use std::{collections::HashMap, fs};

use bevy::{input::mouse::MouseMotion, prelude::*, window::WindowMode};
use bevy_enhanced_input::prelude::*;
use nalgebra::Vector2;
use soukoban::{path_finding::find_path, prelude::*};

use crate::{
    AppState, components::*, events::*, resources::*, systems::level::*, utils::PushState,
};

#[derive(Component)]
pub struct ControlsContext;

#[derive(InputAction)]
#[action_output(f32)]
pub struct ZoomAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleInstantMoveAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleAutomaticSolutionAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleFullscreenAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ResetLevelAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct NextLevelAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PreviousLevelAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct NextUnsolvedLevelAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PreviousUnsolvedLevelAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ImportLevelsFromClipboardAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ExportLevelToClipboardAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct UndoAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct RedoAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct MoveUpAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct MoveDownAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct MoveLeftAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct MoveRightAction;

pub fn setup_controls(mut commands: Commands) {
    commands.spawn((
        ControlsContext,
        actions!(ControlsContext[
            (
                bevy_enhanced_input::prelude::Action::<ZoomAction>::new(),
                Bindings::spawn((
                    Spawn((Binding::mouse_wheel(), SwizzleAxis::YXZ)),
                    Bidirectional::new(KeyCode::Equal, KeyCode::Minus),
                    Bidirectional::new(GamepadButton::RightTrigger2, GamepadButton::LeftTrigger2),
                )),
            ),
            (
                bevy_enhanced_input::prelude::Action::<ToggleInstantMoveAction>::new(),
                bindings![KeyCode::KeyI, GamepadButton::West],
            ),
            (
                bevy_enhanced_input::prelude::Action::<ToggleAutomaticSolutionAction>::new(),
                bindings![KeyCode::KeyP, GamepadButton::North],
            ),
            (
                bevy_enhanced_input::prelude::Action::<ToggleFullscreenAction>::new(),
                bindings![KeyCode::F11],
            ),
            (
                bevy_enhanced_input::prelude::Action::<ResetLevelAction>::new(),
                bindings![KeyCode::Escape],
            ),
            (
                bevy_enhanced_input::prelude::Action::<NextLevelAction>::new(),
                bindings![KeyCode::BracketRight, GamepadButton::RightTrigger],
            ),
            (
                bevy_enhanced_input::prelude::Action::<PreviousLevelAction>::new(),
                bindings![KeyCode::BracketLeft, GamepadButton::LeftTrigger],
            ),
            (
                bevy_enhanced_input::prelude::Action::<NextUnsolvedLevelAction>::new(),
                bindings![
                    Binding::Keyboard {
                        key: KeyCode::BracketRight,
                        mod_keys: ModKeys::CONTROL,
                    }
                ],
            ),
            (
                bevy_enhanced_input::prelude::Action::<PreviousUnsolvedLevelAction>::new(),
                bindings![
                    Binding::Keyboard {
                        key: KeyCode::BracketLeft,
                        mod_keys: ModKeys::CONTROL,
                    }
                ],
            ),
            (
                bevy_enhanced_input::prelude::Action::<ImportLevelsFromClipboardAction>::new(),
                bindings![
                    Binding::Keyboard {
                        key: KeyCode::KeyV,
                        mod_keys: ModKeys::CONTROL,
                    }
                ],
            ),
            (
                bevy_enhanced_input::prelude::Action::<ExportLevelToClipboardAction>::new(),
                bindings![
                    Binding::Keyboard {
                        key: KeyCode::KeyC,
                        mod_keys: ModKeys::CONTROL,
                    }
                ],
            ),
            (
                bevy_enhanced_input::prelude::Action::<UndoAction>::new(),
                bindings![
                    KeyCode::KeyU,
                    Binding::Keyboard {
                        key: KeyCode::KeyZ,
                        mod_keys: ModKeys::CONTROL,
                    },
                    Binding::MouseButton {
                        button: MouseButton::Other(1),
                        mod_keys: ModKeys::empty(),
                    },
                    GamepadButton::East,
                ],
            ),
            (
                bevy_enhanced_input::prelude::Action::<RedoAction>::new(),
                bindings![
                    Binding::Keyboard {
                        key: KeyCode::KeyR,
                        mod_keys: ModKeys::CONTROL,
                    },
                    Binding::Keyboard {
                        key: KeyCode::KeyZ,
                        mod_keys: ModKeys::CONTROL | ModKeys::SHIFT,
                    },
                    Binding::MouseButton {
                        button: MouseButton::Other(2),
                        mod_keys: ModKeys::empty(),
                    },
                    GamepadButton::South,
                ],
            ),
            (
                bevy_enhanced_input::prelude::Action::<MoveUpAction>::new(),
                bindings![KeyCode::KeyW, KeyCode::ArrowUp, KeyCode::KeyK, GamepadButton::DPadUp],
            ),
            (
                bevy_enhanced_input::prelude::Action::<MoveDownAction>::new(),
                bindings![KeyCode::KeyS, KeyCode::ArrowDown, KeyCode::KeyJ, GamepadButton::DPadDown],
            ),
            (
                bevy_enhanced_input::prelude::Action::<MoveLeftAction>::new(),
                bindings![KeyCode::KeyA, KeyCode::ArrowLeft, KeyCode::KeyH, GamepadButton::DPadLeft],
            ),
            (
                bevy_enhanced_input::prelude::Action::<MoveRightAction>::new(),
                bindings![KeyCode::KeyD, KeyCode::ArrowRight, KeyCode::KeyL, GamepadButton::DPadRight],
            ),
        ]),
    ));
}

