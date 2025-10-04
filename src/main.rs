// #![feature(test)]
#![allow(clippy::op_ref)]

mod board;
mod components;
mod database;
mod events;
mod input_map;
mod plugins;
mod resources;
mod settings;
mod solve;
mod state;
mod systems;
mod test;
mod utils;

use events::*;
use input_map::*;
use leafwing_input_manager::{action_diff::ActionDiffMessage, prelude::*};
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
        InputManagerPlugin::<Action>::default(),
    ))
    .init_state::<AppState>();

    app.add_systems(PreStartup, (setup_camera, setup_database));
    app.add_systems(Startup, (set_windows_icon, setup_level));
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

    app.init_resource::<ActionState<Action>>()
        .insert_resource(default_input_map())
        .add_message::<ActionDiffMessage<Action>>();

    app.add_message::<BoxEnterGoal>()
        .add_message::<BoxLeaveGoal>()
        .add_message::<LevelSolved>()
        .add_message::<UpdateGridPositionEvent>();

    app.run();
}
