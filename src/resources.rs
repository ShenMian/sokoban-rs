use bevy::prelude::*;
use bevy::time::Stopwatch;
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::direction::Direction;
use crate::level::PushState;
use crate::solver::solver::*;
use crate::{database, Level};

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

#[derive(Resource, Serialize, Deserialize)]
pub struct Settings {
    /// Disable player movement animation.
    pub instant_move: bool,
    /// Player movement animation speed, seconds per step.
    pub player_move_speed: f32,
    /// Make the floor look like a chessboard with alternating light square and dark square.
    pub even_square_shades: f32,
    /// Audio volume
    pub volume: f64,
    /// Enable auto switch to next unsolved level when the current level is solved.
    pub auto_switch_to_next_unsolved_level: bool,
    pub solver: SolverSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            instant_move: false,
            player_move_speed: 0.1,
            even_square_shades: 0.1,
            volume: 0.5,
            auto_switch_to_next_unsolved_level: true,
            solver: SolverSettings::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SolverSettings {
    pub strategy: Strategy,
    pub lower_bound_method: LowerBoundMethod,
}

impl Default for SolverSettings {
    fn default() -> Self {
        Self {
            strategy: Strategy::Fast,
            lower_bound_method: LowerBoundMethod::MinimumPush,
        }
    }
}

#[derive(Resource, Deref)]
pub struct Database(pub Mutex<database::Database>);

#[derive(Resource, Deref, DerefMut)]
pub struct LevelId(pub u64);

#[derive(Resource)]
pub struct PlayerMovement {
    pub directions: VecDeque<Direction>,
    pub timer: Timer,
}

#[derive(Resource)]
pub struct AutoCratePushState {
    pub selected_crate: Vector2<i32>,
    pub paths: HashMap<PushState, Vec<Vector2<i32>>>,
}

impl Default for AutoCratePushState {
    fn default() -> Self {
        Self {
            selected_crate: Vector2::zeros(),
            paths: HashMap::new(),
        }
    }
}

#[derive(Resource)]
pub struct SolverState {
    pub solver: Mutex<Solver>,
    pub level: Level,
    pub stopwatch: Stopwatch,
}

impl Default for SolverState {
    fn default() -> Self {
        Self {
            solver: Mutex::new(Solver::new(
                Level::empty(),
                Strategy::Fast,
                LowerBoundMethod::MinimumPush,
            )),
            level: Level::empty(),
            stopwatch: Stopwatch::new(),
        }
    }
}
