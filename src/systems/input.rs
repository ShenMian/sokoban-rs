use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use nalgebra::Vector2;

use crate::components::*;
use crate::direction::Direction;
use crate::events::*;
use crate::level::{Level, Tile};
use crate::resources::*;
use crate::solver::{
    solver::{find_path, Error, Solver},
    state::Strategy,
};
use crate::systems::level::*;

pub fn player_move_to(
    target: &Vector2<i32>,
    board: &mut crate::board::Board,
    player_movement: &mut ResMut<PlayerMovement>,
) {
    if let Some(path) = find_path(&board.level.player_position, target, |position| {
        board
            .level
            .at_unchecked(&position)
            .intersects(Tile::Wall | Tile::Crate)
    }) {
        // player_movement.directions.clear();
        let directions = path
            .windows(2)
            .map(|pos| Direction::from_vector(pos[1] - pos[0]).unwrap());
        for direction in directions {
            player_move_or_push(direction, board, player_movement);
        }
    }
}

fn player_move_or_push(
    direction: Direction,
    board: &mut crate::board::Board,
    player_movement: &mut ResMut<PlayerMovement>,
) {
    if board.move_or_push(direction) {
        player_movement.directions.push_front(direction);
    }
}

fn solve_level(board: &crate::board::Board, player_movement: &mut ResMut<PlayerMovement>) {
    info!("Start solving");
    let solver = Solver::from(board.level.clone());

    // Strategy::Fast, Strategy::Mixed
    for strategy in [Strategy::Fast] {
        let mut solver = solver.clone();
        let timer = std::time::Instant::now();
        solver.initial(strategy);
        match solver.solve(std::time::Duration::from_secs(15)) {
            Ok(solution) => {
                let mut board = board.clone();
                for movement in &solution {
                    board.move_or_push(movement.direction);
                }

                let lurd = solution
                    .iter()
                    .map(|x| Into::<char>::into(x.clone()))
                    .collect::<String>();
                info!(
                    "Solved ({:?}): {} sec, ",
                    strategy,
                    timer.elapsed().as_millis() as f32 / 1000.0
                );
                info!(
                    "    Moves: {}, pushes: {}",
                    solution.len(),
                    lurd.chars().filter(|x| x.is_uppercase()).count()
                );
                info!("    Solution: {}", lurd);

                assert!(board.is_solved());

                for movement in solution {
                    player_movement.directions.push_front(movement.direction);
                }
            }
            Err(Error::NoSolution) => {
                info!(
                    "No solution ({:?}): {} sec",
                    strategy,
                    timer.elapsed().as_millis() as f32 / 1000.0
                );
                break;
            }
            Err(Error::Timeout) => {
                info!(
                    "Failed to find a solution within the given time limit ({:?}): {} sec",
                    strategy,
                    timer.elapsed().as_millis() as f32 / 1000.0
                );
                break;
            }
        }
    }
}

pub fn mouse_input(
    mouse_buttons: Res<Input<MouseButton>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut board: Query<&mut Board>,
    windows: Query<&Window>,
    mut camera: Query<(&Camera, &GlobalTransform, &mut MainCamera)>,
    mut player_movement: ResMut<PlayerMovement>,

    mut crate_reachable: ResMut<CrateReachable>,

    mut spawn_crate_reachable_marks_events: EventWriter<SpawnCrateReachableMarks>,
    mut despawn_crate_reachable_marks_events: EventWriter<DespawnCrateReachableMarks>,
) {
    let Board { board, tile_size } = &mut *board.single_mut();
    let (camera, camera_transform, mut main_camera) = camera.single_mut();

    if mouse_buttons.just_pressed(MouseButton::Left) {
        let cursor_position = windows.single().cursor_position().unwrap();
        let position = camera
            .viewport_to_world_2d(camera_transform, cursor_position)
            .unwrap();
        let grid_position = ((position + (tile_size.x / 2.0)) / tile_size.x).as_ivec2();
        let grid_position = Vector2::new(grid_position.x, board.level.size.y - grid_position.y);

        despawn_crate_reachable_marks_events.send(DespawnCrateReachableMarks);
        match &*crate_reachable {
            CrateReachable::None => {
                if board.level.crate_positions.contains(&grid_position) {
                    let came_from = board.level.crate_reachable_path(&grid_position);
                    spawn_crate_reachable_marks_events.send(SpawnCrateReachableMarks);

                    if came_from.is_empty() {
                        *crate_reachable = CrateReachable::None;
                    } else {
                        *crate_reachable = CrateReachable::Some {
                            selected_crate: grid_position,
                            came_from,
                        };
                    }
                    return;
                }
            }
            CrateReachable::Some {
                selected_crate,
                came_from,
            } => {
                if came_from.contains_key(&grid_position) {
                    let mut position = grid_position;
                    let mut crate_path = Vec::new();
                    while position != *selected_crate {
                        crate_path.push(position);
                        position = came_from[&position];
                    }
                    crate_path.push(*selected_crate);
                    crate_path.reverse();

                    for (crate_position, push_direction) in crate_path
                        .windows(2)
                        .map(|pos| (pos[0], Direction::from_vector(pos[1] - pos[0]).unwrap()))
                    {
                        let player_position = crate_position - push_direction.to_vector();
                        player_move_to(&player_position, board, &mut player_movement);
                        player_move_or_push(push_direction, board, &mut player_movement);
                    }
                }
                *crate_reachable = CrateReachable::None;
                return;
            }
        }

        player_move_to(&grid_position, board, &mut player_movement);
    }

    for event in mouse_wheel_events.read() {
        if event.y > 0.0 {
            main_camera.target_scale /= 1.25;
        } else {
            main_camera.target_scale *= 1.25;
        }
    }
}

