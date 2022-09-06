use bevy::prelude::*;
use itertools::{Either, Itertools};
use std::cmp::Ordering;

use crate::card_selection::{CardSelectionPlugin, SelectedCardState};
use crate::common_systems::despawn_entity_with_component;
use crate::game_resources::card::*;
use crate::game_resources::deck::Deck;
use crate::game_resources::discard_pile::DiscardPile;
use crate::game_resources::market::Market;
use crate::game_resources::tokens::*;
use crate::move_execution::{MoveExecutionPlugin, ScreenTransitionDelayTimer, TweenState};
use crate::move_validation::{MoveValidationPlugin, MoveValidity};
use crate::positioning::{
    get_active_player_camel_card_translation, get_active_player_goods_card_translation,
    get_market_card_translation, CARD_DIMENSION, CARD_PADDING, DECK_START_POS, DISCARD_PILE_POS,
    INACTIVE_PLAYER_GOODS_HAND_START_POS,
};
use crate::resources::GameState;
use crate::states::AppState;

#[allow(clippy::too_many_arguments)]
fn handle_when_resources_ready(
    mut state: ResMut<State<AppState>>,
    game_state: Res<GameState>,
    deck: Option<Res<Deck>>,
    market: Option<Res<Market>>,
    tokens: Option<Res<Tokens>>,
    tween_state: Option<Res<TweenState>>,
    timer: Option<Res<ScreenTransitionDelayTimer>>,
    selected_card_state: Option<Res<SelectedCardState>>,
    move_validity: Option<Res<MoveValidity>>,
    discard_pile: Option<Res<DiscardPile>>,
) {
    let resources_are_ready = deck.is_some()
        && market.is_some()
        && tokens.is_some()
        && tween_state.is_some()
        && timer.is_some()
        && selected_card_state.is_some()
        && move_validity.is_some()
        && discard_pile.is_some();

    match (resources_are_ready, game_state.is_playing_ai) {
        (true, true) => state.set(AppState::InGame).unwrap(),
        (true, false) => state.set(AppState::TurnTransition).unwrap(),
        _ => {}
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct TurnTransitionScreen;

fn setup_turn_transition_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<&PlayerName, With<ActivePlayer>>,
) {
    let current_player_name = &query.single().0;

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_grow: 1.0,
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::CRIMSON.into(),
            ..default()
        })
        .insert(TurnTransitionScreen)
        .with_children(|parent| {
            parent.spawn_bundle(
                TextBundle::from_section(
                    format!("{}: your turn", current_player_name),
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                }),
            );
        })
        .with_children(|parent| {
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
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        "Start turn",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
        });
}

fn handle_turn_transition_screen_interaction(
    mut state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                state.set(AppState::InGame).unwrap();
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

fn setup_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    players_query: Query<(&PlayerName, &TokensOwner, &CamelsHandOwner)>,
) {
    let root_entity = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_grow: 1.0,
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::CRIMSON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(
                TextBundle::from_section(
                    "Game Over".to_string(),
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                }),
            );
        })
        .id();

    let mut children: Vec<Entity> = vec![];

    let players: Vec<_> = players_query.iter().collect();

    let first_player = players[0];
    let first_player_num_camels = first_player.2 .0;
    let mut first_player_stats = PlayerStats {
        name: first_player.0 .0.to_string(),
        final_score: get_tokens_score(&first_player.1 .0),
        camel_bonus_awarded: false,
    };

    let second_player = players[1];
    let second_player_num_camels = second_player.2 .0;

    let mut second_player_stats = PlayerStats {
        name: second_player.0 .0.to_string(),
        final_score: get_tokens_score(&second_player.1 .0),
        camel_bonus_awarded: false,
    };

    match first_player_num_camels.cmp(&second_player_num_camels) {
        Ordering::Greater => {
            first_player_stats.camel_bonus_awarded = true;
            first_player_stats.final_score += 5;
        }
        Ordering::Less => {
            second_player_stats.camel_bonus_awarded = true;
            second_player_stats.final_score += 5;
        }
        Ordering::Equal => {}
    }

    for stats in &[first_player_stats.clone(), second_player_stats.clone()] {
        let camel_bonus_text = if stats.camel_bonus_awarded {
            " (Camel bonus awarded)"
        } else {
            ""
        };

        let txt = commands
            .spawn_bundle(
                TextBundle::from_section(
                    format!("{}: {}{}", stats.name, stats.final_score, camel_bonus_text),
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                }),
            )
            .id();

        children.push(txt);
    }

    let winning_player = match first_player_stats
        .final_score
        .cmp(&second_player_stats.final_score)
    {
        Ordering::Greater => Some(first_player_stats),
        Ordering::Less => Some(second_player_stats),
        Ordering::Equal => None,
    };

    let winning_player_str = match winning_player {
        Some(stats) => format!("{} wins!", stats.name),
        None => "It's a tie!".to_string(),
    };

    let winner_text = commands
        .spawn_bundle(
            TextBundle::from_section(
                winning_player_str,
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            )
            .with_style(Style {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            }),
        )
        .id();

    children.push(winner_text);

    commands.entity(root_entity).push_children(&children);
}

