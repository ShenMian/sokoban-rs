use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Main,
    AutomaticSolution,
    AutoCratePush,
}
