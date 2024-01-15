use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};
use nalgebra::Vector2;

use crate::components::*;
use crate::direction::Direction;
use crate::events::*;
use crate::level::{Level, PushState, Tile};
use crate::resources::*;
use crate::solver::solver::*;
use crate::systems::level::*;

#[derive(Actionlike, Reflect, Clone, Hash, PartialEq, Eq)]
pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    Undo,
    Redo,

    PreviousLevel,
    NextLevel,

    ZoomOut,
    ZoomIn,

    InstantMove,
    AutomaticSolution,

    ImportLevelsFromClipboard,
    ExportLevelToClipboard,
}

impl Action {
    pub fn input_map() -> InputMap<Action> {
        InputMap::new([
            // Mouse
            (
                UserInput::Single(InputKind::Mouse(MouseButton::Other(1))),
                Self::Undo,
            ),
            (
                UserInput::Single(InputKind::Mouse(MouseButton::Other(2))),
                Self::Redo,
            ),
            // Keyboard
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::W)),
                Self::MoveUp,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::S)),
                Self::MoveDown,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::A)),
                Self::MoveLeft,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::D)),
                Self::MoveRight,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::Up)),
                Self::MoveUp,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::Down)),
                Self::MoveDown,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::Left)),
                Self::MoveLeft,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::Right)),
                Self::MoveRight,
            ),
            (
                UserInput::Chord(vec![
                    InputKind::Keyboard(KeyCode::ControlLeft),
                    InputKind::Keyboard(KeyCode::Z),
                ]),
                Self::Undo,
            ),
            (
                UserInput::Chord(vec![
                    InputKind::Keyboard(KeyCode::ControlLeft),
                    InputKind::Keyboard(KeyCode::ShiftLeft),
                    InputKind::Keyboard(KeyCode::Z),
                ]),
                Self::Redo,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::BracketLeft)),
                Self::PreviousLevel,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::BracketRight)),
                Self::NextLevel,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::Minus)),
                Self::ZoomOut,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::Equals)),
                Self::ZoomIn,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::I)),
                Self::InstantMove,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::P)),
                Self::AutomaticSolution,
            ),
            // Keyboard (Vim)
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::K)),
                Self::MoveUp,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::J)),
                Self::MoveDown,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::H)),
                Self::MoveLeft,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::L)),
                Self::MoveRight,
            ),
            (
                UserInput::Single(InputKind::Keyboard(KeyCode::U)),
                Self::Undo,
            ),
            (
                UserInput::Chord(vec![
                    InputKind::Keyboard(KeyCode::ControlLeft),
                    InputKind::Keyboard(KeyCode::R),
                ]),
                Self::Redo,
            ),
            // Gamepad
            (
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::DPadUp)),
                Self::MoveUp,
            ),
            (
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::DPadDown)),
                Self::MoveDown,
            ),
            (
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::DPadLeft)),
                Self::MoveLeft,
            ),
            (
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::DPadRight)),
                Self::MoveRight,
            ),
            (
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::East)),
                Self::Undo,
            ),
            (
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::South)),
                Self::Redo,
            ),
            (
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::LeftTrigger)),
                Self::PreviousLevel,
            ),
            (
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::RightTrigger)),
                Self::NextLevel,
            ),
            (
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::LeftTrigger2)),
                Self::ZoomOut,
            ),
            (
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::RightTrigger2)),
                Self::ZoomIn,
            ),
            (
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::West)),
                Self::InstantMove,
            ),
            (
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::North)),
                Self::AutomaticSolution,
            ),
        ])
    }
}

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

#[allow(dead_code)]
fn print_solver_lowerbounds(solver: &Solver) {
    for y in 0..solver.level.dimensions.y {
        for x in 0..solver.level.dimensions.x {
            let position = Vector2::new(x, y);
            if let Some(lower_bound) = solver.lower_bounds().get(&position) {
                print!("{:^2} ", lower_bound);
            } else {
                print!("{:^2} ", "##");
            }
        }
        println!();
    }
}

fn solve_level(board: &mut crate::board::Board, player_movement: &mut ResMut<PlayerMovement>) {
    let mut solver = Solver::new(board.level.clone());
    solver.initial(Strategy::Fast, LowerBoundMethod::PushCount);
    // print_solver_lowerbounds(&solver);

    let timeout = std::time::Duration::from_secs(15);
    info!("Solver: Start (timeout: {:?})", timeout);

    let timer = std::time::Instant::now();
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
                player_move_or_push(movement.direction, board, player_movement);
            }
        }
        Err(SolveError::NoSolution) => {
            info!(
                "Solver: No solution ({} sec)",
                timer.elapsed().as_millis() as f32 / 1000.0
            );
        }
        Err(SolveError::Timeout) => {
            info!(
                "Solver(: Failed to find a solution within the given time limit ({} sec)",
                timer.elapsed().as_millis() as f32 / 1000.0
            );
        }
    }
}

