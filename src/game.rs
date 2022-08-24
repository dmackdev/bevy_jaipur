use bevy::prelude::*;
use enum_map::{enum_map, Enum, EnumMap};
use itertools::{Either, Itertools};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::iter;

use crate::common_systems::despawn_entity_with_component;
use crate::event::ConfirmTurnEvent;
use crate::interaction::InteractionPlugin;
use crate::states::AppState;

#[derive(Clone, Debug, Eq, PartialEq)]
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
}

#[derive(Component, Clone, Debug)]
pub struct Card(pub CardType);

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

#[derive(Clone, Debug, Enum)]
pub enum BonusType {
    Three,
    Four,
    Five,
}

impl Tokens {
    fn create_game_tokens() -> Self {
        let goods = enum_map! {
          GoodType::Diamond => vec![7,7,5,5,5],
          GoodType::Gold => vec![6,6,5,5,5],
          GoodType::Silver => vec![5,5,5,5,5],
          GoodType::Cloth => vec![5,3,3,2,2,1,1],
          GoodType::Spice => vec![5,3,3,2,2,1,1],
          GoodType::Leather => vec![4,3,2,1,1,1,1,1,1],
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

fn handle_when_resources_ready(
    mut state: ResMut<State<AppState>>,
    deck: Option<Res<Deck>>,
    market: Option<Res<Market>>,
    tokens: Option<Res<Tokens>>,
) {
    let resources_are_ready = deck.is_some() && market.is_some() && tokens.is_some();

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

const DECK_START_POS: Vec3 = Vec3::new(300.0, 0.0, 0.0);
const CARD_DIMENSION: Vec2 = Vec2::new(104.0, 150.0);
const GOODS_HAND_START_POS: Vec3 = Vec3::new(-5.0 * 0.5 * CARD_DIMENSION.x, -400.0, 0.0);
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

    debug_deck(deck.clone());
    debug_market(market.clone());
    debug_tokens(tokens.clone());

    commands.insert_resource(deck);
    commands.insert_resource(market);
    commands.insert_resource(tokens);
}

#[derive(Component)]
struct GameRoot;

fn setup_game_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    deck: Res<Deck>,
    market: Res<Market>,
    active_player_query: Query<(&GoodsHandOwner, &CamelsHandOwner), With<ActivePlayer>>,
    inactive_player_query: Query<(&GoodsHandOwner, &CamelsHandOwner), Without<ActivePlayer>>,
) {
    let game_root_entity = commands
        .spawn_bundle(SpatialBundle::default())
        .insert(GameRoot)
        .id();

    // Render deck
    for i in 0..deck.cards.len() {
        let deck_entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("textures/card/back.png"),
                transform: Transform::default()
                    .with_translation(DECK_START_POS + Vec3::new(i as f32, i as f32, i as f32)),
                ..default()
            })
            .id();

        commands.entity(game_root_entity).add_child(deck_entity);
    }

    // Render market
    for (idx, market_card) in market.cards.iter().enumerate() {
        let market_entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load(&market_card.get_card_texture()),
                transform: Transform::default().with_translation(
                    DECK_START_POS
                        - (5 - idx) as f32 * CARD_DIMENSION.x * Vec3::X
                        - (5 - idx) as f32 * CARD_PADDING * Vec3::X,
                ),
                ..default()
            })
            .id();

        commands.entity(game_root_entity).add_child(market_entity);
    }

    let (active_player_goods_hand, active_player_camels_hand) = active_player_query.single();

    // Render active player's goods hand
    for (idx, good) in active_player_goods_hand.0.iter().enumerate() {
        let active_player_goods_hand_entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load(&good.get_card_texture()),
                transform: Transform::default().with_translation(
                    GOODS_HAND_START_POS + Vec3::X * idx as f32 * (CARD_DIMENSION.x + CARD_PADDING),
                ),
                ..default()
            })
            .id();

        commands
            .entity(game_root_entity)
            .add_child(active_player_goods_hand_entity);
    }

    // Render single camel card if active player has at least one - a player need not reveal how many camel cards they have
    if active_player_camels_hand.0 > 0 {
        let active_player_camels_hand_entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("textures/card/camel.png"),
                transform: Transform::default()
                    .with_translation(
                        GOODS_HAND_START_POS
                            + Vec3::Y
                                * (CARD_DIMENSION.y * 0.5 + CARD_DIMENSION.x * 0.5 + CARD_PADDING),
                    )
                    .with_rotation(Quat::from_rotation_z((90.0_f32).to_radians())),
                ..default()
            })
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

fn debug_deck(deck: Deck) {
    println!("Deck:");
    for card in deck.cards.iter() {
        println!("{:?}", card);
    }
}

fn debug_market(market: Market) {
    println!("Market:");
    for card in market.cards.iter() {
        println!("{:?}", card);
    }
}

fn debug_tokens(tokens: Tokens) {
    println!("Tokens:");

    println!("Goods:");
    for (good, tks) in &tokens.goods {
        println!("{:?} => {:?}", good, tks);
    }

    println!("Bonus:");
    for (bonus, tks) in &tokens.bonus {
        println!("{:?} => {:?}", bonus, tks);
    }
}

fn debug_players(
    query: Query<(&PlayerName, &GoodsHandOwner, &CamelsHandOwner, &TokensOwner), With<Player>>,
) {
    println!("Players:");
    for (name, goods_hand, camels_hand, tokens) in query.iter() {
        println!("Name: {:?}", name.0);
        println!("Goods hand: {:?}", goods_hand.0);
        println!("Camels: {:?}", camels_hand.0);
        println!("Tokens: {:?}\n", tokens.0);
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
struct ActivePlayer;

fn partition_hand(hand: Vec<CardType>) -> (usize, Vec<GoodType>) {
    let (camels, goods): (Vec<CardType>, Vec<GoodType>) =
        hand.into_iter().partition_map(|c| match c {
            CardType::Camel => Either::Left(CardType::Camel),
            CardType::Good(good_type) => Either::Right(good_type),
        });

    (camels.len(), goods)
}

fn handle_confirm_turn_event(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    mut ev_confirm_turn: EventReader<ConfirmTurnEvent>,
    active_player_query: Query<Entity, With<ActivePlayer>>,
    inactive_player_query: Query<Entity, (With<Player>, Without<ActivePlayer>)>,
) {
    for _ev in ev_confirm_turn.iter() {
        let active_player_entity = active_player_query.single();
        let inactive_player_entity = inactive_player_query.single();

        commands
            .entity(active_player_entity)
            .remove::<ActivePlayer>();

        commands.entity(inactive_player_entity).insert(ActivePlayer);

        state.set(AppState::TurnTransition).unwrap();
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InteractionPlugin)
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
                SystemSet::on_update(AppState::InGame).with_system(handle_confirm_turn_event),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::InGame)
                    .with_system(despawn_entity_with_component::<GameRoot>),
            );
    }
}
