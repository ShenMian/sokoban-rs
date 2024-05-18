// #![feature(test)]

mod board;
mod components;
mod database;
mod events;
mod input_map;
mod level;
mod plugins;
mod resources;
mod solve;
mod state;
mod systems;
mod test;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use events::*;
use input_map::*;
use leafwing_input_manager::prelude::*;
use level::*;
use plugins::performance_matrix::*;
use resources::*;
use state::*;
use std::fs;
use std::path::Path;
use systems::audio::*;
use systems::auto_move::*;
use systems::auto_solve::*;
use systems::input::*;
use systems::level::*;
use systems::render::*;
use systems::ui::*;

const CONFIG_FILE_PATH: &str = "config.toml";

fn load_config() -> Config {
    if !Path::new(CONFIG_FILE_PATH).is_file() {
        let default_config_toml = toml::to_string(&Config::default()).unwrap();
        fs::write(CONFIG_FILE_PATH, default_config_toml).unwrap();
    }
    let config_toml = fs::read_to_string(CONFIG_FILE_PATH).unwrap();
    let config: Config = toml::from_str(config_toml.as_str()).unwrap();
    config
}

fn load_input_map() -> InputMap<Action> {
    const KEYMAP_FILE_PATH: &str = "keymap.toml";
    if !Path::new(KEYMAP_FILE_PATH).is_file() {
        let default_keymap_toml = toml::to_string(&default_input_map()).unwrap();
        fs::write(KEYMAP_FILE_PATH, default_keymap_toml).unwrap();
    }
    let keymap_toml = fs::read_to_string(KEYMAP_FILE_PATH).unwrap();
    let input_map: InputMap<Action> = toml::from_str(keymap_toml.as_str()).unwrap();
    input_map
}

fn save_config(config: Res<Config>) {
    let config_toml = toml::to_string(&*config).unwrap();
    fs::write(CONFIG_FILE_PATH, config_toml).unwrap();
}

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
        PerformanceMatrixPlugin,
        InputManagerPlugin::<Action>::default(),
    ))
    .init_state::<AppState>()
    .add_systems(PreStartup, (setup_camera, setup_database))
    .add_systems(
        Startup,
        (
            set_windows_icon,
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
            save_config.run_if(resource_changed_or_removed::<Config>()),
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

    let config = load_config();
    let player_movement = PlayerMovement::new(config.player_move_speed);
    app.insert_resource(config)
        .insert_resource(player_movement)
        .insert_resource(AutoMoveState::default());

    let input_map = load_input_map();
    app.init_resource::<ActionState<Action>>()
        .insert_resource(input_map);

    app.add_event::<CrateEnterTarget>()
        .add_event::<CrateLeaveTarget>()
        .add_event::<LevelSolved>()
        .add_event::<UpdateGridPositionEvent>();

    app.run();
}