pub fn on_zoom(trigger: On<Start<ZoomAction>>, mut camera: Query<&mut MainCamera>) {
    let mut main_camera = camera.single_mut().unwrap();
    if trigger.value > 0.0 {
        main_camera.target_scale /= 1.25;
    } else if trigger.value < 0.0 {
        main_camera.target_scale *= 1.25;
    }
}

pub fn on_move_up(
    _trigger: On<Start<MoveUpAction>>,
    mut board: Query<&mut Board>,
    mut player_movement: ResMut<PlayerMovement>,
    state: Res<State<AppState>>,
) {
    if *state.get() == AppState::Main {
        let board = &mut board.single_mut().unwrap().board;
        player_move(Direction::Up, &mut player_movement, board);
    }
}

pub fn on_move_down(
    _trigger: On<Start<MoveDownAction>>,
    mut board: Query<&mut Board>,
    mut player_movement: ResMut<PlayerMovement>,
    state: Res<State<AppState>>,
) {
    if *state.get() == AppState::Main {
        let board = &mut board.single_mut().unwrap().board;
        player_move(Direction::Down, &mut player_movement, board);
    }
}

pub fn on_move_left(
    _trigger: On<Start<MoveLeftAction>>,
    mut board: Query<&mut Board>,
    mut player_movement: ResMut<PlayerMovement>,
    state: Res<State<AppState>>,
) {
    if *state.get() == AppState::Main {
        let board = &mut board.single_mut().unwrap().board;
        player_move(Direction::Left, &mut player_movement, board);
    }
}

pub fn on_move_right(
    _trigger: On<Start<MoveRightAction>>,
    mut board: Query<&mut Board>,
    mut player_movement: ResMut<PlayerMovement>,
    state: Res<State<AppState>>,
) {
    if *state.get() == AppState::Main {
        let board = &mut board.single_mut().unwrap().board;
        player_move(Direction::Right, &mut player_movement, board);
    }
}

pub fn on_reset_level(
    _trigger: On<Start<ResetLevelAction>>,
    mut player_movement: ResMut<PlayerMovement>,
    mut level_id: ResMut<LevelId>,
) {
    player_movement.directions.clear();
    level_id.set_changed();
}

pub fn on_next_level(
    _trigger: On<Start<NextLevelAction>>,
    mut player_movement: ResMut<PlayerMovement>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
) {
    let database = database.lock().unwrap();
    player_movement.directions.clear();
    switch_to_next_level(&mut level_id, &database);
}

pub fn on_previous_level(
    _trigger: On<Start<PreviousLevelAction>>,
    mut player_movement: ResMut<PlayerMovement>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
) {
    let database = database.lock().unwrap();
    player_movement.directions.clear();
    switch_to_previous_level(&mut level_id, &database);
}

pub fn on_next_unsolved_level(
    _trigger: On<Start<NextUnsolvedLevelAction>>,
    mut player_movement: ResMut<PlayerMovement>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
) {
    let database = database.lock().unwrap();
    player_movement.directions.clear();
    switch_to_next_unsolved_level(&mut level_id, &database);
}

pub fn on_previous_unsolved_level(
    _trigger: On<Start<PreviousUnsolvedLevelAction>>,
    mut player_movement: ResMut<PlayerMovement>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
) {
    let database = database.lock().unwrap();
    player_movement.directions.clear();
    switch_to_previous_unsolved_level(&mut level_id, &database);
}

