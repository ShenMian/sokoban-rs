use bevy::prelude::*;
use soukoban::prelude::Point;

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
    pub tile_size: Point,
}

#[derive(Component, Deref, DerefMut)]
pub struct GridPosition(pub Point);

#[derive(Component)]
pub struct Player;

#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(benimator::State);

#[derive(Component)]
pub struct Box;

/// Box entities' grid position and sprite.
pub type BoxGridPositionAndSpriteQuery<'w, 's> =
    Query<'w, 's, (&'static GridPosition, &'static mut Sprite), (With<Box>, Without<Player>)>;

/// The player entity's sprite.
pub type PlayerSpriteQuery<'w, 's> = Query<'w, 's, &'static mut Sprite, With<Player>>;

/// Marks auto-move hint entities (pushable/movable marks).
#[derive(Component)]
pub struct AutoMoveMark;
