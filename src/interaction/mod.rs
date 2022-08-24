use std::collections::HashMap;

use bevy::prelude::*;

use bevy::{
    ecs::event::{Events, ManualEventReader},
    render::camera::Camera,
    window::WindowId,
};
#[cfg(feature = "debug")]
use bevy_prototype_lyon::plugin::ShapePlugin;

pub mod drag;

/// The interaction plugin adds cursor interactions for entities
/// with the Interactable component.
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractionState>()
            .add_system_to_stage(CoreStage::PostUpdate, interaction_state_system)
            .add_system_to_stage(CoreStage::PostUpdate, interaction_system);
    }
}

/// The interaction debug plugin is a drop-in replacement for the interaction
/// plugin that will draw the bounding boxes for Interactable components.
/// This debug plugin requires the bevy_prototype_lyon ShapePlugin
/// in order to function.
#[cfg(feature = "debug")]
pub struct InteractionDebugPlugin;

#[cfg(feature = "debug")]
impl Plugin for InteractionDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InteractionPlugin)
            .add_plugin(ShapePlugin)
            // TODO: what is the correct stage for this?
            // POST_UPDATE doesn't work because then lyon won't draw the bounding mesh
            // check whether that is done in UPDATE or POST_UPDATE
            .add_system_to_stage(CoreStage::PreUpdate, setup_interaction_debug)
            .add_system_to_stage(CoreStage::PostUpdate, cleanup_interaction_debug);
    }
}

/// Using groups it is easy to have systems only interact with
/// draggables in a specific group.
/// An example usecase would be separate groups for draggables and drop zones.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, Default)]
pub struct Group(pub u8);

#[derive(Default)]
pub struct InteractionState {
    pub ordered_interact_list_map: HashMap<Group, Vec<(Entity, Vec2)>>,
    pub cursor_positions: HashMap<Group, Vec2>,
    pub last_window_id: WindowId,
    pub last_cursor_position: Vec2,
}

impl InteractionState {
    pub fn get_group(&self, group: Group) -> Vec<(Entity, Vec2)> {
        match self.ordered_interact_list_map.get(&group) {
            Some(interactions) => interactions.clone(),
            None => vec![],
        }
    }
}

/// Attach an interaction source to cameras you want to interact from
#[derive(Component)]
pub struct InteractionSource {
    pub groups: Vec<Group>,
    pub cursor_events: ManualEventReader<CursorMoved>,
}

impl Default for InteractionSource {
    fn default() -> Self {
        Self {
            groups: vec![Group::default()],
            cursor_events: ManualEventReader::default(),
        }
    }
}

