use bevy::prelude::*;

#[derive(Event, Default)]
pub struct CrateEnterTarget;

#[derive(Event, Default)]
pub struct CrateLeaveTarget;

#[derive(Event, Default)]
pub struct LevelSolved;

#[derive(Event, Default)]
pub struct UpdateGridPositionEvent;
