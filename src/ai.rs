use bevy::prelude::*;
use big_brain::{
    prelude::{ActionBuilder, ActionState, FirstToScore},
    scorers::Score,
    thinker::{Actor, Thinker},
    BigBrainPlugin, BigBrainStage,
};

use crate::{
    card_selection::SelectedCard,
    event::ConfirmTurnEvent,
    game_resources::card::{Card, CardType, MarketCard},
    label::Label,
    move_validation::MoveType,
    states::AppState,
};

#[derive(Component, Debug, Clone)]
pub struct TakeSingleGoodAction;

impl TakeSingleGoodAction {
    pub fn build() -> TakeSingleGoodActionBuilder {
        TakeSingleGoodActionBuilder
    }
}

#[derive(Debug, Clone)]
pub struct TakeSingleGoodActionBuilder;

impl ActionBuilder for TakeSingleGoodActionBuilder {
    fn build(
        &self,
        cmd: &mut bevy::prelude::Commands,
        action: bevy::prelude::Entity,
        actor: bevy::prelude::Entity,
    ) {
        cmd.entity(action).insert(TakeSingleGoodAction);
    }
}

fn take_single_good_action_system(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    all_market_cards_query: Query<(Entity, &Card), With<MarketCard>>,
    mut action_query: Query<(&Actor, &mut ActionState, &TakeSingleGoodAction)>,
    mut ev_confirm_turn: EventWriter<ConfirmTurnEvent>,
) {
    if !matches!(app_state.current(), AppState::AiTurn) {
        return;
    }

    for (Actor(actor), mut state, take_single_good_action) in action_query.iter_mut() {
        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                println!("EXECUTING TAKE SINGLE GOOD");
                let good = all_market_cards_query
                    .iter()
                    .find(|(_, c)| matches!(c.0, CardType::Good(_)))
                    .expect("TODO: handle me better");

                commands.entity(good.0).insert(SelectedCard);
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

pub fn take_single_good_scorer_system(
    mut query: Query<(&Actor, &mut Score), With<TakeSingleGoodScorer>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        score.set(1.0);
    }
}

pub fn init(mut commands: Commands) {
    commands.spawn().insert(
        Thinker::build()
            .picker(FirstToScore { threshold: 0.1 })
            .when(TakeSingleGoodScorer, TakeSingleGoodAction),
    );
}

pub struct JaipurAiPlugin;

impl Plugin for JaipurAiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(BigBrainPlugin)
            .add_startup_system(init)
            .add_system_to_stage(
                BigBrainStage::Actions,
                take_single_good_action_system.label(Label::EventWriter),
            )
            .add_system_to_stage(BigBrainStage::Scorers, take_single_good_scorer_system);
    }
}
