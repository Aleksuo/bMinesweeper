use bevy::{
    app::App,
    ecs::{
        entity::Entity,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, Res, ResMut},
    },
    log::error,
    math::Vec2,
    picking::Pickable,
    sprite::{Sprite, SpriteImageMode},
    state::state::OnEnter,
    transform::components::Transform,
};
use rand::Rng;

use crate::{
    game::tile::{Tile, TileState, tile_on_drag_end, tile_on_pointer_click, tile_on_pointer_press},
    game_state::{GameState, InGameState, OnGameState},
    texture_atlas::{GridBorderSprite, GridBorderSprites, TileSprite, TileSprites},
};

#[derive(Resource)]
pub struct TileGrid {
    max_mines: u32,
    pub(crate) height: i32,
    pub(crate) width: i32,
    pub(crate) tile_size: f32,
    pub(crate) tile_gap: f32,
    pub(crate) tiles: Vec<Vec<Entity>>,
}

impl TileGrid {
    pub fn find_surrounding_tile_handles(&self, x: i32, y: i32) -> Vec<Entity> {
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

    pub fn get_tile_handle(&self, x: i32, y: i32) -> Option<Entity> {
        if (x < 0 || x >= self.width) || (y < 0 || y >= self.height) {
            return None;
        }
        Some(self.tiles[y as usize][x as usize])
    }

    pub fn find_tile_coords(&self, entity: Entity) -> Option<(i32, i32)> {
        for i in 0..self.height {
            for j in 0..self.width {
                let entity_at_coord = self.get_tile_handle(j, i).unwrap();
                if entity == entity_at_coord {
                    return Some((j, i));
                }
            }
        }
        None
    }
}

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(TileGrid {
        max_mines: 8,
        height: 8,
        width: 8,
        tile_gap: 0.16,
        tile_size: 8.,
        tiles: vec![],
    })
    .add_systems(
        OnEnter(InGameState::Playing),
        ((
            reset_grid,
            spawn_grid,
            spawn_grid_borders,
            spawn_mines_on_grid,
            calculate_adjacent_mines,
        )
            .chain(),),
    );
}

fn reset_grid(mut commands: Commands, mut grid_res: ResMut<TileGrid>) {
    for e in grid_res.tiles.iter().flatten() {
        commands.entity(*e).despawn();
    }
    grid_res.tiles.clear();
}

