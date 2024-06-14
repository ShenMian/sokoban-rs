use bevy::prelude::*;

#[derive(Event, Default)]
pub struct BoxEnterTarget;

#[derive(Event, Default)]
pub struct BoxLeaveTarget;

#[derive(Event, Default)]
pub struct LevelSolved;

#[derive(Event, Default)]
pub struct UpdateGridPositionEvent;
