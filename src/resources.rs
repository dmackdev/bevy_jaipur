use crate::game::CardType;

#[derive(Default, Eq, PartialEq)]
pub enum MoveValidity {
    #[default]
    Invalid,
    Valid(MoveType),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MoveType {
    TakeSingleGood,
    TakeAllCamels,
    ExchangeForGoodsFromMarket,
    SellGoods,
}

#[derive(Default)]
pub struct DiscardPile {
    pub cards: Vec<CardType>,
}

#[derive(Default)]
pub struct GameState {
    pub is_game_over: bool,
}
