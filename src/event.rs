use bevy::prelude::*;

use crate::resources::MoveType;

pub struct ConfirmTurnEvent(pub MoveType);

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ConfirmTurnEvent>();
    }
}
