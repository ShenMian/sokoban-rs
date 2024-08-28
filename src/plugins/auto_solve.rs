use crate::{
    resources::*,
    state::*,
    systems::{auto_solve::*, input::*},
};

use bevy::prelude::*;

pub struct AutoSolvePlugin;

impl Plugin for AutoSolvePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::AutoSolve),
            (
                (load_solver, spawn_lowerbound_marks).chain(),
                clear_action_state,
            ),
        );
        app.add_systems(
            Update,
            (
                update_solver,
                update_tile_grid_position,
                update_tile_translation,
            )
                .run_if(in_state(AppState::AutoSolve)),
        );
        app.add_systems(
            OnExit(AppState::AutoSolve),
            (
                reset_board,
                update_tile_grid_position,
                update_tile_translation,
                unload_solver,
            )
                .chain(),
        );
        app.insert_resource(SolverState::default());
    }
}
