use soukoban::{direction::Direction, Action, Actions, Level, Tiles};

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
        let map = self.level.map();
        let player_next_position = map.player_position() + &direction.into();
        if map[player_next_position].intersects(Tiles::Wall) {
            return false;
        }
        if map[player_next_position].intersects(Tiles::Box) {
            let box_next_position = player_next_position + &direction.into();
            if map[box_next_position].intersects(Tiles::Box | Tiles::Wall) {
                return false;
            }
        }
        true
    }

    /// Moves the player or pushes a box in the specified direction.
    pub fn move_or_push(&mut self, direction: Direction) {
        let map = self.level.map_mut();
        let direction_vector = &direction.into();
        let player_next_position = map.player_position() + direction_vector;
        if map[player_next_position].intersects(Tiles::Wall) {
            return;
        }
        if map[player_next_position].intersects(Tiles::Box) {
            let box_next_position = player_next_position + direction_vector;
            if map[box_next_position].intersects(Tiles::Wall | Tiles::Box) {
                return;
            }
            map.set_box_position(player_next_position, box_next_position);

            self.actions.push(Action::Push(direction));
        } else {
            self.actions.push(Action::Move(direction));
        }
        map.set_player_position(player_next_position);
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
        let map = self.level.map_mut();
        let history = self.actions.pop().unwrap();
        let direction = history.direction();
        if history.is_push() {
            let box_position = map.player_position() + &direction.into();
            let player_position = map.player_position();
            map.set_box_position(box_position, player_position);
        }
        let player_prev_position = map.player_position() - &direction.into();
        map.set_player_position(player_prev_position);
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
        let undone_actions = self.undone_actions.clone();
        self.move_or_push(history.direction());
        self.undone_actions = undone_actions;
    }

    /// Checks if the level is solved.
    pub fn is_solved(&self) -> bool {
        self.level.map().box_positions() == self.level.map().goal_positions()
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
}
