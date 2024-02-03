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
            UserInput::Single(Mouse(MouseButton::Other(1))),
            Action::Undo,
        ),
        (
            UserInput::Single(Mouse(MouseButton::Other(2))),
            Action::Redo,
        ),
        (
            UserInput::Single(MouseWheel(MouseWheelDirection::Down)),
            Action::ZoomOut,
        ),
        (
            UserInput::Single(MouseWheel(MouseWheelDirection::Up)),
            Action::ZoomIn,
        ),
    ]);
    let keyboard_input_map = InputMap::new([
        (UserInput::Single(Keyboard(KeyCode::W)), Action::MoveUp),
        (UserInput::Single(Keyboard(KeyCode::S)), Action::MoveDown),
        (UserInput::Single(Keyboard(KeyCode::A)), Action::MoveLeft),
        (UserInput::Single(Keyboard(KeyCode::D)), Action::MoveRight),
        (UserInput::Single(Keyboard(KeyCode::Up)), Action::MoveUp),
        (UserInput::Single(Keyboard(KeyCode::Down)), Action::MoveDown),
        (UserInput::Single(Keyboard(KeyCode::Left)), Action::MoveLeft),
        (
            UserInput::Single(Keyboard(KeyCode::Right)),
            Action::MoveRight,
        ),
        (
            UserInput::Chord(vec![Keyboard(KeyCode::ControlLeft), Keyboard(KeyCode::Z)]),
            Action::Undo,
        ),
        (
            UserInput::Chord(vec![
                Keyboard(KeyCode::ControlLeft),
                Keyboard(KeyCode::ShiftLeft),
                Keyboard(KeyCode::Z),
            ]),
            Action::Redo,
        ),
        (
            UserInput::Single(Keyboard(KeyCode::Escape)),
            Action::ResetLevel,
        ),
        (
            UserInput::Single(Keyboard(KeyCode::BracketRight)),
            Action::NextLevel,
        ),
        (
            UserInput::Single(Keyboard(KeyCode::BracketLeft)),
            Action::PreviousLevel,
        ),
        (
            UserInput::Chord(vec![
                Keyboard(KeyCode::ControlLeft),
                Keyboard(KeyCode::BracketRight),
            ]),
            Action::NextUnsolvedLevel,
        ),
        (
            UserInput::Chord(vec![
                Keyboard(KeyCode::ControlLeft),
                Keyboard(KeyCode::BracketLeft),
            ]),
            Action::PreviousUnsolvedLevel,
        ),
        (UserInput::Single(Keyboard(KeyCode::Equals)), Action::ZoomIn),
        (UserInput::Single(Keyboard(KeyCode::Minus)), Action::ZoomOut),
        (
            UserInput::Single(Keyboard(KeyCode::I)),
            Action::ToggleInstantMove,
        ),
        (
            UserInput::Single(Keyboard(KeyCode::P)),
            Action::ToggleAutomaticSolution,
        ),
        (
            UserInput::Single(Keyboard(KeyCode::F11)),
            Action::ToggleFullscreen,
        ),
        (
            UserInput::Chord(vec![Keyboard(KeyCode::ControlLeft), Keyboard(KeyCode::V)]),
            Action::ImportLevelsFromClipboard,
        ),
        (
            UserInput::Chord(vec![Keyboard(KeyCode::ControlLeft), Keyboard(KeyCode::C)]),
            Action::ExportLevelToClipboard,
        ),
        // Keyboard (Vim)
        (UserInput::Single(Keyboard(KeyCode::K)), Action::MoveUp),
        (UserInput::Single(Keyboard(KeyCode::J)), Action::MoveDown),
        (UserInput::Single(Keyboard(KeyCode::H)), Action::MoveLeft),
        (UserInput::Single(Keyboard(KeyCode::L)), Action::MoveRight),
        (UserInput::Single(Keyboard(KeyCode::U)), Action::Undo),
        (
            UserInput::Chord(vec![Keyboard(KeyCode::ControlLeft), Keyboard(KeyCode::R)]),
            Action::Redo,
        ),
    ]);
    let gamepad_input_map = InputMap::new([
        (
            UserInput::Single(GamepadButton(GamepadButtonType::DPadUp)),
            Action::MoveUp,
        ),
        (
            UserInput::Single(GamepadButton(GamepadButtonType::DPadDown)),
            Action::MoveDown,
        ),
        (
            UserInput::Single(GamepadButton(GamepadButtonType::DPadLeft)),
            Action::MoveLeft,
        ),
        (
            UserInput::Single(GamepadButton(GamepadButtonType::DPadRight)),
            Action::MoveRight,
        ),
        (
            UserInput::Single(GamepadButton(GamepadButtonType::East)),
            Action::Undo,
        ),
        (
            UserInput::Single(GamepadButton(GamepadButtonType::South)),
            Action::Redo,
        ),
        (
            UserInput::Single(GamepadButton(GamepadButtonType::RightTrigger)),
            Action::NextLevel,
        ),
        (
            UserInput::Single(GamepadButton(GamepadButtonType::LeftTrigger)),
            Action::PreviousLevel,
        ),
        (
            UserInput::Single(GamepadButton(GamepadButtonType::RightTrigger2)),
            Action::ZoomIn,
        ),
        (
            UserInput::Single(GamepadButton(GamepadButtonType::LeftTrigger2)),
            Action::ZoomOut,
        ),
        (
            UserInput::Single(GamepadButton(GamepadButtonType::West)),
            Action::ToggleInstantMove,
        ),
        (
            UserInput::Single(GamepadButton(GamepadButtonType::North)),
            Action::ToggleAutomaticSolution,
        ),
    ]);
    InputMap::default()
        .merge(&mouse_input_map)
        .merge(&keyboard_input_map)
        .merge(&gamepad_input_map)
        .clone()
}
