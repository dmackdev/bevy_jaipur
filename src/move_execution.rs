use bevy::prelude::*;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::{Animator, EaseFunction, Tween, TweenCompleted, TweeningPlugin, TweeningType};
use itertools::Itertools;
use std::cmp::Reverse;
use std::time::Duration;

use crate::card_selection::SelectedCard;
use crate::event::ConfirmTurnEvent;
use crate::game::*;
use crate::game_resources::card::*;
use crate::game_resources::deck::Deck;
use crate::game_resources::discard_pile::DiscardPile;
use crate::game_resources::market::Market;
use crate::game_resources::tokens::{BonusType, Tokens};
use crate::label::Label;
use crate::move_validation::MoveType;
use crate::positioning::{
    get_active_player_camel_card_translation, get_active_player_goods_card_translation,
    get_ai_player_goods_card_translation, get_market_card_translation, DISCARD_PILE_POS,
};
use crate::resources::GameState;
use crate::states::AppState;

#[allow(clippy::too_many_arguments)]
fn handle_take_single_good_move_confirmed(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_confirm_turn: EventReader<ConfirmTurnEvent>,
    mut deck: ResMut<Deck>,
    mut market: ResMut<Market>,
    selected_market_cards_query: Query<
        (Entity, &Card, &MarketCard, &Transform),
        With<SelectedCard>,
    >,
    mut active_player_query: Query<&mut GoodsHandOwner, With<ActivePlayer>>,
    mut deck_cards_query: Query<(Entity, &DeckCard, &Card, &Transform, &mut Handle<Image>)>,
    mut tween_state: ResMut<TweenState>,
    mut game_state: ResMut<GameState>,
    app_state: Res<State<AppState>>,
) {
    for _ev in ev_confirm_turn
        .iter()
        .filter(|ev| matches!(ev.0, MoveType::TakeSingleGood))
    {
        let mut active_player_goods_hand = active_player_query.single_mut();

        let (card_entity, card, market_card, transform) = selected_market_cards_query.single();

        let good = card.0.into_good_type();

        // Remove from market resource
        market.cards.remove(market_card.0);

        // add to active player goods hand
        active_player_goods_hand.0.push(good);

        let is_ai_turn = *app_state.current() == AppState::AiTurn;

        let end = if is_ai_turn {
            get_ai_player_goods_card_translation(active_player_goods_hand.0.len() - 1)
        } else {
            get_active_player_goods_card_translation(active_player_goods_hand.0.len() - 1)
        };

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once,
            Duration::from_secs(2),
            TransformPositionLens {
                start: transform.translation,
                end,
            },
        )
        .with_completed_event(1);

        tween_state.tweening_entities.push(card_entity);

        commands
            .entity(card_entity)
            .insert(Animator::new(tween))
            .remove::<MarketCard>();

        if !is_ai_turn {
            commands
                .entity(card_entity)
                .insert(ActivePlayerGoodsCard(active_player_goods_hand.0.len() - 1));
        }

        // Replace with card from deck
        if let Some(replacement_card) = deck.cards.pop() {
            market.cards.insert(market_card.0, replacement_card);

            let (deck_card_entity, _, card, deck_card_transform, mut top_deck_card_texture) =
                deck_cards_query
                    .iter_mut()
                    .max_by_key(|(_, dc, _, _, _)| dc.0)
                    .unwrap();

            // Update the sprite to show the face
            *top_deck_card_texture = asset_server.load(&card.0.get_card_texture());

            // Tween to the market card position
            let second_tween = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: deck_card_transform.translation,
                    end: get_market_card_translation(market_card.0),
                },
            )
            .with_completed_event(2);

            tween_state.tweening_entities.push(deck_card_entity);

            commands
                .entity(deck_card_entity)
                .insert(Animator::new(second_tween))
                .remove::<DeckCard>()
                .insert(MarketCard(market_card.0));
        } else {
            game_state.is_game_over = true;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_take_all_camels_move_confirmed(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_confirm_turn: EventReader<ConfirmTurnEvent>,
    mut deck: ResMut<Deck>,
    mut market: ResMut<Market>,
    selected_camel_market_cards_query: Query<(Entity, &MarketCard, &Transform), With<SelectedCard>>,
    mut active_player_query: Query<&mut CamelsHandOwner, With<ActivePlayer>>,
    mut deck_cards_query: Query<(Entity, &DeckCard, &Card, &Transform, &mut Handle<Image>)>,
    mut tween_state: ResMut<TweenState>,
    mut game_state: ResMut<GameState>,
) {
    for _ev in ev_confirm_turn
        .iter()
        .filter(|ev| matches!(ev.0, MoveType::TakeAllCamels))
    {
        let mut active_player_camel_hand = active_player_query.single_mut();

        for (idx, (card_entity, market_card, transform)) in
            selected_camel_market_cards_query.iter().enumerate()
        {
            // Remove from market resource
            market.cards.remove(market_card.0);

            // add to active player camel hand
            active_player_camel_hand.0 += 1;

            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: transform.translation,
                    end: get_active_player_camel_card_translation(active_player_camel_hand.0 - 1),
                },
            )
            .with_completed_event(1);

            tween_state.tweening_entities.push(card_entity);

            commands
                .entity(card_entity)
                .insert(Animator::new(tween))
                .remove::<MarketCard>()
                .insert(ActivePlayerCamelCard(active_player_camel_hand.0 - 1));

            // Replace with card from deck
            if let Some(replacement_card) = deck.cards.pop() {
                market.cards.insert(market_card.0, replacement_card);

                let l = deck_cards_query.iter().len();
                // Get the top deck card - with the highest index
                let (deck_card_entity, _, card, deck_card_transform, mut top_deck_card_texture) =
                    deck_cards_query
                        .iter_mut()
                        .sorted_by_key(|(_, dc, _, _, _)| dc.0)
                        .nth(l - 1 - idx)
                        .unwrap();

                // Update the sprite to show the face
                *top_deck_card_texture = asset_server.load(&card.0.get_card_texture());

                // Tween to the market card position
                let second_tween = Tween::new(
                    EaseFunction::QuadraticInOut,
                    TweeningType::Once,
                    Duration::from_secs(2),
                    TransformPositionLens {
                        start: deck_card_transform.translation,
                        end: get_market_card_translation(market_card.0),
                    },
                )
                .with_completed_event(2);

                tween_state.tweening_entities.push(deck_card_entity);

                commands
                    .entity(deck_card_entity)
                    .insert(Animator::new(second_tween))
                    .remove::<DeckCard>()
                    .insert(MarketCard(market_card.0));
            } else {
                game_state.is_game_over = true;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_exchange_goods_move_confirmed(
    mut commands: Commands,
    mut ev_confirm_turn: EventReader<ConfirmTurnEvent>,
    mut market: ResMut<Market>,
    mut active_player_query: Query<(&mut GoodsHandOwner, &mut CamelsHandOwner), With<ActivePlayer>>,
    active_player_selected_camel_cards: Query<
        (Entity, &Transform),
        (With<ActivePlayerCamelCard>, With<SelectedCard>),
    >,
    all_player_camel_cards: Query<(Entity, &ActivePlayerCamelCard, &Transform)>,
    active_player_selected_goods_card: Query<
        (Entity, &ActivePlayerGoodsCard, &Transform),
        With<SelectedCard>,
    >,
    selected_market_goods_cards_query: Query<(Entity, &MarketCard, &Transform), With<SelectedCard>>,
    mut tween_state: ResMut<TweenState>,
) {
    for _ev in ev_confirm_turn
        .iter()
        .filter(|ev| matches!(ev.0, MoveType::ExchangeForGoodsFromMarket))
    {
        let (mut goods_hand_owner, mut camels_hand_owner) = active_player_query.single_mut();

        // num player's selected goods <= selected market goods, since camels may fill the remainder
        for (player_good, market_good) in active_player_selected_goods_card
            .iter()
            .zip(selected_market_goods_cards_query.iter())
        {
            let good_type_removed_from_hand = goods_hand_owner.0.remove(player_good.1 .0);
            let good_type_removed_from_market = market.cards.remove(market_good.1 .0);

            goods_hand_owner.0.insert(
                player_good.1 .0,
                good_type_removed_from_market.into_good_type(),
            );
            market.cards.insert(
                market_good.1 .0,
                CardType::Good(good_type_removed_from_hand),
            );

            let tween_hand_to_market = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: player_good.2.translation,
                    end: get_market_card_translation(market_good.1 .0),
                },
            )
            .with_completed_event(1);

            tween_state.tweening_entities.push(player_good.0);

            commands
                .entity(player_good.0)
                .insert(Animator::new(tween_hand_to_market))
                .remove::<ActivePlayerGoodsCard>()
                .insert(MarketCard(market_good.1 .0));

            let tween_market_to_hand = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: market_good.2.translation,
                    end: get_active_player_goods_card_translation(player_good.1 .0),
                },
            )
            .with_completed_event(2);

            tween_state.tweening_entities.push(market_good.0);

            commands
                .entity(market_good.0)
                .insert(Animator::new(tween_market_to_hand))
                .remove::<MarketCard>()
                .insert(ActivePlayerGoodsCard(player_good.1 .0));
        }

        for (camel, market_good) in active_player_selected_camel_cards.iter().zip(
            selected_market_goods_cards_query
                .iter()
                .skip(active_player_selected_goods_card.iter().count()),
        ) {
            camels_hand_owner.0 -= 1;
            let good_type_removed_from_market = market.cards.remove(market_good.1 .0);

            goods_hand_owner
                .0
                .push(good_type_removed_from_market.into_good_type());
            market.cards.insert(market_good.1 .0, CardType::Camel);

            let tween_camel_to_market = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: camel.1.translation,
                    end: get_market_card_translation(market_good.1 .0),
                },
            )
            .with_completed_event(3);

            tween_state.tweening_entities.push(camel.0);

            commands
                .entity(camel.0)
                .insert(Animator::new(tween_camel_to_market))
                .remove::<ActivePlayerCamelCard>()
                .insert(MarketCard(market_good.1 .0));

            let tween_market_to_hand = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: market_good.2.translation,
                    end: get_active_player_goods_card_translation(goods_hand_owner.0.len() - 1),
                },
            )
            .with_completed_event(4);

            tween_state.tweening_entities.push(market_good.0);

            commands
                .entity(market_good.0)
                .insert(Animator::new(tween_market_to_hand))
                .remove::<MarketCard>()
                .insert(ActivePlayerGoodsCard(goods_hand_owner.0.len() - 1));
        }

        let exchanged_camel_entities = active_player_selected_camel_cards
            .iter()
            .map(|(e, _)| e)
            .collect::<Vec<_>>();

        let unexchanged_camel_cards = all_player_camel_cards
            .iter()
            .filter(|(e, _, _)| !exchanged_camel_entities.contains(e))
            .collect::<Vec<_>>();

        let current_indices_ordered = unexchanged_camel_cards
            .iter()
            .map(|(_, camel_card, _)| camel_card.0)
            .sorted()
            .collect::<Vec<_>>();

        for (e, camel_card, transform) in unexchanged_camel_cards {
            let index_in_hand = camel_card.0;
            let correct_index = current_indices_ordered
                .iter()
                .position(|i| *i == index_in_hand)
                .unwrap();

            if index_in_hand != correct_index {
                let tween_shift_in_hand = Tween::new(
                    EaseFunction::QuadraticInOut,
                    TweeningType::Once,
                    Duration::from_secs(2),
                    TransformPositionLens {
                        start: transform.translation,
                        end: get_active_player_camel_card_translation(correct_index),
                    },
                )
                .with_completed_event(e.to_bits());

                tween_state.tweening_entities.push(e);

                commands
                    .entity(e)
                    .insert(Animator::new(tween_shift_in_hand))
                    .remove::<ActivePlayerCamelCard>()
                    .insert(ActivePlayerCamelCard(correct_index));
            }
        }
    }
}

