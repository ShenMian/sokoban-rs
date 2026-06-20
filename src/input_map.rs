use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

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
        ]),
    ));
}

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
            // Vim
            (
                Action::Redo,
                ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::KeyR]),
            ),
        ]);
    let gamepad_input_map = InputMap::default().with_multiple([
        (Action::MoveUp, GamepadButton::DPadUp),
        (Action::MoveDown, GamepadButton::DPadDown),
        (Action::MoveLeft, GamepadButton::DPadLeft),
        (Action::MoveRight, GamepadButton::DPadRight),
        (Action::Undo, GamepadButton::East),
        (Action::Redo, GamepadButton::South),
    ]);
    InputMap::default()
        .merge(&mouse_input_map)
        .merge(&keyboard_input_map)
        .merge(&gamepad_input_map)
        .clone()
}
