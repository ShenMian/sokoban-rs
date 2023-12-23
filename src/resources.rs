use bevy::prelude::*;
use nalgebra::Vector2;

use crate::database;
use crate::direction::Direction;
use crate::level::PushState;

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

#[derive(Resource)]
pub struct Settings {
    pub instant_move: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            instant_move: false,
        }
    }
}

#[derive(Resource, Deref)]
pub struct Database(pub Mutex<database::Database>);

#[derive(Resource, Deref, DerefMut)]
pub struct LevelId(pub u64);

#[derive(Resource)]
pub enum CrateReachable {
    None,
    Some {
        selected_crate: Vector2<i32>,
        path: HashMap<PushState, Vec<Vector2<i32>>>,
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
            timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        }
    }
}
