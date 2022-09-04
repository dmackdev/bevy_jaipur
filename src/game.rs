use bevy::prelude::*;
use bevy_interact_2d::*;
use bevy_prototype_lyon::prelude::ShapePlugin;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::{Animator, EaseFunction, Tween, TweenCompleted, TweeningPlugin, TweeningType};
use enum_map::{enum_map, Enum, EnumMap};
use itertools::{Either, Itertools};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::{Ordering, Reverse};
use std::iter;
use std::time::Duration;

use crate::card_selection::{CardSelectionPlugin, SelectedCard, SelectedCardState};
use crate::common_systems::despawn_entity_with_component;
use crate::event::ConfirmTurnEvent;
use crate::label::Label;
use crate::move_validation::{MoveType, MoveValidationPlugin, MoveValidity};
use crate::resources::{DiscardPile, GameState};
use crate::states::{AppState, TurnState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CardType {
    Camel,
    Good(GoodType),
}

impl CardType {
    fn get_card_texture(&self) -> String {
        match self {
            CardType::Camel => "textures/card/camel.png".to_string(),
            CardType::Good(good) => good.get_card_texture(),
        }
    }

    fn into_good_type(&self) -> GoodType {
        match self {
            CardType::Camel => panic!(),
            CardType::Good(gt) => *gt,
        }
    }
}

#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
pub enum GoodType {
    Diamond,
    Gold,
    Silver,
    Cloth,
    Spice,
    Leather,
}

impl GoodType {
    fn get_card_texture(&self) -> String {
        match self {
            GoodType::Diamond => "textures/card/diamond.png".to_string(),
            GoodType::Gold => "textures/card/gold.png".to_string(),
            GoodType::Silver => "textures/card/silver.png".to_string(),
            GoodType::Cloth => "textures/card/cloth.png".to_string(),
            GoodType::Spice => "textures/card/spice.png".to_string(),
            GoodType::Leather => "textures/card/leather.png".to_string(),
        }
    }

    pub fn is_high_value(&self) -> bool {
        matches!(self, GoodType::Diamond | GoodType::Gold | GoodType::Silver)
    }
}

#[derive(Component, Clone, Debug)]
pub struct Card(pub CardType);

#[derive(Component)]
pub struct MarketCard(usize);

#[derive(Component)]
pub struct ActivePlayerGoodsCard(usize);

#[derive(Component)]
pub struct ActivePlayerCamelCard(usize);

#[derive(Component)]
pub struct CardOutline;

#[derive(Clone)]
pub struct Deck {
    pub cards: Vec<CardType>,
}

const NUM_CAMEL_CARDS: usize = 11;
const NUM_DIAMOND_CARDS: usize = 6;
const NUM_GOLD_CARDS: usize = 6;
const NUM_SILVER_CARDS: usize = 6;
const NUM_CLOTH_CARDS: usize = 8;
const NUM_SPICE_CARDS: usize = 8;
const NUM_LEATHER_CARDS: usize = 10;

impl Default for Deck {
    fn default() -> Self {
        let mut cards = vec![];

        let mut camel_cards = iter::repeat(CardType::Camel)
            .take(NUM_CAMEL_CARDS)
            .collect::<Vec<_>>();

        let mut diamond_cards = iter::repeat(CardType::Good(GoodType::Diamond))
            .take(NUM_DIAMOND_CARDS)
            .collect::<Vec<_>>();

        let mut gold_cards = iter::repeat(CardType::Good(GoodType::Gold))
            .take(NUM_GOLD_CARDS)
            .collect::<Vec<_>>();

        let mut silver_cards = iter::repeat(CardType::Good(GoodType::Silver))
            .take(NUM_SILVER_CARDS)
            .collect::<Vec<_>>();

        let mut cloth_cards = iter::repeat(CardType::Good(GoodType::Cloth))
            .take(NUM_CLOTH_CARDS)
            .collect::<Vec<_>>();

        let mut spice_cards = iter::repeat(CardType::Good(GoodType::Spice))
            .take(NUM_SPICE_CARDS)
            .collect::<Vec<_>>();

        let mut leather_cards = iter::repeat(CardType::Good(GoodType::Leather))
            .take(NUM_LEATHER_CARDS)
            .collect::<Vec<_>>();

        cards.append(&mut camel_cards);
        cards.append(&mut diamond_cards);
        cards.append(&mut gold_cards);
        cards.append(&mut silver_cards);
        cards.append(&mut cloth_cards);
        cards.append(&mut spice_cards);
        cards.append(&mut leather_cards);

        let mut rng = thread_rng();
        cards.shuffle(&mut rng);

        Self { cards }
    }
}

impl Deck {
    pub fn get_cards(&mut self, num_cards: usize) -> Vec<CardType> {
        self.cards.drain(0..num_cards).collect()
    }
}

#[derive(Clone)]
pub struct Market {
    pub cards: Vec<CardType>,
}

impl Market {
    fn new(deck: &mut Deck) -> Self {
        // take 3 camel tokens from deck, and 2 random cards to fill the market
        let mut market_cards = vec![];

        for _ in 0..3 {
            let camel_card_idx = deck
                .cards
                .iter()
                .position(|c| *c == CardType::Camel)
                .unwrap();

            let camel_card = deck.cards.remove(camel_card_idx);
            market_cards.push(camel_card);
        }

        for _ in 0..2 {
            let random_card = deck.cards.remove(0);
            market_cards.push(random_card);
        }

        Self {
            cards: market_cards,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tokens {
    pub goods: EnumMap<GoodType, Vec<usize>>,
    pub bonus: EnumMap<BonusType, Vec<usize>>,
}

#[derive(Clone, Copy, Debug, Enum)]
pub enum BonusType {
    Three,
    Four,
    Five,
}

impl Tokens {
    fn create_game_tokens() -> Self {
        let goods = enum_map! {
          GoodType::Diamond => vec![5,5,5,7,7,],
          GoodType::Gold => vec![5,5,5,6,6,],
          GoodType::Silver => vec![5,5,5,5,5,],
          GoodType::Cloth => vec![1,1,2,2,3,3,5,],
          GoodType::Spice => vec![1,1,2,2,3,3,5,],
          GoodType::Leather => vec![1,1,1,1,1,1,2,3,4,],
        };

        let mut rng = thread_rng();

        let mut three_bonuses = vec![3, 3, 2, 2, 2, 1, 1];
        three_bonuses.shuffle(&mut rng);

        let mut four_bonuses = vec![6, 6, 5, 5, 4, 4];
        four_bonuses.shuffle(&mut rng);

        let mut five_bonuses = vec![10, 10, 9, 8, 8];
        five_bonuses.shuffle(&mut rng);

        let bonus = enum_map! {
          BonusType::Three => three_bonuses.clone(),
          BonusType::Four => four_bonuses.clone(),
          BonusType::Five => five_bonuses.clone(),
        };

        Self { goods, bonus }
    }

    fn create_empty() -> Self {
        Self {
            goods: enum_map! {
              _ => vec![]
            },
            bonus: enum_map! {
              _ => vec![]
            },
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_when_resources_ready(
    mut state: ResMut<State<AppState>>,
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

    if resources_are_ready {
        state.set(AppState::TurnTransition).unwrap();
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

const DECK_START_POS: Vec3 = Vec3::new(300.0, 0.0, 0.0);
const DISCARD_PILE_POS: Vec3 = Vec3::new(
    DECK_START_POS.x + 1.5 * CARD_DIMENSION.x + CARD_PADDING,
    DECK_START_POS.y,
    0.,
);
const CARD_DIMENSION: Vec2 = Vec2::new(104.0, 150.0);
const GOODS_HAND_START_POS: Vec3 = Vec3::new(-5.0 * 0.5 * CARD_DIMENSION.x, -400.0, 0.0);
const CAMEL_HAND_START_POS: Vec3 = Vec3::new(
    GOODS_HAND_START_POS.x,
    GOODS_HAND_START_POS.y + CARD_DIMENSION.y + CARD_PADDING,
    0.0,
);
const INACTIVE_PLAYER_GOODS_HAND_START_POS: Vec3 = Vec3::new(
    GOODS_HAND_START_POS.x,
    GOODS_HAND_START_POS.y * -1.0,
    GOODS_HAND_START_POS.z,
);
const CARD_PADDING: f32 = 20.0;

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
    commands.init_resource::<TweenState>();
}

#[derive(Component)]
struct GameRoot;

#[derive(Component)]
struct DeckCard(usize);

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

#[allow(clippy::too_many_arguments)]
fn handle_take_single_good_move_confirmed(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_confirm_turn: EventReader<ConfirmTurnEvent>,
    mut deck: ResMut<Deck>,
    mut market: ResMut<Market>,
    selected_market_cards_query: Query<
        (Entity, &Card, &MarketCard, &Transform),
        With<SelectedCard>,
    >,
    mut active_player_query: Query<&mut GoodsHandOwner, With<ActivePlayer>>,
    mut deck_cards_query: Query<(Entity, &DeckCard, &Card, &Transform, &mut Handle<Image>)>,
    mut tween_state: ResMut<TweenState>,
    mut game_state: ResMut<GameState>,
) {
    for _ev in ev_confirm_turn
        .iter()
        .filter(|ev| matches!(ev.0, MoveType::TakeSingleGood))
    {
        let mut active_player_goods_hand = active_player_query.single_mut();

        let (card_entity, card, market_card, transform) = selected_market_cards_query.single();

        let good = card.0.into_good_type();

        // Remove from market resource
        market.cards.remove(market_card.0);

        // add to active player goods hand
        active_player_goods_hand.0.push(good);

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once,
            Duration::from_secs(2),
            TransformPositionLens {
                start: transform.translation,
                end: get_active_player_goods_card_translation(active_player_goods_hand.0.len() - 1),
            },
        )
        .with_completed_event(1);

        tween_state.tweening_entities.push(card_entity);

        commands
            .entity(card_entity)
            .insert(Animator::new(tween))
            .remove::<MarketCard>()
            .insert(ActivePlayerGoodsCard(active_player_goods_hand.0.len() - 1));

        // Replace with card from deck
        if let Some(replacement_card) = deck.cards.pop() {
            market.cards.insert(market_card.0, replacement_card);

            let (deck_card_entity, _, card, deck_card_transform, mut top_deck_card_texture) =
                deck_cards_query
                    .iter_mut()
                    .max_by_key(|(_, dc, _, _, _)| dc.0)
                    .unwrap();

            // Update the sprite to show the face
            *top_deck_card_texture = asset_server.load(&card.0.get_card_texture());

            // Tween to the market card position
            let second_tween = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: deck_card_transform.translation,
                    end: get_market_card_translation(market_card.0),
                },
            )
            .with_completed_event(2);

            tween_state.tweening_entities.push(deck_card_entity);

            commands
                .entity(deck_card_entity)
                .insert(Animator::new(second_tween))
                .remove::<DeckCard>()
                .insert(MarketCard(market_card.0));
        } else {
            game_state.is_game_over = true;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_take_all_camels_move_confirmed(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_confirm_turn: EventReader<ConfirmTurnEvent>,
    mut deck: ResMut<Deck>,
    mut market: ResMut<Market>,
    selected_camel_market_cards_query: Query<(Entity, &MarketCard, &Transform), With<SelectedCard>>,
    mut active_player_query: Query<&mut CamelsHandOwner, With<ActivePlayer>>,
    mut deck_cards_query: Query<(Entity, &DeckCard, &Card, &Transform, &mut Handle<Image>)>,
    mut tween_state: ResMut<TweenState>,
    mut game_state: ResMut<GameState>,
) {
    for _ev in ev_confirm_turn
        .iter()
        .filter(|ev| matches!(ev.0, MoveType::TakeAllCamels))
    {
        let mut active_player_camel_hand = active_player_query.single_mut();

        for (idx, (card_entity, market_card, transform)) in
            selected_camel_market_cards_query.iter().enumerate()
        {
            // Remove from market resource
            market.cards.remove(market_card.0);

            // add to active player camel hand
            active_player_camel_hand.0 += 1;

            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: transform.translation,
                    end: get_active_player_camel_card_translation(active_player_camel_hand.0 - 1),
                },
            )
            .with_completed_event(1);

            tween_state.tweening_entities.push(card_entity);

            commands
                .entity(card_entity)
                .insert(Animator::new(tween))
                .remove::<MarketCard>()
                .insert(ActivePlayerCamelCard(active_player_camel_hand.0 - 1));

            // Replace with card from deck
            if let Some(replacement_card) = deck.cards.pop() {
                market.cards.insert(market_card.0, replacement_card);

                let l = deck_cards_query.iter().len();
                // Get the top deck card - with the highest index
                let (deck_card_entity, _, card, deck_card_transform, mut top_deck_card_texture) =
                    deck_cards_query
                        .iter_mut()
                        .sorted_by_key(|(_, dc, _, _, _)| dc.0)
                        .nth(l - 1 - idx)
                        .unwrap();

                // Update the sprite to show the face
                *top_deck_card_texture = asset_server.load(&card.0.get_card_texture());

                // Tween to the market card position
                let second_tween = Tween::new(
                    EaseFunction::QuadraticInOut,
                    TweeningType::Once,
                    Duration::from_secs(2),
                    TransformPositionLens {
                        start: deck_card_transform.translation,
                        end: get_market_card_translation(market_card.0),
                    },
                )
                .with_completed_event(2);

                tween_state.tweening_entities.push(deck_card_entity);

                commands
                    .entity(deck_card_entity)
                    .insert(Animator::new(second_tween))
                    .remove::<DeckCard>()
                    .insert(MarketCard(market_card.0));
            } else {
                game_state.is_game_over = true;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_exchange_goods_move_confirmed(
    mut commands: Commands,
    mut ev_confirm_turn: EventReader<ConfirmTurnEvent>,
    mut market: ResMut<Market>,
    mut active_player_query: Query<(&mut GoodsHandOwner, &mut CamelsHandOwner), With<ActivePlayer>>,
    active_player_selected_camel_cards: Query<
        (Entity, &Transform),
        (With<ActivePlayerCamelCard>, With<SelectedCard>),
    >,
    active_player_selected_goods_card: Query<
        (Entity, &ActivePlayerGoodsCard, &Transform),
        With<SelectedCard>,
    >,
    selected_market_goods_cards_query: Query<(Entity, &MarketCard, &Transform), With<SelectedCard>>,
    mut tween_state: ResMut<TweenState>,
) {
    for _ev in ev_confirm_turn
        .iter()
        .filter(|ev| matches!(ev.0, MoveType::ExchangeForGoodsFromMarket))
    {
        let (mut goods_hand_owner, mut camels_hand_owner) = active_player_query.single_mut();

        // num player's selected goods <= selected market goods, since camels may fill the remainder
        for (player_good, market_good) in active_player_selected_goods_card
            .iter()
            .zip(selected_market_goods_cards_query.iter())
        {
            let good_type_removed_from_hand = goods_hand_owner.0.remove(player_good.1 .0);
            let good_type_removed_from_market = market.cards.remove(market_good.1 .0);

            goods_hand_owner.0.insert(
                player_good.1 .0,
                good_type_removed_from_market.into_good_type(),
            );
            market.cards.insert(
                market_good.1 .0,
                CardType::Good(good_type_removed_from_hand),
            );

            let tween_hand_to_market = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: player_good.2.translation,
                    end: get_market_card_translation(market_good.1 .0),
                },
            )
            .with_completed_event(1);

            tween_state.tweening_entities.push(player_good.0);

            commands
                .entity(player_good.0)
                .insert(Animator::new(tween_hand_to_market))
                .remove::<ActivePlayerGoodsCard>()
                .insert(MarketCard(market_good.1 .0));

            let tween_market_to_hand = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: market_good.2.translation,
                    end: get_active_player_goods_card_translation(player_good.1 .0),
                },
            )
            .with_completed_event(2);

            tween_state.tweening_entities.push(market_good.0);

            commands
                .entity(market_good.0)
                .insert(Animator::new(tween_market_to_hand))
                .remove::<MarketCard>()
                .insert(ActivePlayerGoodsCard(player_good.1 .0));
        }

        for (camel, market_good) in active_player_selected_camel_cards.iter().zip(
            selected_market_goods_cards_query
                .iter()
                .skip(active_player_selected_goods_card.iter().count()),
        ) {
            camels_hand_owner.0 -= 1;
            let good_type_removed_from_market = market.cards.remove(market_good.1 .0);

            goods_hand_owner
                .0
                .push(good_type_removed_from_market.into_good_type());
            market.cards.insert(market_good.1 .0, CardType::Camel);

            let tween_camel_to_market = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: camel.1.translation,
                    end: get_market_card_translation(market_good.1 .0),
                },
            )
            .with_completed_event(3);

            tween_state.tweening_entities.push(camel.0);

            commands
                .entity(camel.0)
                .insert(Animator::new(tween_camel_to_market))
                .remove::<ActivePlayerCamelCard>()
                .insert(MarketCard(market_good.1 .0));

            let tween_market_to_hand = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: market_good.2.translation,
                    end: get_active_player_goods_card_translation(goods_hand_owner.0.len() - 1),
                },
            )
            .with_completed_event(4);

            tween_state.tweening_entities.push(market_good.0);

            commands
                .entity(market_good.0)
                .insert(Animator::new(tween_market_to_hand))
                .remove::<MarketCard>()
                .insert(ActivePlayerGoodsCard(goods_hand_owner.0.len() - 1));
        }
    }
}

fn handle_sell_goods_move_confirmed(
    mut commands: Commands,
    mut ev_confirm_turn: EventReader<ConfirmTurnEvent>,
    mut discard_pile: ResMut<DiscardPile>,
    mut tween_state: ResMut<TweenState>,
    mut game_tokens: ResMut<Tokens>,
    active_player_selected_goods_card: Query<
        (Entity, &ActivePlayerGoodsCard, &Transform),
        With<SelectedCard>,
    >,
    mut active_player_query: Query<(&mut GoodsHandOwner, &mut TokensOwner), With<ActivePlayer>>,
    mut game_state: ResMut<GameState>,
) {
    for _ev in ev_confirm_turn
        .iter()
        .filter(|ev| matches!(ev.0, MoveType::SellGoods))
    {
        let (mut goods_hand_owner, mut tokens_owner) = active_player_query.single_mut();

        for (e, active_player_goods_card, transform) in active_player_selected_goods_card
            .iter()
            // sort in desc order to avoid shifting the indices for subsequent loop iterations after having removed a card from the hand
            .sorted_by_key(|(_, active_player_goods_card, _)| Reverse(active_player_goods_card.0))
        {
            let sold_card = goods_hand_owner.0.remove(active_player_goods_card.0);

            discard_pile.cards.push(CardType::Good(sold_card));

            let tween_goods_hand_to_discard_pile = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: transform.translation,
                    end: DISCARD_PILE_POS,
                },
            )
            .with_completed_event(4);

            tween_state.tweening_entities.push(e);

            commands
                .entity(e)
                .insert(Animator::new(tween_goods_hand_to_discard_pile))
                .remove::<ActivePlayerGoodsCard>();

            let next_goods_token = game_tokens.goods[sold_card].pop();

            if let Some(val) = next_goods_token {
                tokens_owner.0.goods[sold_card].push(val);
            }
        }
        let num_cards_sold = active_player_selected_goods_card.iter().count();
        let bonus_type = match num_cards_sold {
            d if d == 3 => Some(BonusType::Three),
            d if d == 4 => Some(BonusType::Four),
            d if 5 <= d => Some(BonusType::Five),
            _ => None,
        };

        if let Some(bt) = bonus_type {
            if let Some(val) = game_tokens.bonus[bt].pop() {
                tokens_owner.0.bonus[bt].push(val);
            }
        }

        if game_tokens
            .goods
            .iter()
            .filter(|(_, token_values)| token_values.is_empty())
            .count()
            >= 3
        {
            game_state.is_game_over = true;
        }
    }
}

