use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Actionlike, Component, Reflect, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    Undo,
    Redo,

    ResetLevel,
    NextLevel,
    PreviousLevel,
    NextUnsolvedLevel,
    PreviousUnsolvedLevel,

    ZoomIn,
    ZoomOut,

    ToggleInstantMove,
    ToggleAutomaticSolution,
    ToggleFullscreen,

    ImportLevelsFromClipboard,
    ExportLevelToClipboard,
}

pub fn default_input_action_map() -> InputMap<Action> {
    use leafwing_input_manager::user_input::InputKind::*;
    let mouse_input_map = InputMap::new([
        (
            Action::Undo,
            UserInput::Single(Mouse(MouseButton::Other(1))),
        ),
        (
            Action::Redo,
            UserInput::Single(Mouse(MouseButton::Other(2))),
        ),
        (
            Action::ZoomOut,
            UserInput::Single(MouseWheel(MouseWheelDirection::Down)),
        ),
        (
            Action::ZoomIn,
            UserInput::Single(MouseWheel(MouseWheelDirection::Up)),
        ),
    ]);
    let keyboard_input_map = InputMap::new([
        (Action::MoveUp, UserInput::Single(PhysicalKey(KeyCode::KeyW))),
        (Action::MoveDown, UserInput::Single(PhysicalKey(KeyCode::KeyS))),
        (Action::MoveLeft, UserInput::Single(PhysicalKey(KeyCode::KeyA))),
        (Action::MoveRight, UserInput::Single(PhysicalKey(KeyCode::KeyD))),
        (Action::MoveUp, UserInput::Single(PhysicalKey(KeyCode::ArrowUp))),
        (Action::MoveDown, UserInput::Single(PhysicalKey(KeyCode::ArrowDown))),
        (Action::MoveLeft, UserInput::Single(PhysicalKey(KeyCode::ArrowLeft))),
        (
            Action::MoveRight,
            UserInput::Single(PhysicalKey(KeyCode::ArrowRight)),
        ),
        (
            Action::Undo,
            UserInput::Chord(vec![PhysicalKey(KeyCode::ControlLeft), PhysicalKey(KeyCode::KeyZ)]),
        ),
        (
            Action::Redo,
            UserInput::Chord(vec![
                PhysicalKey(KeyCode::ControlLeft),
                PhysicalKey(KeyCode::ShiftLeft),
                PhysicalKey(KeyCode::KeyZ),
            ]),
        ),
        (
            Action::ResetLevel,
            UserInput::Single(PhysicalKey(KeyCode::Escape)),
        ),
        (
            Action::NextLevel,
            UserInput::Single(PhysicalKey(KeyCode::BracketRight)),
        ),
        (
            Action::PreviousLevel,
            UserInput::Single(PhysicalKey(KeyCode::BracketLeft)),
        ),
        (
            Action::NextUnsolvedLevel,
            UserInput::Chord(vec![
                PhysicalKey(KeyCode::ControlLeft),
                PhysicalKey(KeyCode::BracketRight),
            ]),
        ),
        (
            Action::PreviousUnsolvedLevel,
            UserInput::Chord(vec![
                PhysicalKey(KeyCode::ControlLeft),
                PhysicalKey(KeyCode::BracketLeft),
            ]),
        ),
        (Action::ZoomIn, UserInput::Single(PhysicalKey(KeyCode::Equal))),
        (Action::ZoomOut, UserInput::Single(PhysicalKey(KeyCode::Minus))),
        (
            Action::ToggleInstantMove,
            UserInput::Single(PhysicalKey(KeyCode::KeyI)),
        ),
        (
            Action::ToggleAutomaticSolution,
            UserInput::Single(PhysicalKey(KeyCode::KeyP)),
        ),
        (
            Action::ToggleFullscreen,
            UserInput::Single(PhysicalKey(KeyCode::F11)),
        ),
        (
            Action::ImportLevelsFromClipboard,
            UserInput::Chord(vec![PhysicalKey(KeyCode::ControlLeft), PhysicalKey(KeyCode::KeyV)]),
        ),
        (
            Action::ExportLevelToClipboard,
            UserInput::Chord(vec![PhysicalKey(KeyCode::ControlLeft), PhysicalKey(KeyCode::KeyC)]),
        ),
        // PhysicalKey (Vim)
        (Action::MoveUp, UserInput::Single(PhysicalKey(KeyCode::KeyK))),
        (Action::MoveDown, UserInput::Single(PhysicalKey(KeyCode::KeyJ))),
        (Action::MoveLeft, UserInput::Single(PhysicalKey(KeyCode::KeyH))),
        (Action::MoveRight, UserInput::Single(PhysicalKey(KeyCode::KeyL))),
        (Action::Undo, UserInput::Single(PhysicalKey(KeyCode::KeyU))),
        (
            Action::Redo,
            UserInput::Chord(vec![PhysicalKey(KeyCode::ControlLeft), PhysicalKey(KeyCode::KeyR)]),
        ),
    ]);
    let gamepad_input_map = InputMap::new([
        (
            Action::MoveUp,
            UserInput::Single(GamepadButton(GamepadButtonType::DPadUp)),
        ),
        (
            Action::MoveDown,
            UserInput::Single(GamepadButton(GamepadButtonType::DPadDown)),
        ),
        (
            Action::MoveLeft,
            UserInput::Single(GamepadButton(GamepadButtonType::DPadLeft)),
        ),
        (
            Action::MoveRight,
            UserInput::Single(GamepadButton(GamepadButtonType::DPadRight)),
        ),
        (
            Action::Undo,
            UserInput::Single(GamepadButton(GamepadButtonType::East)),
        ),
        (
            Action::Redo,
            UserInput::Single(GamepadButton(GamepadButtonType::South)),
        ),
        (
            Action::NextLevel,
            UserInput::Single(GamepadButton(GamepadButtonType::RightTrigger)),
        ),
        (
            Action::PreviousLevel,
            UserInput::Single(GamepadButton(GamepadButtonType::LeftTrigger)),
        ),
        (
            Action::ZoomIn,
            UserInput::Single(GamepadButton(GamepadButtonType::RightTrigger2)),
        ),
        (
            Action::ZoomOut,
            UserInput::Single(GamepadButton(GamepadButtonType::LeftTrigger2)),
        ),
        (
            Action::ToggleInstantMove,
            UserInput::Single(GamepadButton(GamepadButtonType::West)),
        ),
        (
            Action::ToggleAutomaticSolution,
            UserInput::Single(GamepadButton(GamepadButtonType::North)),
        ),
    ]);
    InputMap::default()
        .merge(&mouse_input_map)
        .merge(&keyboard_input_map)
        .merge(&gamepad_input_map)
        .clone()
}
