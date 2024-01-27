// #![feature(test)]

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use leafwing_input_manager::prelude::*;

use std::collections::VecDeque;
use std::fs;
use std::path::Path;

mod state;
use state::*;

mod level;
use level::*;

mod systems;
use systems::audio::*;
use systems::auto_move::*;
use systems::auto_solve::*;
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

mod input_action_map;
use input_action_map::*;

mod board;
mod components;
mod database;
mod direction;
mod movement;
mod solver;
mod test;

#[allow(unused_imports)]
use bevy_editor_pls::prelude::*;

const SETTINGS_FILE_PATH: &'static str = "settings.toml";
const KEYMAP_FILE_PATH: &'static str = "keymap.toml";

fn save_settings(settings: Res<Settings>) {
    let settings_toml = toml::to_string(&*settings).unwrap();
    fs::write(SETTINGS_FILE_PATH, settings_toml).unwrap();
}

#[bevy_main]
fn main() {
    if !Path::new(SETTINGS_FILE_PATH).is_file() {
        let default_settings_toml = toml::to_string(&Settings::default()).unwrap();
        fs::write(SETTINGS_FILE_PATH, default_settings_toml).unwrap();
    }
    let settings_toml = fs::read_to_string(SETTINGS_FILE_PATH).unwrap();
    let settings: Settings = toml::from_str(settings_toml.as_str()).unwrap();

    if !Path::new(KEYMAP_FILE_PATH).is_file() {
        let default_keymap_toml = toml::to_string(&default_input_action_map()).unwrap();
        fs::write(KEYMAP_FILE_PATH, default_keymap_toml).unwrap();
    }
    let keymap_toml = fs::read_to_string(KEYMAP_FILE_PATH).unwrap();
    let input_action_map: InputMap<Action> = toml::from_str(keymap_toml.as_str()).unwrap();

    let player_movement = PlayerMovement {
        directions: VecDeque::new(),
        timer: Timer::from_seconds(settings.player_move_speed, TimerMode::Repeating),
    };

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        AudioPlugin,
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
    );

    app.add_systems(
        Update,
        (
            button_visual_effect,
            update_button_state,
            handle_audio_event,
            adjust_viewport,
            save_settings.run_if(resource_changed_or_removed::<Settings>()),
            (button_input_to_action, handle_actions).chain(),
        ),
    )
    .add_systems(FixedUpdate, (smooth_camera_motion, animate_player));

    app.add_systems(
        Update,
        (
            (
                mouse_input,
                auto_switch_to_next_unsolved_level.run_if(on_event::<LevelSolved>()),
                spawn_board.run_if(resource_changed_or_removed::<LevelId>()),
            )
                .chain(),
            update_grid_position_from_board.run_if(on_event::<UpdateGridPositionEvent>()),
            update_hud,
            file_drag_and_drop,
        )
            .run_if(in_state(AppState::Main)),
    )
    .add_systems(
        FixedUpdate,
        (handle_player_movement, smooth_tile_motion).run_if(in_state(AppState::Main)),
    );

    app.add_systems(
        OnEnter(AppState::AutoSolve),
        (
            (load_solver, spawn_lowerbound_marks).chain(),
            clear_action_state,
        ),
    )
    .add_systems(
        Update,
        (
            update_solver,
            update_tile_grid_position,
            update_tile_translation,
        )
            .run_if(in_state(AppState::AutoSolve)),
    )
    .add_systems(
        OnExit(AppState::AutoSolve),
        (
            reset_board,
            update_tile_grid_position,
            update_tile_translation,
            unload_solver,
            despawn_lowerbound_marks,
        )
            .chain(),
    )
    .insert_resource(SolverState::default());

    app.add_systems(OnEnter(AppState::AutoMove), spawn_auto_move_marks)
        .add_systems(Update, mouse_input.run_if(in_state(AppState::AutoMove)))
        .add_systems(OnExit(AppState::AutoMove), despawn_auto_move_marks);

    app.init_resource::<ActionState<Action>>()
        .insert_resource(input_action_map);

    app.insert_resource(settings)
        .insert_resource(player_movement)
        .insert_resource(AutoMoveState::default());

    app.add_event::<UpdateGridPositionEvent>()
        .add_event::<CrateEnterTarget>()
        .add_event::<CrateLeaveTarget>()
        .add_event::<LevelSolved>();

    app.run();
}
