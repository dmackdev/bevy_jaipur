use std::fmt;

use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    event::ConfirmTurnEvent,
    label::Label,
    states::{AppState, TurnState},
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

#[derive(Component)]
struct GameButton(GameButtonData);

#[derive(Component)]
struct TakeGameButton;

#[derive(Component)]
struct SellGameButton;

#[derive(Component)]
struct ConfirmGameButton;

const TAKE_BUTTON_DATA: GameButtonData = GameButtonData {
    kind: GameButtonKind::Take,
    normal_color: Color::rgb(0.15, 0.15, 0.15),
    hovered_color: Color::rgb(0.25, 0.25, 0.25),
    pressed_color: Color::rgb(0.35, 0.75, 0.35),
};

const SELL_BUTTON_DATA: GameButtonData = GameButtonData {
    kind: GameButtonKind::Sell,
    normal_color: Color::rgb(0.15, 0.15, 0.15),
    hovered_color: Color::rgb(0.25, 0.25, 0.25),
    pressed_color: Color::rgb(0.35, 0.75, 0.35),
};

const CONFIRM_BUTTON_DATA: GameButtonData = GameButtonData {
    kind: GameButtonKind::Confirm,
    normal_color: Color::rgb(0.15, 0.15, 0.15),
    hovered_color: Color::rgb(0.25, 0.25, 0.25),
    pressed_color: Color::rgb(0.35, 0.75, 0.35),
};

fn create_button<C: Component>(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    button_component: C,
    game_button_data: GameButtonData,
) -> Entity {
    commands
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
            color: game_button_data.normal_color.into(),
            ..default()
        })
        .insert(button_component)
        .insert(GameButton(game_button_data))
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                game_button_data.kind.to_string(),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        })
        .id()
}

#[derive(Component)]
struct GameUiRoot;

fn setup_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let root_node_entity = commands
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
        .insert(GameUiRoot)
        .id();

    let take_button_entity = create_button(
        &mut commands,
        &asset_server,
        TakeGameButton,
        TAKE_BUTTON_DATA,
    );
    let sell_button_entity = create_button(
        &mut commands,
        &asset_server,
        SellGameButton,
        SELL_BUTTON_DATA,
    );
    let confirm_button_entity = create_button(
        &mut commands,
        &asset_server,
        ConfirmGameButton,
        CONFIRM_BUTTON_DATA,
    );

    commands.entity(root_node_entity).push_children(&[
        take_button_entity,
        sell_button_entity,
        confirm_button_entity,
    ]);
}

#[derive(Component)]
struct JustClickedButton;

fn handle_turn_state_button(
    mut commands: Commands,
    mut turn_state: ResMut<State<TurnState>>,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut UiColor, &GameButton),
        (Changed<Interaction>, Without<ConfirmGameButton>),
    >,
) {
    for (interacted_entity, interaction, mut color, game_button) in &mut interaction_query {
        let is_button_selected = *turn_state.current() == game_button.0.kind.into();

        match *interaction {
            Interaction::Clicked => {
                if is_button_selected {
                    *color = game_button.0.normal_color.into();
                    turn_state.set(TurnState::None).unwrap();
                } else {
                    *color = game_button.0.pressed_color.into();
                    turn_state.set(game_button.0.kind.into()).unwrap();
                    commands.entity(interacted_entity).insert(JustClickedButton);
                }
            }
            Interaction::Hovered => {
                if is_button_selected {
                    return;
                }
                *color = game_button.0.hovered_color.into();
            }
            Interaction::None => {
                if is_button_selected {
                    *color = game_button.0.pressed_color.into();
                } else {
                    *color = game_button.0.normal_color.into();
                }
            }
        }
    }
}

fn update_unclicked_turn_move_button_colors(
    mut commands: Commands,
    just_clicked_button_query: Query<Entity, Added<JustClickedButton>>,
    mut other_buttons_query: Query<(Entity, &mut UiColor, &GameButton), Without<ConfirmGameButton>>,
) {
    if just_clicked_button_query.iter().count() == 0 {
        return;
    }

    for e in just_clicked_button_query.iter() {
        commands.entity(e).remove::<JustClickedButton>();
    }

    for (_, mut color, game_button) in other_buttons_query
        .iter_mut()
        .filter(|(e, _, _)| !just_clicked_button_query.iter().contains(e))
    {
        *color = game_button.0.normal_color.into();
    }
}

impl From<GameButtonKind> for TurnState {
    fn from(kind: GameButtonKind) -> Self {
        match kind {
            GameButtonKind::Take => TurnState::Take,
            GameButtonKind::Sell => TurnState::Sell,
            GameButtonKind::Confirm => TurnState::None,
        }
    }
}

fn handle_confirm_button_interaction(
    mut commands: Commands,
    mut turn_state: ResMut<State<TurnState>>,
    mut ev_confirm_turn: EventWriter<ConfirmTurnEvent>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &GameButton),
        (Changed<Interaction>, With<ConfirmGameButton>),
    >,
    ui_root_query: Query<Entity, With<GameUiRoot>>,
) {
    for (interaction, mut color, game_button) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = game_button.0.pressed_color.into();

                let desired_turn_state: TurnState = game_button.0.kind.into();
                if *turn_state.current() != desired_turn_state {
                    turn_state.set(desired_turn_state).unwrap();
                }

                ev_confirm_turn.send(ConfirmTurnEvent);
                commands.entity(ui_root_query.single()).despawn_recursive();
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
                    .with_system(handle_turn_state_button)
                    .with_system(
                        update_unclicked_turn_move_button_colors.after(handle_turn_state_button),
                    )
                    .with_system(
                        handle_confirm_button_interaction
                            .label(Label::EventWriter)
                            .before(Label::EventReader),
                    ),
            );
    }
}
