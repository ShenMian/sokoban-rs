use nalgebra::Vector2;

use crate::direction::Direction;
use crate::level::{normalized_area, Tile};
use crate::movement::Movement;
use crate::solver::solver::*;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Strategy {
    /// Find any solution
    Fast,

    /// Find move optimal solutions with best pushes
    // FIXME: 结果非最优解, 甚至可能比其他策略差. 可能是由于遇到答案就直接返回忽略剩余状态导致的
    OptimalMovePush,

    /// Find push optimal solutions with best moves
    OptimalPushMove,

    Mixed,
}

#[derive(Clone, Eq)]
pub struct State {
    pub player_position: Vector2<i32>,
    pub crate_positions: HashSet<Vector2<i32>>,

    pub heuristic: i32,
    pub movements: Vec<Movement>,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.player_position == other.player_position
            && self.crate_positions == other.crate_positions
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.player_position.hash(state);
        for position in &self.crate_positions {
            position.hash(state);
        }
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.heuristic.cmp(&self.heuristic)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl State {
    pub fn new(
        player_position: Vector2<i32>,
        crate_positions: HashSet<Vector2<i32>>,
        movements: Vec<Movement>,
        solver: &Solver,
    ) -> Self {
        let mut instance = Self {
            player_position,
            crate_positions,
            heuristic: 0,
            movements,
        };
        // TODO: 支持更大的移动和推动数
        assert!(instance.move_count() < 1000);
        assert!(instance.push_count() < 1000);
        instance.heuristic = match solver.strategy {
            Strategy::Fast => instance.lower_bound(solver) * 1000 + instance.move_count(),
            Strategy::Mixed => instance.lower_bound(solver) + instance.move_count(),
            Strategy::OptimalMovePush => {
                instance.move_count() * 1000_000
                    + instance.push_count() * 1000
                    + instance.lower_bound(solver)
            }
            Strategy::OptimalPushMove => {
                instance.push_count() * 1000_000
                    + instance.move_count() * 1000
                    + instance.lower_bound(solver)
            }
        };
        instance
    }

    /// Returns a vector of successor states for the current state.
    pub fn successors(&self, solver: &Solver) -> Vec<State> {
        let mut successors = Vec::new();
        let player_reachable_area = self.player_reachable_area(solver);
        for crate_position in &self.crate_positions {
            for push_direction in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ] {
                let mut new_crate_position = crate_position + push_direction.to_vector();

                let next_player_position = crate_position - push_direction.to_vector();
                if self.can_block_player(&next_player_position, solver)
                    || !player_reachable_area.contains(&next_player_position)
                {
                    continue;
                }

                if self.can_block_crate(&new_crate_position, solver) {
                    continue;
                }

                // slide in tunnel
                let mut slide_movements = Vec::new();
                if solver
                    .level
                    .at_unchecked(&crate_position)
                    .intersects(Tile::Tunnel)
                {
                    while solver
                        .level
                        .at_unchecked(&new_crate_position)
                        .intersects(Tile::Tunnel)
                        && solver
                            .level
                            .at_unchecked(&(new_crate_position + push_direction.to_vector()))
                            .intersects(Tile::Tunnel)
                        && !solver
                            .level
                            .at_unchecked(&new_crate_position)
                            .intersects(Tile::Target)
                    {
                        new_crate_position += push_direction.to_vector();
                        slide_movements.push(Movement::with_push(push_direction));
                    }
                }

                let mut new_movements = self.movements.clone();
                if let Some(path) =
                    find_path(&self.player_position, &next_player_position, |position| {
                        self.can_block_player(position, solver)
                    })
                {
                    new_movements.extend(
                        path.windows(2)
                            .map(|p| Direction::from_vector(p[1] - p[0]).unwrap())
                            .map(Movement::with_move),
                    )
                }
                new_movements.push(Movement::with_push(push_direction));
                new_movements.extend(slide_movements);

                let mut new_crate_positions = self.crate_positions.clone();
                new_crate_positions.remove(crate_position);
                new_crate_positions.insert(new_crate_position);

                let new_player_position = new_crate_position - push_direction.to_vector();

                let new_state = State::new(
                    new_player_position,
                    new_crate_positions,
                    new_movements,
                    solver,
                );
                successors.push(new_state);
            }
        }
        successors
    }

    pub fn is_solved(&self, solver: &Solver) -> bool {
        self.lower_bound(solver) == 0
    }

    pub fn move_count(&self) -> i32 {
        self.movements.len() as i32
    }

    pub fn push_count(&self) -> i32 {
        self.movements.iter().filter(|x| x.is_push).count() as i32
    }

    pub fn normalized(&self, solver: &Solver) -> Self {
        let mut instance = self.clone();
        instance.player_position = self.normalized_player_position(solver);
        instance
    }

    /// Minimum number of pushes required to complete the level.
    fn lower_bound(&self, solver: &Solver) -> i32 {
        self.crate_positions
            .iter()
            .map(|crate_pos| solver.lower_bounds[&crate_pos])
            .sum()
    }

    fn can_block_player(&self, position: &Vector2<i32>, solver: &Solver) -> bool {
        solver.level.at_unchecked(position).intersects(Tile::Wall)
            || self.crate_positions.contains(position)
    }

    fn can_block_crate(&self, position: &Vector2<i32>, solver: &Solver) -> bool {
        solver
            .level
            .at_unchecked(position)
            .intersects(Tile::Wall | Tile::Deadlock)
            || self.crate_positions.contains(position)
    }

    fn normalized_player_position(&self, solver: &Solver) -> Vector2<i32> {
        normalized_area(&self.player_reachable_area(solver))
    }

    fn player_reachable_area(&self, solver: &Solver) -> HashSet<Vector2<i32>> {
        solver
            .level
            .reachable_area(&self.player_position, |position| {
                self.can_block_player(&position, solver)
            })
    }
}
