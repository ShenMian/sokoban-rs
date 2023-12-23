// #![feature(test)]

mod level;
use level::*;

mod systems;
use systems::input::*;
use systems::level::*;
use systems::render::*;

mod plugins;
use plugins::performance_matrix::*;

mod resources;
use resources::*;

mod events;
use events::*;

mod board;
mod components;
mod database;
mod direction;
mod movement;
mod solver;
mod test;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

#[allow(unused_imports)]
use bevy_editor_pls::prelude::*;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin,
            PerformanceMatrixPlugin,
            // EditorPlugin::default(),
        ))
        .add_systems(PreStartup, (setup_database, setup_camera))
        .add_systems(
            Startup,
            (setup_window, setup_version_info, setup_hud, setup_level),
        )
        .add_systems(PostStartup, spawn_board)
        .add_systems(
            Update,
            (
                spawn_crate_reachable_marks,
                despawn_crate_reachable_marks,
                update_grid_position_from_board,
                keyboard_input,
                gamepad_input,
                mouse_input,
                mouse_drag,
                update_hud,
                check_level_solved,
                spawn_board,
                file_drag_and_drop,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                animate_tiles_movement,
                animate_player_movement,
                animate_camera_zoom,
            ),
        )
        .add_event::<UpdateGridPositionEvent>()
        .add_event::<SpawnCrateReachableMarks>()
        .add_event::<DespawnCrateReachableMarks>()
        .insert_resource(PlayerMovement::default())
        .insert_resource(CrateReachable::default())
        .run();
}
