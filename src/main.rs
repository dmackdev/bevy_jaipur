use bevy::prelude::*;
use enum_map::{enum_map, Enum, EnumMap};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::iter;

fn main() {
    App::new()
        .init_resource::<Deck>()
        .init_resource::<Market>()
        .init_resource::<Tokens>()
        .add_startup_system(debug_market)
        .add_startup_system(debug_deck.after(debug_market))
        .add_startup_system(debug_tokens.after(debug_deck))
        .add_startup_system(add_players)
        .add_system(debug_players)
        .run();
}

fn debug_deck(deck: Res<Deck>) {
    println!("Deck:");
    for card in deck.cards.iter() {
        println!("{:?}", card);
    }
}

fn debug_market(market: Res<Market>) {
    println!("Market:");
    for card in market.cards.iter() {
        println!("{:?}", card);
    }
}

fn debug_tokens(tokens: Res<Tokens>) {
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
    query: Query<(&Name, &GoodsHandOwner, &CamelsHandOwner, &TokensOwner), With<Player>>,
) {
    println!("Players:");
    for (name, goods_hand, camels_hand, tokens) in query.iter() {
        println!("Name: {:?}", name.0);
        println!("Goods hand: {:?}", goods_hand.0);
        println!("Camels: {:?}", camels_hand.0);
        println!("Tokens: {:?}\n", tokens.0);
    }
}

#[derive(Clone, Debug)]
enum CardType {
    Camel,
    Good(GoodType),
}

#[derive(Clone, Debug, Enum)]
enum GoodType {
    Diamond,
    Gold,
    Silver,
    Cloth,
    Spice,
    Leather,
}

#[derive(Clone, Debug)]
struct Card(CardType);

struct Deck {
    cards: Vec<Card>,
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

struct Market {
    cards: Vec<Card>,
}

impl FromWorld for Market {
    fn from_world(world: &mut World) -> Self {
        // take 3 camel tokens from deck, and 2 random cards to fill the market
        let mut market_cards = vec![];

        let mut deck = world.get_resource_mut::<Deck>().unwrap();

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

#[derive(Debug)]
struct Tokens {
    goods: EnumMap<GoodType, Vec<usize>>,
    bonus: EnumMap<BonusType, Vec<usize>>,
}

#[derive(Clone, Debug, Enum)]
enum BonusType {
    Three,
    Four,
    Five,
}

impl Default for Tokens {
    fn default() -> Self {
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
}

impl Tokens {
    fn empty() -> Self {
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

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct GoodsHandOwner(Vec<GoodType>);

#[derive(Component)]
struct CamelsHandOwner(usize);

#[derive(Component)]
struct TokensOwner(Tokens);

fn add_players(mut commands: Commands) {
    commands
        .spawn()
        .insert(Player)
        .insert(Name("Player 1".to_string()))
        .insert(GoodsHandOwner(vec![]))
        .insert(CamelsHandOwner(0))
        .insert(TokensOwner(Tokens::empty()));

    commands
        .spawn()
        .insert(Player)
        .insert(Name("Player 2".to_string()))
        .insert(GoodsHandOwner(vec![]))
        .insert(CamelsHandOwner(0))
        .insert(TokensOwner(Tokens::empty()));
}
