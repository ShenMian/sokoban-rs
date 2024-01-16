use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use itertools::Itertools;
use leafwing_input_manager::{prelude::*, user_input::InputKind};
use nalgebra::Vector2;

use crate::direction::Direction;
use crate::events::*;
use crate::level::{Level, PushState, Tile};
use crate::resources::*;
use crate::solver::solver::*;
use crate::systems::level::*;
use crate::{components::*, AppState};

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
    player_movement: &mut PlayerMovement,
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

pub fn player_move_or_push(
    direction: Direction,
    board: &mut crate::board::Board,
    player_movement: &mut PlayerMovement,
) {
    if board.move_or_push(direction) {
        player_movement.directions.push_front(direction);
    }
}

pub fn clear_action_state(mut action_state: ResMut<ActionState<Action>>) {
    action_state.consume_all();
}

pub fn handle_other_action(
    action_state: Res<ActionState<Action>>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
    mut player_movement: ResMut<PlayerMovement>,
    mut settings: ResMut<Settings>,

    mut board: Query<&mut Board>,

    mut update_grid_position_events: EventWriter<UpdateGridPositionEvent>,
) {
    let board = &mut board.single_mut().board;

    if action_state.just_pressed(Action::MoveUp) {
        player_move_or_push(Direction::Up, board, &mut player_movement);
    }
    if action_state.just_pressed(Action::MoveDown) {
        player_move_or_push(Direction::Down, board, &mut player_movement);
    }
    if action_state.just_pressed(Action::MoveLeft) {
        player_move_or_push(Direction::Left, board, &mut player_movement);
    }
    if action_state.just_pressed(Action::MoveRight) {
        player_move_or_push(Direction::Right, board, &mut player_movement);
    }

    if action_state.just_pressed(Action::Undo) {
        board.undo_push();
        player_movement.directions.clear();
        update_grid_position_events.send(UpdateGridPositionEvent);
    }
    if action_state.just_pressed(Action::Redo) {
        board.redo_push();
        player_movement.directions.clear();
        update_grid_position_events.send(UpdateGridPositionEvent);
    }

    if action_state.just_pressed(Action::InstantMove) {
        settings.instant_move = !settings.instant_move;
    }

    let database = database.lock().unwrap();
    if action_state.just_pressed(Action::PreviousLevel) {
        player_movement.directions.clear();
        switch_to_previous_level(&mut level_id, &database);
    }
    if action_state.just_pressed(Action::NextLevel) {
        player_movement.directions.clear();
        switch_to_next_level(&mut level_id, &database);
    }

    if action_state.just_pressed(Action::ImportLevelsFromClipboard) {
        player_movement.directions.clear();
        import_from_clipboard(&mut level_id, &database);
    }
    if action_state.just_pressed(Action::ExportLevelToClipboard) {
        player_movement.directions.clear();
        export_to_clipboard(&board);
    }
}

pub fn handle_viewport_zoom_action(
    action_state: Res<ActionState<Action>>,
    mut camera: Query<&mut MainCamera>,
) {
    let mut main_camera = camera.single_mut();
    if action_state.just_pressed(Action::ZoomOut) {
        main_camera.target_scale *= 1.25;
    }
    if action_state.just_pressed(Action::ZoomIn) {
        main_camera.target_scale /= 1.25;
    }
}

pub fn handle_automatic_solution_action(
    action_state: Res<ActionState<Action>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if action_state.just_pressed(Action::AutomaticSolution) {
        if *state == AppState::Main {
            next_state.set(AppState::AutoSolve);
        } else {
            next_state.set(AppState::Main);
        }
    }
}

pub fn spawn_crate_pushable_marks(
    mut commands: Commands,
    mut auto_crate_push_state: ResMut<AutoCratePushState>,
    board: Query<&Board>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let crate_position = &auto_crate_push_state.selected_crate;
    let Board { board, tile_size } = board.single();

    let paths = board.level.crate_pushable_paths(crate_position);

    // spawn crate pushable marks
    for crate_position in paths.keys().map(|state| state.crate_position).unique() {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::GREEN.with_a(0.8),
                    custom_size: Some(Vec2::new(tile_size.x / 4.0, tile_size.y / 4.0)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    crate_position.x as f32 * tile_size.x,
                    (board.level.dimensions.y - crate_position.y) as f32 * tile_size.y,
                    10.0,
                ),
                ..default()
            },
            CratePushableMark,
        ));
    }

    if paths.is_empty() {
        next_state.set(AppState::Main);
    }
    auto_crate_push_state.paths = paths;
}

pub fn despawn_crate_pushable_marks(
    mut commands: Commands,
    marks: Query<Entity, With<CratePushableMark>>,
) {
    marks.for_each(|entity| commands.entity(entity).despawn());
}

pub fn mouse_input_to_action(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut action_state: ResMut<ActionState<Action>>,
) {
    for event in mouse_wheel_events.read() {
        if event.y < 0.0 {
            action_state.press(Action::ZoomOut);
        } else {
            action_state.press(Action::ZoomIn);
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

    if mouse_buttons.just_pressed(MouseButton::Left) {
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
                    // FIXME: https://github.com/bevyengine/bevy/issues/9130
                    // auto_crate_push_state.selected_crate = grid_position;
                    // next_state.set(AppState::AutoCratePush);
                    next_state.set(AppState::Main);
                    return;
                }
                next_state.set(AppState::Main);
                return;
            }
            _ => unreachable!(),
        }

        player_move_to(&grid_position, board, &mut player_movement);
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
