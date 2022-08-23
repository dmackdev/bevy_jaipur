mod app_state;
mod common_systems;
mod event;
mod game;
mod game_ui;
mod main_menu;

use app_state::AppState;
use bevy::prelude::*;
use event::EventsPlugin;
use game::*;
use game_ui::GameUiPlugin;
use main_menu::MainMenuPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EventsPlugin)
        .add_state(AppState::MainMenu)
        .add_startup_system(setup_app)
        .add_plugin(MainMenuPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(GameUiPlugin)
        .run();
}

fn setup_app(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
