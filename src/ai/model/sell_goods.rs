use bevy::prelude::*;
use big_brain::{prelude::ActionState, scorers::Score, thinker::Actor};

use crate::{
    card_selection::SelectedCard, event::ConfirmTurnEvent,
    game_resources::card::ActivePlayerGoodsCard, move_validation::MoveType, states::AppState,
};

#[derive(Component, Debug, Clone)]
pub struct SellGoodsAction;

#[derive(Debug, Clone)]
pub struct SellGoodsActionBuilder;

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
    active_player_goods_hand_query: Query<Entity, With<ActivePlayerGoodsCard>>,
) {
    if !matches!(app_state.current(), AppState::AiTurn) {
        return;
    }

    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(mut scorer_state) = scorer_states_query.get_mut(*actor) {
            let good = active_player_goods_hand_query
                .iter()
                .last()
                .expect("TODO: handle me better");

            scorer_state.card_entities = Some(vec![good]);
            score.set(1.0);
        } else {
            score.set(0.0);
        }
    }
}
