use std::iter;

use rand::{seq::SliceRandom, thread_rng};

use super::card::{CardType, GoodType};

const NUM_CAMEL_CARDS: usize = 11;
const NUM_DIAMOND_CARDS: usize = 6;
const NUM_GOLD_CARDS: usize = 6;
const NUM_SILVER_CARDS: usize = 6;
const NUM_CLOTH_CARDS: usize = 8;
const NUM_SPICE_CARDS: usize = 8;
const NUM_LEATHER_CARDS: usize = 10;

#[derive(Clone)]
pub struct Deck {
    pub cards: Vec<CardType>,
}

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
