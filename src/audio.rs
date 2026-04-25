use bevy::{platform::collections::HashMap, prelude::*};
use bevy_kira_audio::prelude::*;

#[derive(Resource)]
pub struct SoundPlayer {
    sound_map: HashMap<PlaySoundMessage, Handle<AudioSource>>,
}

impl SoundPlayer {
    fn new(sound_map: HashMap<PlaySoundMessage, Handle<AudioSource>>) -> Self {
        Self { sound_map }
    }
}

#[derive(Message, PartialEq, Eq, Hash)]
pub enum PlaySoundMessage {
    ClickDown,
    ClickUp,
    MineExplosion,
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(AudioPlugin)
        .add_message::<PlaySoundMessage>()
        .add_systems(Startup, load_sound_assets)
        .add_systems(Update, on_play_sound_event);
}

fn load_sound_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let click_down_sound = asset_server.load::<AudioSource>("sounds/click_down.wav");
    let click_up_sound = asset_server.load::<AudioSource>("sounds/click_up.wav");
    let mine_explosion_sound = asset_server.load::<AudioSource>("sounds/mine_explosion.wav");
    let mut sound_map = HashMap::new();
    sound_map.insert(PlaySoundMessage::ClickDown, click_down_sound);
    sound_map.insert(PlaySoundMessage::ClickUp, click_up_sound);
    sound_map.insert(PlaySoundMessage::MineExplosion, mine_explosion_sound);

    commands.insert_resource(SoundPlayer::new(sound_map));
}

fn on_play_sound_event(
    mut event_reader: MessageReader<PlaySoundMessage>,
    sound_player: Res<SoundPlayer>,
    audio: Res<Audio>,
) {
    for event in event_reader.read() {
        if let Some(sound_handle) = sound_player.sound_map.get(event) {
            audio.play(sound_handle.clone());
        } else {
            error!("Loading sound handle failed");
        }
    }
}
