#[allow(unused_imports)]
use bevy::{asset::AssetMetaCheck, prelude::*};
pub struct AppPlugin;

mod audio;
mod camera;
mod game;
mod game_state;
mod texture_atlas;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        #[cfg(target_arch = "wasm32")]
                        canvas: Some("#bevy-canvas".to_string()),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    #[cfg(target_arch = "wasm32")]
                    file_path: "/bMinesweeper/assets".into(),
                    #[cfg(target_arch = "wasm32")]
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
            game_state::plugin,
            camera::plugin,
            game::plugin,
            texture_atlas::plugin,
            audio::plugin,
        ));
    }
}
