use bevy::prelude::*;
use nalgebra::Vector2;

use crate::database;
use crate::direction::Direction;

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

#[derive(Resource)]
pub struct Database(pub Mutex<database::Database>);

#[derive(Resource)]
pub struct LevelId(pub u64);

#[derive(Resource)]
pub enum CrateReachable {
    None,
    Some {
        selected_crate: Vector2<i32>,
        came_from: HashMap<Vector2<i32>, Vector2<i32>>,
    },
}

impl Default for CrateReachable {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Resource)]
pub struct PlayerMovement {
    pub directions: VecDeque<Direction>,
    pub timer: Timer,
}

impl Default for PlayerMovement {
    fn default() -> Self {
        Self {
            directions: VecDeque::new(),
            timer: Timer::from_seconds(0.01, TimerMode::Repeating),
        }
    }
}
