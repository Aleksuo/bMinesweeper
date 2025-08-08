use bevy::prelude::*;
pub struct AppPlugin;

mod game_state;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPlugins, game_state::plugin));
    }
}
