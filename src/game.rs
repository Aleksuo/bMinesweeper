use bevy::{color::palettes::css::GRAY, prelude::*};

use crate::game_state::{GameState, OnGameState};

#[derive(Resource)]
struct TileGrid {
    height: u32,
    width: u32,
    tile_size: f32,
    tile_gap: f32,
    tiles: Vec<Vec<Entity>>,
}

#[derive(Component)]
struct Tile {
    is_opened: bool,
    is_flagged: bool,
    addjacent_mines: u8,
    is_mined: bool,
}

const TILE_SIZE: f32 = 8.;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(TileGrid {
        height: 8,
        width: 8,
        tile_gap: 1.,
        tile_size: 8.,
        tiles: vec![vec![]],
    })
    .add_systems(OnEnter(GameState::InGame), spawn_grid);
}

fn spawn_grid(mut commands: Commands, mut grid_res: ResMut<TileGrid>) {
    let start_x = -(((grid_res.width as f32 * grid_res.tile_size)
        + ((grid_res.width - 1) as f32 * grid_res.tile_gap))
        / 2.);
    let start_y = -(((grid_res.height as f32 * grid_res.tile_size)
        + ((grid_res.height - 1) as f32 * grid_res.tile_gap))
        / 2.);
    let mut x_coord = start_x;
    let mut y_coord = start_y;
    for i in 0..grid_res.height {
        for j in 0..grid_res.width {
            let entity_handle = commands
                .spawn((
                    OnGameState(GameState::InGame),
                    Transform::from_xyz(x_coord, y_coord, 0.),
                    Sprite::from_color(Color::from(GRAY), Vec2::new(TILE_SIZE, TILE_SIZE)),
                    Tile {
                        is_flagged: false,
                        addjacent_mines: 0,
                        is_mined: false,
                        is_opened: false,
                    },
                ))
                .id();
            grid_res.tiles[i as usize].push(entity_handle);
            x_coord += grid_res.tile_size + grid_res.tile_gap;
        }
        x_coord = start_x;
        y_coord += grid_res.tile_size + grid_res.tile_gap;
        grid_res.tiles.push(Vec::new());
    }
}
