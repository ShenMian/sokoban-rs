use bevy::{prelude::*, time::Stopwatch};
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};
use soukoban::{Map, direction::Direction};

use crate::{board::Board, database, solve::solver::*, utils::PushState};

use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

#[derive(Resource, Serialize, Deserialize)]
pub struct Config {
    /// Player movement animation speed, seconds per step.
    pub player_move_speed: f32,
    /// Make the floor look like a chessboard with alternating light square and dark square.
    pub even_square_shades: f32,
    /// Audio volume.
    pub volume: f32,
    /// Disable player movement animation.
    pub instant_move: bool,
    /// Enable auto switch to next unsolved level when the current level is solved.
    pub auto_switch_to_next_unsolved_level: bool,
    pub solver: SolverConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            player_move_speed: 0.1,
            even_square_shades: 0.1,
            volume: 0.5,
            instant_move: false,
            auto_switch_to_next_unsolved_level: true,
            solver: SolverConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct SolverConfig {
    pub strategy: Strategy,
    pub lower_bound_method: LowerBoundMethod,
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

impl PlayerMovement {
    pub fn new(player_move_speed: f32) -> Self {
        Self {
            directions: VecDeque::new(),
            timer: Timer::from_seconds(player_move_speed, TimerMode::Repeating),
        }
    }
}

#[derive(Resource, Default)]
pub enum AutoMoveState {
    #[default]
    Player,
    Box {
        position: Vector2<i32>,
        paths: HashMap<PushState, Vec<Vector2<i32>>>,
    },
}

#[derive(Resource)]
pub struct SolverState {
    pub solver: Mutex<Solver>,
    pub stopwatch: Stopwatch,
    pub origin_board: Board,
}

impl Default for SolverState {
    fn default() -> Self {
        Self {
            solver: Mutex::new(Solver::new(
                Map::with_dimensions(Vector2::new(0, 0)),
                Strategy::default(),
                LowerBoundMethod::default(),
            )),
            stopwatch: Stopwatch::new(),
            origin_board: Board::with_map(Map::with_dimensions(Vector2::new(0, 0))),
        }
    }
}
