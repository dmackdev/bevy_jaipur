use bevy::prelude::*;
use big_brain::{prelude::ActionState, scorers::Score, thinker::Actor};

use crate::{
    card_selection::SelectedCard,
    event::ConfirmTurnEvent,
    game_resources::card::{ActivePlayerGoodsCard, Card, CardType, GoodType, MarketCard},
    move_validation::MoveType,
    states::AppState,
};

use super::math::clamp;

#[derive(Component, Debug, Clone)]
pub struct TakeSingleGoodAction;

pub fn take_single_good_action_system(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    mut action_query: Query<(&Actor, &mut ActionState), With<TakeSingleGoodAction>>,
    mut ev_confirm_turn: EventWriter<ConfirmTurnEvent>,
    scorer_states_query: Query<&TakeSingleGoodScorerState>,
) {
    if !matches!(app_state.current(), AppState::AiTurn) {
        return;
    }

    for (Actor(actor), mut state) in action_query.iter_mut() {
        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                println!("EXECUTING TAKE SINGLE GOOD");

                let good = scorer_states_query
                    .get(*actor)
                    .unwrap()
                    .card_entity
                    .unwrap();

                commands.entity(good).insert(SelectedCard);
                ev_confirm_turn.send(ConfirmTurnEvent(MoveType::TakeSingleGood));
                *state = ActionState::Success;
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Component, Debug)]
pub struct TakeSingleGoodScorer;

#[derive(Default, Component, Debug)]
pub struct TakeSingleGoodScorerState {
    card_entity: Option<Entity>,
}

pub fn take_single_good_scorer_system(
    app_state: Res<State<AppState>>,
    mut query: Query<(&Actor, &mut Score), With<TakeSingleGoodScorer>>,
    mut scorer_states_query: Query<&mut TakeSingleGoodScorerState>,
    market_cards_query: Query<(Entity, &Card), With<MarketCard>>,
    active_player_goods_hand_query: Query<&Card, With<ActivePlayerGoodsCard>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        let mut scorer_state = scorer_states_query.get_mut(*actor).unwrap();

        let num_goods_in_hand = active_player_goods_hand_query.iter().count();

        if !matches!(app_state.current(), AppState::AiTurn) || num_goods_in_hand >= 7 {
            scorer_state.card_entity = None;
            score.set(0.0);
            continue;
        }

        let goods_in_hand = active_player_goods_hand_query
            .iter()
            .filter_map(|c| match c.0 {
                CardType::Good(g) => Some(g),
                _ => None,
            })
            .collect::<Vec<_>>();

        let best_good_to_take = market_cards_query
            .iter()
            .filter_map(|(ent, c)| match c.0 {
                CardType::Good(good_type) => Some((
                    ent,
                    good_type,
                    // TODO: refactor to pass good_type and goods_in_hand to calculate_score
                    calculate_score(
                        get_num_goods_in_hand(good_type, &goods_in_hand),
                        good_type.is_high_value(),
                    ),
                )),
                _ => None,
            })
            .max_by(|(_, _, score_a), (_, _, score_b)| score_a.total_cmp(score_b));

        match best_good_to_take {
            Some((e, good_type, score_for_good)) => {
                println!(
                    "TAKE SINGLE GOOD {:?}, SCORE: {}",
                    good_type, score_for_good
                );
                scorer_state.card_entity = Some(e);
                score.set(score_for_good);
            }
            None => {
                println!("NO GOOD TO TAKE");
                scorer_state.card_entity = None;
                score.set(0.0);
            }
        }
    }
}

fn get_num_goods_in_hand(good_type: GoodType, goods_in_hand: &[GoodType]) -> usize {
    goods_in_hand.iter().filter(|g| **g == good_type).count()
}
// If there is a good in the market that would give you
// 5 of that good in your hand => 100%
// 4 of that good in your hand => 80%
// 3 of that good in your hand => 60%
// 2 of that good in your hand => 40%
// 1 of that good in your hand => 20%
fn calculate_score(num_good_in_hand: usize, is_high_value_good: bool) -> f32 {
    let mut raw_score = ((num_good_in_hand + 1) * 2) as f32;

    if is_high_value_good {
        raw_score *= 1.5;
    }

    raw_score /= 10.0;

    clamp(raw_score, 0.0, 1.0)
}
