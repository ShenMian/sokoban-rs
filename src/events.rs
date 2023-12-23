use bevy::prelude::*;

#[derive(Event)]
pub struct UpdateGridPositionEvent;

#[derive(Event)]
pub struct SpawnCrateReachableMarks;

#[derive(Event)]
pub struct DespawnCrateReachableMarks;
