use std::{collections::HashMap, fs};

use bevy::{input::mouse::MouseMotion, prelude::*, window::WindowMode};
use leafwing_input_manager::{action_diff::ActionDiffEvent, prelude::*};
use nalgebra::Vector2;
use soukoban::{direction::Direction, path_finding::find_path, Level, Tiles};

use crate::{
    components::*, events::*, resources::*, systems::level::*, utils::PushState, Action, AppState,
};

/// Clears the action state by consuming all stored actions.
pub fn clear_action_state(mut action_diff_events: EventReader<ActionDiffEvent<Action>>) {
    action_diff_events.clear();
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
    mut config: ResMut<Config>,

    mut update_grid_position_events: EventWriter<UpdateGridPositionEvent>,
) {
    let board = &mut board.single_mut().board;
    let main_camera = &mut *camera.single_mut();
    let database = database.lock().unwrap();
    let Ok(mut window) = window.get_single_mut() else {
        return;
    };
    let window = &mut *window;
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
            handle_toggle_fullscreen_action(&action_state, window);
            handle_undo_redo_action(
                &action_state,
                &mut player_movement,
                board,
                &mut update_grid_position_events,
            );
            handle_toggle_instant_move_action(&action_state, &mut config);
            handle_automatic_solution_action(
                &action_state,
                &state,
                &mut next_state,
                &mut player_movement,
            );
        }
        AppState::AutoMove => {
            handle_viewport_zoom_action(&action_state, main_camera);
            handle_toggle_fullscreen_action(&action_state, window);
        }
        AppState::AutoSolve => {
            handle_viewport_zoom_action(&action_state, main_camera);
            handle_toggle_fullscreen_action(&action_state, window);
            handle_toggle_instant_move_action(&action_state, &mut config);
            handle_automatic_solution_action(
                &action_state,
                &state,
                &mut next_state,
                &mut player_movement,
            );
        }
    }
}

/// Adds movement without checking for moveability.
pub fn player_move_unchecked(direction: Direction, player_movement: &mut PlayerMovement) {
    player_movement.directions.push_front(direction);
}

/// Moves the player to the specified target position on the board.
fn player_move_to(
    target: &Vector2<i32>,
    player_movement: &mut PlayerMovement,
    board: &crate::board::Board,
) {
    if let Some(path) = find_path(board.level.map().player_position(), *target, |position| {
        !board.level.map()[position].intersects(Tiles::Wall | Tiles::Box)
    }) {
        let directions = path
            .windows(2)
            .map(|pos| Direction::try_from(pos[1] - pos[0]).unwrap());
        for direction in directions {
            player_move_unchecked(direction, player_movement);
        }
    }
}

/// Adds movement if the move is valid.
fn player_move(
    direction: Direction,
    player_movement: &mut PlayerMovement,
    board: &crate::board::Board,
) {
    player_movement.directions.push_back(direction);
}

fn instant_player_move_to(
    target: &Vector2<i32>,
    board_clone: &mut crate::board::Board,
    player_movement: &mut PlayerMovement,
) {
    if let Some(path) = find_path(
        board_clone.level.map().player_position(),
        *target,
        |position| !board_clone.level.map()[position].intersects(Tiles::Wall | Tiles::Box),
    ) {
        let directions = path
            .windows(2)
            .map(|pos| Direction::try_from(pos[1] - pos[0]).unwrap());
        for direction in directions {
            instant_player_move(direction, board_clone, player_movement);
        }
    }
}

fn instant_player_move(
    direction: Direction,
    board_clone: &mut crate::board::Board,
    player_movement: &mut PlayerMovement,
) {
    board_clone.move_or_push(direction);
    player_movement.directions.push_front(direction);
}

fn handle_viewport_zoom_action(action_state: &ActionState<Action>, main_camera: &mut MainCamera) {
    if action_state.just_pressed(&Action::ZoomIn) {
        main_camera.target_scale /= 1.25;
    } else if action_state.just_pressed(&Action::ZoomOut) {
        main_camera.target_scale *= 1.25;
    }
}

