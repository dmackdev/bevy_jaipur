use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    card_selection::{SelectedCard, SelectedCardState},
    game_resources::card::{
        ActivePlayerCamelCard, ActivePlayerGoodsCard, Card, CardType, GoodType, MarketCard,
    },
    states::TurnState,
};

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

#[allow(clippy::too_many_arguments)]
fn handle_selected_card_state_change_for_take(
    turn_state: Res<State<TurnState>>,
    selected_card_state: Res<SelectedCardState>,
    mut move_validity_state: ResMut<MoveValidity>,
    market_selected_card_query: Query<&Card, (With<MarketCard>, With<SelectedCard>)>,
    all_market_card_query: Query<&Card, With<MarketCard>>,
    camel_hand_selected_card_query: Query<&Card, (With<ActivePlayerCamelCard>, With<SelectedCard>)>,
    goods_hand_selected_card_query: Query<&Card, (With<ActivePlayerGoodsCard>, With<SelectedCard>)>,
    all_goods_hand_card_query: Query<&Card, With<ActivePlayerGoodsCard>>,
) {
    if *turn_state.current() != TurnState::Take
        || (!selected_card_state.is_changed() && !turn_state.is_changed())
    {
        return;
    }

    let num_selected_market_goods_cards = market_selected_card_query
        .iter()
        .filter(|c| matches!(c.0, CardType::Good(_)))
        .count();

    let num_selected_camels_from_hand = camel_hand_selected_card_query.iter().count();

    let num_total_selected_cards_in_market = market_selected_card_query.iter().count();

    let num_selected_goods_from_hand = goods_hand_selected_card_query.iter().count();

    let num_total_goods_in_hand = all_goods_hand_card_query.iter().count();

    // Take single good from market rule
    if num_selected_market_goods_cards == 1
        && num_total_selected_cards_in_market == 1
        && num_selected_camels_from_hand == 0
        && num_selected_goods_from_hand == 0
        && num_total_goods_in_hand < 7
    {
        // println!("TAKE SINGLE GOOD");
        *move_validity_state = MoveValidity::Valid(MoveType::TakeSingleGood);
        return;
    }

    // Take all camels from market rule
    let total_num_camels_in_market = all_market_card_query
        .iter()
        .filter(|c| matches!(c.0, CardType::Camel))
        .count();

    if num_total_selected_cards_in_market > 0
        && market_selected_card_query
            .iter()
            .all(|c| matches!(c.0, CardType::Camel))
        && market_selected_card_query.iter().count() == total_num_camels_in_market
        && goods_hand_selected_card_query.iter().count() == 0
    {
        // println!("TAKE ALL CAMELS");
        *move_validity_state = MoveValidity::Valid(MoveType::TakeAllCamels);
        return;
    }

    // Exchange at least two goods from the market with combination of camels and goods from player's hand
    let do_market_goods_set_and_hand_goods_set_intersect = market_selected_card_query
        .iter()
        .filter_map(|c| match c.0 {
            CardType::Camel => None,
            CardType::Good(g) => Some(g),
        })
        .any(|g| {
            goods_hand_selected_card_query
                .iter()
                .filter_map(|c| match c.0 {
                    CardType::Camel => None,
                    CardType::Good(g) => Some(g),
                })
                .contains(&g)
        });

    let num_selected_camels_from_market = market_selected_card_query
        .iter()
        .filter(|c| matches!(c.0, CardType::Camel))
        .count();

    if !do_market_goods_set_and_hand_goods_set_intersect
        && num_selected_camels_from_market == 0
        && num_selected_market_goods_cards > 1
        && num_selected_market_goods_cards
            == num_selected_camels_from_hand + num_selected_goods_from_hand
        && num_selected_market_goods_cards + num_total_goods_in_hand - num_selected_goods_from_hand
            <= 7
    {
        // println!("EXCHANGE");
        *move_validity_state = MoveValidity::Valid(MoveType::ExchangeForGoodsFromMarket);
        return;
    }

    *move_validity_state = MoveValidity::Invalid;
}

fn handle_selected_card_state_change_for_sell(
    turn_state: Res<State<TurnState>>,
    selected_card_state: Res<SelectedCardState>,
    mut move_validity_state: ResMut<MoveValidity>,
    goods_hand_selected_card_query: Query<&Card, (With<ActivePlayerGoodsCard>, With<SelectedCard>)>,
    camel_hand_selected_card_query: Query<&Card, (With<ActivePlayerCamelCard>, With<SelectedCard>)>,
    market_selected_card_query: Query<&Card, (With<MarketCard>, With<SelectedCard>)>,
) {
    if *turn_state.current() != TurnState::Sell
        || (!selected_card_state.is_changed() && !turn_state.is_changed())
    {
        return;
    }

    let num_selected_goods_from_hand = goods_hand_selected_card_query.iter().count();
    let num_selected_camels_from_hand = camel_hand_selected_card_query.iter().count();
    let num_selected_cards_from_market = market_selected_card_query.iter().count();

    if num_selected_goods_from_hand > 0
        && num_selected_camels_from_hand == 0
        && num_selected_cards_from_market == 0
    {
        let selected_goods_types: Vec<GoodType> = goods_hand_selected_card_query
            .iter()
            .filter_map(|c| match c.0 {
                CardType::Camel => None,
                CardType::Good(g) => Some(g),
            })
            .collect();
        let are_all_goods_the_same = selected_goods_types.windows(2).all(|w| w[0] == w[1]);

        if are_all_goods_the_same {
            let good_type = selected_goods_types[0];

            if !good_type.is_high_value() || num_selected_goods_from_hand > 1 {
                *move_validity_state = MoveValidity::Valid(MoveType::SellGoods);
                return;
            }
        }
    }

    *move_validity_state = MoveValidity::Invalid;
}

fn handle_no_turn_state_selected(
    turn_state: Res<State<TurnState>>,
    mut move_validity_state: ResMut<MoveValidity>,
) {
    if turn_state.is_changed() && *turn_state.current() == TurnState::None {
        *move_validity_state = MoveValidity::Invalid;
    }
}

pub struct MoveValidationPlugin;

impl Plugin for MoveValidationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<MoveValidity>()
            // component removal occurs at the end of the stage (i.e. update stage), so this system needs to go in PostUpdate
            .add_system_to_stage(
                CoreStage::PostUpdate,
                handle_selected_card_state_change_for_take,
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                handle_selected_card_state_change_for_sell,
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                handle_no_turn_state_selected
                    .after(handle_selected_card_state_change_for_take)
                    .after(handle_selected_card_state_change_for_sell),
            );
    }
}