pub fn mouse_drag(
    mouse_buttons: Res<Input<MouseButton>>,
    mut motion_events: EventReader<MouseMotion>,
    mut camera: Query<(&mut Transform, &mut MainCamera), With<MainCamera>>,
) {
    if mouse_buttons.pressed(MouseButton::Right) {
        let (mut camera_transform, main_camera) = camera.single_mut();
        if mouse_buttons.pressed(MouseButton::Right) {
            for event in motion_events.read() {
                camera_transform.translation.x -= event.delta.x * main_camera.target_scale * 0.6;
                camera_transform.translation.y += event.delta.y * main_camera.target_scale * 0.6;
            }
        }
    } else {
        motion_events.clear();
    }
}

pub fn keyboard_input(
    keyboard: Res<Input<KeyCode>>,
    mut board: Query<&mut Board>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
    mut camera: Query<&mut MainCamera>,
    mut player_movement: ResMut<PlayerMovement>,

    mut update_grid_position_events: EventWriter<UpdateGridPositionEvent>,
    mut despawn_crate_reachable_marks_events: EventWriter<DespawnCrateReachableMarks>,
) {
    let board = &mut board.single_mut().board;

    if keyboard.any_just_pressed([KeyCode::W, KeyCode::Up, KeyCode::K]) {
        player_move_or_push(Direction::Up, board, &mut player_movement);
    }
    if keyboard.any_just_pressed([KeyCode::S, KeyCode::Down, KeyCode::J]) {
        player_move_or_push(Direction::Down, board, &mut player_movement);
    }
    if keyboard.any_just_pressed([KeyCode::A, KeyCode::Left, KeyCode::H]) {
        player_move_or_push(Direction::Left, board, &mut player_movement);
    }
    if keyboard.any_just_pressed([KeyCode::D, KeyCode::Right, KeyCode::L]) {
        player_move_or_push(Direction::Right, board, &mut player_movement);
    }

    if (keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::Z))
        || keyboard.just_pressed(KeyCode::U)
    {
        board.undo_push();
        player_movement.directions.clear();
        update_grid_position_events.send(UpdateGridPositionEvent);
    }

    let database = database.0.lock().unwrap();
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::V) {
        despawn_crate_reachable_marks_events.send(DespawnCrateReachableMarks);
        import_from_clipboard(&mut level_id, &database);
    }
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::C) {
        despawn_crate_reachable_marks_events.send(DespawnCrateReachableMarks);
        export_to_clipboard(&board);
    }

    if keyboard.just_pressed(KeyCode::BracketRight) {
        despawn_crate_reachable_marks_events.send(DespawnCrateReachableMarks);
        switch_to_next_level(&mut level_id, &database);
    }
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        despawn_crate_reachable_marks_events.send(DespawnCrateReachableMarks);
        switch_to_previous_level(&mut level_id, &database);
    }

    let mut main_camera = camera.single_mut();
    if keyboard.just_pressed(KeyCode::Equals) {
        main_camera.target_scale /= 1.25;
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        main_camera.target_scale *= 1.25;
    }

    if keyboard.just_pressed(KeyCode::P) {
        solve_level(&board, &mut player_movement);
    }
}

pub fn gamepad_input(
    gamepads: Res<Gamepads>,
    button_inputs: Res<Input<GamepadButton>>,
    mut board: Query<&mut Board>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
) {
    let database = database.0.lock().unwrap();
    let board = &mut board.single_mut().board;

    for gamepad in gamepads.iter() {
        if button_inputs.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadUp)) {
            board.move_or_push(Direction::Up);
        }
        if button_inputs.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadDown)) {
            board.move_or_push(Direction::Down);
        }
        if button_inputs.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadLeft)) {
            board.move_or_push(Direction::Left);
        }
        if button_inputs.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadRight)) {
            board.move_or_push(Direction::Right);
        }

        if button_inputs.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::East)) {
            // TODO
            board.undo_push();
        }

        if button_inputs.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::RightTrigger))
        {
            switch_to_next_level(&mut level_id, &database);
        }
        if button_inputs.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::LeftTrigger)) {
            switch_to_previous_level(&mut level_id, &database);
        }
    }
}

pub fn file_drag_and_drop(mut events: EventReader<FileDragAndDrop>) {
    for event in events.read() {
        if let FileDragAndDrop::DroppedFile {
            window: _,
            path_buf,
        } = event
        {
            info!("Load levels from file {:?}", path_buf);
            match Level::load_from_file(path_buf) {
                Ok(levels) => info!("Done, {} levels loaded", levels.len()),
                Err(msg) => warn!("Failed to load levels from file: {}", msg),
            }
        }
    }
}
