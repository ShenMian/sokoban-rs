use bevy::prelude::*;

#[derive(Event, Default)]
pub struct BoxEnterGoal;

#[derive(Event, Default)]
pub struct BoxLeaveGoal;

#[derive(Event, Default)]
pub struct LevelSolved;

#[derive(Event, Default)]
pub struct UpdateGridPositionEvent;
