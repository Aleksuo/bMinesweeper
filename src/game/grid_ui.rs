use bevy::{
    app::App,
    ecs::{
        component::Component,
        observer::On,
        query::With,
        system::{Commands, Res, ResMut, Single},
    },
    math::Vec2,
    picking::{
        Pickable,
        events::{Click, DragEnd, Pointer, Press},
    },
    sprite::Sprite,
    state::state::{NextState, OnEnter},
    transform::components::Transform,
};

use crate::{
    game::grid::TileGrid,
    game_state::{GameState, InGameState, OnGameState},
    texture_atlas::{RetryButtonSprite, RetryButtonSprites},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InGame), spawn_restart_button);
}

#[derive(Component)]
struct RestartButton;

fn spawn_restart_button(
    mut commands: Commands,
    grid_res: Res<TileGrid>,
    retry_sprites: Res<RetryButtonSprites>,
) {
    let grid_top_edge = ((grid_res.height - 1) as f32 * (grid_res.tile_size + grid_res.tile_gap))
        / 2.
        + grid_res.tile_size / 2.;
    let button_size = Vec2::splat(grid_res.tile_size * 1.5);
    let button_y = grid_top_edge + grid_res.tile_size;

    let mut sprite = Sprite::from_atlas_image(
        retry_sprites.texture_handle.clone(),
        retry_sprites.get(RetryButtonSprite::Unpressed),
    );
    sprite.custom_size = Some(button_size);

    commands
        .spawn((
            OnGameState(GameState::InGame),
            RestartButton,
            Transform::from_xyz(0., button_y, 0.),
            sprite,
            Pickable::default(),
        ))
        .observe(restart_button_on_pointer_press)
        .observe(restart_button_on_pointer_click)
        .observe(restart_button_on_drag_end);
}

fn restart_button_on_pointer_press(
    _press: On<Pointer<Press>>,
    mut restart_sprite: Single<&mut Sprite, With<RestartButton>>,
    retry_sprites: Res<RetryButtonSprites>,
) {
    restart_sprite.texture_atlas = Some(retry_sprites.get(RetryButtonSprite::Pressed));
}

fn restart_button_on_pointer_click(
    _click: On<Pointer<Click>>,
    mut sub_state: ResMut<NextState<InGameState>>,
    mut restart_sprite: Single<&mut Sprite, With<RestartButton>>,
    retry_sprites: Res<RetryButtonSprites>,
) {
    restart_sprite.texture_atlas = Some(retry_sprites.get(RetryButtonSprite::Unpressed));
    sub_state.set(InGameState::Playing);
}

fn restart_button_on_drag_end(
    _drag_end: On<Pointer<DragEnd>>,
    mut restart_sprite: Single<&mut Sprite, With<RestartButton>>,
    retry_sprites: Res<RetryButtonSprites>,
) {
    restart_sprite.texture_atlas = Some(retry_sprites.get(RetryButtonSprite::Unpressed));
}
