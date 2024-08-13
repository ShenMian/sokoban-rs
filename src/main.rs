// #![feature(test)]
#![allow(clippy::op_ref)]

mod board;
mod components;
mod database;
mod events;
mod input_map;
mod plugins;
mod resources;
mod solve;
mod state;
mod systems;
mod test;
mod utils;

use std::{fs, path::Path};

use events::*;
use input_map::*;
use leafwing_input_manager::{action_diff::ActionDiffEvent, prelude::*};
use plugins::{performance_matrix::*, version_information::*};
use resources::*;
use state::*;
use systems::{audio::*, auto_move::*, auto_solve::*, input::*, level::*, render::*, ui::*};
use utils::*;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

const CONFIG_FILE_PATH: &str = "config.toml";

fn load_config() -> Config {
    let config_toml = fs::read_to_string(CONFIG_FILE_PATH).unwrap();
    let config: Config = toml::from_str(config_toml.as_str()).unwrap();
    config
}

fn save_config(config: &Config) {
    let config_toml = toml::to_string(&config).unwrap();
    fs::write(CONFIG_FILE_PATH, config_toml).unwrap();
}

fn save_config_system(config: Res<Config>) {
    save_config(&config);
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
        VersionInformationPlugin,
        InputManagerPlugin::<Action>::default(),
    ))
    .init_state::<AppState>()
    .enable_state_scoped_entities::<AppState>()
    .add_systems(PreStartup, (setup_camera, setup_database))
    .add_systems(
        Startup,
        (set_windows_icon, setup_button, setup_hud, setup_level),
    );

    app.add_systems(
        Update,
        (
            button_visual_effect,
            update_button_state,
            handle_audio_event,
            adjust_viewport,
            adjust_camera_scale,
            save_config_system.run_if(resource_changed_or_removed::<Config>()),
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
        )
            .chain(),
    )
    .insert_resource(SolverState::default());

    app.add_systems(OnEnter(AppState::AutoMove), spawn_auto_move_marks)
        .add_systems(Update, mouse_input.run_if(in_state(AppState::AutoMove)))
        .add_systems(OnExit(AppState::AutoMove), cleanup_sprite_color);

    if !Path::new(CONFIG_FILE_PATH).is_file() {
        let default_config_toml = toml::to_string(&Config::default()).unwrap();
        fs::write(CONFIG_FILE_PATH, default_config_toml).unwrap();
        save_config(&Config::default());
    }
    let config = load_config();
    let player_movement = PlayerMovement::new(config.player_move_speed);
    app.insert_resource(config)
        .insert_resource(player_movement)
        .insert_resource(AutoMoveState::default());

    app.init_resource::<ActionState<Action>>()
        .insert_resource(default_input_map())
        .add_event::<ActionDiffEvent<Action>>();

    app.add_event::<BoxEnterTarget>()
        .add_event::<BoxLeaveTarget>()
        .add_event::<LevelSolved>()
        .add_event::<UpdateGridPositionEvent>();

    app.run();
}
