use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{events::*, resources::*};

/// Play audio based on events.
pub fn handle_audio_event(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    config: Res<Config>,
    mut box_enter_goal_events: EventReader<BoxEnterGoal>,
    mut _box_leave_goal_events: EventReader<BoxLeaveGoal>,
    mut level_solved_events: EventReader<LevelSolved>,
) {
    for _ in level_solved_events.read() {
        audio
            .play(asset_server.load("audio/success.ogg"))
            .with_volume(config.volume);
        box_enter_goal_events.clear();
    }
    for _ in box_enter_goal_events.read() {
        audio
            .play(asset_server.load("audio/correct.ogg"))
            .with_volume(config.volume);
    }
}
