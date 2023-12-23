use nalgebra::Vector2;

use crate::direction::Direction;
use crate::level::{Level, Tile};
use crate::movement::Movement;
use crate::solver::state::*;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;
use std::time;

use std::io::Write;

#[derive(Clone)]
pub struct Solver {
    pub level: Level,
    pub strategy: Strategy,
    pub lower_bounds: HashMap<Vector2<i32>, i32>,
    visited: HashSet<State>,
    heap: BinaryHeap<State>,
}

impl From<Level> for Solver {
    fn from(mut level: Level) -> Self {
        level.clear(Tile::Player | Tile::Crate);
        let mut instance = Self {
            level,
            strategy: Strategy::Fast,
            lower_bounds: HashMap::new(),
            visited: HashSet::new(),
            heap: BinaryHeap::new(),
        };
        instance.pre_calculate_dead_positions();
        instance.pre_calculate_tunnel_positions();
        instance.pre_calculate_lower_bounds();

        instance
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    Timeout,
    NoSolution,
}

type Result<T> = std::result::Result<T, Error>;

impl Solver {
    pub fn initial(&mut self, strategy: Strategy) {
        self.strategy = strategy;
        self.heap.push(State::new(
            self.level.player_position,
            self.level.crate_positions.clone(),
            Vec::new(),
            self,
        ));
    }

    /// Solves the current level using the provided strategy and maximum pressure.
    ///
    /// This function uses a heuristic approach to explore the state space of the level.
    /// It keeps track of visited states to avoid re-exploration and uses a heuristic to guide the search.
    ///
    /// # Arguments
    ///
    /// - `strategy` - A Strategy object that determines the method of
    ///                exploration.
    /// - `timeout`  - Timeout duration.
    pub fn solve(&mut self, timeout: time::Duration) -> Result<Vec<Movement>> {
        let timer = std::time::Instant::now();
        while let Some(state) = self.heap.pop() {
            self.visited.insert(state.normalized(&self));

            if timer.elapsed() >= timeout {
                return Err(Error::Timeout);
            }

            // Solver::shrink_heap(&mut self.heap);
            Solver::print_info(&self.visited, &self.heap, &state);

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

        Err(Error::NoSolution)
    }

    fn pre_calculate_dead_positions(&mut self) {
        for x in 1..self.level.size.x - 1 {
            for y in 1..self.level.size.y - 1 {
                let position = Vector2::new(x, y);
                if !self.level.at_unchecked(&position).intersects(Tile::Floor)
                    || self
                        .level
                        .at_unchecked(&position)
                        .intersects(Tile::Target | Tile::Deadlock)
                {
                    continue;
                }

                for directions in [
                    Direction::Up,
                    Direction::Right,
                    Direction::Down,
                    Direction::Left,
                    Direction::Up,
                ]
                .windows(2)
                {
                    let neighbor = [
                        position + directions[0].to_vector(),
                        position + directions[1].to_vector(),
                    ];
                    if !(self.level.at_unchecked(&neighbor[0]).intersects(Tile::Wall)
                        && self.level.at_unchecked(&neighbor[1]).intersects(Tile::Wall))
                    {
                        continue;
                    }

                    self.level
                        .at_unchecked_mut(&position)
                        .insert(Tile::Deadlock);

                    let mut dead_positions = HashSet::new();
                    let mut next_position = position;
                    while !self
                        .level
                        .at_unchecked(&next_position)
                        .intersects(Tile::Wall)
                        && self
                            .level
                            .at_unchecked(&(next_position + directions[1].to_vector()))
                            .intersects(Tile::Wall)
                    {
                        dead_positions.insert(next_position);
                        next_position += -directions[0].to_vector();
                        if self
                            .level
                            .at_unchecked(&next_position)
                            .intersects(Tile::Target)
                        {
                            break;
                        }
                        if self
                            .level
                            .at_unchecked(&next_position)
                            .intersects(Tile::Wall)
                        {
                            for dead_position in dead_positions {
                                self.level
                                    .at_unchecked_mut(&dead_position)
                                    .insert(Tile::Deadlock);
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    fn pre_calculate_tunnel_positions(&mut self) {
        for x in 1..self.level.size.x - 1 {
            for y in 1..self.level.size.y - 1 {
                let position = Vector2::new(x, y);
                if !self.level.at_unchecked(&position).intersects(Tile::Floor) {
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
                    if !(self.level.at_unchecked(&neighbor[0]).intersects(Tile::Wall)
                        && self.level.at_unchecked(&neighbor[1]).intersects(Tile::Wall))
                    {
                        continue;
                    }

                    self.level.at_unchecked_mut(&position).insert(Tile::Tunnel);
                }
            }
        }
    }

    fn pre_calculate_lower_bounds(&mut self) {
        for x in 1..self.level.size.x - 1 {
            for y in 1..self.level.size.y - 1 {
                let position = Vector2::new(x, y);
                if !self.level.at_unchecked(&position).intersects(Tile::Floor)
                    || self
                        .level
                        .at_unchecked(&position)
                        .intersects(Tile::Deadlock)
                {
                    continue;
                }
                let closest_target_position = self
                    .level
                    .target_positions
                    .iter()
                    .min_by_key(|crate_pos| manhattan_distance(crate_pos, &position))
                    .unwrap();
                let movements = find_path(&position, &closest_target_position, |position| {
                    self.level.at_unchecked(&position).intersects(Tile::Wall)
                })
                .unwrap();
                self.lower_bounds
                    .insert(position, movements.len() as i32 - 1);

                // simple lower bound
                // let position = Vector2::new(x, y);
                // if !self.level.at(&position).intersects(Tile::Floor)
                //     || self.dead_positions.contains(&position)
                // {
                //     continue;
                // }
                // let closest_target_distance = self
                //     .level
                //     .target_positions
                //     .iter()
                //     .map(|crate_pos| manhattan_distance(crate_pos, &position))
                //     .min()
                //     .unwrap();
                // self.lower_bounds.insert(position, closest_target_distance);
            }
        }
    }

    #[allow(dead_code)]
    fn shrink_heap(heap: &mut BinaryHeap<State>) {
        let max_pressure = 200_000;
        if heap.len() > max_pressure {
            let mut heuristics: Vec<_> = heap.iter().map(|state| state.heuristic).collect();
            heuristics.sort_unstable();
            let mut costs: Vec<_> = heap.iter().map(|state| state.move_count()).collect();
            costs.sort_unstable();

            let alpha = 0.8;
            let heuristic_median = heuristics[(heuristics.len() as f32 * alpha) as usize];
            let cost_median = costs[(costs.len() as f32 * alpha) as usize];
            *heap = heap
                .clone()
                .into_iter()
                .filter(|state| {
                    state.heuristic <= heuristic_median && state.move_count() <= cost_median
                })
                .collect();
        }
    }

    fn print_info(visited: &HashSet<State>, heap: &BinaryHeap<State>, state: &State) {
        print!(
            "Visited: {:<6}, Heuristic: {:<4}, Moves: {:<4}, Pushes: {:<4}, Pressure: {:<4}\r",
            visited.len(),
            state.heuristic,
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
