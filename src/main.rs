mod common_systems;
mod event;
mod game;
mod game_ui;
mod label;
mod main_menu;
mod states;

use bevy::prelude::*;
use bevy_interact_2d::{Group, InteractionSource};
use event::EventsPlugin;
use game::*;
use game_ui::GameUiPlugin;
use main_menu::MainMenuPlugin;
use states::{AppState, TurnState};

#[allow(clippy::type_complexity)]

fn main() {
    App::new()
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
        .spawn_bundle(Camera2dBundle::default())
        .insert(InteractionSource {
            groups: vec![Group(0)],
            ..default()
        });
}
