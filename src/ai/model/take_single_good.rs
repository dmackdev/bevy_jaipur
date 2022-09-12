use bevy::prelude::*;
use big_brain::{prelude::ActionState, scorers::Score, thinker::Actor};

use crate::{
    card_selection::SelectedCard,
    event::ConfirmTurnEvent,
    game_resources::card::{ActivePlayerGoodsCard, Card, CardType, MarketCard},
    move_validation::MoveType,
    states::AppState,
};

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
    use rand::Rng;

    let mut rng = rand::thread_rng();

    for (Actor(actor), mut score) in query.iter_mut() {
        let mut scorer_state = scorer_states_query.get_mut(*actor).unwrap();

        let num_goods_in_hand = active_player_goods_hand_query.iter().count();

        if !matches!(app_state.current(), AppState::AiTurn) || num_goods_in_hand >= 7 {
            scorer_state.card_entity = None;
            score.set(0.0);
            continue;
        }

        let good_to_take = market_cards_query
            .iter()
            .find(|(_, c)| matches!(c.0, CardType::Good(_)));

        match good_to_take {
            Some((e, _)) => {
                scorer_state.card_entity = Some(e);
                score.set(rng.gen_range(0..=1) as f32);
            }
            None => {
                println!("NO GOOD TO TAKE");
                scorer_state.card_entity = None;
                score.set(0.0);
            }
        }
    }
}
