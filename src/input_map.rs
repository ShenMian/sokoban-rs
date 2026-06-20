use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
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

#[derive(
    Component, Reflect, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Debug,
)]
pub enum Action {
    ToggleInstantMove,
    ToggleAutomaticSolution,
    PreviousLevel,
    NextLevel,
}
