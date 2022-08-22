use bevy::prelude::*;
use enum_map::{enum_map, Enum, EnumMap};
use itertools::{Either, Itertools};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::iter;

use crate::app_state::AppState;

#[derive(Clone, Debug)]
pub enum CardType {
    Camel,
    Good(GoodType),
}

#[derive(Clone, Debug, Enum)]
pub enum GoodType {
    Diamond,
    Gold,
    Silver,
    Cloth,
    Spice,
    Leather,
}

#[derive(Clone, Debug)]
pub struct Card(pub CardType);

#[derive(Clone)]
pub struct Deck {
    pub cards: Vec<Card>,
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

        let mut camel_cards = iter::repeat(Card(CardType::Camel))
            .take(NUM_CAMEL_CARDS)
            .collect::<Vec<_>>();

        let mut diamond_cards = iter::repeat(Card(CardType::Good(GoodType::Diamond)))
            .take(NUM_DIAMOND_CARDS)
            .collect::<Vec<_>>();

        let mut gold_cards = iter::repeat(Card(CardType::Good(GoodType::Gold)))
            .take(NUM_GOLD_CARDS)
            .collect::<Vec<_>>();

        let mut silver_cards = iter::repeat(Card(CardType::Good(GoodType::Silver)))
            .take(NUM_SILVER_CARDS)
            .collect::<Vec<_>>();

        let mut cloth_cards = iter::repeat(Card(CardType::Good(GoodType::Cloth)))
            .take(NUM_CLOTH_CARDS)
            .collect::<Vec<_>>();

        let mut spice_cards = iter::repeat(Card(CardType::Good(GoodType::Spice)))
            .take(NUM_SPICE_CARDS)
            .collect::<Vec<_>>();

        let mut leather_cards = iter::repeat(Card(CardType::Good(GoodType::Leather)))
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
    pub fn get_cards(&mut self, num_cards: usize) -> Vec<Card> {
        self.cards.drain(0..num_cards).collect()
    }
}

#[derive(Clone)]
pub struct Market {
    pub cards: Vec<Card>,
}

impl Market {
    fn new(deck: &mut Deck) -> Self {
        // take 3 camel tokens from deck, and 2 random cards to fill the market
        let mut market_cards = vec![];

        for _ in 0..3 {
            let camel_card_idx = deck
                .cards
                .iter()
                .position(|c| matches!(c.0, CardType::Camel))
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

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_game));
        app.add_system_set(SystemSet::on_update(AppState::InGame).with_system(debug_players));
    }
}

fn setup_game(mut commands: Commands) {
    let mut deck = Deck::default();
    let market = Market::new(&mut deck);
    let tokens = Tokens::create_game_tokens();

    commands.spawn_bundle(PlayerBundle::new("Player 1".to_string(), deck.get_cards(5)));
    commands.spawn_bundle(PlayerBundle::new("Player 2".to_string(), deck.get_cards(5)));

    debug_deck(deck.clone());
    debug_market(market.clone());
    debug_tokens(tokens.clone());

    commands.insert_resource(deck);
    commands.insert_resource(market);
    commands.insert_resource(tokens);
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
    fn new(name: String, initial_cards: Vec<Card>) -> Self {
        let (camels, goods): (Vec<CardType>, Vec<GoodType>) =
            initial_cards.into_iter().partition_map(|c| match c.0 {
                CardType::Camel => Either::Left(CardType::Camel),
                CardType::Good(good_type) => Either::Right(good_type),
            });

        Self {
            player: Player {},
            name: PlayerName(name),
            goods_hand_owner: GoodsHandOwner(goods),
            camels_hand_owner: CamelsHandOwner(camels.len()),
            tokens_owner: TokensOwner(Tokens::create_empty()),
        }
    }
}
