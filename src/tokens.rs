use enum_map::{enum_map, Enum, EnumMap};
use rand::{seq::SliceRandom, thread_rng};

use crate::card::GoodType;

#[derive(Clone, Debug)]
pub struct Tokens {
    pub goods: EnumMap<GoodType, Vec<usize>>,
    pub bonus: EnumMap<BonusType, Vec<usize>>,
}

#[derive(Clone, Copy, Debug, Enum)]
pub enum BonusType {
    Three,
    Four,
    Five,
}

impl Tokens {
    pub fn create_game_tokens() -> Self {
        let goods = enum_map! {
          GoodType::Diamond => vec![5,5,5,7,7,],
          GoodType::Gold => vec![5,5,5,6,6,],
          GoodType::Silver => vec![5,5,5,5,5,],
          GoodType::Cloth => vec![1,1,2,2,3,3,5,],
          GoodType::Spice => vec![1,1,2,2,3,3,5,],
          GoodType::Leather => vec![1,1,1,1,1,1,2,3,4,],
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

    pub fn create_empty() -> Self {
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
