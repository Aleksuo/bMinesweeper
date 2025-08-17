use bevy::{
    color::palettes::css::{BLUE, GRAY, WHITE_SMOKE},
    prelude::*,
};

use crate::game_state::{GameState, OnGameState};

#[derive(PartialEq)]
enum TileState {
    Unopened,
    Opened,
    Flagged,
}

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
    state: TileState,
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
        for _ in 0..grid_res.width {
            let entity_handle = commands
                .spawn((
                    OnGameState(GameState::InGame),
                    Transform::from_xyz(x_coord, y_coord, 0.),
                    Sprite::from_color(Color::from(GRAY), Vec2::new(TILE_SIZE, TILE_SIZE)),
                    Tile {
                        state: TileState::Unopened,
                        addjacent_mines: 0,
                        is_mined: false,
                    },
                    Pickable::default(),
                ))
                .observe(tile_on_pointer_click)
                .id();
            grid_res.tiles[i as usize].push(entity_handle);
            x_coord += grid_res.tile_size + grid_res.tile_gap;
        }
        x_coord = start_x;
        y_coord += grid_res.tile_size + grid_res.tile_gap;
        grid_res.tiles.push(Vec::new());
    }
}

fn tile_on_pointer_click(
    click: Trigger<Pointer<Click>>,
    mut query: Query<(&mut Tile, &mut Sprite)>,
) {
    let Ok((mut tile, mut sprite)) = query.get_mut(click.target) else {
        return;
    };

    match click.button {
        PointerButton::Primary => {
            if can_open_tile(&tile) {
                open_tile(&mut tile, &mut sprite);
            }
        }
        PointerButton::Secondary => {
            if can_flag_tile(&tile) {
                flag_tile(&mut tile, &mut sprite);
            }
        }
        _ => {}
    }
}

fn can_open_tile(tile: &Tile) -> bool {
    return tile.state != TileState::Opened;
}

fn open_tile(tile: &mut Tile, sprite: &mut Sprite) {
    tile.state = TileState::Opened;
    sprite.color = Color::from(WHITE_SMOKE);
}

fn can_flag_tile(tile: &Tile) -> bool {
    return tile.state != TileState::Opened;
}

fn flag_tile(tile: &mut Tile, sprite: &mut Sprite) {
    if tile.state == TileState::Unopened {
        tile.state = TileState::Flagged;
        sprite.color = Color::from(BLUE)
    } else if tile.state == TileState::Flagged {
        tile.state = TileState::Unopened;
        sprite.color = Color::from(GRAY)
    }
}
