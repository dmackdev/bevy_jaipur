mod app_state;
mod game;
mod main_menu;

use app_state::AppState;
use bevy::prelude::*;
use game::*;
use main_menu::MainMenuPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state(AppState::MainMenu)
        .add_startup_system(setup_app)
        .add_plugin(MainMenuPlugin)
        .add_plugin(GamePlugin)
        .run();
}

fn setup_app(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
