use crate::direction::Direction;
use crate::level::Level;
use crate::movement::{Movement, Movements};
use crate::Tile;

use nalgebra::Vector2;

#[derive(Clone)]
pub struct Board {
    pub level: Level,
    pub movements: Movements,
    undone_movements: Movements,
}

impl Board {
    pub fn with_level(level: Level) -> Self {
        Self {
            level,
            movements: Movements::new(),
            undone_movements: Movements::new(),
        }
    }

    pub fn move_or_push(&mut self, direction: Direction) {
        let direction_vector = direction.to_vector();
        let player_next_position = self.level.player_position + direction_vector;
        if self
            .level
            .get_unchecked(&player_next_position)
            .intersects(Tile::Wall)
        {
            return;
        }
        if self
            .level
            .get_unchecked(&player_next_position)
            .intersects(Tile::Crate)
        {
            let crate_next_position = player_next_position + direction_vector;
            if self
                .level
                .get_unchecked(&crate_next_position)
                .intersects(Tile::Wall | Tile::Crate)
            {
                return;
            }
            self.move_crate(player_next_position, crate_next_position);

            self.movements.push(Movement::with_push(direction));
        } else {
            self.movements.push(Movement::with_move(direction));
        }
        self.move_player(player_next_position);
        self.undone_movements.clear();
    }

    pub fn reset(&mut self) {
        while !self.movements.is_empty() {
            self.undo_move();
        }
    }

    pub fn undo_push(&mut self) {
        while let Some(history) = self.movements.last() {
            if history.is_push {
                self.undo_move();
                return;
            }
            self.undo_move();
        }
    }

    pub fn undo_move(&mut self) {
        debug_assert!(!self.movements.is_empty());
        let history = self.movements.pop().unwrap();
        let direction = history.direction;
        if history.is_push {
            let crate_position = self.level.player_position + direction.to_vector();
            self.move_crate(crate_position, self.level.player_position.clone());
        }
        let player_prev_position = self.level.player_position - direction.to_vector();
        self.move_player(player_prev_position);
        self.undone_movements.push(history);
    }

    pub fn redo_push(&mut self) {
        while let Some(history) = self.undone_movements.last() {
            if history.is_push {
                self.redo_move();
                return;
            }
            self.redo_move();
        }
    }

    pub fn redo_move(&mut self) {
        debug_assert!(!self.undone_movements.is_empty());
        let history = self.undone_movements.pop().unwrap();
        let undone_movements = self.undone_movements.clone();
        self.move_or_push(history.direction);
        self.undone_movements = undone_movements;
    }

    pub fn is_solved(&self) -> bool {
        return self.level.crate_positions == self.level.target_positions;
    }

    #[allow(dead_code)]
    pub fn player_facing_direction(&self) -> Direction {
        self.movements
            .last()
            .map(|movement| movement.direction)
            .unwrap_or(Direction::Down)
    }

    fn move_player(&mut self, to: Vector2<i32>) {
        self.level
            .get_unchecked_mut(&self.level.player_position.clone())
            .remove(Tile::Player);
        self.level.get_unchecked_mut(&to).insert(Tile::Player);
        self.level.player_position = to;
    }

    fn move_crate(&mut self, from: Vector2<i32>, to: Vector2<i32>) {
        self.level.get_unchecked_mut(&from).remove(Tile::Crate);
        self.level.get_unchecked_mut(&to).insert(Tile::Crate);
        self.level.crate_positions.remove(&from);
        self.level.crate_positions.insert(to);
    }
}
