use crate::direction::Direction;
use crate::level::Level;
use crate::movement::Movement;
use crate::Tile;

use nalgebra::Vector2;

#[derive(Clone)]
pub struct Board {
    pub level: Level,
    pub movements: Vec<Movement>,
}

impl Board {
    pub fn move_or_push(&mut self, direction: Direction) -> bool {
        let direction_vector = direction.to_vector();
        let player_next_position = self.level.player_position + direction_vector;
        if self
            .level
            .at_unchecked(&player_next_position)
            .intersects(Tile::Wall)
        {
            return false;
        }
        if self
            .level
            .at_unchecked(&player_next_position)
            .intersects(Tile::Crate)
        {
            let crate_next_position = player_next_position + direction_vector;
            if self
                .level
                .at_unchecked(&crate_next_position)
                .intersects(Tile::Wall | Tile::Crate)
            {
                return false;
            }
            self.move_crate(player_next_position, crate_next_position);

            self.movements.push(Movement::with_push(direction));
        } else {
            self.movements.push(Movement::with_move(direction));
        }
        self.move_player(player_next_position);
        return true;
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

    fn undo_move(&mut self) {
        debug_assert!(!self.movements.is_empty());
        let history = self.movements.pop().unwrap();
        let direction = history.direction;
        if history.is_push {
            let crate_position = self.level.player_position + direction.to_vector();
            self.move_crate(crate_position, self.level.player_position.clone());
        }
        let player_prev_position = self.level.player_position - direction.to_vector();
        self.move_player(player_prev_position);
    }

    pub fn is_solved(&self) -> bool {
        return self.level.crate_positions == self.level.target_positions;
    }

    pub fn move_count(&self) -> usize {
        self.movements.len()
    }

    pub fn push_count(&self) -> usize {
        self.movements.iter().filter(|x| x.is_push).count()
    }

    pub fn export_movements(&self) -> String {
        self.movements
            .iter()
            .map(|x| Into::<char>::into(x.clone()))
            .collect()
    }

    fn move_player(&mut self, to: Vector2<i32>) {
        self.level
            .at_unchecked_mut(&self.level.player_position.clone())
            .remove(Tile::Player);
        self.level.at_unchecked_mut(&to).insert(Tile::Player);
        self.level.player_position = to;
    }

    fn move_crate(&mut self, from: Vector2<i32>, to: Vector2<i32>) {
        self.level.at_unchecked_mut(&from).remove(Tile::Crate);
        self.level.at_unchecked_mut(&to).insert(Tile::Crate);
        self.level.crate_positions.remove(&from);
        self.level.crate_positions.insert(to);
    }
}
