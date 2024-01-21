use nalgebra::Vector2;

use crate::direction::Direction;
use crate::level::{normalized_area, Tile};
use crate::movement::{Movement, Movements};
use crate::solver::solver::*;

use std::cell::OnceCell;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Clone, Eq)]
pub struct State {
    pub player_position: Vector2<i32>,
    pub crate_positions: HashSet<Vector2<i32>>,
    pub movements: Movements,
    heuristic: usize,
    lower_bound: OnceCell<usize>,
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
        movements: Movements,
        solver: &Solver,
    ) -> Self {
        let mut instance = Self {
            player_position,
            crate_positions,
            movements,
            heuristic: 0,
            lower_bound: OnceCell::new(),
        };
        debug_assert!(instance.move_count() < 10000);
        debug_assert!(instance.push_count() < 10000);
        debug_assert!(instance.lower_bound(solver) < 10000);
        instance.heuristic = match solver.strategy() {
            Strategy::Fast => instance.lower_bound(solver) * 10000 + instance.move_count(),
            Strategy::Mixed => instance.lower_bound(solver) + instance.move_count(),
            Strategy::OptimalMovePush => {
                instance.move_count() * 10000_0000
                    + instance.push_count() * 10000
                    + instance.lower_bound(solver)
            }
            Strategy::OptimalPushMove => {
                instance.push_count() * 10000_0000
                    + instance.move_count() * 10000
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

                let mut new_movements = self.movements.clone();
                let path = find_path(&self.player_position, &next_player_position, |position| {
                    self.can_block_player(position, solver)
                })
                .unwrap();
                new_movements.extend(
                    path.windows(2)
                        .map(|p| Direction::from_vector(p[1] - p[0]).unwrap())
                        .map(Movement::with_move),
                );
                new_movements.push(Movement::with_push(push_direction));

                // skip tunnels
                while solver.tunnels().contains(&(
                    (new_crate_position - push_direction.to_vector()),
                    push_direction,
                )) {
                    if self
                        .can_block_crate(&(new_crate_position + push_direction.to_vector()), solver)
                    {
                        break;
                    }
                    new_crate_position += push_direction.to_vector();
                    new_movements.push(Movement::with_push(push_direction));
                }

                let mut new_crate_positions = self.crate_positions.clone();
                new_crate_positions.remove(crate_position);
                new_crate_positions.insert(new_crate_position);

                // skip deadlocks
                if !solver.level.target_positions.contains(&new_crate_position)
                    && self.is_freeze_deadlock(
                        &new_crate_position,
                        &new_crate_positions,
                        solver,
                        &mut HashSet::new(),
                    )
                {
                    continue;
                }

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

    pub fn heuristic(&self) -> usize {
        self.heuristic
    }

    pub fn move_count(&self) -> usize {
        self.movements.len()
    }

    pub fn push_count(&self) -> usize {
        self.movements.iter().filter(|x| x.is_push).count()
    }

    pub fn normalized(&self, solver: &Solver) -> Self {
        let mut instance = self.clone();
        instance.player_position = self.normalized_player_position(solver);
        instance
    }

    fn is_freeze_deadlock(
        &self,
        crate_position: &Vector2<i32>,
        crate_positions: &HashSet<Vector2<i32>>,
        solver: &Solver,
        visited: &mut HashSet<Vector2<i32>>,
    ) -> bool {
        if !visited.insert(*crate_position) {
            return true;
        }

        for direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
        .chunks(2)
        {
            let neighbors = [
                crate_position + direction[0].to_vector(),
                crate_position + direction[1].to_vector(),
            ];
            if solver
                .level
                .get_unchecked(&neighbors[0])
                .intersects(Tile::Wall)
                || solver
                    .level
                    .get_unchecked(&neighbors[1])
                    .intersects(Tile::Wall)
            {
                continue;
            }
            if (crate_positions.contains(&neighbors[0])
                && self.is_freeze_deadlock(&neighbors[0], crate_positions, solver, visited))
                || (crate_positions.contains(&neighbors[1])
                    && self.is_freeze_deadlock(&neighbors[1], crate_positions, solver, visited))
            {
                continue;
            }
            return false;
        }
        return true;
    }

    /// Minimum number of pushes required to complete the level.
    fn lower_bound(&self, solver: &Solver) -> usize {
        *self
            .lower_bound
            .get_or_init(|| self.calculate_lower_bound(solver))
    }

    fn calculate_lower_bound(&self, solver: &Solver) -> usize {
        // FIXME: Panic on Microban #155
        self.crate_positions
            .iter()
            .map(|crate_position| solver.lower_bounds()[&crate_position])
            .sum()
    }

    fn can_block_player(&self, position: &Vector2<i32>, solver: &Solver) -> bool {
        solver.level.get_unchecked(position).intersects(Tile::Wall)
            || self.crate_positions.contains(position)
    }

    fn can_block_crate(&self, position: &Vector2<i32>, solver: &Solver) -> bool {
        solver
            .level
            .get_unchecked(position)
            .intersects(Tile::Wall | Tile::Deadlock)
            || !solver.lower_bounds().contains_key(position)
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
