use bevy::{platform::collections::HashSet, prelude::*, text::FontSmoothing};

use crate::{
    game::{constants::TILE_SIZE, grid::TileGrid},
    game_state::InGameState,
    texture_atlas::{TileSprite, TileSprites},
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
    pub(crate) is_exploded: bool,
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, render_tiles);
}

pub fn tile_on_pointer_click(
    click: On<Pointer<Click>>,
    grid_res: ResMut<TileGrid>,
    mut query: Query<&mut Tile>,
    mut sub_state: ResMut<NextState<InGameState>>,
) {
    match click.button {
        PointerButton::Primary => {
            let (can_open, is_mined) = if let Ok(clicked_tile) = query.get(click.event_target()) {
                (can_open_tile(clicked_tile), clicked_tile.is_mined)
            } else {
                (false, false)
            };
            if !can_open {
                return;
            }
            if is_mined {
                update_tile_states_on_game_over(click.event_target(), &grid_res, &mut query);
                sub_state.set(InGameState::Lost);
                info!("Game lost");
            } else {
                propagate_open_tiles(click.event_target(), &grid_res, &mut query);
                if check_win_condition(&grid_res, query) {
                    sub_state.set(InGameState::Won);
                    info!("Game won");
                }
            }
        }
        PointerButton::Secondary => {
            if let Ok(mut clicked_tile) = query.get_mut(click.event_target())
                && can_flag_tile(&clicked_tile)
            {
                flag_tile(&mut clicked_tile);
            }
        }
        _ => {}
    }
}

fn update_tile_states_on_game_over(
    clicked_entity: Entity,
    grid_res: &ResMut<TileGrid>,
    query: &mut Query<&mut Tile>,
) {
    if let Ok((mut clicked_tile)) = query.get_mut(clicked_entity) {
        clicked_tile.is_exploded = true;
    }
    for i in 0..grid_res.height {
        for j in 0..grid_res.width {
            if let Some(entity) = grid_res.get_tile_handle(j, i)
                && let Ok((mut tile)) = query.get_mut(entity)
            {
                tile.state = TileState::Opened;
            }
        }
    }
}

fn propagate_open_tiles(
    clicked_tile: Entity,
    grid_res: &ResMut<TileGrid>,
    query: &mut Query<&mut Tile>,
) {
    let mut tiles_to_open = vec![(clicked_tile)];
    let mut handled_tiles = HashSet::new();

    while let Some(entity) = tiles_to_open.pop() {
        if let Ok((mut tile)) = query.get_mut(entity) {
            if tile.is_mined || tile.state == TileState::Opened {
                continue;
            }
            open_tile(&mut tile);
            if tile.adjacent_mines == 0 {
                let (tile_x, tile_y) = grid_res.find_tile_coords(entity).unwrap();
                let adjacent_tiles = grid_res.find_surrounding_tile_handles(tile_x, tile_y);
                for e in adjacent_tiles {
                    if !handled_tiles.contains(&e) {
                        tiles_to_open.push(e);
                    }
                }
            }
        }
        handled_tiles.insert(entity);
    }
}

fn can_open_tile(tile: &Tile) -> bool {
    tile.state != TileState::Opened
}

fn open_tile(tile: &mut Tile) {
    tile.state = TileState::Opened;
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

fn flag_tile(tile: &mut Tile) {
    if tile.state == TileState::Unopened {
        tile.state = TileState::Flagged;
    } else if tile.state == TileState::Flagged {
        tile.state = TileState::Unopened;
    }
}

fn check_win_condition(tile_grid: &TileGrid, tile_query: Query<&mut Tile>) -> bool {
    for tile_ref in tile_grid.tiles.iter().flatten() {
        let tile = tile_query.get(*tile_ref).unwrap();
        if tile.state == TileState::Unopened && !tile.is_mined {
            return false;
        }
    }
    true
}

fn render_tiles(
    mut commands: Commands,
    mut tiles: Query<(Entity, &Tile, &mut Sprite), Changed<Tile>>,
    sprites: Res<TileSprites>,
) {
    for (entity, tile, mut sprite) in &mut tiles {
        commands.entity(entity).despawn_children();
        let base_sprite = match (&tile.state, tile.is_mined, tile.is_exploded) {
            (TileState::Unopened, _, _) => TileSprite::Unopened,
            (TileState::Opened, true, true) => TileSprite::Exploded,
            (TileState::Opened, _, _) => TileSprite::Opened,
            (TileState::Flagged, _, _) => TileSprite::Unopened,
        };

        *sprite =
            Sprite::from_atlas_image(sprites.texture_handle.clone(), sprites.get(base_sprite));

        let overlay = match (&tile.state, tile.is_mined) {
            (TileState::Opened, true) => Some(TileSprite::Mine),
            (TileState::Flagged, _) => Some(TileSprite::Flag),
            _ => None,
        };

        if overlay.is_none() && tile.state == TileState::Opened && tile.adjacent_mines > 0 {
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

        if let Some(kind) = overlay {
            commands.entity(entity).with_children(|p| {
                p.spawn((
                    Transform::from_xyz(0., 0., 1.),
                    Sprite::from_atlas_image(sprites.texture_handle.clone(), sprites.get(kind)),
                ));
            });
        }
    }
}
