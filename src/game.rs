use bevy::{
    color::palettes::css::{BLUE, GRAY, RED, WHITE_SMOKE},
    prelude::*,
    text::FontSmoothing,
};

use rand::Rng;

use crate::game_state::{GameState, OnGameState};

#[derive(PartialEq)]
enum TileState {
    Unopened,
    Opened,
    Flagged,
}

#[derive(Resource)]
struct TileGrid {
    max_mines: u32,
    remaining_mines: u32,
    height: i32,
    width: i32,
    tile_size: f32,
    tile_gap: f32,
    tiles: Vec<Vec<Entity>>,
}

impl TileGrid {
    fn find_surrounding_tile_handles(&self, x: i32, y: i32) -> Vec<Entity> {
        let mut surrounding_tiles = vec![];
        for i in -1..=1 {
            for j in -1..=1 {
                if j == 0 && i == 0 {
                    continue;
                }
                let tile_x = x + j;
                let tile_y = y + i;
                if let Some(handle) = self.get_tile_handle(tile_x, tile_y) {
                    surrounding_tiles.push(handle);
                }
            }
        }
        surrounding_tiles
    }

    fn get_tile_handle(&self, x: i32, y: i32) -> Option<Entity> {
        if (x < 0 || x >= self.width) || (y < 0 || y >= self.height) {
            return None;
        }
        Some(self.tiles[y as usize][x as usize])
    }
}

#[derive(Component)]
struct Tile {
    state: TileState,
    adjacent_mines: u32,
    is_mined: bool,
}

const TILE_SIZE: f32 = 8.;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(TileGrid {
        max_mines: 9,
        remaining_mines: 9,
        height: 8,
        width: 8,
        tile_gap: 1.,
        tile_size: 8.,
        tiles: vec![vec![]],
    })
    .add_systems(
        OnEnter(GameState::InGame),
        (spawn_grid, spawn_mines_on_grid, calculate_adjacent_mines).chain(),
    );
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
                        adjacent_mines: 0,
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

fn spawn_mines_on_grid(grid_res: ResMut<TileGrid>, mut tile_query: Query<&mut Tile>) {
    let mut spawnable_tiles = {
        let mut coords: Vec<(i32, i32)> = Vec::new();
        for i in 0..grid_res.height {
            for j in 0..grid_res.width {
                coords.push((i, j));
            }
        }
        coords
    };

    let mut remaining_placements = grid_res.max_mines;
    let mut rng = rand::rng();
    while remaining_placements > 0 {
        let next_selection = rng.random_range(0..(spawnable_tiles.len() - 1));
        let selection = spawnable_tiles.get(next_selection).unwrap().clone();
        spawnable_tiles.remove(next_selection);
        let entity_handle = grid_res.tiles[selection.0 as usize][selection.1 as usize];

        let Ok(mut tile) = tile_query.get_mut(entity_handle) else {
            error!("Tried to spawn a mine on a tile that does not exist!");
            return;
        };
        tile.is_mined = true;

        remaining_placements -= 1;
    }
}

fn calculate_adjacent_mines(grid_res: ResMut<TileGrid>, mut tile_query: Query<&mut Tile>) {
    for i in 0..grid_res.height {
        for j in 0..grid_res.width {
            let adjacent_mine_count: u32 = grid_res
                .find_surrounding_tile_handles(j, i)
                .iter()
                .map(|handle| -> u32 {
                    if let Ok(tile) = tile_query.get(*handle) {
                        return tile.is_mined as u32;
                    } else {
                        return 0;
                    }
                })
                .sum();
            if let Ok(mut tile) = tile_query.get_mut(grid_res.get_tile_handle(j, i).unwrap()) {
                tile.adjacent_mines = adjacent_mine_count;
                info!(j, i, adjacent_mine_count);
            }
        }
    }
}

fn tile_on_pointer_click(
    click: Trigger<Pointer<Click>>,
    mut query: Query<(&mut Tile, &mut Sprite)>,
    commands: Commands,
) {
    let Ok((mut tile, mut sprite)) = query.get_mut(click.target) else {
        return;
    };

    match click.button {
        PointerButton::Primary => {
            if can_open_tile(&tile) {
                open_tile(&mut tile, &mut sprite, click.target, commands);
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

fn open_tile(tile: &mut Tile, sprite: &mut Sprite, entity: Entity, mut commands: Commands) {
    tile.state = TileState::Opened;
    if (tile.is_mined) {
        sprite.color = Color::from(RED);
    } else {
        sprite.color = Color::from(WHITE_SMOKE);
        if tile.adjacent_mines > 0 {
            // maybe render the text here?
            commands.entity(entity).with_children(|p| {
                p.spawn((
                    Text2d::new(format!("{:?}", tile.adjacent_mines)),
                    Transform::from_xyz(0., 0., 1.),
                    TextFont {
                        font_size: TILE_SIZE * 1.1,
                        font_smoothing: FontSmoothing::None,
                        ..default()
                    },
                    TextColor::from(get_adjacent_text_color(tile.adjacent_mines)),
                ));
            });
        }
    }
}

fn get_adjacent_text_color(count: u32) -> Srgba {
    let text_color = match count {
        1 => Srgba::hex("#0000FF"),
        2 => Srgba::hex("#008000"),
        3 => Srgba::hex("#FF0000"),
        4 => Srgba::hex("#000080"),
        5 => Srgba::hex("#800000"),
        6 => Srgba::hex("#008080"),
        7 => Srgba::hex("#000000"),
        _ => Srgba::hex("#808080"),
    };
    text_color.unwrap()
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
