use bevy::prelude::Plugin;
use big_brain::BigBrainPlugin;

pub struct JaipurAiPlugin;

impl Plugin for JaipurAiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(BigBrainPlugin);
    }
}
