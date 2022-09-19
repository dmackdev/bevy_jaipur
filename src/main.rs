mod ai;
mod card_selection;
mod common_systems;
mod event;
mod game;
mod game_resources;
mod label;
mod move_execution;
mod move_validation;
mod positioning;
mod resources;
mod states;
mod ui;

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_interact_2d::{Group, InteractionSource};
use event::EventsPlugin;
use game::*;
use resources::GameState;
use states::{AppState, TurnState};
use ui::game_ui::GameUiPlugin;
use ui::main_menu::MainMenuPlugin;

#[allow(clippy::type_complexity)]

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Jaipur".to_string(),
            resizable: true,
            ..default()
        })
        .init_resource::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(EventsPlugin)
        .add_state(AppState::MainMenu)
        .add_state(TurnState::None)
        .add_startup_system(setup_app)
        .add_plugin(MainMenuPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(GameUiPlugin)
        .run();
}

fn setup_app(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle {
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::Auto {
                    min_width: 800.,
                    min_height: 600.,
                },
                scale: 2.0,
                ..default()
            },
            ..default()
        })
        .insert(InteractionSource {
            groups: vec![Group(0)],
            ..default()
        });
}
