use bevy::prelude::*;
mod tile;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(tile::plugin);
}
