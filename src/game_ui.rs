use std::fmt;

use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    event::ConfirmTurnEvent,
    game::{
        ActivePlayerCamelCard, ActivePlayerGoodsCard, Card, CardType, GoodType, MarketCard,
        SelectedCard,
    },
    label::Label,
    resources::{MoveValidity, SelectedCardState},
    states::{AppState, TurnState},
};

#[derive(Debug, Copy, Clone)]
enum GameButtonKind {
    Take,
    Sell,
    Confirm,
}

impl fmt::Display for GameButtonKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone)]
struct GameButtonData {
    kind: GameButtonKind,
    normal_color: Color,
    hovered_color: Color,
    pressed_color: Color,
}

#[derive(Component)]
struct GameButton(GameButtonData);

#[derive(Component)]
struct TakeGameButton;

#[derive(Component)]
struct SellGameButton;

#[derive(Component)]
struct ConfirmGameButton;

const TAKE_BUTTON_DATA: GameButtonData = GameButtonData {
    kind: GameButtonKind::Take,
    normal_color: Color::rgb(0.15, 0.15, 0.15),
    hovered_color: Color::GRAY,
    pressed_color: Color::BLUE,
};

const SELL_BUTTON_DATA: GameButtonData = GameButtonData {
    kind: GameButtonKind::Sell,
    normal_color: Color::rgb(0.15, 0.15, 0.15),
    hovered_color: Color::GRAY,
    pressed_color: Color::BLUE,
};

// Colors only apply when move is valid and confirm button is enabled
const CONFIRM_BUTTON_DATA: GameButtonData = GameButtonData {
    kind: GameButtonKind::Confirm,
    normal_color: Color::DARK_GREEN,
    hovered_color: Color::rgba(0.0, 1.0, 0.0, 0.5),
    pressed_color: Color::DARK_GREEN,
};

fn create_button<C: Component>(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    button_component: C,
    game_button_data: GameButtonData,
) -> Entity {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(45.0)),
                // center button
                margin: UiRect::all(Val::Px(10.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: game_button_data.normal_color.into(),
            ..default()
        })
        .insert(button_component)
        .insert(GameButton(game_button_data))
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                game_button_data.kind.to_string(),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        })
        .id()
}

#[derive(Component)]
struct GameUiRoot;

fn setup_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let root_node_entity = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position: UiRect::new(Val::Auto, Val::Px(0.0), Val::Auto, Val::Px(0.0)),
                ..default()
            },
            color: Color::DARK_GRAY.into(),
            transform: Transform::default().with_translation(Vec3::new(300.0, 0.0, 0.0)),
            ..default()
        })
        .insert(GameUiRoot)
        .id();

    let take_button_entity = create_button(
        &mut commands,
        &asset_server,
        TakeGameButton,
        TAKE_BUTTON_DATA,
    );
    let sell_button_entity = create_button(
        &mut commands,
        &asset_server,
        SellGameButton,
        SELL_BUTTON_DATA,
    );
    let confirm_button_entity = create_button(
        &mut commands,
        &asset_server,
        ConfirmGameButton,
        CONFIRM_BUTTON_DATA,
    );

    commands.entity(root_node_entity).push_children(&[
        take_button_entity,
        sell_button_entity,
        confirm_button_entity,
    ]);
}

#[derive(Component)]
struct JustClickedButton;

fn handle_turn_state_button(
    mut commands: Commands,
    mut turn_state: ResMut<State<TurnState>>,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut UiColor, &GameButton),
        (Changed<Interaction>, Without<ConfirmGameButton>),
    >,
) {
    for (interacted_entity, interaction, mut color, game_button) in &mut interaction_query {
        let is_button_selected = *turn_state.current() == game_button.0.kind.into();

        match (*interaction, is_button_selected) {
            (Interaction::Clicked, true) => {
                *color = game_button.0.normal_color.into();
                turn_state.set(TurnState::None).unwrap();
            }
            (Interaction::Clicked, false) => {
                *color = game_button.0.pressed_color.into();
                turn_state.set(game_button.0.kind.into()).unwrap();
                commands.entity(interacted_entity).insert(JustClickedButton);
            }
            (Interaction::Hovered, false) => {
                *color = game_button.0.hovered_color.into();
            }
            (Interaction::None, true) => {
                *color = game_button.0.pressed_color.into();
            }
            (Interaction::None, false) => {
                *color = game_button.0.normal_color.into();
            }
            (_, _) => return,
        }
    }
}