fn handle_confirm_turn_event(
    mut state: ResMut<State<AppState>>,
    mut ev_confirm_turn: EventReader<ConfirmTurnEvent>,
) {
    for _ev in ev_confirm_turn.iter() {
        state.set(AppState::WaitForTweensToFinish).unwrap();
    }
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

fn setup_for_take_action(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<MarketCard>,
            With<ActivePlayerGoodsCard>,
            With<ActivePlayerCamelCard>,
        )>,
    >,
) {
    // TODO: remove other Interactables, only add to cards that can be interacted with for Take

    for entity in query.iter() {
        commands.entity(entity).insert(Interactable {
            groups: vec![Group(0)],
            bounding_box: (-0.5 * CARD_DIMENSION, 0.5 * CARD_DIMENSION),
        });
    }
}

fn setup_for_sell_action(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<MarketCard>,
            With<ActivePlayerGoodsCard>,
            With<ActivePlayerCamelCard>,
        )>,
    >,
) {
    // TODO: remove other Interactables, only add to cards that can be interacted with for Sell
    for entity in query.iter() {
        commands.entity(entity).insert(Interactable {
            groups: vec![Group(0)],
            bounding_box: (-0.5 * CARD_DIMENSION, 0.5 * CARD_DIMENSION),
        });
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScreenTransitionDelayTimer(Timer::from_seconds(2.0, true)))
            .init_resource::<GameState>()
            .add_plugin(InteractionPlugin)
            .add_plugin(ShapePlugin)
            .add_plugin(TweeningPlugin)
            .add_plugin(CardSelectionPlugin)
            .add_plugin(MoveValidationPlugin)
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
                SystemSet::on_update(AppState::InGame)
                    .with_system(
                        handle_take_single_good_move_confirmed
                            .label(Label::EventReader)
                            .before(handle_confirm_turn_event)
                            .after(Label::EventWriter),
                    )
                    .with_system(
                        handle_take_all_camels_move_confirmed
                            .label(Label::EventReader)
                            .before(handle_confirm_turn_event)
                            .after(Label::EventWriter),
                    )
                    .with_system(
                        handle_exchange_goods_move_confirmed
                            .label(Label::EventReader)
                            .before(handle_confirm_turn_event)
                            .after(Label::EventWriter),
                    )
                    .with_system(
                        handle_sell_goods_move_confirmed
                            .label(Label::EventReader)
                            .before(handle_confirm_turn_event)
                            .after(Label::EventWriter),
                    )
                    .with_system(
                        handle_confirm_turn_event
                            .label(Label::EventReader)
                            .after(Label::EventWriter),
                    ),
            )
            .add_system_set(
                SystemSet::on_update(AppState::WaitForTweensToFinish)
                    .with_system(wait_for_tweens_to_finish),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::WaitForTweensToFinish)
                    .with_system(update_active_player)
                    .with_system(despawn_entity_with_component::<GameRoot>),
            )
            .add_system_set(
                SystemSet::on_enter(AppState::GameOver).with_system(setup_game_over_screen),
            )
            .add_system_set(SystemSet::on_enter(TurnState::Take).with_system(setup_for_take_action))
            .add_system_set(
                SystemSet::on_enter(TurnState::Sell).with_system(setup_for_sell_action),
            );
    }
}

