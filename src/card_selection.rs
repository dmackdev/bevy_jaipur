use bevy::prelude::*;
use bevy_interact_2d::{Group, Interactable, InteractionState};
use bevy_prototype_lyon::prelude::ShapePlugin;
use bevy_prototype_lyon::{
    prelude::{DrawMode, GeometryBuilder, StrokeMode},
    shapes::Polygon,
};
use itertools::Itertools;

use crate::{
    event::ConfirmTurnEvent,
    game::{ActivePlayerCamelCard, ActivePlayerGoodsCard, Card, MarketCard},
    label::Label,
    states::AppState,
};

#[derive(Component)]
struct ClickedCard;

#[derive(Component)]
pub struct SelectedCard;

#[derive(Component)]
pub struct CardOutline;

#[derive(Default)]
pub struct SelectedCardState(pub Vec<Entity>);

fn update_card_as_clicked(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    interaction_state: Res<InteractionState>,
    mut card_query: Query<
        Entity,
        Or<(
            With<MarketCard>,
            With<ActivePlayerGoodsCard>,
            With<ActivePlayerCamelCard>,
        )>,
    >,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        for card_entity in card_query.iter_mut() {
            if interaction_state
                .get_group(Group(0))
                .iter()
                .any(|(e, _)| *e == card_entity)
            {
                commands.entity(card_entity).insert(ClickedCard);
            }
        }
    }
}

fn update_card_as_selected(
    mut commands: Commands,
    mut selected_card_state: ResMut<SelectedCardState>,
    clicked_card_query: Query<(Entity, &Interactable), (Added<ClickedCard>, Without<SelectedCard>)>,
) {
    for (clicked_card_entity, interactable) in clicked_card_query.iter() {
        let bounding_mesh = Polygon {
            points: vec![
                Vec2::new(interactable.bounding_box.0.x, interactable.bounding_box.0.y),
                Vec2::new(interactable.bounding_box.1.x, interactable.bounding_box.0.y),
                Vec2::new(interactable.bounding_box.1.x, interactable.bounding_box.1.y),
                Vec2::new(interactable.bounding_box.0.x, interactable.bounding_box.1.y),
            ],
            closed: true,
        };

        let card_outline_entity = commands
            .spawn_bundle(SpatialBundle::default())
            .insert_bundle(GeometryBuilder::build_as(
                &bounding_mesh,
                DrawMode::Stroke(StrokeMode::new(Color::YELLOW, 10.0)),
                Transform::default(),
            ))
            .insert(CardOutline)
            .id();

        commands
            .entity(clicked_card_entity)
            .insert(SelectedCard)
            .remove::<ClickedCard>()
            .add_child(card_outline_entity);

        selected_card_state.0.push(clicked_card_entity);
    }
}

fn update_card_as_unselected(
    mut commands: Commands,
    mut selected_card_state: ResMut<SelectedCardState>,
    clicked_card_query: Query<Entity, (Added<ClickedCard>, With<SelectedCard>)>,
) {
    for clicked_card_entity in clicked_card_query.iter() {
        commands
            .entity(clicked_card_entity)
            .remove::<SelectedCard>();

        let idx = selected_card_state
            .0
            .iter()
            .position(|e| *e == clicked_card_entity)
            .unwrap();

        selected_card_state.0.remove(idx);
    }
}

// This system reacts to removal of SelectedCard components for removing the outline - this means that it can be reused for both deselecting cards via clicking, or when the turn's move is confirmed.
fn handle_selected_card_removed(
    mut commands: Commands,
    removed_selected_card: RemovedComponents<SelectedCard>,
    card_query: Query<(Entity, &Children), (With<Card>, Without<SelectedCard>)>,
    card_outline_query: Query<Entity, With<CardOutline>>,
) {
    if removed_selected_card.iter().len() == 0 {
        return;
    }

    let cards_with_selected_removed_query = card_query
        .iter()
        .filter(|(e, _)| removed_selected_card.iter().contains(e));

    for (clicked_card_entity, children) in cards_with_selected_removed_query {
        for &child in children.iter() {
            let card_outline_entity = card_outline_query.get(child).unwrap();
            commands.entity(card_outline_entity).despawn_recursive();
        }
        commands.entity(clicked_card_entity).remove::<ClickedCard>();
    }
}

fn remove_card_selections_on_confirm_turn(
    mut commands: Commands,
    mut ev_confirm_turn: EventReader<ConfirmTurnEvent>,
    selected_cards_query: Query<Entity, With<SelectedCard>>,
) {
    for _ev in ev_confirm_turn.iter() {
        for entity in selected_cards_query.iter() {
            commands.entity(entity).remove::<SelectedCard>();
        }
    }
}

pub struct CardSelectionPlugin;

impl Plugin for CardSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .init_resource::<SelectedCardState>()
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(update_card_as_clicked)
                    .with_system(update_card_as_selected.after(update_card_as_clicked))
                    .with_system(update_card_as_unselected.after(update_card_as_clicked))
                    .with_system(
                        remove_card_selections_on_confirm_turn
                            .label(Label::EventReader)
                            .after(Label::EventWriter),
                    ),
            )
            // component removal occurs at the end of the stage (i.e. update stage), so this system needs to go in PostUpdate
            .add_system_to_stage(CoreStage::PostUpdate, handle_selected_card_removed);
    }
}
