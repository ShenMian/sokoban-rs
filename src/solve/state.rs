use std::{
    cell::OnceCell,
    cmp::Ordering,
    collections::HashSet,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::solve::solver::*;

use nalgebra::Vector2;
use soukoban::{
    Action, Actions, Tiles, deadlock,
    direction::Direction,
    path_finding::{compute_area_anchor, compute_reachable_area, find_path},
};

#[derive(Clone, Eq)]
pub struct State {
    pub player_position: Vector2<i32>,
    pub box_positions: HashSet<Vector2<i32>>,
    pub actions: Actions,
    heuristic: usize,
    lower_bound: OnceCell<usize>,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.player_position == other.player_position && self.box_positions == other.box_positions
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.player_position.hash(state);
        for position in &self.box_positions {
            position.hash(state);
        }
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        self.heuristic.cmp(&other.heuristic).reverse()
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
        box_positions: HashSet<Vector2<i32>>,
        actions: Actions,
        solver: &Solver,
    ) -> Self {
        let mut instance = Self {
            player_position,
            box_positions,
            actions,
            heuristic: 0,
            lower_bound: OnceCell::new(),
        };
        debug_assert!(instance.actions.moves() < 10_000);
        debug_assert!(instance.actions.pushes() < 10_000);
        debug_assert!(instance.lower_bound(solver) < 10_000);
        instance.heuristic = match solver.strategy() {
            Strategy::Fast => instance.lower_bound(solver) * 10_000 + instance.actions.moves(),
            Strategy::Mixed => instance.lower_bound(solver) + instance.actions.moves(),
            Strategy::OptimalMovePush => {
                instance.actions.moves() * 100_000_000
                    + instance.actions.pushes() * 10_000
                    + instance.lower_bound(solver)
            }
            Strategy::OptimalPushMove => {
                instance.actions.pushes() * 100_000_000
                    + instance.actions.moves() * 10_000
                    + instance.lower_bound(solver)
            }
        };
        instance.box_positions.shrink_to_fit();
        instance.actions.shrink_to_fit();
        instance
    }

    /// Returns a vector of successor states for the current state.
    pub fn successors(&self, solver: &Solver) -> Vec<State> {
        let mut successors = Vec::new();
        let player_reachable_area = self.player_reachable_area(solver);
        for box_position in &self.box_positions {
            for push_direction in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ] {
                let mut new_box_position = box_position + &push_direction.into();
                if self.can_block_box(new_box_position, solver) {
                    continue;
                }

                let next_player_position = box_position - &push_direction.into();
                if self.can_block_player(next_player_position, solver)
                    || !player_reachable_area.contains(&next_player_position)
                {
                    continue;
                }

                let mut new_actions = self.actions.clone();
                let path = find_path(self.player_position, next_player_position, |position| {
                    !self.can_block_player(position, solver)
                })
                .unwrap();
                new_actions.extend(
                    path.windows(2)
                        .map(|pos| Direction::try_from(pos[1] - pos[0]).unwrap())
                        .map(Action::Move),
                );
                new_actions.push(Action::Push(push_direction));

                // skip tunnels
                while solver
                    .tunnels()
                    .contains(&((new_box_position - &push_direction.into()), push_direction))
                {
                    if self.can_block_box(new_box_position + &push_direction.into(), solver) {
                        break;
                    }
                    new_box_position += &push_direction.into();
                    new_actions.push(Action::Push(push_direction));
                }

                let mut new_box_positions = self.box_positions.clone();
                new_box_positions.remove(box_position);
                new_box_positions.insert(new_box_position);

                // skip deadlocks
                if !solver.map[new_box_position].intersects(Tiles::Goal)
                    && deadlock::is_freeze_deadlock(
                        &solver.map,
                        new_box_position,
                        &new_box_positions,
                        &mut HashSet::new(),
                    )
                {
                    continue;
                }

                let new_player_position = new_box_position - &push_direction.into();

                let new_state =
                    State::new(new_player_position, new_box_positions, new_actions, solver);
                successors.push(new_state);
            }
        }
        successors
    }

    /// Checks if the current state represents a solved level.
    pub fn is_solved(&self, solver: &Solver) -> bool {
        self.lower_bound(solver) == 0
    }

    /// Returns the heuristic value of the current state.
    pub fn heuristic(&self) -> usize {
        self.heuristic
    }

    /// Returns a normalized clone of the current state.
    pub fn normalized(&self, solver: &Solver) -> Self {
        let mut instance = self.clone();
        instance.player_position = self.normalized_player_position(solver);
        instance
    }

    /// Returns a normalized hash of the current state.
    pub fn normalized_hash(&self, solver: &Solver) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.normalized(solver).hash(&mut hasher);
        hasher.finish()
    }

    /// Returns the lower bound value for the current state.
    fn lower_bound(&self, solver: &Solver) -> usize {
        *self
            .lower_bound
            .get_or_init(|| self.calculate_lower_bound(solver))
    }

    /// Calculates and returns the lower bound value for the current state.
    fn calculate_lower_bound(&self, solver: &Solver) -> usize {
        let mut sum: usize = 0;
        for box_position in &self.box_positions {
            match solver.lower_bounds().get(box_position) {
                Some(lower_bound) => sum += lower_bound,
                None => return 10_000 - 1,
            }
        }
        sum
    }

    /// Checks if a position can block the player's movement.
    fn can_block_player(&self, position: Vector2<i32>, solver: &Solver) -> bool {
        solver.map[position].intersects(Tiles::Wall) || self.box_positions.contains(&position)
    }

    /// Checks if a position can block a box's movement.
    fn can_block_box(&self, position: Vector2<i32>, solver: &Solver) -> bool {
        solver.map[position].intersects(Tiles::Wall /* | Tiles::Deadlock */)
            || !solver.lower_bounds().contains_key(&position)
            || self.box_positions.contains(&position)
    }

    /// Returns the normalized player position based on reachable area.
    fn normalized_player_position(&self, solver: &Solver) -> Vector2<i32> {
        compute_area_anchor(&self.player_reachable_area(solver)).unwrap()
    }

    /// Returns the reachable area for the player in the current state.
    fn player_reachable_area(&self, solver: &Solver) -> HashSet<Vector2<i32>> {
        compute_reachable_area(self.player_position, |position| {
            !self.can_block_player(position, solver)
        })
    }
}
