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
pub struct Hud;

#[derive(Component)]
pub struct Board {
    pub board: board::Board,
    pub tile_size: Vector2<f32>,
}

#[derive(Component, Deref, DerefMut)]
pub struct GridPosition(pub Vector2<i32>);

#[derive(Component)]
pub struct Player;

#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(benimator::State);

#[derive(Component)]
pub struct Crate;

#[derive(Component)]
pub struct PlayerMovableMark;

#[derive(Component)]
pub struct CratePushableMark;

#[derive(Component)]
pub struct LowerBoundMark;
