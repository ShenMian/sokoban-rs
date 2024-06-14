use nalgebra::Vector2;
use soukoban::{direction::Direction, path_finding::reachable_area, Level, Tiles};

use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PushState {
    pub push_direction: Direction,
    pub box_position: Vector2<i32>,
}

pub fn box_pushable_paths_with_positions(
    level: &Level,
    box_position: &Vector2<i32>,
    initial_box_positions: &HashSet<Vector2<i32>>,
) -> HashMap<PushState, Vec<Vector2<i32>>> {
    let mut paths = HashMap::<PushState, Vec<Vector2<i32>>>::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    let player_reachable_area = reachable_area(level.player_position(), |position| {
        !level[position].intersects(Tiles::Wall) && !initial_box_positions.contains(&position)
    });
    for push_direction in [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ] {
        let player_position = box_position - &push_direction.into();
        if level[player_position].intersects(Tiles::Wall)
            || !player_reachable_area.contains(&player_position)
        {
            continue;
        }
        let new_state = PushState {
            push_direction,
            box_position: *box_position,
        };
        paths.insert(new_state.clone(), vec![*box_position]);
        queue.push_front(new_state);
    }

    while let Some(state) = queue.pop_back() {
        let mut box_positions = initial_box_positions.clone();
        box_positions.remove(box_position);
        box_positions.insert(state.box_position);

        let player_position = state.box_position - &state.push_direction.into();
        let player_reachable_area = reachable_area(player_position, |position| {
            !level[position].intersects(Tiles::Wall) && !box_positions.contains(&position)
        });

        for push_direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let new_box_position = state.box_position + &push_direction.into();
            let player_position = state.box_position - &push_direction.into();

            if level[new_box_position].intersects(Tiles::Wall /* | Tiles::Deadlock */)
                || box_positions.contains(&new_box_position)
            {
                continue;
            }

            if level[player_position].intersects(Tiles::Wall)
                || !player_reachable_area.contains(&player_position)
            {
                continue;
            }

            let new_state = PushState {
                push_direction,
                box_position: new_box_position,
            };

            if !visited.insert(new_state.clone()) {
                continue;
            }

            let mut new_path = paths[&state].clone();
            new_path.push(new_box_position);
            paths.insert(new_state.clone(), new_path);

            queue.push_front(new_state);
        }
    }

    paths.retain(|state, _| state.box_position != *box_position);
    paths
}

/// Finds paths for pushing a box from `box_position` to other positions.
pub fn box_pushable_paths(
    level: &Level,
    box_position: &Vector2<i32>,
) -> HashMap<PushState, Vec<Vector2<i32>>> {
    debug_assert!(level.box_positions().contains(box_position));
    box_pushable_paths_with_positions(level, box_position, level.box_positions())
}
