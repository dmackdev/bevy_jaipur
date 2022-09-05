use crate::game_resources::card::CardType;

#[derive(Default)]
pub struct DiscardPile {
    pub cards: Vec<CardType>,
}

#[derive(Default)]
pub struct GameState {
    pub is_game_over: bool,
}