fn get_tokens_score(tokens: &Tokens) -> usize {
    let all_goods_tokens_values = tokens.goods.iter().flat_map(|(_, values)| values);
    let all_bonus_tokens_values = tokens.bonus.iter().flat_map(|(_, values)| values);

    all_goods_tokens_values.chain(all_bonus_tokens_values).sum()
}

#[derive(Clone)]
struct PlayerStats {
    name: String,
    final_score: usize,
    camel_bonus_awarded: bool,
}

fn setup_game(mut commands: Commands) {
    let mut deck = Deck::default();
    let market = Market::new(&mut deck);
    let tokens = Tokens::create_game_tokens();

    let player_one_cards = deck.get_cards(5);
    let player_two_cards = deck.get_cards(5);

    let (player_one_num_camels, player_one_goods_hand) = partition_hand(player_one_cards);
    let (player_two_num_camels, player_two_goods_hand) = partition_hand(player_two_cards);

    commands
        .spawn_bundle(PlayerBundle::new(
            "Player 1".to_string(),
            player_one_goods_hand,
            player_one_num_camels,
        ))
        .insert(ActivePlayer);

    commands.spawn_bundle(PlayerBundle::new(
        "Player 2".to_string(),
        player_two_goods_hand,
        player_two_num_camels,
    ));

    commands.insert_resource(deck);
    commands.insert_resource(market);
    commands.insert_resource(tokens);
    commands.init_resource::<DiscardPile>();
}

#[derive(Component)]
struct GameRoot;

#[derive(Component)]
pub struct DeckCard(pub usize);

fn setup_game_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    deck: Res<Deck>,
    market: Res<Market>,
    discard_pile: Res<DiscardPile>,
    active_player_query: Query<(&GoodsHandOwner, &CamelsHandOwner), With<ActivePlayer>>,
    inactive_player_query: Query<(&GoodsHandOwner, &CamelsHandOwner), Without<ActivePlayer>>,
) {
    let game_root_entity = commands
        .spawn_bundle(SpatialBundle::default())
        .insert(GameRoot)
        .id();

    // Render deck
    for (i, card_type) in deck.cards.iter().enumerate() {
        let deck_entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("textures/card/back.png"),
                transform: Transform::default()
                    .with_translation(DECK_START_POS + Vec3::new(i as f32, i as f32, i as f32)),
                ..default()
            })
            .insert(Card(*card_type))
            .insert(DeckCard(i))
            .id();

        commands.entity(game_root_entity).add_child(deck_entity);
    }

    // Render market
    for (idx, market_card) in market.cards.iter().enumerate() {
        let market_entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load(&market_card.get_card_texture()),
                transform: Transform::default().with_translation(get_market_card_translation(idx)),
                ..default()
            })
            .insert(Card(*market_card))
            .insert(MarketCard(idx))
            .id();

        commands.entity(game_root_entity).add_child(market_entity);
    }

    // Render discard pile - only last card need be shown
    if let Some(card_type) = discard_pile.cards.last() {
        let discard_pile_entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load(&card_type.get_card_texture()),
                transform: Transform::default().with_translation(DISCARD_PILE_POS),
                ..default()
            })
            .id();

        commands
            .entity(game_root_entity)
            .add_child(discard_pile_entity);
    }

    let (active_player_goods_hand, active_player_camels_hand) = active_player_query.single();

    // Render active player's goods hand
    for (idx, good) in active_player_goods_hand.0.iter().enumerate() {
        let active_player_goods_hand_entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load(&good.get_card_texture()),
                transform: Transform::default()
                    .with_translation(get_active_player_goods_card_translation(idx)),
                ..default()
            })
            .insert(Card(CardType::Good(*good)))
            .insert(ActivePlayerGoodsCard(idx))
            .id();

        commands
            .entity(game_root_entity)
            .add_child(active_player_goods_hand_entity);
    }

    // Render active player's camel hand
    for idx in 0..active_player_camels_hand.0 {
        let active_player_camels_hand_entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("textures/card/camel.png"),
                transform: Transform::default()
                    .with_translation(get_active_player_camel_card_translation(idx)),
                ..default()
            })
            .insert(Card(CardType::Camel))
            .insert(ActivePlayerCamelCard(idx))
            .id();

        commands
            .entity(game_root_entity)
            .add_child(active_player_camels_hand_entity);
    }

    let (inactive_player_goods_hand, inactive_player_camels_hand) = inactive_player_query.single();

    for idx in 0..inactive_player_goods_hand.0.len() {
        let inactive_player_goods_hand_entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("textures/card/back.png"),
                transform: Transform::default()
                    .with_translation(
                        INACTIVE_PLAYER_GOODS_HAND_START_POS
                            + Vec3::X * idx as f32 * (CARD_DIMENSION.x + CARD_PADDING),
                    )
                    .with_rotation(Quat::from_rotation_z((180.0_f32).to_radians())),
                ..default()
            })
            .id();

        commands
            .entity(game_root_entity)
            .add_child(inactive_player_goods_hand_entity);
    }

    if inactive_player_camels_hand.0 > 0 {
        let inactive_player_camels_hand_entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("textures/card/camel.png"),
                transform: Transform::default()
                    .with_translation(
                        INACTIVE_PLAYER_GOODS_HAND_START_POS
                            - Vec3::Y
                                * (CARD_DIMENSION.y * 0.5 + CARD_DIMENSION.x * 0.5 + CARD_PADDING),
                    )
                    .with_rotation(Quat::from_rotation_z((90.0_f32).to_radians())),

                ..default()
            })
            .id();

        commands
            .entity(game_root_entity)
            .add_child(inactive_player_camels_hand_entity);
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerName(pub String);

