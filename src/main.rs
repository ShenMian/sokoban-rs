// #![feature(test)]

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use std::collections::VecDeque;
use std::fs;
use std::path::Path;

mod state;
use state::*;

mod level;
use level::*;

mod systems;
use systems::input::*;
use systems::level::*;
use systems::render::*;
use systems::solver::*;
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

#[allow(unused_imports)]
use bevy_editor_pls::prelude::*;

#[bevy_main]
fn main() {
    const SETTINGS_FILE_PATH: &'static str = "settings.toml";
    if !Path::new(SETTINGS_FILE_PATH).is_file() {
        let default_settings_toml = toml::to_string(&Settings::default()).unwrap();
        fs::write(SETTINGS_FILE_PATH, default_settings_toml);
    }
    let settings_toml = fs::read_to_string(SETTINGS_FILE_PATH).unwrap();
    let settings: Settings = toml::from_str(settings_toml.as_str()).unwrap();

    let player_movement = PlayerMovement {
        directions: VecDeque::new(),
        timer: Timer::from_seconds(settings.player_move_speed, TimerMode::Repeating),
    };

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        FrameTimeDiagnosticsPlugin,
        PerformanceMatrixPlugin,
        InputManagerPlugin::<Action>::default(),
        // EditorPlugin::default(),
    ))
    .add_state::<AppState>()
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
    .add_systems(PostStartup, spawn_board);

    app.add_systems(
        Update,
        (
            button_visual_effect,
            update_button_state,
            adjust_viewport,
            (
                button_input_to_action,
                mouse_input_to_action,
                handle_automatic_solution_action,
                handle_viewport_zoom_action,
            )
                .chain(),
        ),
    )
    .add_systems(
        FixedUpdate,
        (
            animate_tiles_movement,
            animate_player_movement,
            animate_camera_zoom,
        ),
    );

    app.add_systems(
        Update,
        (
            (
                handle_other_action,
                mouse_input,
                check_level_solved,
                spawn_board.run_if(resource_changed_or_removed::<LevelId>()),
            )
                .chain(),
            update_grid_position_from_board.run_if(on_event::<UpdateGridPositionEvent>()),
            update_hud,
            file_drag_and_drop,
        )
            .run_if(in_state(AppState::Main)),
    );

    app.add_systems(
        OnEnter(AppState::AutoSolve),
        (
            (setup_solver, spawn_lowerbound_marks).chain(),
            clear_action_state,
        ),
    )
    .add_systems(Update, update_solver.run_if(in_state(AppState::AutoSolve)))
    .add_systems(OnExit(AppState::AutoSolve), despawn_lowerbound_marks)
    .insert_resource(SolverState::default());

    app.add_systems(OnEnter(AppState::AutoCratePush), spawn_crate_pushable_marks)
        .add_systems(
            Update,
            mouse_input.run_if(in_state(AppState::AutoCratePush)),
        )
        .add_systems(
            OnExit(AppState::AutoCratePush),
            despawn_crate_pushable_marks,
        );

    app.init_resource::<ActionState<Action>>()
        .insert_resource(Action::input_map());

    app.insert_resource(settings)
        .insert_resource(player_movement)
        .insert_resource(AutoCratePushState::default());

    app.add_event::<UpdateGridPositionEvent>();

    app.run();
}
