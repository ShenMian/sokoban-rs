use bevy::prelude::*;
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::database;
use crate::direction::Direction;
use crate::level::PushState;
use crate::solver::solver::*;

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

#[derive(Resource, Serialize, Deserialize)]
pub struct Settings {
    pub instant_move: bool,
    pub player_move_speed: f32,
    pub solver: SolverSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            instant_move: false,
            player_move_speed: 0.05,
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
            lower_bound_method: LowerBoundMethod::PushCount,
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
pub enum CrateReachable {
    None,
    Some {
        selected_crate: Vector2<i32>,
        paths: HashMap<PushState, Vec<Vector2<i32>>>,
    },
}

impl Default for CrateReachable {
    fn default() -> Self {
        Self::None
    }
}
