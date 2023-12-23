use bevy::prelude::*;
use nalgebra::Vector2;

#[derive(Event)]
pub struct SelectCrate(pub Vector2<i32>);

#[derive(Event)]
pub struct UnselectCrate;

#[derive(Event)]
pub struct UpdateGridPositionEvent;
