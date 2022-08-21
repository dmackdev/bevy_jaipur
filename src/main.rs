use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::iter;

fn main() {
    App::new()
        .init_resource::<Deck>()
        .add_startup_system(debug_deck)
        .run();
}

fn debug_deck(deck: Res<Deck>) {
    for card in deck.cards.iter() {
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
