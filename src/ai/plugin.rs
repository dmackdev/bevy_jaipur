use bevy::prelude::*;
use big_brain::{thinker::Thinker, BigBrainPlugin, BigBrainStage};

use crate::label::Label;

use super::{
    model::{
        sell_goods::{
            sell_goods_action_system, sell_goods_scorer_system, SellGoodsAction, SellGoodsScorer,
            SellGoodsScorerState,
        },
        take_all_camels::{
            take_all_camels_action_system, take_all_camels_scorer_system, TakeAllCamelsAction,
            TakeAllCamelsScorer, TakeAllCamelsScorerState,
        },
        take_single_good::{
            take_single_good_action_system, take_single_good_scorer_system, TakeSingleGoodAction,
            TakeSingleGoodScorer, TakeSingleGoodScorerState,
        },
    },
    picker::highest_score::HighestScorePicker,
};

pub fn init(mut commands: Commands) {
    commands
        .spawn()
        .insert(SellGoodsScorerState::default())
        .insert(TakeSingleGoodScorerState::default())
        .insert(TakeAllCamelsScorerState::default())
        .insert(
            Thinker::build()
                .picker(HighestScorePicker { threshold: 0.1 })
                .when(TakeSingleGoodScorer, TakeSingleGoodAction)
                .when(SellGoodsScorer, SellGoodsAction)
                .when(TakeAllCamelsScorer, TakeAllCamelsAction),
        );
}

pub struct JaipurAiPlugin;

impl Plugin for JaipurAiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(BigBrainPlugin)
            .add_startup_system(init)
            .add_system_set_to_stage(
                BigBrainStage::Actions,
                SystemSet::new()
                    .label(Label::EventWriter)
                    .with_system(take_single_good_action_system)
                    .with_system(sell_goods_action_system)
                    .with_system(take_all_camels_action_system),
            )
            .add_system_set_to_stage(
                BigBrainStage::Scorers,
                SystemSet::new()
                    .with_system(take_single_good_scorer_system)
                    .with_system(sell_goods_scorer_system)
                    .with_system(take_all_camels_scorer_system),
            );
    }
}
