mod common_systems;
mod event;
mod game;
mod game_ui;
mod interaction;
mod main_menu;
mod states;

use bevy::prelude::*;
use event::EventsPlugin;
use game::*;
use game_ui::GameUiPlugin;
use interaction::{Group, InteractionSource};
use main_menu::MainMenuPlugin;
use states::{AppState, TurnState};

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
