use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::solver::solver::*;
use crate::systems::input::*;
use crate::AppState;

use std::time::{Duration, Instant};

/// Loads the solver state with the current board data and initializes a new solver.
pub fn load_solver(
    mut solver_state: ResMut<SolverState>,
    board: Query<&Board>,
    config: Res<Config>,
) {
    let board = &board.single().board;
    let SolverState {
        solver,
        level,
        stopwatch,
    } = &mut *solver_state;
    *level = board.level.clone();
    let solver = solver.get_mut().unwrap();
    *solver = Solver::new(
        level.clone(),
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
    let Board { board, tile_size } = &mut *board.single_mut();
    let solver = solver_state.solver.lock().unwrap();

    let lowerbounds = solver.lower_bounds().clone();
    let max_lowerbound = lowerbounds.values().cloned().max().unwrap();
    for (position, lowerbound) in lowerbounds {
        let alpha = lowerbound as f32 / max_lowerbound as f32;
        let color = Color::BLUE * alpha + Color::RED * (1.0 - alpha);
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: color.with_a(0.5),
                    custom_size: Some(Vec2::new(tile_size.x, tile_size.y)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    position.x as f32 * tile_size.x,
                    (board.level.dimensions().y - position.y) as f32 * tile_size.y,
                    10.0,
                ),
                ..default()
            },
            LowerBoundMark,
        ));
    }
}

/// Despawns lower bound marks on the board.
pub fn despawn_lowerbound_marks(
    mut commands: Commands,
    marks: Query<Entity, With<LowerBoundMark>>,
) {
    marks
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
}

/// Resets the board to the state before automatic solution
pub fn reset_board(mut board: Query<&mut Board>, solver_state: Res<SolverState>) {
    let board = &mut board.single_mut().board;
    *board = crate::board::Board::with_level(solver_state.level.clone());
}

pub fn update_solver(
    mut solver_state: ResMut<SolverState>,
    mut board: Query<&mut Board>,
    mut player_movement: ResMut<PlayerMovement>,

    mut next_state: ResMut<NextState<AppState>>,
) {
    let board = &mut board.single_mut().board;
    let SolverState {
        solver,
        level,
        stopwatch,
    } = &mut *solver_state;

    *board = crate::board::Board::with_level(level.clone());

    let solver = solver.get_mut().unwrap();
    let timeout = Duration::from_millis(50);
    let timer = Instant::now();
    match solver.search(timeout) {
        Ok(solution) => {
            let mut verify_board = board.clone();
            for movement in &*solution {
                verify_board.move_or_push(movement.direction());
            }
            assert!(verify_board.is_solved());

            stopwatch.tick(timer.elapsed());
            info!(
                "Solver: Solved ({} sec)",
                stopwatch.elapsed().as_millis() as f32 / 1000.0
            );
            info!(
                "    Moves: {}, pushes: {}",
                solution.move_count(),
                solution.push_count()
            );
            info!("    Solution: {}", solution.lurd());

            for movement in &*solution {
                player_move_unchecked(movement.direction(), &mut player_movement);
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
            let _ = stopwatch.tick(timer.elapsed());
        }
    }
    if let Some(best_state) = solver.best_state() {
        // println!(
        //     "lower bound: {:3}, moves: {:3}, pushes: {:3}",
        //     best_state.lower_bound(&solver),
        //     best_state.movements.move_count(),
        //     best_state.movements.push_count()
        // );
        for movement in &*best_state.movements {
            board.move_or_push(movement.direction());
        }
    }
}

pub fn update_tile_translation(
    mut tiles: Query<(&mut Transform, &GridPosition)>,
    board: Query<&Board>,
) {
    let Board { board, tile_size } = &board.single();
    for (mut transform, grid_position) in tiles.iter_mut() {
        transform.translation.x = grid_position.x as f32 * tile_size.x;
        transform.translation.y =
            board.level.dimensions().y as f32 * tile_size.y - grid_position.y as f32 * tile_size.y;
    }
}

pub fn update_tile_grid_position(
    mut player_grid_positions: Query<&mut GridPosition, With<Player>>,
    mut crate_grid_positions: Query<&mut GridPosition, (With<Crate>, Without<Player>)>,
    board: Query<&Board>,
) {
    let board = &board.single().board;
    let mut player_grid_positions = player_grid_positions.single_mut();
    **player_grid_positions = board.level.player_position;

    for (mut crate_grid_position, crate_position) in crate_grid_positions
        .iter_mut()
        .zip(board.level.crate_positions.iter())
    {
        **crate_grid_position = *crate_position;
    }
}
