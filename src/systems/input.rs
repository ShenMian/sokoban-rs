use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::WindowMode;
use leafwing_input_manager::prelude::*;
use nalgebra::Vector2;

use crate::direction::Direction;
use crate::events::*;
use crate::level::{Level, PushState, Tile};
use crate::resources::*;
use crate::solver::solver::*;
use crate::systems::level::*;
use crate::AppState;
use crate::{components::*, Action};

pub fn player_move_to(
    target: &Vector2<i32>,
    player_movement: &mut PlayerMovement,
    board: &crate::board::Board,
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
            player_move(direction, player_movement);
        }
    }
}

pub fn player_move(direction: Direction, player_movement: &mut PlayerMovement) {
    player_movement.directions.push_front(direction);
}

pub fn player_move_with_check(
    direction: Direction,
    player_movement: &mut PlayerMovement,
    board: &crate::board::Board,
) {
    if !board.moveable(direction) {
        return;
    }
    player_movement.directions.push_front(direction);
}

pub fn instant_player_move_to(
    target: &Vector2<i32>,
    board_clone: &mut crate::board::Board,
    player_movement: &mut PlayerMovement,
) {
    if let Some(path) = find_path(&board_clone.level.player_position, target, |position| {
        board_clone
            .level
            .get_unchecked(&position)
            .intersects(Tile::Wall | Tile::Crate)
    }) {
        let directions = path
            .windows(2)
            .map(|pos| Direction::from_vector(pos[1] - pos[0]).unwrap());
        for direction in directions {
            instant_player_move(direction, board_clone, player_movement);
        }
    }
}

pub fn instant_player_move(
    direction: Direction,
    board_clone: &mut crate::board::Board,
    player_movement: &mut PlayerMovement,
) {
    board_clone.move_or_push(direction);
    player_movement.directions.push_front(direction);
}

pub fn clear_action_state(mut action_state: ResMut<ActionState<Action>>) {
    action_state.consume_all();
}

pub fn handle_actions(
    action_state: Res<ActionState<Action>>,

    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,

    mut camera: Query<&mut MainCamera>,
    mut board: Query<&mut Board>,
    mut window: Query<&mut Window>,

    mut player_movement: ResMut<PlayerMovement>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
    mut settings: ResMut<Settings>,

    mut update_grid_position_events: EventWriter<UpdateGridPositionEvent>,
) {
    let board = &mut board.single_mut().board;
    let main_camera = &mut *camera.single_mut();
    let database = database.lock().unwrap();
    let window = &mut *window.single_mut();
    match state.get() {
        AppState::Main => {
            handle_viewport_zoom_action(&action_state, main_camera);
            handle_player_movement_action(&action_state, &mut player_movement, board);
            handle_level_switch_action(
                &action_state,
                &mut player_movement,
                &mut level_id,
                &database,
            );
            handle_clipboard_action(
                &action_state,
                &mut player_movement,
                &mut level_id,
                &database,
                board,
            );
            handle_toggle_instant_move_action(&action_state, &mut settings);
            handle_toggle_toggle_fullscreen_action(&action_state, window);
            handle_undo_redo_action(
                &action_state,
                &mut player_movement,
                board,
                &mut update_grid_position_events,
            );
            handle_automatic_solution_action(
                &action_state,
                &state,
                &mut next_state,
                &mut player_movement,
            );
        }
        AppState::AutoCratePush => handle_viewport_zoom_action(&action_state, main_camera),
        AppState::AutoSolve => handle_viewport_zoom_action(&action_state, main_camera),
    }
}

fn handle_viewport_zoom_action(action_state: &ActionState<Action>, main_camera: &mut MainCamera) {
    if action_state.just_pressed(Action::ZoomOut) {
        main_camera.target_scale *= 1.25;
    } else if action_state.just_pressed(Action::ZoomIn) {
        main_camera.target_scale /= 1.25;
    }
}

fn handle_player_movement_action(
    action_state: &ActionState<Action>,
    player_movement: &mut ResMut<PlayerMovement>,
    board: &crate::board::Board,
) {
    // TODO: 通过 PlayerMovement 移动角色受角色移动速度限制, 会给玩家带来输入的迟滞感
    if action_state.just_pressed(Action::MoveUp) {
        player_move_with_check(Direction::Up, player_movement, board);
    }
    if action_state.just_pressed(Action::MoveDown) {
        player_move_with_check(Direction::Down, player_movement, board);
    }
    if action_state.just_pressed(Action::MoveLeft) {
        player_move_with_check(Direction::Left, player_movement, board);
    }
    if action_state.just_pressed(Action::MoveRight) {
        player_move_with_check(Direction::Right, player_movement, board);
    }
}

fn handle_level_switch_action(
    action_state: &ActionState<Action>,
    player_movement: &mut ResMut<PlayerMovement>,
    level_id: &mut ResMut<LevelId>,
    database: &crate::database::Database,
) {
    if action_state.just_pressed(Action::ResetLevel) {
        player_movement.directions.clear();
        level_id.0 = level_id.0;
    } else if action_state.just_pressed(Action::PreviousLevel) {
        player_movement.directions.clear();
        switch_to_previous_level(level_id, database);
    } else if action_state.just_pressed(Action::NextLevel) {
        player_movement.directions.clear();
        switch_to_next_level(level_id, database);
    }
}