fn handle_sell_goods_move_confirmed(
    mut commands: Commands,
    mut ev_confirm_turn: EventReader<ConfirmTurnEvent>,
    mut discard_pile: ResMut<DiscardPile>,
    mut tween_state: ResMut<TweenState>,
    mut game_tokens: ResMut<Tokens>,
    active_player_selected_goods_card: Query<
        (Entity, &ActivePlayerGoodsCard, &Transform),
        With<SelectedCard>,
    >,
    all_active_player_goods_cards: Query<(Entity, &ActivePlayerGoodsCard, &Transform)>,
    mut active_player_query: Query<(&mut GoodsHandOwner, &mut TokensOwner), With<ActivePlayer>>,
    mut game_state: ResMut<GameState>,
) {
    for _ev in ev_confirm_turn
        .iter()
        .filter(|ev| matches!(ev.0, MoveType::SellGoods))
    {
        let (mut goods_hand_owner, mut tokens_owner) = active_player_query.single_mut();

        for (e, active_player_goods_card, transform) in active_player_selected_goods_card
            .iter()
            // sort in desc order to avoid shifting the indices for subsequent loop iterations after having removed a card from the hand
            .sorted_by_key(|(_, active_player_goods_card, _)| Reverse(active_player_goods_card.0))
        {
            let sold_card = goods_hand_owner.0.remove(active_player_goods_card.0);

            discard_pile.cards.push(CardType::Good(sold_card));

            let tween_goods_hand_to_discard_pile = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(2),
                TransformPositionLens {
                    start: transform.translation,
                    end: DISCARD_PILE_POS,
                },
            )
            .with_completed_event(4);

            tween_state.tweening_entities.push(e);

            commands
                .entity(e)
                .insert(Animator::new(tween_goods_hand_to_discard_pile))
                .remove::<ActivePlayerGoodsCard>();

            let next_goods_token = game_tokens.goods[sold_card].pop();

            if let Some(val) = next_goods_token {
                tokens_owner.0.goods[sold_card].push(val);
            }
        }

        // As we have just sold cards and removed them from the hand, the current indices of ActivePlayerGoodsCards may be misaligned to the index of the card in the hand
        let sold_entities: Vec<Entity> = active_player_selected_goods_card
            .iter()
            .map(|(e, _, _)| e)
            .collect();

        let unsold_goods_cards = all_active_player_goods_cards
            .iter()
            .filter(|(e, _, _)| !sold_entities.contains(e))
            .collect::<Vec<_>>();

        let current_indices_ordered = unsold_goods_cards
            .iter()
            .map(|(_, active_player_good_card, _)| active_player_good_card.0)
            .sorted()
            .collect::<Vec<_>>();

        for (e, active_player_good_card, transform) in unsold_goods_cards {
            let index_in_hand = active_player_good_card.0;
            let correct_index = current_indices_ordered
                .iter()
                .position(|i| *i == index_in_hand)
                .unwrap();

            if index_in_hand != correct_index {
                let tween_shift_in_hand = Tween::new(
                    EaseFunction::QuadraticInOut,
                    TweeningType::Once,
                    Duration::from_secs(2),
                    TransformPositionLens {
                        start: transform.translation,
                        end: get_active_player_goods_card_translation(correct_index),
                    },
                )
                .with_completed_event(e.to_bits());

                tween_state.tweening_entities.push(e);

                commands
                    .entity(e)
                    .insert(Animator::new(tween_shift_in_hand))
                    .remove::<ActivePlayerGoodsCard>()
                    .insert(ActivePlayerGoodsCard(correct_index));
            }
        }

        let num_cards_sold = active_player_selected_goods_card.iter().count();
        let bonus_type = match num_cards_sold {
            d if d == 3 => Some(BonusType::Three),
            d if d == 4 => Some(BonusType::Four),
            d if 5 <= d => Some(BonusType::Five),
            _ => None,
        };

        if let Some(bt) = bonus_type {
            if let Some(val) = game_tokens.bonus[bt].pop() {
                tokens_owner.0.bonus[bt].push(val);
            }
        }

        if game_tokens
            .goods
            .iter()
            .filter(|(_, token_values)| token_values.is_empty())
            .count()
            >= 3
        {
            game_state.is_game_over = true;
        }
    }
}

