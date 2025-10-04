use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{events::*, resources::*};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, handle_audio_event);
}

/// Play audio based on events.
pub fn handle_audio_event(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    config: Res<Config>,
    mut box_enter_goal_events: MessageReader<BoxEnterGoal>,
    mut _box_leave_goal_events: MessageReader<BoxLeaveGoal>,
    mut level_solved_events: MessageReader<LevelSolved>,
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