fn handle_clipboard_action(
    action_state: &ActionState<Action>,
    player_movement: &mut ResMut<PlayerMovement>,
    level_id: &mut ResMut<LevelId>,
    database: &crate::database::Database,
    board: &crate::board::Board,
) {
    if action_state.just_pressed(Action::ImportLevelsFromClipboard) {
        player_movement.directions.clear();
        import_from_clipboard(level_id, database);
    }
    if action_state.just_pressed(Action::ExportLevelToClipboard) {
        player_movement.directions.clear();
        export_to_clipboard(board);
    }
}

fn handle_toggle_instant_move_action(
    action_state: &ActionState<Action>,
    settings: &mut ResMut<Settings>,
) {
    if action_state.just_pressed(Action::ToggleInstantMove) {
        settings.instant_move = !settings.instant_move;
    }
}

fn handle_toggle_toggle_fullscreen_action(action_state: &ActionState<Action>, window: &mut Window) {
    if action_state.just_pressed(Action::ToggleFullscreen) {
        window.mode = match window.mode {
            WindowMode::BorderlessFullscreen => WindowMode::Windowed,
            WindowMode::Windowed => WindowMode::BorderlessFullscreen,
            _ => unreachable!(),
        };
    }
}

fn handle_undo_redo_action(
    action_state: &ActionState<Action>,
    player_movement: &mut PlayerMovement,
    board: &mut crate::board::Board,
    update_grid_position_events: &mut EventWriter<UpdateGridPositionEvent>,
) {
    if action_state.just_pressed(Action::Undo) {
        player_movement.directions.clear();
        board.undo_push();
        update_grid_position_events.send(UpdateGridPositionEvent);
    }
    if action_state.just_pressed(Action::Redo) {
        player_movement.directions.clear();
        board.redo_push();
        update_grid_position_events.send(UpdateGridPositionEvent);
    }
}

pub fn handle_automatic_solution_action(
    action_state: &ActionState<Action>,
    state: &State<AppState>,
    next_state: &mut ResMut<NextState<AppState>>,
    player_movement: &mut ResMut<PlayerMovement>,
) {
    if action_state.just_pressed(Action::ToggleAutomaticSolution) {
        player_movement.directions.clear();
        if *state == AppState::Main {
            next_state.set(AppState::AutoSolve);
        } else {
            next_state.set(AppState::Main);
        }
    }
}

pub fn mouse_input(
    mouse_buttons: Res<Input<MouseButton>>,
    mut board: Query<&mut Board>,
    windows: Query<&Window>,
    mut camera: Query<(&Camera, &GlobalTransform)>,

    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,

    mut player_movement: ResMut<PlayerMovement>,
    mut auto_crate_push_state: ResMut<AutoCratePushState>,
) {
    let Board { board, tile_size } = &mut *board.single_mut();
    let (camera, camera_transform) = camera.single_mut();

    if mouse_buttons.just_pressed(MouseButton::Left) && player_movement.directions.is_empty() {
        let cursor_position = windows.single().cursor_position();
        if cursor_position.is_none() {
            return;
        }
        let cursor_position = cursor_position.unwrap();
        let position = camera
            .viewport_to_world_2d(camera_transform, cursor_position)
            .unwrap();
        let grid_position = ((position + (tile_size.x / 2.0)) / tile_size.x).as_ivec2();
        let grid_position =
            Vector2::new(grid_position.x, board.level.dimensions.y - grid_position.y);

        match state.get() {
            AppState::Main => {
                if board.level.crate_positions.contains(&grid_position) {
                    auto_crate_push_state.selected_crate = grid_position;
                    next_state.set(AppState::AutoCratePush);
                    return;
                }
            }
            AppState::AutoCratePush => {
                let AutoCratePushState {
                    selected_crate,
                    paths,
                } = &mut *auto_crate_push_state;
                let mut crate_paths = Vec::new();
                for &push_direction in [
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ]
                .iter()
                {
                    let push_state = PushState {
                        push_direction,
                        crate_position: grid_position,
                    };
                    if paths.contains_key(&push_state) {
                        if *selected_crate == grid_position {
                            next_state.set(AppState::Main);
                            return;
                        }
                        let crate_path = paths[&push_state].clone();
                        crate_paths.push(crate_path);
                    }
                }
                if let Some(min_crate_path) =
                    crate_paths.iter().min_by_key(|crate_path| crate_path.len())
                {
                    let mut board_clone = board.clone();
                    for (crate_position, push_direction) in min_crate_path
                        .windows(2)
                        .map(|pos| (pos[0], Direction::from_vector(pos[1] - pos[0]).unwrap()))
                    {
                        let player_position = crate_position - push_direction.to_vector();
                        instant_player_move_to(
                            &player_position,
                            &mut board_clone,
                            &mut player_movement,
                        );
                        instant_player_move(push_direction, &mut board_clone, &mut player_movement);
                    }
                } else if grid_position != *selected_crate
                    && board.level.crate_positions.contains(&grid_position)
                {
                    // auto_crate_push_state.selected_crate = grid_position;
                    // FIXME: Re-entering AppState::AutoCratePush https://github.com/bevyengine/bevy/issues/9130
                    // next_state.set(AppState::AutoCratePush);
                    next_state.set(AppState::Main);
                    return;
                }
                next_state.set(AppState::Main);
                return;
            }
            _ => unreachable!(),
        }

        player_move_to(&grid_position, &mut player_movement, board);
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
