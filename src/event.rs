use bevy::prelude::*;

pub struct ConfirmTurnEvent;

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ConfirmTurnEvent>();
    }
}
