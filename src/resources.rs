use bevy::prelude::Entity;

#[derive(Default)]
pub struct SelectedCardState(pub Vec<Entity>);

#[derive(Default, Eq, PartialEq)]
pub enum MoveValidity {
    #[default]
    Invalid,
    Valid(MoveType),
}

#[derive(Eq, PartialEq)]
pub enum MoveType {
    TakeSingleGood,
    TakeAllCamels,
    ExchangeForGoodsFromMarket,
    SellGoods,
}
