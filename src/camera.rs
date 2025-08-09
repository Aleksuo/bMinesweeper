use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_camera);
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
