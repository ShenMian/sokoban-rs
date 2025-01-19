use crate::{
    resources::AutoMoveState,
    state::*,
    systems::{auto_move::*, input::*},
};

use bevy::prelude::*;

pub struct AutoMovePlugin;

impl Plugin for AutoMovePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::AutoMove), spawn_auto_move_marks)
            .add_systems(Update, mouse_input.run_if(in_state(AppState::AutoMove)))
            .add_systems(OnExit(AppState::AutoMove), cleanup_sprite_color);
        app.insert_resource(AutoMoveState::default());
    }
}
