use soukoban::{direction::Direction, Action, Actions, Level, Tiles};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Board {
    pub level: Level,
    actions: Actions,
    undone_actions: Actions,
    input_queue: VecDeque<Direction>,
}

impl Board {
    /// Creates a new board with the specified level.
    pub fn with_level(level: Level) -> Self {
        Self {
            level,
            actions: Actions::new(),
            undone_actions: Actions::new(),
            input_queue: VecDeque::new(),
        }
    }

    /// Moves the player or pushes a box in the specified direction.
    pub fn move_or_push(&mut self, direction: Direction) {
        self.input_queue.push_back(direction);
        while let Some(direction
