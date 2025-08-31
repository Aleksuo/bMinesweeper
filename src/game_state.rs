use bevy::prelude::*;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
#[allow(dead_code)]
pub enum GameState {
    MainMenu,
    SelectLevel,
    #[default]
    InGame,
    GameOver,
    Settings,
}

#[derive(Component)]
pub struct OnGameState(pub GameState);

pub(super) fn plugin(app: &mut App) {
    app.init_state::<GameState>().add_systems(
        PreUpdate,
        despawn_not_in_state.run_if(resource_changed::<NextState<GameState>>),
    );
}

fn despawn_not_in_state(
    mut commands: Commands,
    query: Query<(Entity, &OnGameState)>,
    next_state: Res<NextState<GameState>>,
) {
    let NextState::Pending(target) = *next_state else {
        return;
    };
    for (entity, on_state) in query.iter() {
        if on_state.0 != target {
            commands.entity(entity).despawn();
        }
    }
}
