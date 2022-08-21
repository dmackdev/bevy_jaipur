use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::iter;

fn main() {
    App::new()
        .init_resource::<Deck>()
        .init_resource::<Market>()
        .add_startup_system(debug_market)
        .add_startup_system(debug_deck.after(debug_market))
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

#[derive(Clone, Debug)]
enum CardType {
    Camel,
    Diamond,
    Gold,
    Silver,
    Cloth,
    Spice,
    Leather,
}

#[derive(Component, Clone, Debug)]
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

impl FromWorld for Deck {
    fn from_world(_world: &mut World) -> Self {
        let mut cards = vec![];

        let mut camel_cards = iter::repeat(Card(CardType::Camel))
            .take(NUM_CAMEL_CARDS)
            .collect::<Vec<_>>();

        let mut diamond_cards = iter::repeat(Card(CardType::Diamond))
            .take(NUM_DIAMOND_CARDS)
            .collect::<Vec<_>>();

        let mut gold_cards = iter::repeat(Card(CardType::Gold))
            .take(NUM_GOLD_CARDS)
            .collect::<Vec<_>>();

        let mut silver_cards = iter::repeat(Card(CardType::Silver))
            .take(NUM_SILVER_CARDS)
            .collect::<Vec<_>>();

        let mut cloth_cards = iter::repeat(Card(CardType::Cloth))
            .take(NUM_CLOTH_CARDS)
            .collect::<Vec<_>>();

        let mut spice_cards = iter::repeat(Card(CardType::Spice))
            .take(NUM_SPICE_CARDS)
            .collect::<Vec<_>>();

        let mut leather_cards = iter::repeat(Card(CardType::Leather))
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