fn handle_player_movement_action(
    action_state: &ActionState<Action>,
    player_movement: &mut ResMut<PlayerMovement>,
    board: &crate::board::Board,
) {
    // TODO: If you move the character through PlayerMovement, the character's
    // movement speed will be limited, giving the player a sense of input lag.
    if action_state.just_pressed(&Action::MoveUp) {
        player_move(Direction::Up, player_movement, board);
    }
    if action_state.just_pressed(&Action::MoveDown) {
        player_move(Direction::Down, player_movement, board);
    }
    if action_state.just_pressed(&Action::MoveLeft) {
        player_move(Direction::Left, player_movement, board);
    }
    if action_state.just_pressed(&Action::MoveRight) {
        player_move(Direction::Right, player_movement, board);
    }
}

fn handle_level_switch_action(
    action_state: &ActionState<Action>,
    player_movement: &mut ResMut<PlayerMovement>,
    level_id: &mut ResMut<LevelId>,
    database: &crate::database::Database,
) {
    if action_state.just_pressed(&Action::ResetLevel) {
        player_movement.directions.clear();
        level_id.set_changed();
    } else if action_state.just_pressed(&Action::NextLevel) {
        player_movement.directions.clear();
        switch_to_next_level(level_id, database);
    } else if action_state.just_pressed(&Action::PreviousLevel) {
        player_movement.directions.clear();
        switch_to_previous_level(level_id, database);
    } else if action_state.just_pressed(&Action::NextUnsolvedLevel) {
        player_movement.directions.clear();
        switch_to_next_unsolved_level(level_id, database);
    } else if action_state.just_pressed(&Action::PreviousUnsolvedLevel) {
        player_movement.directions.clear();
        switch_to_previous_unsolved_level(level_id, database);
    }
}

fn handle_clipboard_action(
    action_state: &ActionState<Action>,
    player_movement: &mut ResMut<PlayerMovement>,
    level_id: &mut ResMut<LevelId>,
    database: &crate::database::Database,
    board: &crate::board::Board,
) {
    if action_state.just_pressed(&Action::ImportLevelsFromClipboard) {
        player_movement.directions.clear();
        import_from_clipboard(level_id, database);
    }
    if action_state.just_pressed(&Action::ExportLevelToClipboard) {
        player_movement.directions.clear();
        export_to_clipboard(board);
    }
}

fn handle_toggle_instant_move_action(
    action_state: &ActionState<Action>,
    config: &mut ResMut<Config>,
) {
    if action_state.just_pressed(&Action::ToggleInstantMove) {
        config.instant_move = !config.instant_move;
    }
}

