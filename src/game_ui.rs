use std::fmt;

use bevy::prelude::*;

use crate::{
    common_systems::despawn_entity_with_component, event::ConfirmTurnEvent, states::AppState,
};

#[derive(Debug, Copy, Clone)]
enum GameButtonKind {
    Take,
    Sell,
    Confirm,
}

impl fmt::Display for GameButtonKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone)]
struct GameButtonData {
    kind: GameButtonKind,
    normal_color: Color,
    hovered_color: Color,
    pressed_color: Color,
}

trait GameButton {
    fn get_data(&self) -> GameButtonData;
}

#[derive(Component, Copy, Clone)]
struct TakeGameButton(GameButtonData);

impl GameButton for TakeGameButton {
    fn get_data(&self) -> GameButtonData {
        self.0
    }
}

#[derive(Component, Copy, Clone)]
struct SellGameButton(GameButtonData);

impl GameButton for SellGameButton {
    fn get_data(&self) -> GameButtonData {
        self.0
    }
}

#[derive(Component, Copy, Clone)]
struct ConfirmGameButton(GameButtonData);

impl GameButton for ConfirmGameButton {
    fn get_data(&self) -> GameButtonData {
        self.0
    }
}

const TAKE_BUTTON: TakeGameButton = TakeGameButton(GameButtonData {
    kind: GameButtonKind::Take,
    normal_color: Color::rgb(0.15, 0.15, 0.15),
    hovered_color: Color::rgb(0.25, 0.25, 0.25),
    pressed_color: Color::rgb(0.35, 0.75, 0.35),
});

const SELL_BUTTON: SellGameButton = SellGameButton(GameButtonData {
    kind: GameButtonKind::Sell,
    normal_color: Color::rgb(0.15, 0.15, 0.15),
    hovered_color: Color::rgb(0.25, 0.25, 0.25),
    pressed_color: Color::rgb(0.35, 0.75, 0.35),
});

const CONFIRM_BUTTON: ConfirmGameButton = ConfirmGameButton(GameButtonData {
    kind: GameButtonKind::Confirm,
    normal_color: Color::rgb(0.15, 0.15, 0.15),
    hovered_color: Color::rgb(0.25, 0.25, 0.25),
    pressed_color: Color::rgb(0.35, 0.75, 0.35),
});

fn create_button(
    parent: &mut ChildBuilder,
    game_button: impl GameButton + Component + Copy,
    asset_server: &Res<AssetServer>,
) {
    parent
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(200.0), Val::Px(65.0)),
                // center button
                margin: UiRect::all(Val::Px(10.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: game_button.get_data().normal_color.into(),
            ..default()
        })
        .insert(game_button)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                game_button.get_data().kind.to_string(),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        });
}

fn setup_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position: UiRect::new(Val::Auto, Val::Px(50.0), Val::Auto, Val::Px(50.0)),
                ..default()
            },
            color: Color::BLUE.into(),
            transform: Transform::default().with_translation(Vec3::new(300.0, 0.0, 0.0)),
            ..default()
        })
        .with_children(|parent| create_button(parent, TAKE_BUTTON, &asset_server))
        .with_children(|parent| create_button(parent, SELL_BUTTON, &asset_server))
        .with_children(|parent| create_button(parent, CONFIRM_BUTTON, &asset_server));
}

fn handle_confirm_button_interaction(
    mut ev_confirm_turn: EventWriter<ConfirmTurnEvent>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &ConfirmGameButton),
        Changed<Interaction>,
    >,
) {
    for (interaction, mut color, game_button) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = game_button.0.pressed_color.into();
                ev_confirm_turn.send(ConfirmTurnEvent);
            }
            Interaction::Hovered => {
                *color = game_button.0.hovered_color.into();
            }
            Interaction::None => {
                *color = game_button.0.normal_color.into();
            }
        }
    }
}

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_game_ui))
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(handle_confirm_button_interaction),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::InGame)
                    .with_system(despawn_entity_with_component::<Node>),
            );
    }
}
