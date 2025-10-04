use std::time::{Duration, Instant};

use bevy::{color::palettes::css::*, prelude::*};

use crate::{
    components::{Board, Box, GridPosition, Player},
    resources::*,
    solve::solver::*,
    systems::input::*,
    AppState,
};

pub fn plugin(app: &mut App) {
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

/// Loads the solver state with the current board data and initializes a new solver.
pub fn load_solver(
    mut solver_state: ResMut<SolverState>,
    board: Query<&Board>,
    config: Res<Config>,
) {
    let board = &board.single().unwrap().board;
    let SolverState {
        solver,
        stopwatch,
        origin_board,
    } = &mut *solver_state;
    *origin_board = board.clone();
    let solver = solver.get_mut().unwrap();
    *solver = Solver::new(
        origin_board.map.clone(),
        config.solver.strategy,
        config.solver.lower_bound_method,
    );
    stopwatch.reset();
}

/// Unloads the solver state by resetting it to default values.
pub fn unload_solver(mut solver_state: ResMut<SolverState>) {
    *solver_state = SolverState::default();
}

/// Spawns lower bound marks on the board based on the solver's lower bounds.
pub fn spawn_lowerbound_marks(
    solver_state: Res<SolverState>,
    mut commands: Commands,
    mut board: Query<&mut Board>,
) {
    let Board { board, tile_size } = &mut *board.single_mut().unwrap();
    let solver = solver_state.solver.lock().unwrap();

    let lowerbounds = solver.lower_bounds().clone();
    let max_lowerbound = lowerbounds.values().cloned().max().unwrap();
    for (position, lowerbound) in lowerbounds {
        let alpha = lowerbound as f32 / max_lowerbound as f32;
        let color = BLUE * alpha + RED * (1.0 - alpha);
        commands.spawn((
            Name::new("Lower bound mark"),
            Sprite::from_color(
                color.with_alpha(0.5),
                Vec2::new(tile_size.x as f32, tile_size.y as f32),
            ),
            Transform::from_xyz(
                position.x as f32 * tile_size.x as f32,
                (board.map.dimensions().y - position.y) as f32 * tile_size.y as f32,
                10.0,
            ),
            DespawnOnExit(AppState::AutoSolve),
        ));
    }
}

/// Resets the board to the state before automatic solution
pub fn reset_board(mut board: Query<&mut Board>, solver_state: Res<SolverState>) {
    let board = &mut board.single_mut().unwrap().board;
    *board = solver_state.origin_board.clone();
}

pub fn update_solver(
    mut solver_state: ResMut<SolverState>,
    mut board: Query<&mut Board>,
    mut player_movement: ResMut<PlayerMovement>,

    mut next_state: ResMut<NextState<AppState>>,
) {
    let board = &mut board.single_mut().unwrap().board;
    let SolverState {
        solver,
        stopwatch,
        origin_board,
    } = &mut *solver_state;

    *board = crate::board::Board::with_map(origin_board.map.clone());

    let solver = solver.get_mut().unwrap();
    let timeout = Duration::from_millis(50);
    let timer = Instant::now();
    match solver.search(timeout) {
        Ok(solution) => {
            let mut verify_board = board.clone();
            for action in &*solution {
                verify_board.do_action(action.direction());
            }
            assert!(verify_board.is_solved());

            stopwatch.tick(timer.elapsed());
            info!(
                "Solver: Solved ({} sec)",
                stopwatch.elapsed().as_millis() as f32 / 1000.0
            );
            info!(
                "    Moves: {}, pushes: {}",
                solution.moves(),
                solution.pushes()
            );
            info!("    Solution: {}", solution.to_string());

            for action in &*solution {
                player_move_unchecked(action.direction(), &mut player_movement);
            }
            next_state.set(AppState::Main);
            return;
        }
        Err(SolveError::NoSolution) => {
            stopwatch.tick(timer.elapsed());
            info!(
                "Solver: No solution ({} sec)",
                stopwatch.elapsed().as_millis() as f32 / 1000.0
            );
            next_state.set(AppState::Main);
            return;
        }
        Err(SolveError::Timeout) => {
            stopwatch.tick(timer.elapsed());
        }
    }
    if let Some(best_state) = solver.best_state() {
        // println!(
        //     "lower bound: {:3}, moves: {:3}, pushes: {:3}",
        //     best_state.lower_bound(&solver),
        //     best_state.actions.moves(),
        //     best_state.actions.pushes()
        // );
        for action in &*best_state.actions {
            board.do_action(action.direction());
        }
    }
}

pub fn update_tile_translation(
    mut tiles: Query<(&mut Transform, &GridPosition)>,
    board: Query<&Board>,
) {
    let Board { board, tile_size } = &board.single().unwrap();
    for (mut transform, grid_position) in tiles.iter_mut() {
        transform.translation.x = grid_position.x as f32 * tile_size.x as f32;
        transform.translation.y = board.map.dimensions().y as f32 * tile_size.y as f32
            - grid_position.y as f32 * tile_size.y as f32;
    }
}

pub fn update_tile_grid_position(
    mut player_grid_positions: Query<&mut GridPosition, With<Player>>,
    mut box_grid_positions: Query<&mut GridPosition, (With<Box>, Without<Player>)>,
    board: Query<&Board>,
) {
    let map = &board.single().unwrap().board.map;
    let mut player_grid_positions = player_grid_positions.single_mut().unwrap();
    **player_grid_positions = map.player_position();

    for (mut box_grid_position, box_position) in box_grid_positions
        .iter_mut()
        .zip(map.box_positions().iter())
    {
        **box_grid_position = *box_position;
    }
}
