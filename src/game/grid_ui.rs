use bevy::{
    app::App,
    color::{Color, palettes::css::WHITE_SMOKE},
    ecs::{
        component::Component,
        observer::On,
        system::{Commands, Res, ResMut},
    },
    math::{Vec2, Vec3},
    picking::{
        Pickable,
        events::{Click, Pointer},
    },
    sprite::{Sprite, Text2d},
    state::state::{NextState, OnEnter},
    text::{TextColor, TextFont},
    transform::components::Transform,
    utils::default,
};

use crate::{
    game::{constants::TILE_SIZE, grid::TileGrid},
    game_state::{GameState, InGameState, OnGameState},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InGame), spawn_restart_button);
}

#[derive(Component)]
struct RestartButton;

fn spawn_restart_button(mut commands: Commands, grid_res: Res<TileGrid>) {
    let grid_top_edge = ((grid_res.height - 1) as f32 * (grid_res.tile_size + grid_res.tile_gap))
        / 2.
        + grid_res.tile_size / 2.;
    let button_size = Vec2::new(grid_res.tile_size * 3., grid_res.tile_size * 1.25);
    let button_y = grid_top_edge + grid_res.tile_size;

    commands
        .spawn((
            OnGameState(GameState::InGame),
            RestartButton,
            Transform::from_xyz(0., button_y, 0.),
            Sprite::from_color(Color::from(WHITE_SMOKE), button_size),
            Pickable::default(),
        ))
        .observe(restart_button_on_pointer_click)
        .with_children(|p| {
            p.spawn((
                Text2d::new("Reset"),
                Transform::from_xyz(0., 0., 1.).with_scale(Vec3::splat(0.2)),
                TextFont {
                    font_size: TILE_SIZE * 4.,
                    ..default()
                },
                TextColor::from(Color::BLACK),
            ));
        });
}

fn restart_button_on_pointer_click(
    _click: On<Pointer<Click>>,
    mut sub_state: ResMut<NextState<InGameState>>,
) {
    sub_state.set(InGameState::Playing);
}