/// This system calculates the interaction point for each group
/// whenever the cursor is moved.
fn interaction_state_system(
    mut interaction_state: ResMut<InteractionState>,
    cursor_moved: Res<Events<CursorMoved>>,
    windows: Res<Windows>,
    mut sources: Query<(&mut InteractionSource, &GlobalTransform, Option<&Camera>)>,
) {
    interaction_state.cursor_positions.clear();

    for (mut interact_source, global_transform, camera) in sources.iter_mut() {
        if let Some(evt) = interact_source.cursor_events.iter(&cursor_moved).last() {
            interaction_state.last_window_id = evt.id;
            interaction_state.last_cursor_position = evt.position;
        }
        let projection_matrix = match camera {
            Some(camera) => camera.projection_matrix(),
            None => panic!("Interacting without camera not supported."),
        };
        if let Some(window) = windows.get(interaction_state.last_window_id) {
            let screen_size = Vec2::from([window.width() as f32, window.height() as f32]);
            let cursor_position = interaction_state.last_cursor_position;
            let cursor_position_ndc =
                (cursor_position / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
            let camera_matrix = global_transform.compute_matrix();
            let ndc_to_world: Mat4 = camera_matrix * projection_matrix.inverse();
            let cursor_position = ndc_to_world
                .transform_point3(cursor_position_ndc.extend(1.0))
                .truncate();

            for group in &interact_source.groups {
                if interaction_state
                    .cursor_positions
                    .insert(*group, cursor_position)
                    .is_some()
                {
                    panic!(
                        "Multiple interaction sources have been added to interaction group {:?}",
                        group
                    );
                }
            }
        }
    }
}

/// This component makes an entity interactable with the mouse cursor
#[derive(Component)]
pub struct Interactable {
    /// The interaction groups this interactable entity belongs to
    pub groups: Vec<Group>,
    /// The interaction area for the interactable entity
    pub bounding_box: (Vec2, Vec2),
}

impl Default for Interactable {
    fn default() -> Self {
        Self {
            groups: vec![Group::default()],
            bounding_box: (Vec2::default(), Vec2::default()),
        }
    }
}

/// This system checks what for what groups an entity is currently interacted with
fn interaction_system(
    mut interaction_state: ResMut<InteractionState>,
    interactables: Query<(Entity, &GlobalTransform, &Interactable)>,
) {
    interaction_state.ordered_interact_list_map.clear();

    for (entity, global_transform, interactable) in interactables.iter() {
        let cursor_positions = interaction_state.cursor_positions.clone();
        for (group, cursor_position) in cursor_positions {
            if !interactable.groups.contains(&group) {
                continue;
            }
            // TODO: use bounding_mesh
            let relative_cursor_position = (cursor_position
                - global_transform.translation().truncate())
                / Transform::from(*global_transform).scale.truncate();
            if (interactable.bounding_box.0.x..interactable.bounding_box.1.x)
                .contains(&relative_cursor_position.x)
                && (interactable.bounding_box.0.y..interactable.bounding_box.1.y)
                    .contains(&relative_cursor_position.y)
            {
                let interaction = (entity, cursor_position);
                if let Some(list) = interaction_state.ordered_interact_list_map.get_mut(&group) {
                    list.push(interaction)
                } else {
                    interaction_state
                        .ordered_interact_list_map
                        .insert(group, vec![interaction]);
                }
            }
        }
    }
}

#[cfg(feature = "debug")]
#[derive(Component)]
pub struct DebugInteractable {
    pub child: Entity,
}

#[cfg(feature = "debug")]
fn setup_interaction_debug(
    mut commands: Commands,
    interactables: Query<(Entity, &Interactable), Added<Interactable>>,
) {
    use bevy_prototype_lyon::prelude::*;

    for (entity, interactable) in interactables.iter() {
        let group_sum = interactable.groups.iter().fold(0, |acc, Group(n)| acc + n);
        let (red, green) = match group_sum {
            0 => (0, 0),
            1 => (255, 0),
            2 => (0, 255),
            _ => (0, 0),
        };
        let blue = match interactable.groups.is_empty() {
            true => 0,
            false => 255,
        };

        let bounding_mesh = shapes::Polygon {
            points: vec![
                Vec2::new(interactable.bounding_box.0.x, interactable.bounding_box.0.y),
                Vec2::new(interactable.bounding_box.1.x, interactable.bounding_box.0.y),
                Vec2::new(interactable.bounding_box.1.x, interactable.bounding_box.1.y),
                Vec2::new(interactable.bounding_box.0.x, interactable.bounding_box.1.y),
            ],
            closed: true,
        };

        let child = commands
            .spawn_bundle(GeometryBuilder::build_as(
                &bounding_mesh,
                DrawMode::Stroke(StrokeMode::new(Color::rgb_u8(red, green, blue), 1.0)),
                Transform::default(),
            ))
            .id();

        commands
            .entity(entity)
            .push_children(&vec![child])
            .insert(DebugInteractable { child });
    }
}

#[cfg(feature = "debug")]
pub fn cleanup_interaction_debug(
    mut commands: Commands,
    removed_interactables: RemovedComponents<Interactable>,
    interactables: Query<(Entity, &DebugInteractable)>,
) {
    for entity in removed_interactables.iter() {
        if let Ok(debug_interactable) = interactables.get_component::<DebugInteractable>(entity) {
            commands.entity(debug_interactable.child).despawn();
        } else {
            warn!("Could not remove interactable debug from entity. Was already despawned?");
        }
    }
}
