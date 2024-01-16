use bevy::prelude::*;
use nalgebra::Vector2;

#[derive(Event)]
pub struct SelectCrateEvent(pub Vector2<i32>);

#[derive(Event)]
pub struct UnselectCrateEvent;

#[derive(Event)]
pub struct UpdateGridPositionEvent;