fn spawn_grid(mut commands: Commands, mut grid_res: ResMut<TileGrid>, sprites: Res<TileSprites>) {
    grid_res.tiles = vec![];
    let start_x = -(((grid_res.width - 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);
    let start_y = -(((grid_res.height - 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);
    let mut x_coord = start_x;
    let mut y_coord = start_y;
    for i in 0..grid_res.height {
        grid_res.tiles.push(Vec::new());
        for _ in 0..grid_res.width {
            let entity_handle = commands
                .spawn((
                    OnGameState(GameState::InGame),
                    Transform::from_xyz(x_coord, y_coord, 0.),
                    Sprite::from_atlas_image(
                        sprites.texture_handle.clone(),
                        sprites.get(TileSprite::Unopened),
                    ),
                    Tile {
                        is_exploded: false,
                        state: TileState::Unopened,
                        adjacent_mines: 0,
                        is_mined: false,
                        show_incorrect: false,
                    },
                    Pickable::default(),
                ))
                .observe(tile_on_pointer_click)
                .observe(tile_on_pointer_press)
                .observe(tile_on_drag_end)
                .id();
            grid_res.tiles[i as usize].push(entity_handle);
            x_coord += grid_res.tile_size + grid_res.tile_gap;
        }
        x_coord = start_x;
        y_coord += grid_res.tile_size + grid_res.tile_gap;
    }
}

fn spawn_grid_borders(
    mut commands: Commands,
    sprites: Res<GridBorderSprites>,
    grid_res: Res<TileGrid>,
) {
    // Left border
    let left_start_x =
        -(((grid_res.width + 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);
    let border_height =
        grid_res.height as f32 * (grid_res.tile_size + grid_res.tile_gap) + 2. * grid_res.tile_gap;
    commands.spawn((
        OnGameState(GameState::InGame),
        Transform::from_xyz(left_start_x, 0., 0.),
        Sprite {
            image_mode: SpriteImageMode::Tiled {
                tile_y: true,
                tile_x: false,
                stretch_value: 1.0,
            },
            custom_size: Some(Vec2::new(grid_res.tile_size, border_height)),
            ..Sprite::from_atlas_image(
                sprites.texture_handle.clone(),
                sprites.get(GridBorderSprite::Left),
            )
        },
    ));

    let right_start_x =
        ((grid_res.width + 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.;

    commands.spawn((
        OnGameState(GameState::InGame),
        Transform::from_xyz(right_start_x, 0., 0.),
        Sprite {
            image_mode: SpriteImageMode::Tiled {
                tile_y: true,
                tile_x: false,
                stretch_value: 1.0,
            },
            custom_size: Some(Vec2::new(grid_res.tile_size, border_height)),
            ..Sprite::from_atlas_image(
                sprites.texture_handle.clone(),
                sprites.get(GridBorderSprite::Right),
            )
        },
    ));

    let border_width =
        grid_res.width as f32 * (grid_res.tile_size + grid_res.tile_gap) + 2. * grid_res.tile_gap;
    let bottom_start_y =
        -(((grid_res.height + 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);

    commands.spawn((
        OnGameState(GameState::InGame),
        Transform::from_xyz(0., bottom_start_y, 0.),
        Sprite {
            image_mode: SpriteImageMode::Tiled {
                tile_y: false,
                tile_x: true,
                stretch_value: 1.0,
            },
            custom_size: Some(Vec2::new(border_width, grid_res.tile_size)),
            ..Sprite::from_atlas_image(
                sprites.texture_handle.clone(),
                sprites.get(GridBorderSprite::Bottom),
            )
        },
    ));

    let bottom_left_y =
        -(((grid_res.height + 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);
    let bottom_left_x =
        -(((grid_res.width + 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);

    commands.spawn((
        OnGameState(GameState::InGame),
        Transform::from_xyz(bottom_left_x, bottom_left_y, 0.),
        Sprite::from_atlas_image(
            sprites.texture_handle.clone(),
            sprites.get(GridBorderSprite::BottomLeft),
        ),
    ));

    let bottom_right_y =
        -(((grid_res.height + 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);
    let bottom_right_x =
        (((grid_res.width + 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);

    commands.spawn((
        OnGameState(GameState::InGame),
        Transform::from_xyz(bottom_right_x, bottom_right_y, 0.),
        Sprite::from_atlas_image(
            sprites.texture_handle.clone(),
            sprites.get(GridBorderSprite::BottomRight),
        ),
    ));

    let top_start_y =
        (((grid_res.height + 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);

    commands.spawn((
        OnGameState(GameState::InGame),
        Transform::from_xyz(0., top_start_y, 0.),
        Sprite {
            image_mode: SpriteImageMode::Tiled {
                tile_y: false,
                tile_x: true,
                stretch_value: 1.0,
            },
            custom_size: Some(Vec2::new(border_width, grid_res.tile_size)),
            ..Sprite::from_atlas_image(
                sprites.texture_handle.clone(),
                sprites.get(GridBorderSprite::Top),
            )
        },
    ));

    let top_right_y =
        (((grid_res.height + 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);
    let top_right_x =
        (((grid_res.width + 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);

    commands.spawn((
        OnGameState(GameState::InGame),
        Transform::from_xyz(top_right_x, top_right_y, 0.),
        Sprite::from_atlas_image(
            sprites.texture_handle.clone(),
            sprites.get(GridBorderSprite::TopRight),
        ),
    ));

    let top_left_y =
        (((grid_res.height + 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);
    let top_left_x =
        -(((grid_res.width + 1) as f32 * (grid_res.tile_size + grid_res.tile_gap)) / 2.);

    commands.spawn((
        OnGameState(GameState::InGame),
        Transform::from_xyz(top_left_x, top_left_y, 0.),
        Sprite::from_atlas_image(
            sprites.texture_handle.clone(),
            sprites.get(GridBorderSprite::TopLeft),
        ),
    ));
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
        let next_selection = rng.random_range(0..spawnable_tiles.len());
        let selection = *spawnable_tiles.get(next_selection).unwrap();
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
                        tile.is_mined as u32
                    } else {
                        0
                    }
                })
                .sum();
            if let Ok(mut tile) = tile_query.get_mut(grid_res.get_tile_handle(j, i).unwrap()) {
                tile.adjacent_mines = adjacent_mine_count;
            }
        }
    }
}
