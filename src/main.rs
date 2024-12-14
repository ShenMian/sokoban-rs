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

use events::*;
use input_map::*;
use leafwing_input_manager::{action_diff::ActionDiffEvent, prelude::*};
use plugins::{
    auto_move::AutoMovePlugin, auto_solve::AutoSolvePlugin, camera::CameraPlugin,
    config::ConfigPlugin, performance_matrix, ui::UiPlugin, version_information,
};
use resources::*;
use state::*;
use systems::{audio::*, input::*, level::*, render::*, ui::*};
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
    ))
    .add_plugins((
        performance_matrix::plugin,
        version_information::plugin,
        InputManagerPlugin::<Action>::default(),
    ))
    .init_state::<AppState>()
    .enable_state_scoped_entities::<AppState>();

    app.add_systems(PreStartup, (setup_camera, setup_database));
    app.add_systems(
        Startup,
        (set_windows_icon, setup_button, setup_hud, setup_level),
    );
    app.add_systems(Update, handle_audio_event);
    app.add_systems(FixedUpdate, animate_player);

    app.add_systems(
        Update,
        (
            (
                mouse_input,
                auto_switch_to_next_unsolved_level.run_if(on_event::<LevelSolved>),
                spawn_board.run_if(resource_changed_or_removed::<LevelId>),
            )
                .chain(),
            update_grid_position_from_board.run_if(on_event::<UpdateGridPositionEvent>),
            file_drag_and_drop,
        )
            .run_if(in_state(AppState::Main)),
    )
    .add_systems(
        FixedUpdate,
        (handle_player_movement, smooth_tile_motion).run_if(in_state(AppState::Main)),
    );

    app.add_plugins((
        ConfigPlugin,
        UiPlugin,
        CameraPlugin,
        AutoSolvePlugin,
        AutoMovePlugin,
    ));

    app.init_resource::<ActionState<Action>>()
        .insert_resource(default_input_map())
        .add_event::<ActionDiffEvent<Action>>();

    app.add_event::<BoxEnterTarget>()
        .add_event::<BoxLeaveTarget>()
        .add_event::<LevelSolved>()
        .add_event::<UpdateGridPositionEvent>();

    app.run();
}