fn update_unclicked_turn_move_button_colors(
    mut commands: Commands,
    just_clicked_button_query: Query<Entity, Added<JustClickedButton>>,
    mut other_buttons_query: Query<(Entity, &mut UiColor, &GameButton), Without<ConfirmGameButton>>,
) {
    if just_clicked_button_query.iter().count() == 0 {
        return;
    }

    for e in just_clicked_button_query.iter() {
        commands.entity(e).remove::<JustClickedButton>();
    }

    for (_, mut color, game_button) in other_buttons_query
        .iter_mut()
        .filter(|(e, _, _)| !just_clicked_button_query.iter().contains(e))
    {
        *color = game_button.0.normal_color.into();
    }
}

impl From<GameButtonKind> for TurnState {
    fn from(kind: GameButtonKind) -> Self {
        match kind {
            GameButtonKind::Take => TurnState::Take,
            GameButtonKind::Sell => TurnState::Sell,
            GameButtonKind::Confirm => TurnState::None,
        }
    }
}

fn handle_confirm_button_interaction(
    mut commands: Commands,
    mut turn_state: ResMut<State<TurnState>>,
    mut ev_confirm_turn: EventWriter<ConfirmTurnEvent>,
    mut move_validity_state: ResMut<MoveValidity>,
    mut selected_card_state: ResMut<SelectedCardState>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &GameButton),
        (Changed<Interaction>, With<ConfirmGameButton>),
    >,
    ui_root_query: Query<Entity, With<GameUiRoot>>,
) {
    for (interaction, mut color, game_button) in &mut interaction_query {
        if *move_validity_state.as_ref() == MoveValidity::Invalid {
            return;
        }

        match *interaction {
            Interaction::Clicked => {
                *color = game_button.0.pressed_color.into();

                let desired_turn_state: TurnState = game_button.0.kind.into();
                if *turn_state.current() != desired_turn_state {
                    turn_state.set(desired_turn_state).unwrap();
                }

                selected_card_state.0.clear();
                *move_validity_state = MoveValidity::default();

                ev_confirm_turn.send(ConfirmTurnEvent);
                commands.entity(ui_root_query.single()).despawn_recursive();
            }
            Interaction::Hovered => {
                *color = game_button.0.hovered_color.into();
            }
            Interaction::None => {
                *color = game_button.0.normal_color.into();
            }
        }
    }
}

// TODO: extract to game_logic
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
        println!("TAKE SINGLE GOOD");
        *move_validity_state = MoveValidity::Valid;
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
        println!("TAKE ALL CAMELS");
        *move_validity_state = MoveValidity::Valid;
        return;
    }

    // Exchange at least two goods from the market with combination of camels and goods from player's hand
    let num_selected_camels_from_market = market_selected_card_query
        .iter()
        .filter(|c| matches!(c.0, CardType::Camel))
        .count();

    if num_selected_camels_from_market == 0
        && num_selected_market_goods_cards > 1
        && num_selected_market_goods_cards
            == num_selected_camels_from_hand + num_selected_goods_from_hand
        && num_selected_market_goods_cards + num_total_goods_in_hand - num_selected_goods_from_hand
            <= 7
    {
        println!("EXCHANGE");
        *move_validity_state = MoveValidity::Valid;
        return;
    }

    *move_validity_state = MoveValidity::Invalid;
}

// TODO: extract to game_logic
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
                *move_validity_state = MoveValidity::Valid;
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

fn handle_move_validity_change(
    move_validity_state: Res<MoveValidity>,
    mut confirm_button_query: Query<(&mut UiColor, &GameButton), With<ConfirmGameButton>>,
) {
    if !move_validity_state.is_changed() {
        return;
    }

    let (mut confirm_button_color, game_button) = confirm_button_query.single_mut();

    match move_validity_state.as_ref() {
        MoveValidity::Invalid => *confirm_button_color = Color::RED.into(),
        MoveValidity::Valid => *confirm_button_color = game_button.0.normal_color.into(),
    }
}

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_game_ui))
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(handle_turn_state_button)
                    .with_system(
                        update_unclicked_turn_move_button_colors.after(handle_turn_state_button),
                    )
                    .with_system(
                        handle_confirm_button_interaction
                            .label(Label::EventWriter)
                            .before(Label::EventReader),
                    )
                    .with_system(handle_move_validity_change),
            )
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