fn handle_confirm_turn_event(
    mut state: ResMut<State<AppState>>,
    mut ev_confirm_turn: EventReader<ConfirmTurnEvent>,
) {
    for _ev in ev_confirm_turn.iter() {
        state.set(AppState::WaitForTweensToFinish).unwrap();
    }
}

#[derive(Default)]
pub struct TweenState {
    pub tweening_entities: Vec<Entity>,
    // Distinguishes between there never being any tweening entities in the first place, and actual tweens that started completing
    pub did_all_tweens_complete: bool,
}

pub struct ScreenTransitionDelayTimer(Timer);

fn wait_for_tweens_to_finish(
    mut commands: Commands,
    mut ev_tween_completed: EventReader<TweenCompleted>,
    mut tween_state: ResMut<TweenState>,
    mut app_state: ResMut<State<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<ScreenTransitionDelayTimer>,
    game_state: Res<GameState>,
    active_player_query: Query<(Entity, Option<&AiPlayer>), With<ActivePlayer>>,
    inactive_player_query: Query<Entity, (With<Player>, Without<ActivePlayer>)>,
    active_player_goods_cards: Query<(Entity, &ActivePlayerGoodsCard)>,
    active_player_camel_cards: Query<(Entity, &ActivePlayerCamelCard)>,
    inactive_player_goods_cards: Query<(Entity, &InactivePlayerGoodsCard)>,
    inactive_player_camel_cards: Query<(Entity, &InactivePlayerCamelCard)>,
) {
    for ev in ev_tween_completed.iter() {
        let index = tween_state
            .tweening_entities
            .iter()
            .position(|e| *e == ev.entity)
            .unwrap();
        tween_state.tweening_entities.remove(index);

        if tween_state.tweening_entities.is_empty() {
            tween_state.did_all_tweens_complete = true;
        }
    }

    if tween_state.did_all_tweens_complete && timer.0.tick(time.delta()).just_finished() {
        tween_state.did_all_tweens_complete = false;

        let (active_player_entity, ai_player_option) = active_player_query.single();
        let is_current_player_ai = ai_player_option.is_some();
        let inactive_player_entity = inactive_player_query.single();

        commands
            .entity(active_player_entity)
            .remove::<ActivePlayer>();

        commands.entity(inactive_player_entity).insert(ActivePlayer);

        for (e, active_good) in active_player_goods_cards.iter() {
            commands
                .entity(e)
                .remove::<ActivePlayerGoodsCard>()
                .insert(InactivePlayerGoodsCard(active_good.0));
        }

        for (e, active_camel) in active_player_camel_cards.iter() {
            commands
                .entity(e)
                .remove::<ActivePlayerCamelCard>()
                .insert(InactivePlayerCamelCard(active_camel.0));
        }

        for (e, inactive_good) in inactive_player_goods_cards.iter() {
            commands
                .entity(e)
                .remove::<InactivePlayerGoodsCard>()
                .insert(ActivePlayerGoodsCard(inactive_good.0));
        }

        for (e, inactive_camel) in inactive_player_camel_cards.iter() {
            commands
                .entity(e)
                .remove::<InactivePlayerCamelCard>()
                .insert(ActivePlayerCamelCard(inactive_camel.0));
        }

        if game_state.is_game_over {
            app_state.set(AppState::GameOver).unwrap();
        } else if game_state.is_playing_ai {
            if is_current_player_ai {
                println!("TRANSITION TO PLAYER MOVE");
                app_state.set(AppState::InGame).unwrap();
            } else {
                println!("TRANSITION TO AI MOVE");
                app_state.set(AppState::AiTurn).unwrap();
            }
        } else {
            app_state.set(AppState::TurnTransition).unwrap();
        }
    }
}