fn get_active_player_goods_card_translation(idx: usize) -> Vec3 {
    GOODS_HAND_START_POS + Vec3::X * idx as f32 * (CARD_DIMENSION.x + CARD_PADDING)
}

fn get_market_card_translation(idx: usize) -> Vec3 {
    DECK_START_POS
        - (5 - idx) as f32 * CARD_DIMENSION.x * Vec3::X
        - (5 - idx) as f32 * CARD_PADDING * Vec3::X
}

fn get_active_player_camel_card_translation(idx: usize) -> Vec3 {
    CAMEL_HAND_START_POS + Vec3::X * idx as f32 * (CARD_DIMENSION.x + CARD_PADDING)
}

#[derive(Default)]
struct TweenState {
    tweening_entities: Vec<Entity>,
    // Distinguishes between there never being any tweening entities in the first place, and actual tweens that started completing
    did_all_tweens_complete: bool,
}

struct ScreenTransitionDelayTimer(Timer);

fn wait_for_tweens_to_finish(
    mut ev_tween_completed: EventReader<TweenCompleted>,
    mut tween_state: ResMut<TweenState>,
    mut app_state: ResMut<State<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<ScreenTransitionDelayTimer>,
    game_state: Res<GameState>,
) {
    for ev in ev_tween_completed.iter() {
        let index = tween_state
            .tweening_entities
            .iter()
            .position(|e| *e == ev.entity)
            .unwrap();
        tween_state.tweening_entities.remove(index);

        if tween_state.tweening_entities.is_empty() {
            tween_state.did_all_tweens_complete = true;
        }
    }

    if tween_state.did_all_tweens_complete && timer.0.tick(time.delta()).just_finished() {
        tween_state.did_all_tweens_complete = false;

        if game_state.is_game_over {
            app_state.set(AppState::GameOver).unwrap();
        } else {
            app_state.set(AppState::TurnTransition).unwrap();
        }
    }
}