fn handle_toggle_fullscreen_action(action_state: &ActionState<Action>, window: &mut Window) {
    if action_state.just_pressed(&Action::ToggleFullscreen) {
        window.mode = match window.mode {
            WindowMode::BorderlessFullscreen(_) => WindowMode::Windowed,
            WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
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
    if action_state.just_pressed(&Action::Undo) {
        player_movement.directions.clear();
        board.undo_push();
        update_grid_position_events.send_default();
    }
    if action_state.just_pressed(&Action::Redo) {
        player_movement.directions.clear();
        board.redo_push();
        update_grid_position_events.send_default();
    }
}

pub fn handle_automatic_solution_action(
    action_state: &ActionState<Action>,
    state: &State<AppState>,
    next_state: &mut ResMut<NextState<AppState>>,
    player_movement: &mut ResMut<PlayerMovement>,
) {
    if action_state.just_pressed(&Action::ToggleAutomaticSolution) {
        player_movement.directions.clear();
        if *state == AppState::Main {
            next_state.set(AppState::AutoSolve);
        } else {
            next_state.set(AppState::Main);
        }
    }
}

/// Handles mouse input events.
pub fn mouse_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut board: Query<&mut Board>,
    windows: Query<&Window>,
    mut camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,

    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,

    mut player_movement: ResMut<PlayerMovement>,
    mut auto_move_state: ResMut<AutoMoveState>,
) {
    let Board { board, tile_size } = &mut *board.single_mut();
    let map = board.level.map();
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
        let grid_position =
            ((position + (tile_size.x as f32 / 2.0)) / tile_size.x as f32).as_ivec2();
        let grid_position = Vector2::new(grid_position.x, map.dimensions().y - grid_position.y);

        match state.get() {
            AppState::Main => {
                if map.box_positions().contains(&grid_position) {
                    *auto_move_state = AutoMoveState::Box {
                        position: grid_position,
                        paths: HashMap::new(),
                    };
                    next_state.set(AppState::AutoMove);
                    return;
                } else if map.player_position() == grid_position {
                    *auto_move_state = AutoMoveState::Player;
                    next_state.set(AppState::AutoMove);
                    return;
                }
            }
            AppState::AutoMove => {
                match &mut *auto_move_state {
                    AutoMoveState::Box {
                        position: box_position,
                        paths,
                    } => {
                        let mut box_paths = Vec::new();
                        for push_direction in [
                            Direction::Up,
                            Direction::Down,
                            Direction::Left,
                            Direction::Right,
                        ] {
                            let push_state = PushState {
                                push_direction,
                                box_position: grid_position,
                            };
                            if paths.contains_key(&push_state) {
                                if *box_position == grid_position {
                                    next_state.set(AppState::Main);
                                    return;
                                }
                                let box_path = paths[&push_state].clone();
                                box_paths.push(box_path);
                            }
                        }
                        if let Some(min_box_path) =
                            box_paths.iter().min_by_key(|box_path| box_path.len())
                        {
                            let mut board_clone = board.clone();
                            for (box_position, push_direction) in min_box_path
                                .windows(2)
                                .map(|pos| (pos[0], Direction::try_from(pos[1] - pos[0]).unwrap()))
                            {
                                let player_position = box_position - &push_direction.into();
                                instant_player_move_to(
                                    &player_position,
                                    &mut board_clone,
                                    &mut player_movement,
                                );
                                instant_player_move(
                                    push_direction,
                                    &mut board_clone,
                                    &mut player_movement,
                                );
                            }
                        } else if grid_position != *box_position
                            && map.box_positions().contains(&grid_position)
                        {
                            // box_position = grid_position;
                            // FIXME: Re-entering AppState::AutoMove https://github.com/bevyengine/bevy/issues/9130 https://github.com/bevyengine/bevy/pull/13579
                            // next_state.set(AppState::AutoMove);
                            next_state.set(AppState::Main);
                            return;
                        }
                        next_state.set(AppState::Main);
                        return;
                    }
                    AutoMoveState::Player => {
                        player_move_to(&grid_position, &mut player_movement, board);
                        next_state.set(AppState::Main);
                        return;
                    }
                }
            }
            _ => unreachable!(),
        }

        player_move_to(&grid_position, &mut player_movement, board);
    }
}

/// Adjusts the viewport based on various input events.
pub fn adjust_viewport(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    gamepads: Query<(Entity, &Gamepad)>,
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

    for (_entity, gamepad) in &gamepads {
        let right_stick = Vec2::new(
            gamepad.get(GamepadAxis::RightStickX).unwrap(),
            gamepad.get(GamepadAxis::RightStickY).unwrap(),
        );
        camera_transform.translation.x += right_stick.x * main_camera.target_scale * 1.6;
        camera_transform.translation.y += right_stick.y * main_camera.target_scale * 1.6;
    }
}

/// Handles file drag-and-drop events.
pub fn file_drag_and_drop(
    mut events: EventReader<FileDragAndDrop>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
) {
    for event in events.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            let database = database.lock().unwrap();
            info!("Load levels from file {:?}", path_buf);
            match Level::load_from_str(&fs::read_to_string(path_buf).unwrap())
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(levels) => {
                    info!("Done, {} levels loaded", levels.len());
                    database.import_levels(&levels);
                    level_id.0 = database.get_level_id(&levels[0]).unwrap();
                }
                Err(msg) => warn!("Failed to load levels from file: {}", msg),
            }
        }
    }
}