pub fn on_import_levels(
    _trigger: On<Start<ImportLevelsFromClipboardAction>>,
    mut player_movement: ResMut<PlayerMovement>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
) {
    let database = database.lock().unwrap();
    player_movement.directions.clear();
    import_from_clipboard(&mut level_id, &database);
}

pub fn on_export_level(
    _trigger: On<Start<ExportLevelToClipboardAction>>,
    mut player_movement: ResMut<PlayerMovement>,
    board: Query<&Board>,
) {
    let board = &board.single().unwrap().board;
    player_movement.directions.clear();
    export_to_clipboard(board);
}

pub fn on_toggle_instant_move(
    _trigger: On<Start<ToggleInstantMoveAction>>,
    mut config: ResMut<Config>,
) {
    config.instant_move = !config.instant_move;
}

pub fn on_toggle_fullscreen(
    _trigger: On<Start<ToggleFullscreenAction>>,
    mut window: Query<&mut Window>,
) {
    let Ok(mut window) = window.single_mut() else {
        return;
    };
    window.mode = match window.mode {
        WindowMode::BorderlessFullscreen(_) => WindowMode::Windowed,
        WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
        _ => unreachable!(),
    };
}

pub fn on_toggle_automatic_solution(
    _trigger: On<Start<ToggleAutomaticSolutionAction>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut player_movement: ResMut<PlayerMovement>,
) {
    player_movement.directions.clear();
    if *state.get() == AppState::Main {
        next_state.set(AppState::AutoSolve);
    } else {
        next_state.set(AppState::Main);
    }
}

pub fn on_undo(
    _trigger: On<Start<UndoAction>>,
    mut board: Query<&mut Board>,
    mut player_movement: ResMut<PlayerMovement>,
    mut update_grid_position_events: MessageWriter<UpdateGridPositionEvent>,
    state: Res<State<AppState>>,
) {
    if *state.get() == AppState::Main {
        let board = &mut board.single_mut().unwrap().board;
        player_movement.directions.clear();
        board.undo_push();
        update_grid_position_events.write_default();
    }
}

pub fn on_redo(
    _trigger: On<Start<RedoAction>>,
    mut board: Query<&mut Board>,
    mut player_movement: ResMut<PlayerMovement>,
    mut update_grid_position_events: MessageWriter<UpdateGridPositionEvent>,
    state: Res<State<AppState>>,
) {
    if *state.get() == AppState::Main {
        let board = &mut board.single_mut().unwrap().board;
        player_movement.directions.clear();
        board.redo_push();
        update_grid_position_events.write_default();
    }
}

pub fn player_move_unchecked(direction: Direction, player_movement: &mut PlayerMovement) {
    player_movement.directions.push_front(direction);
}

fn player_move_to(
    target: &Vector2<i32>,
    player_movement: &mut PlayerMovement,
    board: &crate::board::Board,
) {
    if let Some(path) = find_path(board.map.player_position(), *target, |position| {
        !board.map[position].intersects(Tiles::Wall | Tiles::Box)
    }) {
        let directions = path
            .windows(2)
            .map(|pos| Direction::try_from(pos[1] - pos[0]).unwrap());
        for direction in directions {
            player_move_unchecked(direction, player_movement);
        }
    }
}

fn player_move(
    direction: Direction,
    player_movement: &mut PlayerMovement,
    board: &crate::board::Board,
) {
    if !board.moveable(direction) {
        return;
    }
    player_movement.directions.push_front(direction);
}

fn instant_player_move_to(
    target: &Vector2<i32>,
    board_clone: &mut crate::board::Board,
    player_movement: &mut PlayerMovement,
) {
    if let Some(path) = find_path(board_clone.map.player_position(), *target, |position| {
        !board_clone.map[position].intersects(Tiles::Wall | Tiles::Box)
    }) {
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
    board_clone.do_action(direction);
    player_movement.directions.push_front(direction);
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
    let Board { board, tile_size } = &mut *board.single_mut().unwrap();
    let map = &board.map;
    let (camera, camera_transform) = camera.single_mut().unwrap();

    if mouse_buttons.just_pressed(MouseButton::Left) && player_movement.directions.is_empty() {
        let cursor_position = windows.single().unwrap().cursor_position();
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
                        for push_direction in Direction::iter() {
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
    mut motion_events: MessageReader<MouseMotion>,
    mut camera: Query<(&mut Transform, &MainCamera)>,
) {
    let (mut camera_transform, main_camera) = camera.single_mut().unwrap();
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
    mut events: MessageReader<FileDragAndDrop>,
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
