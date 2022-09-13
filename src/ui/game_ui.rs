use bevy::prelude::*;
use itertools::Itertools;
use std::{fmt, ops::DerefMut};

use crate::{
    card_selection::SelectedCardState,
    event::ConfirmTurnEvent,
    game::{ActivePlayer, TokensOwner},
    game_resources::tokens::Tokens,
    label::Label,
    move_validation::MoveValidity,
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
    hovered_color: Color::GRAY,
    pressed_color: Color::BLUE,
};

const SELL_BUTTON_DATA: GameButtonData = GameButtonData {
    kind: GameButtonKind::Sell,
    normal_color: Color::rgb(0.15, 0.15, 0.15),
    hovered_color: Color::GRAY,
    pressed_color: Color::BLUE,
};

// Colors only apply when move is valid and confirm button is enabled
const CONFIRM_BUTTON_DATA: GameButtonData = GameButtonData {
    kind: GameButtonKind::Confirm,
    normal_color: Color::DARK_GREEN,
    hovered_color: Color::rgba(0.0, 1.0, 0.0, 0.5),
    pressed_color: Color::DARK_GREEN,
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
                size: Size::new(Val::Px(150.0), Val::Px(45.0)),
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
                    font_size: 20.0,
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
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position: UiRect::new(Val::Auto, Val::Px(0.0), Val::Auto, Val::Px(0.0)),
                ..default()
            },
            color: Color::DARK_GRAY.into(),
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
struct GameTokensUiRoot;

fn setup_tokens_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tokens: Res<Tokens>,
    active_player_tokens_query: Query<&TokensOwner, With<ActivePlayer>>,
) {
    let game_tokens_root_node_entity = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                position: UiRect::new(Val::Px(0.), Val::Auto, Val::Px(0.), Val::Auto),
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(GameTokensUiRoot)
        .id();

    let game_tokens_children = create_tokens_ui(
        &mut commands,
        &asset_server,
        tokens.as_ref(),
        "Remaining game tokens".to_string(),
    );

    commands
        .entity(game_tokens_root_node_entity)
        .push_children(&game_tokens_children);

    let player_tokens_root_node_entity = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                position: UiRect::new(Val::Auto, Val::Px(0.), Val::Px(0.), Val::Auto),
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(GameTokensUiRoot)
        .id();

    let player_tokens_children = create_tokens_ui(
        &mut commands,
        &asset_server,
        &active_player_tokens_query.single().0,
        "Your tokens".to_string(),
    );

    commands
        .entity(player_tokens_root_node_entity)
        .push_children(&player_tokens_children);
}

fn create_tokens_ui(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    tokens: &Tokens,
    title: String,
) -> Vec<Entity> {
    let mut v: Vec<Entity> = vec![];
    v.push(
        commands
            .spawn_bundle(
                TextBundle::from_section(
                    title,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.)),
                    ..default()
                }),
            )
            .id(),
    );
    tokens.goods.iter().for_each(|(good_type, token_values)| {
        let t = commands
            .spawn_bundle(
                TextBundle::from_section(
                    format!("{:?}: {:?}", good_type, token_values),
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.)),
                    ..default()
                }),
            )
            .id();

        v.push(t);
    });

    tokens.bonus.iter().for_each(|(bonus_type, token_values)| {
        let t = commands
            .spawn_bundle(
                TextBundle::from_section(
                    format!("{:?} bonus tokens: {:?}", bonus_type, token_values.len()),
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.)),
                    ..default()
                }),
            )
            .id();

        v.push(t);
    });
    v
}

fn cleanup_tokens_ui(
    mut commands: Commands,
    tokens_ui_query: Query<Entity, With<GameTokensUiRoot>>,
) {
    for e in tokens_ui_query.iter() {
        commands.entity(e).despawn_recursive();
    }
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

        match (*interaction, is_button_selected) {
            (Interaction::Clicked, true) => {
                *color = game_button.0.normal_color.into();
                turn_state.set(TurnState::None).unwrap();
            }
            (Interaction::Clicked, false) => {
                *color = game_button.0.pressed_color.into();
                turn_state.set(game_button.0.kind.into()).unwrap();
                commands.entity(interacted_entity).insert(JustClickedButton);
            }
            (Interaction::Hovered, false) => {
                *color = game_button.0.hovered_color.into();
            }
            (Interaction::None, true) => {
                *color = game_button.0.pressed_color.into();
            }
            (Interaction::None, false) => {
                *color = game_button.0.normal_color.into();
            }
            (_, _) => return,
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
    mut move_validity_state: ResMut<MoveValidity>,
    mut selected_card_state: ResMut<SelectedCardState>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &GameButton),
        (Changed<Interaction>, With<ConfirmGameButton>),
    >,
    ui_root_query: Query<Entity, With<GameUiRoot>>,
) {
    for (interaction, mut color, game_button) in &mut interaction_query {
        if *move_validity_state.as_ref() == MoveValidity::Invalid {
            return;
        }

        let move_type = match move_validity_state.deref_mut() {
            MoveValidity::Invalid => return,
            MoveValidity::Valid(m) => m,
        };

        match *interaction {
            Interaction::Clicked => {
                *color = game_button.0.pressed_color.into();

                let desired_turn_state: TurnState = game_button.0.kind.into();
                if *turn_state.current() != desired_turn_state {
                    turn_state.set(desired_turn_state).unwrap();
                }

                selected_card_state.0.clear();

                ev_confirm_turn.send(ConfirmTurnEvent(*move_type));
                *move_validity_state = MoveValidity::default();

                for root_entity in ui_root_query.iter() {
                    commands.entity(root_entity).despawn_recursive();
                }
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

fn handle_move_validity_change(
    move_validity_state: Res<MoveValidity>,
    mut confirm_button_query: Query<(&mut UiColor, &GameButton), With<ConfirmGameButton>>,
) {
    if !move_validity_state.is_changed() {
        return;
    }

    let (mut confirm_button_color, game_button) = confirm_button_query.single_mut();

    match move_validity_state.as_ref() {
        MoveValidity::Invalid => *confirm_button_color = Color::RED.into(),
        MoveValidity::Valid(_) => *confirm_button_color = game_button.0.normal_color.into(),
    }
}

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::InGame)
                .with_system(setup_game_ui)
                .with_system(setup_tokens_ui),
        )
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
                )
                .with_system(handle_move_validity_change),
        )
        .add_system_set(
            SystemSet::on_exit(TurnState::Sell)
                .with_system(cleanup_tokens_ui.before(setup_tokens_ui))
                .with_system(setup_tokens_ui),
        )
        .add_system_set(
            SystemSet::on_enter(AppState::TurnTransition).with_system(cleanup_tokens_ui),
        )
        .add_system_set(SystemSet::on_enter(AppState::GameOver).with_system(cleanup_tokens_ui));
    }
}
