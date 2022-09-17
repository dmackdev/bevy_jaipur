use std::cmp;

use bevy::prelude::*;
use big_brain::{prelude::ActionState, scorers::Score, thinker::Actor};
use itertools::{Either, Itertools};

use crate::{
    ai::model::math::clamp,
    card_selection::SelectedCard,
    event::ConfirmTurnEvent,
    game_resources::card::{
        ActivePlayerCamelCard, ActivePlayerGoodsCard, Card, CardType, MarketCard,
    },
    move_validation::MoveType,
    states::AppState,
};

#[derive(Component, Debug, Clone)]
pub struct ExchangeGoodsAction;

pub fn exchange_goods_action_system(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    mut action_query: Query<(&Actor, &mut ActionState, &ExchangeGoodsAction)>,
    mut ev_confirm_turn: EventWriter<ConfirmTurnEvent>,
    scorer_states_query: Query<&ExchangeGoodsScorerState>,
) {
    if !matches!(app_state.current(), AppState::AiTurn) {
        return;
    }

    for (Actor(actor), mut state, _) in action_query.iter_mut() {
        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                if let Ok(scorer_state) = scorer_states_query.get(*actor) {
                    for good in scorer_state.card_entities.clone().unwrap() {
                        commands.entity(good).insert(SelectedCard);
                    }
                    ev_confirm_turn.send(ConfirmTurnEvent(MoveType::ExchangeForGoodsFromMarket));
                    *state = ActionState::Success;
                    return;
                }
                *state = ActionState::Failure;
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Component, Debug)]
pub struct ExchangeGoodsScorer;

#[derive(Default, Component, Debug)]
pub struct ExchangeGoodsScorerState {
    card_entities: Option<Vec<Entity>>,
}

pub fn exchange_goods_scorer_system(
    app_state: Res<State<AppState>>,
    mut query: Query<(&Actor, &mut Score), With<ExchangeGoodsScorer>>,
    mut scorer_states_query: Query<&mut ExchangeGoodsScorerState>,
    active_player_cards_hand_query: Query<
        (Entity, &Card),
        Or<(With<ActivePlayerGoodsCard>, With<ActivePlayerCamelCard>)>,
    >,
    market_cards_query: Query<(Entity, &Card), With<MarketCard>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        let mut scorer_state = scorer_states_query.get_mut(*actor).unwrap();

        if !matches!(app_state.current(), AppState::AiTurn) {
            scorer_state.card_entities = None;
            score.set(0.0);
            continue;
        }

        let market_goods = market_cards_query
            .iter()
            .filter_map(|(_, card)| match card.0 {
                CardType::Good(good_type) => Some(good_type),
                _ => None,
            })
            .collect::<Vec<_>>();

        let (goods_in_hand, camels_in_hand): (Vec<_>, Vec<_>) = active_player_cards_hand_query
            .iter()
            .partition_map(|(e, c)| match c.0 {
                CardType::Good(g) => Either::Left((e, g)),
                _ => Either::Right(e),
            });

        let num_goods_in_hand = goods_in_hand.len();
        let num_camels_in_hand = camels_in_hand.len();

        // Exchanging camels will add extra goods to your hand - ensure this would not exceed max number of goods allowed in hand
        // And you cannot exchange more camels than you have
        let num_camels_permitted_to_exchange =
            cmp::min(7 - num_goods_in_hand as i32, num_camels_in_hand as i32);

        // For each good in the market, if you took all of that type of good, how many would you get in your hand?
        let goods_hand_counts = goods_in_hand.iter().counts_by(|(_, good)| good);
        let mut goods_hand_counts_after_market_take = market_goods.iter().counts_by(|good| good);

        for (good, count) in goods_hand_counts_after_market_take.iter_mut() {
            *count += goods_hand_counts.get(good).unwrap_or(&0);
        }
        // TODO: filter out goods_hand_counts_after_market_take for which there is count of one, unless its a high value good
        println!("{:?}", goods_hand_counts_after_market_take);

        // Find goods in hand of which there are only one, that are not in the market
        // TODO: Filter out high value goods to not exchange them, unless there are no more tokens for it
        let eligible_single_goods_in_hand = goods_in_hand
            .iter()
            .filter_map(|(ent, good_type)| {
                let count = goods_hand_counts.get(good_type);
                match count {
                    Some(count) => {
                        if *count == 1 && !market_goods.iter().contains(good_type) {
                            Some(*ent)
                        } else {
                            None
                        }
                    }
                    None => None,
                }
            })
            .collect::<Vec<_>>();

        // TODO:
        // 1 Order market goods descending by how many of that good you would end up with in your hand
        // 2 Zip [...camel entities permitted to exchange, ...single goods in hand entities] with market entities from step 1
        // 3 If there are at least 2 tuples, then set the score above 0 proportionate to the new counts of goods in your hand after the exchange, else 0

        let market_goods_with_ents = market_cards_query
            .iter()
            .filter_map(|(ent, card)| match card.0 {
                CardType::Good(good_type) => Some((ent, good_type)),
                _ => None,
            })
            .collect::<Vec<_>>();

        let entities_to_exchange_from_hand = camels_in_hand
            .iter()
            .take(num_camels_permitted_to_exchange as usize)
            .chain(eligible_single_goods_in_hand.iter());

        let sorted_market_entities = market_goods_with_ents
            .iter()
            .sorted_by_key(|(_, g)| goods_hand_counts_after_market_take.get(g).unwrap_or(&0))
            .map(|(ent, _)| ent)
            .rev()
            .collect::<Vec<_>>();

        let zipped = entities_to_exchange_from_hand
            .zip(sorted_market_entities)
            .collect::<Vec<_>>();

        let zipped_len = zipped.len();

        if zipped_len < 2 {
            println!("COULD NOT EXCHANGE CARDS",);
            scorer_state.card_entities = None;
            score.set(0.0);
        } else {
            println!("FOUND {} CARDS TO EXCHANGE", zipped_len);
            let mut ents = vec![];
            for (hand_ent, market_ent) in zipped {
                ents.push(*hand_ent);
                ents.push(*market_ent);
            }
            scorer_state.card_entities = Some(ents);

            let highest_count_after_take = goods_hand_counts_after_market_take
                .iter()
                .sorted_by_key(|(_, count)| *count)
                .rev()
                .next()
                .unwrap()
                .1;

            // TODO: I am adding one to make this a higher score than take single good for the corresponding number of goods in hand after
            // This is because the exchange must happen on at least two different goods types, so we are in total taking more cards in of value
            let score_value = clamp(((highest_count_after_take + 1) * 2) as f32 / 10.0, 0.0, 1.0);
            println!("SCORE: {}", score_value);

            score.set(score_value);
        }
    }
}
