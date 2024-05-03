use itertools::Itertools;
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::direction::Direction;
use crate::level::{Level, Tile};
use crate::movement::Movements;
use crate::solve::state::*;

use std::cell::OnceCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;
use std::time::{Duration, Instant};

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
    pub fn new(mut level: Level, strategy: Strategy, lower_bound_method: LowerBoundMethod) -> Self {
        level.clear(Tile::Player | Tile::Crate);
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
            instance.level.player_position,
            instance.level.crate_positions.clone(),
            Movements::new(),
            &instance,
        ));
        instance
    }

    /// Searches for solution using the A* algorithm.
    pub fn search(&mut self, timeout: Duration) -> Result<Movements> {
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
        let mut tunnels = HashSet::new();
        for x in 1..self.level.dimensions().x - 1 {
            for y in 1..self.level.dimensions().y - 1 {
                let player_position = Vector2::new(x, y);
                if !self
                    .level
                    .get_unchecked(&player_position)
                    .intersects(Tile::Floor)
                {
                    continue;
                }

                for (up, right, _down, left) in [
                    Direction::Up,
                    Direction::Right,
                    Direction::Down,
                    Direction::Left,
                    Direction::Up,
                    Direction::Right,
                    Direction::Down,
                ]
                .iter()
                .tuple_windows::<(_, _, _, _)>()
                {
                    // #$#
                    // #@#
                    if self
                        .level
                        .get_unchecked(&(player_position + left.to_vector()))
                        .intersects(Tile::Wall)
                        && self
                            .level
                            .get_unchecked(&(player_position + right.to_vector()))
                            .intersects(Tile::Wall)
                        && self
                            .level
                            .get_unchecked(&(player_position + up.to_vector() + left.to_vector()))
                            .intersects(Tile::Wall)
                        && self
                            .level
                            .get_unchecked(&(player_position + up.to_vector() + right.to_vector()))
                            .intersects(Tile::Wall)
                        && self
                            .level
                            .get_unchecked(&(player_position + up.to_vector()))
                            .intersects(Tile::Floor)
                        && !self
                            .level
                            .get_unchecked(&(player_position + up.to_vector()))
                            .intersects(Tile::Target)
                    {
                        tunnels.insert((player_position, *up));
                    }

                    // #$_ _$#
                    // #@# #@#
                    if self
                        .level
                        .get_unchecked(&(player_position + left.to_vector()))
                        .intersects(Tile::Wall)
                        && self
                            .level
                            .get_unchecked(&(player_position + right.to_vector()))
                            .intersects(Tile::Wall)
                        && (self
                            .level
                            .get_unchecked(&(player_position + up.to_vector() + right.to_vector()))
                            .intersects(Tile::Wall)
                            && self
                                .level
                                .get_unchecked(
                                    &(player_position + up.to_vector() + left.to_vector()),
                                )
                                .intersects(Tile::Floor)
                            || self
                                .level
                                .get_unchecked(
                                    &(player_position + up.to_vector() + right.to_vector()),
                                )
                                .intersects(Tile::Floor)
                                && self
                                    .level
                                    .get_unchecked(
                                        &(player_position + up.to_vector() + left.to_vector()),
                                    )
                                    .intersects(Tile::Wall))
                        && self
                            .level
                            .get_unchecked(&(player_position + up.to_vector()))
                            .intersects(Tile::Floor)
                        && !self
                            .level
                            .get_unchecked(&(player_position + up.to_vector()))
                            .intersects(Tile::Target)
                    {
                        tunnels.insert((player_position, *up));
                    }
                }
            }
        }
        tunnels.shrink_to_fit();
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
        for target_position in &self.level.target_positions {
            lower_bounds.insert(*target_position, 0);
            let mut player_position = None;
            for pull_direction in [
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ] {
                let next_crate_position = target_position + pull_direction.to_vector();
                let next_player_position = next_crate_position + pull_direction.to_vector();
                if self.level.in_bounds(&next_player_position)
                    && !self
                        .level
                        .get_unchecked(&next_player_position)
                        .intersects(Tile::Wall)
                    && !self
                        .level
                        .get_unchecked(&next_crate_position)
                        .intersects(Tile::Wall)
                {
                    player_position = Some(next_player_position);
                    break;
                }
            }
            if let Some(player_position) = player_position {
                self.minimum_push_to(
                    target_position,
                    &player_position,
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
        crate_position: &Vector2<i32>,
        player_position: &Vector2<i32>,
        lower_bounds: &mut HashMap<Vector2<i32>, usize>,
        visited: &mut HashSet<(Vector2<i32>, Direction)>,
    ) {
        let player_reachable_area = self.level.reachable_area(player_position, |position| {
            self.level.get_unchecked(position).intersects(Tile::Wall) || position == crate_position
        });
        for pull_direction in [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ] {
            let next_crate_position = crate_position + pull_direction.to_vector();
            if self
                .level
                .get_unchecked(&next_crate_position)
                .intersects(Tile::Wall)
            {
                continue;
            }

            let next_player_position = next_crate_position + pull_direction.to_vector();
            if !self.level.in_bounds(&next_player_position)
                || self
                    .level
                    .get_unchecked(&next_player_position)
                    .intersects(Tile::Wall)
            {
                continue;
            }
            if !player_reachable_area.contains(&next_player_position) {
                continue;
            }

            let lower_bound = *lower_bounds
                .get(&next_crate_position)
                .unwrap_or(&usize::MAX);
            let new_lower_bound = lower_bounds[crate_position] + 1;
            if !visited.insert((next_crate_position, pull_direction)) {
                continue;
            }
            if new_lower_bound < lower_bound {
                lower_bounds.insert(next_crate_position, new_lower_bound);
            }
            self.minimum_push_to(
                &next_crate_position,
                &next_player_position,
                lower_bounds,
                visited,
            );
        }
    }

    /// Calculates and returns the lower bounds using the minimum move method.
    fn minimum_move_lower_bounds(&self) -> HashMap<Vector2<i32>, usize> {
        let mut lower_bounds = HashMap::new();
        for x in 1..self.level.dimensions().x - 1 {
            for y in 1..self.level.dimensions().y - 1 {
                let position = Vector2::new(x, y);
                // There may be situations in the level where the box is
                // already on the target and cannot be reached by the player.
                if self.level.get_unchecked(&position).intersects(Tile::Target) {
                    lower_bounds.insert(position, 0);
                    continue;
                }
                if !self.level.get_unchecked(&position).intersects(Tile::Floor)
                    || self
                        .level
                        .get_unchecked(&position)
                        .intersects(Tile::Deadlock)
                {
                    continue;
                }

                let paths = self
                    .level
                    .crate_pushable_paths_with_crate_positions(&position, &HashSet::new());
                if let Some(lower_bound) = paths
                    .iter()
                    .filter(|path| {
                        self.level
                            .get_unchecked(&path.0.crate_position)
                            .intersects(Tile::Target)
                    })
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
        for x in 1..self.level.dimensions().x - 1 {
            for y in 1..self.level.dimensions().y - 1 {
                let position = Vector2::new(x, y);
                // There may be situations in the level where the box is
                // already on the target and cannot be reached by the player.
                if self.level.get_unchecked(&position).intersects(Tile::Target) {
                    lower_bounds.insert(position, 0);
                    continue;
                }
                if !self.level.get_unchecked(&position).intersects(Tile::Floor)
                    || self
                        .level
                        .get_unchecked(&position)
                        .intersects(Tile::Deadlock)
                {
                    continue;
                }
                let lower_bound = self
                    .level
                    .target_positions
                    .iter()
                    .map(|crate_pos| manhattan_distance(crate_pos, &position))
                    .min()
                    .unwrap() as usize;
                lower_bounds.insert(position, lower_bound);
            }
        }
        lower_bounds
    }

    /// Shrinks the heap by retaining only a subset of states based on heuristics.
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn print_lower_bounds(&self) {
        for y in 0..self.level.dimensions().y {
            for x in 0..self.level.dimensions().x {
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

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Node {
    position: Vector2<i32>,
    heuristic: i32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.heuristic.cmp(&self.heuristic)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Finds a path from `from` point to `to` point using the A* algorithm.
pub fn find_path(
    from: &Vector2<i32>,
    to: &Vector2<i32>,
    is_block: impl Fn(&Vector2<i32>) -> bool,
) -> Option<Vec<Vector2<i32>>> {
    let mut open_set = BinaryHeap::new();
    let mut came_from = HashMap::new();
    let mut cost = HashMap::new();

    open_set.push(Node {
        position: *from,
        heuristic: manhattan_distance(from, to),
    });
    cost.insert(*from, 0);

    while let Some(node) = open_set.pop() {
        if node.position == *to {
            let mut path = Vec::new();
            let mut current = *to;
            while current != *from {
                path.push(current);
                current = came_from[&current];
            }
            path.push(*from);
            path.reverse();
            return Some(path);
        }

        for direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let new_position = node.position + direction.to_vector();
            if is_block(&new_position) {
                continue;
            }

            let new_cost = cost[&node.position] + 1;
            if !cost.contains_key(&new_position) || new_cost < cost[&new_position] {
                cost.insert(new_position, new_cost);
                let priority = new_cost + manhattan_distance(&new_position, to);
                open_set.push(Node {
                    position: new_position,
                    heuristic: priority,
                });
                came_from.insert(new_position, node.position);
            }
        }
    }

    None
}

/// Calculates the Manhattan distance between two 2D vectors.
fn manhattan_distance(a: &Vector2<i32>, b: &Vector2<i32>) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}
