use bevy::prelude::{Vec2, Vec3};

pub const DECK_START_POS: Vec3 = Vec3::new(300.0, 0.0, 0.0);

pub const DISCARD_PILE_POS: Vec3 = Vec3::new(
    DECK_START_POS.x + 1.5 * CARD_DIMENSION.x + CARD_PADDING,
    DECK_START_POS.y,
    0.,
);

pub const CARD_DIMENSION: Vec2 = Vec2::new(104.0, 150.0);
pub const CARD_PADDING: f32 = 20.0;

pub const GOODS_HAND_START_POS: Vec3 = Vec3::new(-5.0 * 0.5 * CARD_DIMENSION.x, -400.0, 0.0);

pub const CAMEL_HAND_START_POS: Vec3 = Vec3::new(
    GOODS_HAND_START_POS.x,
    GOODS_HAND_START_POS.y + CARD_DIMENSION.y + CARD_PADDING,
    0.0,
);

pub const INACTIVE_PLAYER_GOODS_HAND_START_POS: Vec3 = Vec3::new(
    GOODS_HAND_START_POS.x,
    GOODS_HAND_START_POS.y * -1.0,
    GOODS_HAND_START_POS.z,
);

pub fn get_active_player_goods_card_translation(idx: usize) -> Vec3 {
    GOODS_HAND_START_POS + Vec3::X * idx as f32 * (CARD_DIMENSION.x + CARD_PADDING)
}

pub fn get_market_card_translation(idx: usize) -> Vec3 {
    DECK_START_POS
        - (5 - idx) as f32 * CARD_DIMENSION.x * Vec3::X
        - (5 - idx) as f32 * CARD_PADDING * Vec3::X
}

pub fn get_active_player_camel_card_translation(idx: usize) -> Vec3 {
    CAMEL_HAND_START_POS + Vec3::X * idx as f32 * (CARD_DIMENSION.x + CARD_PADDING)
}
