use crate::level::Level;
use crate::Tile;

use nalgebra::Vector2;
use soukoban::{direction::Direction, Action, Actions};

#[derive(Clone)]
pub struct Board {
    pub level: Level,
    actions: Actions,
    undone_actions: Actions,
}

impl Board {
    /// Creates a new board with the specified level.
    pub fn with_level(level: Level) -> Self {
        Self {
            level,
            actions: Actions::new(),
            undone_actions: Actions::new(),
        }
    }

    /// Checks if the player can move or push in the specified direction.
    pub fn moveable(&self, direction: Direction) -> bool {
        let player_next_position = self.level.player_position + &direction.into();
        if self.level.get(&player_next_position).intersects(Tile::Wall) {
            return false;
        }
        if self
            .level
            .get(&player_next_position)
            .intersects(Tile::Crate)
        {
            let crate_next_position = player_next_position + &direction.into();
            if self
                .level
                .get(&crate_next_position)
                .intersects(Tile::Crate | Tile::Wall)
            {
                return false;
            }
        }
        true
    }

    /// Moves the player or pushes a crate in the specified direction.
    pub fn move_or_push(&mut self, direction: Direction) {
        let direction_vector = &direction.into();
        let player_next_position = self.level.player_position + direction_vector;
        if self.level.get(&player_next_position).intersects(Tile::Wall) {
            return;
        }
        if self
            .level
            .get(&player_next_position)
            .intersects(Tile::Crate)
        {
            let crate_next_position = player_next_position + direction_vector;
            if self
                .level
                .get(&crate_next_position)
                .intersects(Tile::Wall | Tile::Crate)
            {
                return;
            }
            self.move_crate(player_next_position, crate_next_position);

            self.actions.push(Action::Push(direction));
        } else {
            self.actions.push(Action::Move(direction));
        }
        self.move_player(player_next_position);
        self.undone_actions.clear();
    }

    /// Undoes the last push.
    pub fn undo_push(&mut self) {
        while let Some(history) = self.actions.last() {
            if history.is_push() {
                self.undo_move();
                return;
            }
            self.undo_move();
        }
    }

    /// Undoes the last move.
    pub fn undo_move(&mut self) {
        debug_assert!(!self.actions.is_empty());
        let history = self.actions.pop().unwrap();
        let direction = history.direction();
        if history.is_push() {
            let crate_position = self.level.player_position + &direction.into();
            self.move_crate(crate_position, self.level.player_position);
        }
        let player_prev_position = self.level.player_position - &direction.into();
        self.move_player(player_prev_position);
        self.undone_actions.push(history);
    }

    /// Redoes the last push.
    pub fn redo_push(&mut self) {
        while let Some(history) = self.undone_actions.last() {
            if history.is_push() {
                self.redo_move();
                return;
            }
            self.redo_move();
        }
    }

    /// Redoes the last move.
    pub fn redo_move(&mut self) {
        debug_assert!(!self.undone_actions.is_empty());
        let history = self.undone_actions.pop().unwrap();
        let undone_movements = self.undone_actions.clone();
        self.move_or_push(history.direction());
        self.undone_actions = undone_movements;
    }

    /// Checks if the level is solved.
    pub fn is_solved(&self) -> bool {
        self.level.crate_positions == self.level.target_positions
    }

    pub fn actions(&self) -> &Actions {
        &self.actions
    }

    /// Returns the player's current orientation.
    pub fn player_orientation(&self) -> Direction {
        self.actions
            .last()
            .map(|action| action.direction())
            .unwrap_or(Direction::Down)
    }

    fn move_player(&mut self, to: Vector2<i32>) {
        self.level
            .get_mut(&self.level.player_position.clone())
            .remove(Tile::Player);
        self.level.get_mut(&to).insert(Tile::Player);
        self.level.player_position = to;
    }

    fn move_crate(&mut self, from: Vector2<i32>, to: Vector2<i32>) {
        self.level.get_mut(&from).remove(Tile::Crate);
        self.level.get_mut(&to).insert(Tile::Crate);
        self.level.crate_positions.remove(&from);
        self.level.crate_positions.insert(to);
    }
}
