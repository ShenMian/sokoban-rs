// #![feature(test)]
#![allow(clippy::op_ref)]

mod board;
mod components;
mod database;
mod events;
mod plugins;
mod resources;
mod settings;
mod solve;
mod state;
mod systems;
mod test;
mod utils;

use bevy_enhanced_input::prelude::*;
use events::*;
use plugins::*;
use resources::*;
use state::*;
use systems::{input::*, level::*, render::*};
use utils::*;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

#[bevy_main]
fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sokoban".to_string(),
                ..default()
            }),
            ..default()
        }),
        AudioPlugin,
        EnhancedInputPlugin,
    ))
    .add_input_context::<ControlsContext>()
    .add_observer(on_zoom)
    .add_observer(on_toggle_instant_move)
    .add_observer(on_toggle_automatic_solution)
    .add_observer(on_toggle_fullscreen)
    .add_observer(on_reset_level)
    .add_observer(on_next_level)
    .add_observer(on_previous_level)
    .add_observer(on_next_unsolved_level)
    .add_observer(on_previous_unsolved_level)
    .add_observer(on_import_levels)
    .add_observer(on_export_level)
    .add_observer(on_move_up)
    .add_observer(on_move_down)
    .add_observer(on_move_left)
    .add_observer(on_move_right)
    .add_observer(on_undo)
    .add_observer(on_redo)
    .init_state::<AppState>();

    app.add_systems(PreStartup, (setup_camera, setup_database));
    app.add_systems(Startup, (set_windows_icon, setup_level, setup_controls));
    app.add_systems(FixedUpdate, animate_player);
    app.add_systems(
        Update,
        (
            (
                mouse_input,
                auto_switch_to_next_unsolved_level.run_if(on_message::<LevelSolved>),
                spawn_board.run_if(resource_changed_or_removed::<LevelId>),
            )
                .chain(),
            update_grid_position_from_board.run_if(on_message::<UpdateGridPositionEvent>),
            file_drag_and_drop,
        )
            .run_if(in_state(AppState::Main)),
    )
    .add_systems(
        FixedUpdate,
        (handle_player_movement, smooth_tile_motion).run_if(in_state(AppState::Main)),
    );

    app.add_plugins((
        performance_matrix::plugin,
        version_information::plugin,
        ui::plugin,
        audio::plugin,
        config::plugin,
        camera::plugin,
        auto_move::plugin,
        auto_solve::plugin,
    ));

    app.add_message::<BoxEnterGoal>()
        .add_message::<BoxLeaveGoal>()
        .add_message::<LevelSolved>()
        .add_message::<UpdateGridPositionEvent>();

    app.run();
}
