use crate::{
    state::*,
    systems::{input::*, ui::*},
};

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                button_visual_effect,
                update_button_state,
                (button_input_to_action, handle_actions).chain(),
            ),
        );

        app.add_systems(Update, update_hud.run_if(in_state(AppState::Main)));
    }
}
