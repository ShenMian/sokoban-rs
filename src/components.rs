use bevy::prelude::*;
use nalgebra::Vector2;

use crate::board;

#[derive(Component)]
pub struct MainCamera {
    pub target_scale: f32,
}

impl Default for MainCamera {
    fn default() -> Self {
        Self { target_scale: 1.0 }
    }
}

#[derive(Component)]
pub struct HUD;

#[derive(Component)]
pub struct Board {
    pub board: board::Board,
    pub tile_size: Vector2<f32>,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Crate;

#[derive(Component)]
pub struct ReachableMark;

#[derive(Component, Deref, DerefMut)]
pub struct GridPosition(pub Vector2<i32>);
