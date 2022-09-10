use bevy::prelude::*;
use big_brain::{thinker::Thinker, BigBrainPlugin, BigBrainStage};

use crate::label::Label;

use super::{
    model::take_single_good::{
        take_single_good_action_system, take_single_good_scorer_system, TakeSingleGoodAction,
        TakeSingleGoodScorer,
    },
    picker::highest_score::HighestScorePicker,
};

pub fn init(mut commands: Commands) {
    commands.spawn().insert(
        Thinker::build()
            .picker(HighestScorePicker { threshold: 0.1 })
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
