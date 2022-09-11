use bevy::prelude::*;
use big_brain::{prelude::ActionState, scorers::Score, thinker::Actor};

use crate::{
    card_selection::SelectedCard,
    event::ConfirmTurnEvent,
    game_resources::card::{Card, CardType, MarketCard},
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
) {
    use rand::Rng;

    let mut rng = rand::thread_rng();

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

        let num_camel_cards = market_camel_cards.len();

        if 0 < num_camel_cards {
            scorer_state.card_entities = Some(market_camel_cards);
            score.set(rng.gen_range(0..=1) as f32);
        } else {
            scorer_state.card_entities = None;
            score.set(0.0);
        }
    }
}
