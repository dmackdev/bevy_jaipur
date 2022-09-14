use bevy::prelude::SystemLabel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum Label {
    ConfirmTurnEventWriter,
    ConfirmTurnEventReader,
}
