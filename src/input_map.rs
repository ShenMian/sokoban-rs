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
            (Action::ResetLevel, KeyCode::Escape),
            (Action::NextLevel, KeyCode::BracketRight),
            (Action::PreviousLevel, KeyCode::BracketLeft),
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
        (Action::MoveUp, GamepadButton::DPadUp),
        (Action::MoveDown, GamepadButton::DPadDown),
        (Action::MoveLeft, GamepadButton::DPadLeft),
        (Action::MoveRight, GamepadButton::DPadRight),
        (Action::Undo, GamepadButton::East),
        (Action::Redo, GamepadButton::South),
        (Action::NextLevel, GamepadButton::RightTrigger),
        (Action::PreviousLevel, GamepadButton::LeftTrigger),
    ]);
    InputMap::default()
        .merge(&mouse_input_map)
        .merge(&keyboard_input_map)
        .merge(&gamepad_input_map)
        .clone()
}
