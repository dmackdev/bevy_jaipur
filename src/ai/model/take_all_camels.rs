use bevy::prelude::*;
use big_brain::{prelude::ActionState, scorers::Score, thinker::Actor};

use crate::{
    ai::model::math::clamp,
    card_selection::SelectedCard,
    event::ConfirmTurnEvent,
    game_resources::card::{
        ActivePlayerGoodsCard, Card, CardType, InactivePlayerGoodsCard, MarketCard,
    },
    move_validation::MoveType,
    states::AppState,
};

#[derive(Component, Debug, Clone)]
pub struct TakeAllCamelsAction;

pub fn take_all_camels_action_system(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    mut action_query: Query<(&Actor, &mut ActionState, &TakeAllCamelsAction)>,
    mut ev_confirm_turn: EventWriter<ConfirmTurnEvent>,
    scorer_states_query: Query<&TakeAllCamelsScorerState>,
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
                println!("EXECUTING TAKE ALL CAMELS");
                if let Ok(scorer_state) = scorer_states_query.get(*actor) {
                    for good in scorer_state.card_entities.clone().unwrap() {
                        commands.entity(good).insert(SelectedCard);
                    }
                    ev_confirm_turn.send(ConfirmTurnEvent(MoveType::TakeAllCamels));
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
pub struct TakeAllCamelsScorer;

#[derive(Default, Component, Debug)]
pub struct TakeAllCamelsScorerState {
    card_entities: Option<Vec<Entity>>,
}

pub fn take_all_camels_scorer_system(
    app_state: Res<State<AppState>>,
    mut query: Query<(&Actor, &mut Score), With<TakeAllCamelsScorer>>,
    mut scorer_states_query: Query<&mut TakeAllCamelsScorerState>,
    all_market_card_query: Query<(Entity, &Card), With<MarketCard>>,
    active_player_goods_cards: Query<Entity, With<ActivePlayerGoodsCard>>,
    opponent_goods_cards: Query<Entity, With<InactivePlayerGoodsCard>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        let mut scorer_state = scorer_states_query.get_mut(*actor).unwrap();

        if !matches!(app_state.current(), AppState::AiTurn) {
            scorer_state.card_entities = None;
            score.set(0.0);
            continue;
        }

        let market_camel_cards = all_market_card_query
            .iter()
            .filter_map(|(e, card)| match card.0 {
                CardType::Camel => Some(e),
                CardType::Good(_) => None,
            })
            .collect::<Vec<_>>();

        let num_camels_in_market = market_camel_cards.len();

        if num_camels_in_market > 0 {
            let num_goods_in_hand = active_player_goods_cards.iter().count();
            let num_goods_in_opponent_hand = opponent_goods_cards.iter().count();

            let score_value = calculate_score(
                num_camels_in_market,
                num_goods_in_hand,
                num_goods_in_opponent_hand,
            );

            println!("START CAMELS SCORE INFO");
            println!("NUMBER OF CAMELS IN MARKET: {}", num_camels_in_market);
            println!("NUMBER OF GOODS IN AI HAND: {}", num_goods_in_hand);
            println!(
                "NUMBER OF GOODS IN HUMAN HAND: {}",
                num_goods_in_opponent_hand
            );
            println!("SCORE: {}", score_value);
            println!("END CAMELS SCORE INFO");

            scorer_state.card_entities = Some(market_camel_cards);
            score.set(score_value);
        } else {
            scorer_state.card_entities = None;
            score.set(0.0);
        }
    }
}

fn calculate_score(
    num_camels_in_market: usize,
    num_goods_in_hand: usize,
    num_goods_in_opponent_hand: usize,
) -> f32 {
    let weighted_num_camels_in_market = num_camels_in_market.pow(2) as f32;
    let weighted_num_goods_in_hand = 2.0 * ((0.5 * num_goods_in_hand as f32).powf(2.0));
    let raw_score = (weighted_num_camels_in_market - weighted_num_goods_in_hand
        + num_goods_in_opponent_hand as f32)
        * 0.8
        / 32.0;

    clamp(raw_score, 0.0, 1.0)
}

// The best time to take camels is when:
// market is full of camels, and current player has few goods cards, and opponent has a full hand, so
// num_camels_in_market - num_goods_in_hand + num_goods_in_opponent_hand
// At the best case this would be: 5 - 0 + 7 = 12, but num_goods_in_opponent_hand has too much influence - even with 0 camels this component takes up more than half of the best score.
// We need to add weightings to put more importance on num_camels_in_market and less on num_goods_in_hand
// num_camels_in_market^2 - 2 * (0.5 * num_goods_in_hand)^2 + num_goods_in_opponent_hand
// Best case: 5^2 - 0 + 7 = 32, which we map to be equal score of selling 4 goods: 80%
// So our formula becomes [num_camels_in_market^2 - 2 * (0.5 * num_goods_in_hand)^2 + num_goods_in_opponent_hand] * 0.8/32
// If current player has a full goods hand, this would be a bad time to take all camels
// since they cannot exchange them for goods on their next turn
// These weightings cause the num_camels_in_market and num_goods_in_hand components to roughly cancel out when each is maximised, and yields a low score overall, but not zero since this is still a viable move
// [5^2 - 2 * (0.5 * 7)^2 + 7] * 0.8/32 = [25 - 24.5 + 7] * 0.8/32 = 0.1875
