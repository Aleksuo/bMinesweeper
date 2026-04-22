use bevy::{
    app::{App, Update},
    asset::{AssetServer, Assets, Handle, LoadState, UntypedAssetId, UntypedHandle},
    ecs::{
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, Res, ResMut},
    },
    image::{Image, TextureAtlas, TextureAtlasLayout},
    log::info,
    math::UVec2,
    state::{
        condition::in_state,
        state::{NextState, OnEnter},
    },
};

use crate::game_state::GameState;

pub enum TileSprite {
    PressedUnopened,
    Unopened,
    Opened,
    Exploded,
    Flag,
    Mine,
    WrongFlag,
}

#[derive(Resource)]
pub struct TileSprites {
    pub texture_handle: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
}

impl TileSprites {
    pub fn get(&self, kind: TileSprite) -> TextureAtlas {
        TextureAtlas {
            layout: self.atlas_layout.clone(),
            index: kind as usize,
        }
    }
}

pub enum RetryButtonSprite {
    Unpressed,
    Pressed,
}

#[derive(Resource)]
pub struct RetryButtonSprites {
    pub texture_handle: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
}

impl RetryButtonSprites {
    pub fn get(&self, kind: RetryButtonSprite) -> TextureAtlas {
        TextureAtlas {
            layout: self.atlas_layout.clone(),
            index: kind as usize,
        }
    }
}

#[derive(Resource)]
struct AssetsLoading(Vec<UntypedHandle>);

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(AssetsLoading(vec![]))
        .add_systems(OnEnter(GameState::LoadingAssets), setup)
        .add_systems(
            Update,
            check_assets_ready.run_if(in_state(GameState::LoadingAssets)),
        );
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut loading: ResMut<AssetsLoading>,
) {
    let texture: Handle<Image> = asset_server.load("minesweeper-tiles.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(8), 6, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    loading.0.push(texture.clone().untyped());

    commands.insert_resource(TileSprites {
        texture_handle: texture,
        atlas_layout: texture_atlas_layout,
    });

    let retry_texture: Handle<Image> = asset_server.load("retry-button.png");
    let retry_layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 2, 1, None, None);
    let retry_atlas_layout = texture_atlas_layouts.add(retry_layout);

    loading.0.push(retry_texture.clone().untyped());

    commands.insert_resource(RetryButtonSprites {
        texture_handle: retry_texture,
        atlas_layout: retry_atlas_layout,
    });
}

fn check_assets_ready(
    mut commands: Commands,
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    match get_group_load_state(&server, loading.0.iter().map(|handle| handle.id())) {
        LoadState::Failed(_) => {
            panic!("loading assets failed")
        }
        LoadState::Loaded => {
            commands.remove_resource::<AssetsLoading>();
            game_state.set(GameState::InGame);
        }
        _ => {
            info!("Assets loading")
        }
    }
}

fn get_group_load_state(
    server: &AssetServer,
    handles: impl IntoIterator<Item = UntypedAssetId>,
) -> LoadState {
    let mut load_state = LoadState::Loaded;
    for handle_id in handles {
        match server.get_load_state(handle_id) {
            Some(LoadState::Loaded) => continue,
            Some(LoadState::Loading) => {
                load_state = LoadState::Loading;
            }
            Some(LoadState::Failed(x)) => return LoadState::Failed(x),
            Some(LoadState::NotLoaded) => return LoadState::NotLoaded,
            None => return LoadState::NotLoaded,
        }
    }

    load_state
}
