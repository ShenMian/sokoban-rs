use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::solver::solver::*;
use crate::systems::input::*;
use crate::AppState;

pub fn spawn_lowerbound_marks(
    solver_state: Res<SolverState>,
    mut commands: Commands,
    mut board: Query<&mut Board>,
) {
    let Board { board, tile_size } = &mut *board.single_mut();

    let lowerbounds = solver_state.solver.lock().unwrap().lower_bounds().clone();
    let max_lowerbound = lowerbounds
        .iter()
        .map(|(_, lowerbound)| *lowerbound)
        .max()
        .unwrap();
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
                    (board.level.dimensions.y - position.y) as f32 * tile_size.y,
                    10.0,
                ),
                ..default()
            },
            LowerBoundMark,
        ));
    }
}

pub fn despawn_lowerbound_marks(
    mut commands: Commands,
    marks: Query<Entity, With<LowerBoundMark>>,
) {
    marks.for_each(|entity| commands.entity(entity).despawn());
}

pub fn update_solver(
    mut solver_state: ResMut<SolverState>,
    mut board: Query<&mut Board>,
    mut player_movement: ResMut<PlayerMovement>,

    mut next_state: ResMut<NextState<AppState>>,
) {
    let board = &mut board.single_mut().board;
    let SolverState { solver, timer } = &mut *solver_state;

    let mut solver = solver.lock().unwrap();
    let timeout = std::time::Duration::from_millis(100);
    match solver.solve(timeout) {
        Ok(solution) => {
            let mut verify_board = board.clone();
            for movement in &*solution {
                verify_board.move_or_push(movement.direction);
            }
            assert!(verify_board.is_solved());

            info!(
                "Solver: Solved ({} sec)",
                timer.elapsed().as_millis() as f32 / 1000.0
            );
            info!(
                "    Moves: {}, pushes: {}",
                solution.move_count(),
                solution.push_count()
            );
            info!("    Solution: {}", solution.lurd());

            for movement in &*solution {
                player_move_or_push(movement.direction, board, &mut player_movement);
            }
            next_state.set(AppState::Main);
        }
        Err(SolveError::NoSolution) => {
            info!(
                "Solver: No solution ({} sec)",
                timer.elapsed().as_millis() as f32 / 1000.0
            );
            next_state.set(AppState::Main);
        }
        Err(SolveError::Timeout) => (),
    }
}
