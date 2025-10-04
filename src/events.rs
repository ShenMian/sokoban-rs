use bevy::prelude::*;

#[derive(Message, Default)]
pub struct BoxEnterGoal;

#[derive(Message, Default)]
pub struct BoxLeaveGoal;

#[derive(Message, Default)]
pub struct LevelSolved;

#[derive(Message, Default)]
pub struct UpdateGridPositionEvent;
