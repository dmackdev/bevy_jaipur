use bevy::prelude::*;

use crate::{common_systems::despawn_entity_with_component, states::AppState};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct MenuRootNode;

trait ClickHandler {
    fn on_click(self, state: &mut ResMut<State<AppState>>);
}

#[derive(Component, Copy, Clone)]
struct PlayLocalMultiplayerButton;

impl ClickHandler for PlayLocalMultiplayerButton {
    fn on_click(self, state: &mut ResMut<State<AppState>>) {
        state.set(AppState::InitGame).unwrap();
    }
}

#[derive(Component, Copy, Clone)]
struct PlayAIButton;

impl ClickHandler for PlayAIButton {
    fn on_click(self, state: &mut ResMut<State<AppState>>) {
        state.set(AppState::InitGame).unwrap();
    }
}

fn create_button<C: ClickHandler + Component>(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    click_handler_component: C,
    text: String,
) -> Entity {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                margin: UiRect::all(Val::Px(10.0)),
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .insert(click_handler_component)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                text,
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        })
        .id()
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let root_node_entity = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_grow: 1.0,
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::GRAY.into(),
            ..default()
        })
        .insert(MenuRootNode)
        .id();

    let play_human_button_entity = create_button(
        &mut commands,
        &asset_server,
        PlayLocalMultiplayerButton,
        "Play Local Multiplayer".to_string(),
    );

    let play_ai_button_entity = create_button(
        &mut commands,
        &asset_server,
        PlayAIButton,
        "Play Computer".to_string(),
    );

    commands
        .entity(root_node_entity)
        .push_children(&[play_human_button_entity, play_ai_button_entity]);
}

fn handle_menu_interaction<T: ClickHandler + Component + Copy>(
    mut state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &T),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, click_handler) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                click_handler.on_click(&mut state);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(setup_menu))
            .add_system_set(
                SystemSet::on_update(AppState::MainMenu)
                    .with_system(handle_menu_interaction::<PlayLocalMultiplayerButton>)
                    .with_system(handle_menu_interaction::<PlayAIButton>),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::MainMenu)
                    .with_system(despawn_entity_with_component::<MenuRootNode>),
            );
    }
}
