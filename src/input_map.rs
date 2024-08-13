use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Actionlike, Component, Reflect, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Debug,
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

pub fn default_input_map() -> InputMap<Action> {
    let mouse_input_map = InputMap::default()
        .with_multiple([
            (Action::Undo, MouseButton::Other(1)),
            (Action::Redo, MouseButton::Other(2)),
        ])
        .with_multiple([
            (Action::ZoomOut, MouseScrollDirection::DOWN),
            (Action::ZoomIn, MouseScrollDirection::UP),
        ]);
    let keyboard_input_map = InputMap::default()
        .with_multiple([
            (Action::MoveUp, KeyCode::KeyW),
            (Action::MoveDown, KeyCode::KeyS),
            (Action::MoveLeft, KeyCode::KeyA),
            (Action::MoveRight, KeyCode::KeyD),
            (Action::MoveUp, KeyCode::ArrowUp),
            (Action::MoveDown, KeyCode::ArrowDown),
            (Action::MoveLeft, KeyCode::ArrowLeft),
            (Action::MoveRight, KeyCode::ArrowRight),
            (Action::ResetLevel, KeyCode::Escape),
            (Action::NextLevel, KeyCode::BracketRight),
            (Action::PreviousLevel, KeyCode::BracketLeft),
            (Action::ZoomIn, KeyCode::Equal),
            (Action::ZoomOut, KeyCode::Minus),
            (Action::ToggleInstantMove, KeyCode::KeyI),
            (Action::ToggleAutomaticSolution, KeyCode::KeyP),
            (Action::ToggleFullscreen, KeyCode::F11),
            // Vim
            (Action::MoveUp, KeyCode::KeyK),
            (Action::MoveDown, KeyCode::KeyJ),
            (Action::MoveLeft, KeyCode::KeyH),
            (Action::MoveRight, KeyCode::KeyL),
            (Action::Undo, KeyCode::KeyU),
        ])
        .with_multiple([
            (
                Action::Undo,
                ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::KeyZ]),
            ),
            (
                Action::Redo,
                ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::ShiftLeft, KeyCode::KeyZ]),
            ),
            (
                Action::NextUnsolvedLevel,
                ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::BracketRight]),
            ),
            (
                Action::PreviousUnsolvedLevel,
                ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::BracketLeft]),
            ),
            (
                Action::ImportLevelsFromClipboard,
                ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::KeyV]),
            ),
            (
                Action::ExportLevelToClipboard,
                ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::KeyC]),
            ),
            // Vim
            (
                Action::Redo,
                ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::KeyR]),
            ),
        ]);
    let gamepad_input_map = InputMap::default().with_multiple([
        (Action::MoveUp, GamepadButtonType::DPadUp),
        (Action::MoveDown, GamepadButtonType::DPadDown),
        (Action::MoveLeft, GamepadButtonType::DPadLeft),
        (Action::MoveRight, GamepadButtonType::DPadRight),
        (Action::Undo, GamepadButtonType::East),
        (Action::Redo, GamepadButtonType::South),
        (Action::NextLevel, GamepadButtonType::RightTrigger),
        (Action::PreviousLevel, GamepadButtonType::LeftTrigger),
        (Action::ZoomIn, GamepadButtonType::RightTrigger2),
        (Action::ZoomOut, GamepadButtonType::LeftTrigger2),
        (Action::ToggleInstantMove, GamepadButtonType::West),
        (Action::ToggleAutomaticSolution, GamepadButtonType::North),
    ]);
    InputMap::default()
        .merge(&mouse_input_map)
        .merge(&keyboard_input_map)
        .merge(&gamepad_input_map)
        .clone()
}