#[derive(Component)]
pub struct GoodsHandOwner(pub Vec<GoodType>);

#[derive(Component)]
pub struct CamelsHandOwner(pub usize);

#[derive(Component)]
pub struct TokensOwner(pub Tokens);

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    name: PlayerName,
    goods_hand_owner: GoodsHandOwner,
    camels_hand_owner: CamelsHandOwner,
    tokens_owner: TokensOwner,
}

impl PlayerBundle {
    fn new(name: String, initial_goods_hand: Vec<GoodType>, initial_camels: usize) -> Self {
        Self {
            player: Player {},
            name: PlayerName(name),
            goods_hand_owner: GoodsHandOwner(initial_goods_hand),
            camels_hand_owner: CamelsHandOwner(initial_camels),
            tokens_owner: TokensOwner(Tokens::create_empty()),
        }
    }
}

#[derive(Component)]
pub struct ActivePlayer;

fn partition_hand(hand: Vec<CardType>) -> (usize, Vec<GoodType>) {
    let (camels, goods): (Vec<CardType>, Vec<GoodType>) =
        hand.into_iter().partition_map(|c| match c {
            CardType::Camel => Either::Left(CardType::Camel),
            CardType::Good(good_type) => Either::Right(good_type),
        });

    (camels.len(), goods)
}

fn update_active_player(
    mut commands: Commands,
    active_player_query: Query<Entity, With<ActivePlayer>>,
    inactive_player_query: Query<Entity, (With<Player>, Without<ActivePlayer>)>,
) {
    let active_player_entity = active_player_query.single();
    let inactive_player_entity = inactive_player_query.single();

    commands
        .entity(active_player_entity)
        .remove::<ActivePlayer>();

    commands.entity(inactive_player_entity).insert(ActivePlayer);
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CardSelectionPlugin)
            .add_plugin(MoveValidationPlugin)
            .add_plugin(MoveExecutionPlugin)
            .add_system_set(SystemSet::on_enter(AppState::InitGame).with_system(setup_game))
            .add_system_set(
                SystemSet::on_update(AppState::InitGame).with_system(handle_when_resources_ready),
            )
            .add_system_set(
                SystemSet::on_enter(AppState::TurnTransition)
                    .with_system(setup_turn_transition_screen),
            )
            .add_system_set(
                SystemSet::on_update(AppState::TurnTransition)
                    .with_system(handle_turn_transition_screen_interaction),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::TurnTransition)
                    .with_system(despawn_entity_with_component::<TurnTransitionScreen>),
            )
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_game_screen))
            .add_system_set(
                SystemSet::on_exit(AppState::WaitForTweensToFinish)
                    .with_system(update_active_player)
                    .with_system(despawn_entity_with_component::<GameRoot>),
            )
            .add_system_set(
                SystemSet::on_enter(AppState::GameOver).with_system(setup_game_over_screen),
            );
    }
}
