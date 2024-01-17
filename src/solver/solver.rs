use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::direction::Direction;
use crate::level::{Level, Tile};
use crate::movement::Movements;
use crate::solver::state::*;

use std::cell::OnceCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;
use std::time;

use std::io::Write;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Strategy {
    /// Find any solution
    Fast,

    /// Find move optimal solutions with best pushes
    // FIXME: 结果非最优解, 可能是由于遇到答案就直接返回忽略剩余状态导致的
    OptimalMovePush,

    /// Find push optimal solutions with best moves
    OptimalPushMove,

    Mixed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum LowerBoundMethod {
    PushCount,
    MoveCount,
    ManhattanDistance,
}

pub struct Solver {
    pub level: Level,
    strategy: Strategy,
    lower_bound_method: LowerBoundMethod,
    lower_bounds: OnceCell<HashMap<Vector2<i32>, usize>>,
    visited: HashSet<State>,
    heap: BinaryHeap<State>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SolveError {
    Timeout,
    NoSolution,
}

type Result<T> = std::result::Result<T, SolveError>;

impl Solver {
    pub fn new(mut level: Level) -> Self {
        level.clear(Tile::Player | Tile::Crate);
        let mut instance = Self {
            level,
            strategy: Strategy::Fast,
            lower_bound_method: LowerBoundMethod::PushCount,
            lower_bounds: OnceCell::new(),
            visited: HashSet::new(),
            heap: BinaryHeap::new(),
        };
        instance.calculate_tunnel_positions();
        instance
    }

    pub fn initial(&mut self, strategy: Strategy, lower_bound_method: LowerBoundMethod) {
        self.strategy = strategy;
        self.lower_bound_method = lower_bound_method;
        self.heap.push(State::new(
            self.level.player_position,
            self.level.crate_positions.clone(),
            Movements::new(),
            self,
        ));
    }

    pub fn solve(&mut self, timeout: time::Duration) -> Result<Movements> {
        debug_assert!(!self.heap.is_empty());
        let timer = std::time::Instant::now();
        while let Some(state) = self.heap.pop() {
            self.visited.insert(state.normalized(&self));

            if timer.elapsed() >= timeout {
                return Err(SolveError::Timeout);
            }

            // Solver::shrink_heap(&mut self.heap);
            // Solver::print_info(&self.visited, &self.heap, &state);

            for successor in state.successors(&self) {
                if self.visited.contains(&successor.normalized(&self)) {
                    continue;
                }
                if successor.is_solved(&self) {
                    return Ok(successor.movements);
                }
                self.heap.push(successor);
            }
        }

        Err(SolveError::NoSolution)
    }

    pub fn strategy(&self) -> Strategy {
        self.strategy
    }

    pub fn lower_bounds(&self) -> &HashMap<Vector2<i32>, usize> {
        self.lower_bounds
            .get_or_init(|| self.calculate_lower_bounds())
    }

    pub fn best_state(&self) -> Option<&State> {
        self.heap.peek()
    }

    fn minimum_push_count_to_nearest_target(&self, position: &Vector2<i32>) -> Option<usize> {
        if self.level.target_positions.contains(position) {
            return Some(0);
        }

        let paths = self
            .level
            .crate_pushable_paths_with_crate_positions(position, &HashSet::new());

        paths
            .iter()
            .filter(|path| self.level.target_positions.contains(&path.0.crate_position))
            .map(|path| path.1.len() - 1)
            .min()
    }

    fn minimum_move_count_to_nearest_target(&self, position: &Vector2<i32>) -> Option<usize> {
        let nearest_target_position = self
            .level
            .target_positions
            .iter()
            .min_by_key(|crate_pos| manhattan_distance(crate_pos, &position))
            .unwrap();
        let movements = find_path(&position, &nearest_target_position, |position| {
            self.level.get_unchecked(&position).intersects(Tile::Wall)
        })
        .unwrap();
        Some(movements.len() - 1)
    }

    fn manhattan_distance_to_nearest_target(&self, position: &Vector2<i32>) -> Option<usize> {
        Some(
            self.level
                .target_positions
                .iter()
                .map(|crate_pos| manhattan_distance(crate_pos, &position))
                .min()
                .unwrap() as usize,
        )
    }

    fn calculate_lower_bounds(&self) -> HashMap<Vector2<i32>, usize> {
        let mut lower_bounds = HashMap::new();
        for x in 1..self.level.dimensions.x - 1 {
            for y in 1..self.level.dimensions.y - 1 {
                let position = Vector2::new(x, y);
                if !self.level.get_unchecked(&position).intersects(Tile::Floor)
                    || self
                        .level
                        .get_unchecked(&position)
                        .intersects(Tile::Deadlock)
                {
                    continue;
                }
                let lower_bound = match self.lower_bound_method {
                    LowerBoundMethod::PushCount => {
                        self.minimum_push_count_to_nearest_target(&position)
                    }
                    LowerBoundMethod::MoveCount => {
                        self.minimum_move_count_to_nearest_target(&position)
                    }
                    LowerBoundMethod::ManhattanDistance => {
                        self.manhattan_distance_to_nearest_target(&position)
                    }
                };
                if let Some(lower_bound) = lower_bound {
                    lower_bounds.insert(position, lower_bound);
                }
            }
        }
        lower_bounds
    }

    fn calculate_tunnel_positions(&mut self) {
        for x in 1..self.level.dimensions.x - 1 {
            for y in 1..self.level.dimensions.y - 1 {
                let position = Vector2::new(x, y);
                if !self.level.get_unchecked(&position).intersects(Tile::Floor) {
                    continue;
                }
                for directions in [
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ]
                .chunks(2)
                {
                    let neighbor = [
                        position + directions[0].to_vector(),
                        position + directions[1].to_vector(),
                    ];
                    if !(self
                        .level
                        .get_unchecked(&neighbor[0])
                        .intersects(Tile::Wall)
                        && self
                            .level
                            .get_unchecked(&neighbor[1])
                            .intersects(Tile::Wall))
                    {
                        continue;
                    }

                    self.level.get_unchecked_mut(&position).insert(Tile::Tunnel);
                }
            }
        }
    }

    #[allow(dead_code)]
    fn shrink_heap(heap: &mut BinaryHeap<State>) {
        let max_pressure = 200_000;
        if heap.len() > max_pressure {
            let mut heuristics: Vec<_> = heap.iter().map(|state| state.heuristic()).collect();
            heuristics.sort_unstable();
            let mut costs: Vec<_> = heap.iter().map(|state| state.move_count()).collect();
            costs.sort_unstable();

            let alpha = 0.8;
            let heuristic_median = heuristics[(heuristics.len() as f32 * alpha) as usize];
            let cost_median = costs[(costs.len() as f32 * alpha) as usize];
            heap.retain(|state| {
                state.heuristic() <= heuristic_median && state.move_count() <= cost_median
            });
        }
    }

    #[allow(dead_code)]
    fn print_info(visited: &HashSet<State>, heap: &BinaryHeap<State>, state: &State) {
        print!(
            "Visited: {:<6}, Heuristic: {:<4}, Moves: {:<4}, Pushes: {:<4}, Pressure: {:<4}\r",
            visited.len(),
            state.heuristic(),
            state.move_count(),
            state.push_count(),
            heap.len()
        );
        std::io::stdout().flush().unwrap();
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

fn manhattan_distance(a: &Vector2<i32>, b: &Vector2<i32>) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}
