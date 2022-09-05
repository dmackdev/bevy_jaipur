use super::card::CardType;

#[derive(Default)]
pub struct DiscardPile {
    pub cards: Vec<CardType>,
}
