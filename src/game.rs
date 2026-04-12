use bevy::prelude::*;
mod constants;
mod grid;
mod grid_ui;
mod tile;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((tile::plugin, grid_ui::plugin, grid::plugin));
}