pub fn action_input(
    action_state: Res<ActionState<Action>>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
    mut player_movement: ResMut<PlayerMovement>,
    mut settings: ResMut<Settings>,

    mut board: Query<&mut Board>,
    mut camera: Query<&mut MainCamera>,

    mut update_grid_position_events: EventWriter<UpdateGridPositionEvent>,
    mut unselect_crate_events: EventWriter<UnselectCrate>,
) {
    let database = database.lock().unwrap();
    let board = &mut board.single_mut().board;
    let mut any_pressed = false;

    if action_state.just_pressed(Action::MoveUp) {
        player_move_or_push(Direction::Up, board, &mut player_movement);
        any_pressed = true;
    }
    if action_state.just_pressed(Action::MoveDown) {
        player_move_or_push(Direction::Down, board, &mut player_movement);
        any_pressed = true;
    }
    if action_state.just_pressed(Action::MoveLeft) {
        player_move_or_push(Direction::Left, board, &mut player_movement);
        any_pressed = true;
    }
    if action_state.just_pressed(Action::MoveRight) {
        player_move_or_push(Direction::Right, board, &mut player_movement);
        any_pressed = true;
    }

    if action_state.just_pressed(Action::Undo) {
        board.undo_push();
        player_movement.directions.clear();
        update_grid_position_events.send(UpdateGridPositionEvent);
        any_pressed = true;
    }
    if action_state.just_pressed(Action::Redo) {
        board.redo_push();
        player_movement.directions.clear();
        update_grid_position_events.send(UpdateGridPositionEvent);
        any_pressed = true;
    }

    if action_state.just_pressed(Action::PreviousLevel) {
        player_movement.directions.clear();
        switch_to_previous_level(&mut level_id, &database);
        any_pressed = true;
    }
    if action_state.just_pressed(Action::NextLevel) {
        player_movement.directions.clear();
        switch_to_next_level(&mut level_id, &database);
        any_pressed = true;
    }

    let mut main_camera = camera.single_mut();
    if action_state.just_pressed(Action::ZoomOut) {
        main_camera.target_scale *= 1.25;
        any_pressed = true;
    }
    if action_state.just_pressed(Action::ZoomIn) {
        main_camera.target_scale /= 1.25;
        any_pressed = true;
    }

    if action_state.just_pressed(Action::InstantMove) {
        settings.instant_move = !settings.instant_move;
        any_pressed = true;
    }
    if action_state.just_pressed(Action::AutomaticSolution) {
        solve_level(board, &mut player_movement);
        any_pressed = true;
    }

    if action_state.just_pressed(Action::ImportLevelsFromClipboard) {
        player_movement.directions.clear();
        import_from_clipboard(&mut level_id, &database);
        any_pressed = true;
    }
    if action_state.just_pressed(Action::ExportLevelToClipboard) {
        player_movement.directions.clear();
        export_to_clipboard(&board);
        any_pressed = true;
    }

    if any_pressed {
        unselect_crate_events.send(UnselectCrate);
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
) {
    let Board { board, tile_size } = &mut *board.single_mut();
    let (camera, camera_transform, mut main_camera) = camera.single_mut();

    if mouse_buttons.just_pressed(MouseButton::Left) {
        let cursor_position = windows.single().cursor_position().unwrap();
        let position = camera
            .viewport_to_world_2d(camera_transform, cursor_position)
            .unwrap();
        let grid_position = ((position + (tile_size.x / 2.0)) / tile_size.x).as_ivec2();
        let grid_position =
            Vector2::new(grid_position.x, board.level.dimensions.y - grid_position.y);

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
                paths,
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
                    if paths.contains_key(&PushState {
                        push_direction,
                        crate_position: grid_position,
                    }) {
                        if *selected_crate == grid_position {
                            *crate_reachable = CrateReachable::None;
                            return;
                        }
                        let crate_path = paths[&PushState {
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

pub fn adjust_viewport(
    mouse_buttons: Res<Input<MouseButton>>,
    gamepads: Res<Gamepads>,
    axes: Res<Axis<GamepadAxis>>,
    mut motion_events: EventReader<MouseMotion>,
    mut camera: Query<(&mut Transform, &MainCamera)>,
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

    for gamepad in gamepads.iter() {
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
    }
}

pub fn file_drag_and_drop(
    mut events: EventReader<FileDragAndDrop>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
) {
    for event in events.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            let database = database.lock().unwrap();
            info!("Load levels from file {:?}", path_buf);
            match Level::load_from_file(path_buf) {
                Ok(levels) => {
                    info!("Done, {} levels loaded", levels.len());
                    database.import_levels(&levels);
                    **level_id = database.get_level_id(&levels[0]).unwrap();
                }
                Err(msg) => warn!("Failed to load levels from file: {}", msg),
            }
        }
    }
}
