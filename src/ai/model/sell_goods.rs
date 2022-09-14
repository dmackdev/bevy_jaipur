use bevy::prelude::*;
use big_brain::{prelude::ActionState, scorers::Score, thinker::Actor};
use itertools::Itertools;

use crate::{
    ai::model::math::clamp,
    card_selection::SelectedCard,
    event::ConfirmTurnEvent,
    game_resources::card::{ActivePlayerGoodsCard, Card, CardType},
    move_validation::MoveType,
    states::AppState,
};

#[derive(Component, Debug, Clone)]
pub struct SellGoodsAction;

pub fn sell_goods_action_system(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    mut action_query: Query<(&Actor, &mut ActionState, &SellGoodsAction)>,
    mut ev_confirm_turn: EventWriter<ConfirmTurnEvent>,
    scorer_states_query: Query<&SellGoodsScorerState>,
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
                println!("EXECUTING SELL GOODS");
                if let Ok(scorer_state) = scorer_states_query.get(*actor) {
                    for good in scorer_state.card_entities.clone().unwrap() {
                        commands.entity(good).insert(SelectedCard);
                    }
                    ev_confirm_turn.send(ConfirmTurnEvent(MoveType::SellGoods));
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
pub struct SellGoodsScorer;

#[derive(Default, Component, Debug)]
pub struct SellGoodsScorerState {
    card_entities: Option<Vec<Entity>>,
}

pub fn sell_goods_scorer_system(
    app_state: Res<State<AppState>>,
    mut query: Query<(&Actor, &mut Score), With<SellGoodsScorer>>,
    mut scorer_states_query: Query<&mut SellGoodsScorerState>,
    active_player_goods_hand_query: Query<(Entity, &Card), With<ActivePlayerGoodsCard>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        let mut scorer_state = scorer_states_query.get_mut(*actor).unwrap();

        if !matches!(app_state.current(), AppState::AiTurn) {
            scorer_state.card_entities = None;
            score.set(0.0);
            continue;
        }

        let goods_in_hand = active_player_goods_hand_query
            .iter()
            .filter_map(|(e, c)| match c.0 {
                CardType::Good(g) => Some((e, g)),
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut counts = goods_in_hand.iter().counts_by(|(_, good)| good);

        // Prevent selling a single high value good
        counts.retain(|good_type, count| !(good_type.is_high_value() && *count < 2));

        let most_frequent_good = counts.iter().max_by_key(|(_, freq)| *freq);

        match most_frequent_good {
            Some((good_type_to_sell, freq)) => {
                let entities_to_sell = goods_in_hand
                    .iter()
                    .filter_map(|(e, g)| {
                        if *good_type_to_sell == g {
                            Some(*e)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                scorer_state.card_entities = Some(entities_to_sell);
                let score_value = calculate_score(*freq, good_type_to_sell.is_high_value());

                println!("COULD SELL {} GOODS, SCORE {}", freq, score_value);

                score.set(score_value);
            }
            None => {
                println!("NO GOOD TO SELL");
                scorer_state.card_entities = None;
                score.set(0.0);
            }
        }
    }
}

// >=5 of same good in hand => 100%
// 4 of same good in hand => 80%
// 3 of same good in hand => 60%
// 2 of same good in hand => 40%
// 1 of same good in hand => 20%
fn calculate_score(highest_frequency_of_good: usize, is_high_value_good: bool) -> f32 {
    let mut raw_score = (highest_frequency_of_good as f32 * 20.0) / 100.0;

    if is_high_value_good {
        raw_score *= 1.5;
    }

    clamp(raw_score, 0.0, 1.0)
}