pub struct MoveExecutionPlugin;

impl Plugin for MoveExecutionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ScreenTransitionDelayTimer(Timer::from_seconds(2.0, true)))
            .init_resource::<TweenState>()
            .add_plugin(TweeningPlugin)
            // TODO: since these respond to events, can remove the AppState stage criteria. see if you can apply the labels to the whole system set instead of each system
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(
                        handle_take_single_good_move_confirmed
                            .label(Label::EventReader)
                            .before(handle_confirm_turn_event)
                            .after(Label::EventWriter),
                    )
                    .with_system(
                        handle_take_all_camels_move_confirmed
                            .label(Label::EventReader)
                            .before(handle_confirm_turn_event)
                            .after(Label::EventWriter),
                    )
                    .with_system(
                        handle_exchange_goods_move_confirmed
                            .label(Label::EventReader)
                            .before(handle_confirm_turn_event)
                            .after(Label::EventWriter),
                    )
                    .with_system(
                        handle_sell_goods_move_confirmed
                            .label(Label::EventReader)
                            .before(handle_confirm_turn_event)
                            .after(Label::EventWriter),
                    )
                    .with_system(
                        handle_confirm_turn_event
                            .label(Label::EventReader)
                            .after(Label::EventWriter),
                    ),
            )
            .add_system_set(
                SystemSet::on_update(AppState::AiTurn)
                    .with_system(
                        handle_take_single_good_move_confirmed
                            .label(Label::EventReader)
                            .before(handle_confirm_turn_event)
                            .after(Label::EventWriter),
                    )
                    .with_system(
                        handle_take_all_camels_move_confirmed
                            .label(Label::EventReader)
                            .before(handle_confirm_turn_event)
                            .after(Label::EventWriter),
                    )
                    .with_system(
                        handle_exchange_goods_move_confirmed
                            .label(Label::EventReader)
                            .before(handle_confirm_turn_event)
                            .after(Label::EventWriter),
                    )
                    .with_system(
                        handle_sell_goods_move_confirmed
                            .label(Label::EventReader)
                            .before(handle_confirm_turn_event)
                            .after(Label::EventWriter),
                    )
                    .with_system(
                        handle_confirm_turn_event
                            .label(Label::EventReader)
                            .after(Label::EventWriter),
                    ),
            )
            .add_system_set(
                SystemSet::on_update(AppState::WaitForTweensToFinish)
                    .with_system(wait_for_tweens_to_finish),
            );
    }
}
