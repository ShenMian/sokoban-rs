use nalgebra::Vector2;
use soukoban::{Map, Tiles, direction::Direction, path_finding::compute_reachable_area};

use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PushState {
    pub box_position: Vector2<i32>,
    pub push_direction: Direction,
}

pub fn box_pushable_paths_with_positions(
    map: &Map,
    box_position: &Vector2<i32>,
    initial_box_positions: &HashSet<Vector2<i32>>,
) -> HashMap<PushState, Vec<Vector2<i32>>> {
    let mut paths = HashMap::<PushState, Vec<Vector2<i32>>>::new();
    let mut queue = VecDeque::new();

    let player_reachable_area = compute_reachable_area(map.player_position(), |position| {
        !map[position].intersects(Tiles::Wall) && !initial_box_positions.contains(&position)
    });
    for push_direction in Direction::iter() {
        let player_position = box_position - &push_direction.into();
        if map[player_position].intersects(Tiles::Wall)
            || !player_reachable_area.contains(&player_position)
        {
            continue;
        }
        let new_state = PushState {
            box_position: *box_position,
            push_direction,
        };
        paths.insert(new_state.clone(), vec![*box_position]);
        queue.push_front(new_state);
    }

    while let Some(state) = queue.pop_back() {
        let mut box_positions = initial_box_positions.clone();
        box_positions.remove(box_position);
        box_positions.insert(state.box_position);

        let player_position = state.box_position - &state.push_direction.into();
        let player_reachable_area = compute_reachable_area(player_position, |position| {
            !map[position].intersects(Tiles::Wall) && !box_positions.contains(&position)
        });

        for push_direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let new_box_position = state.box_position + &push_direction.into();
            if map[new_box_position].intersects(Tiles::Wall /* | Tiles::Deadlock */)
                || box_positions.contains(&new_box_position)
            {
                continue;
            }

            let player_position = state.box_position - &push_direction.into();
            if map[player_position].intersects(Tiles::Wall)
                || !player_reachable_area.contains(&player_position)
            {
                continue;
            }

            let new_state = PushState {
                box_position: new_box_position,
                push_direction,
            };

            if paths.contains_key(&new_state) {
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
    map: &Map,
    box_position: &Vector2<i32>,
) -> HashMap<PushState, Vec<Vector2<i32>>> {
    debug_assert!(map.box_positions().contains(box_position));
    box_pushable_paths_with_positions(map, box_position, map.box_positions())
}
