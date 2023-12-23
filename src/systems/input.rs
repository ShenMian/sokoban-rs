use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use nalgebra::Vector2;

use crate::components::*;
use crate::direction::Direction;
use crate::events::*;
use crate::level::{Level, PushState, Tile};
use crate::resources::*;
use crate::solver::{
    solver::{find_path, SolveError, Solver},
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
            .get_unchecked(&position)
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

fn solve_level(board: &mut crate::board::Board, player_movement: &mut ResMut<PlayerMovement>) {
    let timeout = std::time::Duration::from_secs(15);
    info!("Start solving (timeout: {:?})", timeout);
    let solver = Solver::from(board.level.clone());

    // Strategy::Fast, Strategy::Mixed
    for strategy in [Strategy::Fast] {
        let mut solver = solver.clone();
        let timer = std::time::Instant::now();
        solver.initial(strategy);
        match solver.solve(timeout) {
            Ok(solution) => {
                let mut verify_board = board.clone();
                for movement in &*solution {
                    verify_board.move_or_push(movement.direction);
                }
                assert!(verify_board.is_solved());

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

                for movement in &*solution {
                    player_move_or_push(movement.direction, board, player_movement);
                }
            }
            Err(SolveError::NoSolution) => {
                info!(
                    "No solution ({:?}): {} sec",
                    strategy,
                    timer.elapsed().as_millis() as f32 / 1000.0
                );
                break;
            }
            Err(SolveError::Timeout) => {
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
    mut select_crate_events: EventWriter<SelectCrate>,
    mut unselect_crate_events: EventWriter<UnselectCrate>,
    mut update_grid_position_events: EventWriter<UpdateGridPositionEvent>,
) {
    let Board { board, tile_size } = &mut *board.single_mut();
    let (camera, camera_transform, mut main_camera) = camera.single_mut();

    if mouse_buttons.just_pressed(MouseButton::Other(1)) {
        board.undo_push();
        player_movement.directions.clear();
        update_grid_position_events.send(UpdateGridPositionEvent);
        unselect_crate_events.send(UnselectCrate);
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Other(2)) {
        board.redo_push();
        player_movement.directions.clear();
        update_grid_position_events.send(UpdateGridPositionEvent);
        unselect_crate_events.send(UnselectCrate);
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        let cursor_position = windows.single().cursor_position().unwrap();
        let position = camera
            .viewport_to_world_2d(camera_transform, cursor_position)
            .unwrap();
        let grid_position = ((position + (tile_size.x / 2.0)) / tile_size.x).as_ivec2();
        let grid_position =
            Vector2::new(grid_position.x, board.level.dimensions.y - grid_position.y);

        // TODO: 将选择箱子作为一个 State
        unselect_crate_events.send(UnselectCrate);
        match &*crate_reachable {
            CrateReachable::None => {
                if board.level.crate_positions.contains(&grid_position) {
                    select_crate_events.send(SelectCrate(grid_position));
                    return;
                }
            }
            CrateReachable::Some {
                selected_crate,
                path,
            } => {
                let mut crate_paths = Vec::new();
                for &push_direction in [
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ]
                .iter()
                {
                    if path.contains_key(&PushState {
                        push_direction,
                        crate_position: grid_position,
                    }) {
                        if *selected_crate == grid_position {
                            *crate_reachable = CrateReachable::None;
                            return;
                        }
                        let crate_path = path[&PushState {
                            push_direction,
                            crate_position: grid_position,
                        }]
                            .clone();
                        crate_paths.push(crate_path);
                    }
                }
                if let Some(min_crate_path) =
                    crate_paths.iter().min_by_key(|crate_path| crate_path.len())
                {
                    for (crate_position, push_direction) in min_crate_path
                        .windows(2)
                        .map(|pos| (pos[0], Direction::from_vector(pos[1] - pos[0]).unwrap()))
                    {
                        let player_position = crate_position - push_direction.to_vector();
                        player_move_to(&player_position, board, &mut player_movement);
                        player_move_or_push(push_direction, board, &mut player_movement);
                    }
                } else if grid_position != *selected_crate
                    && board.level.crate_positions.contains(&grid_position)
                {
                    select_crate_events.send(SelectCrate(grid_position));
                    return;
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
    mut camera: Query<(&mut Transform, &mut MainCamera)>,
) {
    let (mut camera_transform, main_camera) = camera.single_mut();
    if mouse_buttons.pressed(MouseButton::Right) {
        for event in motion_events.read() {
            camera_transform.translation.x -= event.delta.x * main_camera.target_scale * 0.6;
            camera_transform.translation.y += event.delta.y * main_camera.target_scale * 0.6;
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
    mut settings: ResMut<Settings>,

    mut unselect_crate_events: EventWriter<UnselectCrate>,
    mut update_grid_position_events: EventWriter<UpdateGridPositionEvent>,
) {
    let board = &mut board.single_mut().board;

    let mut any_pressed = false;

    if keyboard.any_just_pressed([KeyCode::W, KeyCode::Up, KeyCode::K]) {
        player_move_or_push(Direction::Up, board, &mut player_movement);
        any_pressed = true;
    }
    if keyboard.any_just_pressed([KeyCode::S, KeyCode::Down, KeyCode::J]) {
        player_move_or_push(Direction::Down, board, &mut player_movement);
        any_pressed = true;
    }
    if keyboard.any_just_pressed([KeyCode::A, KeyCode::Left, KeyCode::H]) {
        player_move_or_push(Direction::Left, board, &mut player_movement);
        any_pressed = true;
    }
    if keyboard.any_just_pressed([KeyCode::D, KeyCode::Right, KeyCode::L]) {
        player_move_or_push(Direction::Right, board, &mut player_movement);
        any_pressed = true;
    }

    if (keyboard.pressed(KeyCode::ControlLeft)
        && keyboard.just_pressed(KeyCode::Z)
        && !keyboard.pressed(KeyCode::ShiftLeft))
        || keyboard.just_pressed(KeyCode::U)
    {
        board.undo_push();
        player_movement.directions.clear();
        update_grid_position_events.send(UpdateGridPositionEvent);
        any_pressed = true;
    }

    if (keyboard.pressed(KeyCode::ControlLeft)
        && keyboard.pressed(KeyCode::ShiftLeft)
        && keyboard.just_pressed(KeyCode::Z))
        || (keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::R))
    {
        board.redo_push();
        player_movement.directions.clear();
        update_grid_position_events.send(UpdateGridPositionEvent);
        any_pressed = true;
    }

    if player_movement.directions.is_empty() {
        let database = database.lock().unwrap();
        if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::V) {
            import_from_clipboard(&mut level_id, &database);
            any_pressed = true;
        }
        if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::C) {
            export_to_clipboard(&board);
            any_pressed = true;
        }

        if keyboard.just_pressed(KeyCode::BracketRight) {
            switch_to_next_level(&mut level_id, &database);
            any_pressed = true;
        }
        if keyboard.just_pressed(KeyCode::BracketLeft) {
            switch_to_previous_level(&mut level_id, &database);
            any_pressed = true;
        }
    }

    let mut main_camera = camera.single_mut();
    if keyboard.just_pressed(KeyCode::Equals) {
        main_camera.target_scale /= 1.25;
        any_pressed = true;
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        main_camera.target_scale *= 1.25;
        any_pressed = true;
    }

    if keyboard.just_pressed(KeyCode::P) {
        solve_level(board, &mut player_movement);
        any_pressed = true;
    }

    if keyboard.just_pressed(KeyCode::I) {
        settings.instant_move = !settings.instant_move;
        any_pressed = true;
    }

    if any_pressed {
        unselect_crate_events.send(UnselectCrate);
    }
}

pub fn gamepad_input(
    gamepads: Res<Gamepads>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    mut board: Query<&mut Board>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
    mut camera: Query<(&mut Transform, &mut MainCamera)>,

    mut player_movement: ResMut<PlayerMovement>,
    mut update_grid_position_events: EventWriter<UpdateGridPositionEvent>,
    mut unselect_crate_events: EventWriter<UnselectCrate>,
) {
    let database = database.lock().unwrap();
    let board = &mut board.single_mut().board;

    let mut any_pressed = false;

    for gamepad in gamepads.iter() {
        if buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadUp)) {
            player_move_or_push(Direction::Up, board, &mut player_movement);
            any_pressed = true;
        }
        if buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadDown)) {
            player_move_or_push(Direction::Down, board, &mut player_movement);
            any_pressed = true;
        }
        if buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadLeft)) {
            player_move_or_push(Direction::Left, board, &mut player_movement);
            any_pressed = true;
        }
        if buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadRight)) {
            player_move_or_push(Direction::Right, board, &mut player_movement);
            any_pressed = true;
        }

        if buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::East)) {
            board.undo_push();
            player_movement.directions.clear();
            update_grid_position_events.send(UpdateGridPositionEvent);
            any_pressed = true;
        }
        if buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::South)) {
            board.redo_push();
            player_movement.directions.clear();
            update_grid_position_events.send(UpdateGridPositionEvent);
            any_pressed = true;
        }

        if buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::RightTrigger)) {
            switch_to_next_level(&mut level_id, &database);
            any_pressed = true;
        }
        if buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::LeftTrigger)) {
            switch_to_previous_level(&mut level_id, &database);
            any_pressed = true;
        }

        let (mut camera_transform, mut main_camera) = camera.single_mut();
        if buttons.just_pressed(GamepadButton::new(
            gamepad,
            GamepadButtonType::RightTrigger2,
        )) {
            main_camera.target_scale /= 1.25;
            any_pressed = true;
        }
        if buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::LeftTrigger2)) {
            main_camera.target_scale *= 1.25;
            any_pressed = true;
        }

        if let (Some(x), Some(y)) = (
            axes.get(GamepadAxis::new(gamepad, GamepadAxisType::RightStickX)),
            axes.get(GamepadAxis::new(gamepad, GamepadAxisType::RightStickY)),
        ) {
            let right_stick_position = Vector2::new(x, y);
            camera_transform.translation.x +=
                right_stick_position.x * main_camera.target_scale * 1.6;
            camera_transform.translation.y +=
                right_stick_position.y * main_camera.target_scale * 1.6;
        }

        if buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::North)) {
            solve_level(board, &mut player_movement);
            any_pressed = true;
        }
    }

    if any_pressed {
        unselect_crate_events.send(UnselectCrate);
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
