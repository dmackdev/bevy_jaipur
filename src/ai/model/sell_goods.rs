use bevy::prelude::*;
use big_brain::{prelude::ActionState, scorers::Score, thinker::Actor};

use crate::{
    card_selection::SelectedCard,
    event::ConfirmTurnEvent,
    game_resources::card::{ActivePlayerGoodsCard, Card},
    move_validation::MoveType,
    states::AppState,
};

#[derive(Component, Debug, Clone)]
pub struct SellGoodsAction;

#[derive(Debug, Clone)]
pub struct SellGoodsActionBuilder;

pub fn sell_goods_action_system(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    active_player_goods_hand_query: Query<(Entity, &Card), With<ActivePlayerGoodsCard>>,
    mut action_query: Query<(&Actor, &mut ActionState, &SellGoodsAction)>,
    mut ev_confirm_turn: EventWriter<ConfirmTurnEvent>,
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
                let good = active_player_goods_hand_query
                    .iter()
                    .next()
                    .expect("TODO: handle me better");

                commands.entity(good.0).insert(SelectedCard);
                ev_confirm_turn.send(ConfirmTurnEvent(MoveType::SellGoods));
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
pub struct SellGoodsScorer;

pub fn sell_goods_scorer_system(mut query: Query<(&Actor, &mut Score), With<SellGoodsScorer>>) {
    for (Actor(actor), mut score) in query.iter_mut() {
        score.set(1.0);
    }
}
