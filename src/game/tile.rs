use bevy::{
    color::palettes::css::{BLUE, GRAY, RED, WHITE_SMOKE},
    platform::collections::HashSet,
    prelude::*,
    text::FontSmoothing,
};

use crate::{
    game::{constants::TILE_SIZE, grid::TileGrid},
    game_state::InGameState,
};

#[derive(PartialEq)]
pub enum TileState {
    Unopened,
    Opened,
    Flagged,
}

#[derive(Component)]
pub struct Tile {
    pub(crate) state: TileState,
    pub(crate) adjacent_mines: u32,
    pub(crate) is_mined: bool,
}

pub(super) fn plugin(_app: &mut App) {}

pub fn tile_on_pointer_click(
    click: On<Pointer<Click>>,
    grid_res: ResMut<TileGrid>,
    mut query: Query<(&mut Tile, &mut Sprite)>,
    mut commands: Commands,
    mut sub_state: ResMut<NextState<InGameState>>,
) {
    match click.button {
        PointerButton::Primary => {
            let (can_open, is_mined) =
                if let Ok((clicked_tile, _)) = query.get(click.event_target()) {
                    (can_open_tile(clicked_tile), clicked_tile.is_mined)
                } else {
                    (false, false)
                };
            if !can_open {
                return;
            }
            if is_mined {
                for i in 0..grid_res.height {
                    for j in 0..grid_res.width {
                        if let Some(entity) = grid_res.get_tile_handle(j, i)
                            && let Ok((mut tile, mut sprite)) = query.get_mut(entity)
                        {
                            open_tile(&mut tile, &mut sprite, entity, &mut commands);
                        }
                    }
                }
                sub_state.set(InGameState::Lost);
                info!("Game lost");
            } else {
                let mut tiles_to_open = vec![(click.event_target())];
                let mut handled_tiles = HashSet::new();

                while let Some(entity) = tiles_to_open.pop() {
                    if let Ok((mut tile, mut sprite)) = query.get_mut(entity) {
                        if tile.is_mined || tile.state == TileState::Opened {
                            continue;
                        }
                        open_tile(&mut tile, &mut sprite, entity, &mut commands);
                        if tile.adjacent_mines == 0 {
                            let (tile_x, tile_y) = grid_res.find_tile_coords(entity).unwrap();
                            let adjacent_tiles =
                                grid_res.find_surrounding_tile_handles(tile_x, tile_y);
                            for e in adjacent_tiles {
                                if !handled_tiles.contains(&e) {
                                    tiles_to_open.push(e);
                                }
                            }
                        }
                    }
                    handled_tiles.insert(entity);
                }
                if check_win_condition(&grid_res, query) {
                    sub_state.set(InGameState::Won);
                    info!("Game won");
                }
            }
        }
        PointerButton::Secondary => {
            if let Ok((mut clicked_tile, mut clicked_sprite)) = query.get_mut(click.event_target())
                && can_flag_tile(&clicked_tile)
            {
                flag_tile(&mut clicked_tile, &mut clicked_sprite);
            }
        }
        _ => {}
    }
}

fn can_open_tile(tile: &Tile) -> bool {
    tile.state != TileState::Opened
}

fn open_tile(tile: &mut Tile, sprite: &mut Sprite, entity: Entity, commands: &mut Commands) {
    tile.state = TileState::Opened;
    if tile.is_mined {
        sprite.color = Color::from(RED);
    } else {
        sprite.color = Color::from(WHITE_SMOKE);
        if tile.adjacent_mines > 0 {
            commands.entity(entity).with_children(|p| {
                p.spawn((
                    Text2d::new(format!("{:?}", tile.adjacent_mines)),
                    Transform::from_xyz(0., 0., 1.).with_scale(Vec3::splat(0.2)),
                    TextFont {
                        font_size: TILE_SIZE * 4.,
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
    tile.state != TileState::Opened
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

fn check_win_condition(tile_grid: &TileGrid, tile_query: Query<(&mut Tile, &mut Sprite)>) -> bool {
    for tile_ref in tile_grid.tiles.iter().flatten() {
        let (tile, _) = tile_query.get(*tile_ref).unwrap();
        if tile.state == TileState::Unopened && !tile.is_mined {
            return false;
        }
    }
    true
}
