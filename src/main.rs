// #![feature(test)]

mod level;
use level::*;

mod systems;
use systems::input::*;
use systems::level::*;
use systems::render::*;
use systems::ui::*;

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
use leafwing_input_manager::prelude::*;

#[allow(unused_imports)]
use bevy_editor_pls::prelude::*;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin,
            PerformanceMatrixPlugin,
            InputManagerPlugin::<Action>::default(),
            // EditorPlugin::default(),
        ))
        .add_systems(PreStartup, (setup_camera, setup_database))
        .add_systems(
            Startup,
            (
                setup_window,
                setup_version_info,
                setup_button,
                setup_hud,
                setup_level,
            ),
        )
        .add_systems(PostStartup, spawn_board)
        .add_systems(
            Update,
            (
                button_visual_effect,
                update_button_state,
                button_pressed,
                update_grid_position_from_board,
                select_crate,
                unselect_crate,
                (
                    action_input,
                    adjust_viewport,
                    mouse_input,
                    check_level_solved,
                    spawn_board,
                )
                    .chain(),
                update_hud,
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
        .add_event::<SelectCrate>()
        .add_event::<UnselectCrate>()
        .add_event::<UpdateGridPositionEvent>()
        .insert_resource(Settings::default())
        .init_resource::<ActionState<Action>>()
        .insert_resource(Action::input_map())
        .insert_resource(PlayerMovement::default())
        .insert_resource(CrateReachable::default())
        .run();
}
