use bevy::prelude::Component;
use enum_map::Enum;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CardType {
    Camel,
    Good(GoodType),
}

impl CardType {
    pub fn get_card_texture(&self) -> String {
        match self {
            CardType::Camel => "textures/card/camel.png".to_string(),
            CardType::Good(good) => good.get_card_texture(),
        }
    }

    pub fn into_good_type(&self) -> GoodType {
        match self {
            CardType::Camel => panic!(),
            CardType::Good(gt) => *gt,
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
    pub fn get_card_texture(&self) -> String {
        match self {
            GoodType::Diamond => "textures/card/diamond.png".to_string(),
            GoodType::Gold => "textures/card/gold.png".to_string(),
            GoodType::Silver => "textures/card/silver.png".to_string(),
            GoodType::Cloth => "textures/card/cloth.png".to_string(),
            GoodType::Spice => "textures/card/spice.png".to_string(),
            GoodType::Leather => "textures/card/leather.png".to_string(),
        }
    }

    pub fn is_high_value(&self) -> bool {
        matches!(self, GoodType::Diamond | GoodType::Gold | GoodType::Silver)
    }
}

#[derive(Component, Clone, Debug)]
pub struct Card(pub CardType);

#[derive(Component)]
pub struct MarketCard(pub usize);

#[derive(Component)]
pub struct ActivePlayerGoodsCard(pub usize);

#[derive(Component)]
pub struct ActivePlayerCamelCard(pub usize);
