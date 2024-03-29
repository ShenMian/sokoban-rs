use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::events::*;
use crate::resources::*;

/// Play audio based on events.
pub fn handle_audio_event(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    config: Res<Config>,
    mut crate_enter_target_events: EventReader<CrateEnterTarget>,
    mut _crate_leave_target_events: EventReader<CrateLeaveTarget>,
    mut level_solved_events: EventReader<LevelSolved>,
) {
    for _ in level_solved_events.read() {
        audio
            .play(asset_server.load("audio/success.ogg"))
            .with_volume(config.volume);
        crate_enter_target_events.clear();
    }
    for _ in crate_enter_target_events.read() {
        audio
            .play(asset_server.load("audio/correct.ogg"))
            .with_volume(config.volume);
    }
}
