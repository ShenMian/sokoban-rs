use std::{
    cell::OnceCell,
    collections::{BinaryHeap, HashMap, HashSet},
    time::{Duration, Instant},
};

use crate::{box_pushable_paths_with_positions, solve::state::*};

use itertools::Itertools;
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};
use soukoban::{direction::Direction, path_finding::reachable_area, Actions, Level, Tiles};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum Strategy {
    /// Speed priority
    #[default]
    Fast,

    /// Balanced speed and steps
    Mixed,

    /// Find move optimal solutions with best pushes
    OptimalMovePush,

    /// Find push optimal solutions with best moves
    OptimalPushMove,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum LowerBoundMethod {
    /// Minimum push count to nearest target
    MinimumPush,

    /// Minimum move count to nearest target
    #[default]
    MinimumMove,

    /// Manhattan distance to nearest target
    ManhattanDistance,
}

pub struct Solver {
    pub level: Level,
    strategy: Strategy,
    lower_bound_method: LowerBoundMethod,
    lower_bounds: OnceCell<HashMap<Vector2<i32>, usize>>,
    tunnels: OnceCell<HashSet<(Vector2<i32>, Direction)>>,
    visited: HashSet<u64>,
    heap: BinaryHeap<State>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SolveError {
    Timeout,
    NoSolution,
}

type Result<T> = std::result::Result<T, SolveError>;

impl Solver {
    /// Creates a new solver.
    pub fn new(level: Level, strategy: Strategy, lower_bound_method: LowerBoundMethod) -> Self {
        let mut instance = Self {
            level,
            strategy,
            lower_bound_method,
            lower_bounds: OnceCell::new(),
            tunnels: OnceCell::new(),
            visited: HashSet::new(),
            heap: BinaryHeap::new(),
        };
        instance.heap.push(State::new(
            instance.level.map().player_position(),
            instance.level.map().box_positions().clone(),
            Actions::new(),
            &instance,
        ));
        instance
    }

    /// Searches for solution using the A* algorithm.
    pub fn search(&mut self, timeout: Duration) -> Result<Actions> {
        let timer = Instant::now();
        self.visited
            .insert(self.heap.peek().unwrap().normalized_hash(self));
        while let Some(state) = self.heap.pop() {
            if timer.elapsed() >= timeout {
                return Err(SolveError::Timeout);
            }
            if state.is_solved(self) {
                return Ok(state.movements);
            }

            for successor in state.successors(self) {
                if !self.visited.insert(successor.normalized_hash(self)) {
                    continue;
                }
                self.heap.push(successor);
            }

            // Solver::shrink_heap(&mut self.heap);
        }

        Err(SolveError::NoSolution)
    }

    pub fn strategy(&self) -> Strategy {
        self.strategy
    }

    /// Returns the best state in the binary heap, or `None` if it is empty.
    pub fn best_state(&self) -> Option<&State> {
        self.heap.peek()
    }

    /// Returns a reference to the set of tunnels.
    pub fn tunnels(&self) -> &HashSet<(Vector2<i32>, Direction)> {
        self.tunnels.get_or_init(|| self.calculate_tunnels())
    }

    /// Calculates and returns the set of tunnels in the level.
    fn calculate_tunnels(&self) -> HashSet<(Vector2<i32>, Direction)> {
        let map = self.level.map();
        let mut tunnels = HashSet::new();
        for x in 1..map.dimensions().x - 1 {
            for y in 1..map.dimensions().y - 1 {
                let box_position = Vector2::new(x, y);
                if !map[box_position].intersects(Tiles::Floor) {
                    continue;
                }

                for (up, right, down, left) in [
                    Direction::Up,
                    Direction::Right,
                    Direction::Down,
                    Direction::Left,
                    Direction::Up,
                    Direction::Right,
                    Direction::Down,
                ]
                .into_iter()
                .tuple_windows()
                {
                    let player_position = box_position + &down.into();

                    //  .
                    // #$#
                    // #@#
                    if map[player_position + &left.into()].intersects(Tiles::Wall)
                        && map[player_position + &right.into()].intersects(Tiles::Wall)
                        && map[box_position + &left.into()].intersects(Tiles::Wall)
                        && map[box_position + &right.into()].intersects(Tiles::Wall)
                        && map[box_position].intersects(Tiles::Floor)
                        && self
                            .lower_bounds()
                            .contains_key(&(box_position + &up.into()))
                        && !map[box_position].intersects(Tiles::Goal)
                    {
                        tunnels.insert((player_position, up));
                    }

                    //  .      .
                    // #$_ or _$#
                    // #@#    #@#
                    if map[player_position + &left.into()].intersects(Tiles::Wall)
                        && map[player_position + &right.into()].intersects(Tiles::Wall)
                        && (map[box_position + &right.into()].intersects(Tiles::Wall)
                            && map[box_position + &left.into()].intersects(Tiles::Floor)
                            || map[box_position + &right.into()].intersects(Tiles::Floor)
                                && map[box_position + &left.into()].intersects(Tiles::Wall))
                        && map[box_position].intersects(Tiles::Floor)
                        && self
                            .lower_bounds()
                            .contains_key(&(box_position + &up.into()))
                        && !map[box_position].intersects(Tiles::Goal)
                    {
                        tunnels.insert((player_position, up));
                    }
                }
            }
        }
        tunnels
    }

    /// Returns a reference to the set of lower bounds.
    pub fn lower_bounds(&self) -> &HashMap<Vector2<i32>, usize> {
        self.lower_bounds
            .get_or_init(|| self.calculate_lower_bounds())
    }

    /// Calculates and returns the set of lower bounds.
    fn calculate_lower_bounds(&self) -> HashMap<Vector2<i32>, usize> {
        match self.lower_bound_method {
            LowerBoundMethod::MinimumPush => self.minimum_push_lower_bounds(),
            LowerBoundMethod::MinimumMove => self.minimum_move_lower_bounds(),
            LowerBoundMethod::ManhattanDistance => self.manhattan_distance_lower_bounds(),
        }
    }

    /// Calculates and returns the lower bounds using the minimum push method.
    fn minimum_push_lower_bounds(&self) -> HashMap<Vector2<i32>, usize> {
        let mut lower_bounds = HashMap::new();
        let map = self.level.map();
        for target_position in map.goal_positions() {
            lower_bounds.insert(*target_position, 0);
            let mut player_position = None;
            for pull_direction in [
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ] {
                let next_box_position = target_position + &pull_direction.into();
                let next_player_position = next_box_position + &pull_direction.into();
                if map.in_bounds(next_player_position)
                    && !map[next_player_position].intersects(Tiles::Wall)
                    && !map[next_box_position].intersects(Tiles::Wall)
                {
                    player_position = Some(next_player_position);
                    break;
                }
            }
            if let Some(player_position) = player_position {
                self.minimum_push_to(
                    *target_position,
                    player_position,
                    &mut lower_bounds,
                    &mut HashSet::new(),
                );
            } else {
                continue;
            }
        }
        lower_bounds
    }

    fn minimum_push_to(
        &self,
        box_position: Vector2<i32>,
        player_position: Vector2<i32>,
        lower_bounds: &mut HashMap<Vector2<i32>, usize>,
        visited: &mut HashSet<(Vector2<i32>, Direction)>,
    ) {
        let map = self.level.map();
        let player_reachable_area = reachable_area(player_position, |position| {
            !map[position].intersects(Tiles::Wall) && position != box_position
        });
        for pull_direction in [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ] {
            let next_box_position = box_position + &pull_direction.into();
            if map[next_box_position].intersects(Tiles::Wall) {
                continue;
            }

            let next_player_position = next_box_position + &pull_direction.into();
            if !map.in_bounds(next_player_position)
                || map[next_player_position].intersects(Tiles::Wall)
            {
                continue;
            }
            if !player_reachable_area.contains(&next_player_position) {
                continue;
            }

            let lower_bound = *lower_bounds.get(&next_box_position).unwrap_or(&usize::MAX);
            let new_lower_bound = lower_bounds[&box_position] + 1;
            if !visited.insert((next_box_position, pull_direction)) {
                continue;
            }
            if new_lower_bound < lower_bound {
                lower_bounds.insert(next_box_position, new_lower_bound);
            }
            self.minimum_push_to(
                next_box_position,
                next_player_position,
                lower_bounds,
                visited,
            );
        }
    }

    /// Calculates and returns the lower bounds using the minimum move method.
    fn minimum_move_lower_bounds(&self) -> HashMap<Vector2<i32>, usize> {
        let mut lower_bounds = HashMap::new();
        let map = self.level.map();
        for x in 1..map.dimensions().x - 1 {
            for y in 1..map.dimensions().y - 1 {
                let position = Vector2::new(x, y);
                // There may be situations in the level where the box is
                // already on the target and cannot be reached by the player.
                if map[position].intersects(Tiles::Goal) {
                    lower_bounds.insert(position, 0);
                    continue;
                }
                if !map[position].intersects(Tiles::Floor)
                // || map[position].intersects(Tiles::Deadlock)
                {
                    continue;
                }

                let paths =
                    box_pushable_paths_with_positions(&self.level, &position, &HashSet::new());
                if let Some(lower_bound) = paths
                    .iter()
                    .filter(|path| map[path.0.box_position].intersects(Tiles::Goal))
                    .map(|path| path.1.len() - 1)
                    .min()
                {
                    lower_bounds.insert(position, lower_bound);
                }
            }
        }
        lower_bounds
    }

    /// Calculates and returns the lower bounds using the Manhattan distance method.
    fn manhattan_distance_lower_bounds(&self) -> HashMap<Vector2<i32>, usize> {
        let mut lower_bounds = HashMap::new();
        let map = self.level.map();
        for x in 1..map.dimensions().x - 1 {
            for y in 1..map.dimensions().y - 1 {
                let position = Vector2::new(x, y);
                // There may be situations in the level where the box is
                // already on the target and cannot be reached by the player.
                if map[position].intersects(Tiles::Goal) {
                    lower_bounds.insert(position, 0);
                    continue;
                }
                if !map[position].intersects(Tiles::Floor)
                // || map.get(&position).intersects(Tiles::Deadlock)
                {
                    continue;
                }
                let lower_bound = map
                    .goal_positions()
                    .iter()
                    .map(|box_pos| manhattan_distance(box_pos, &position))
                    .min()
                    .unwrap() as usize;
                lower_bounds.insert(position, lower_bound);
            }
        }
        lower_bounds
    }

    /// Shrinks the heap by retaining only a subset of states based on heuristics.
    #[expect(dead_code)]
    fn shrink_heap(heap: &mut BinaryHeap<State>) {
        let max_pressure = 200_000;
        if heap.len() > max_pressure {
            let mut heuristics: Vec<_> = heap.iter().map(|state| state.heuristic()).collect();
            heuristics.sort_unstable();

            let alpha = 0.8;
            let heuristic_median = heuristics[(heuristics.len() as f32 * alpha) as usize];
            heap.retain(|state| state.heuristic() <= heuristic_median);
        }
    }

    /// Prints the lower bounds for each position in the level.
    #[expect(dead_code)]
    pub fn print_lower_bounds(&self) {
        let map = self.level.map();
        for y in 0..map.dimensions().y {
            for x in 0..map.dimensions().x {
                let position = Vector2::new(x, y);
                if let Some(lower_bound) = self.lower_bounds().get(&position) {
                    print!("{:3} ", lower_bound);
                } else {
                    print!("{:3} ", "###");
                }
            }
            println!();
        }
    }
}

/// Calculates the Manhattan distance between two 2D vectors.
fn manhattan_distance(a: &Vector2<i32>, b: &Vector2<i32>) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}
