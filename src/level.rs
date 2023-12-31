use bitflags::bitflags;
use nalgebra::Vector2;
use siphasher::sip::SipHasher24;

use crate::direction::Direction;

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::{fmt, fs};

#[derive(Clone, PartialEq, Eq, Hash)]
struct Node {
    // push_direction: Direction,
    crate_position: Vector2<i32>,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub struct Tile: u8 {
        const Void = 1 << 0;
        const Floor = 1 << 1;
        const Wall = 1 << 2;
        const Crate = 1 << 3;
        const Target = 1 << 4;
        const Player = 1 << 5;

        const Deadlock = 1 << 6;
        const Tunnel = 1 << 7;
    }
}

#[derive(Clone)]
pub struct Level {
    pub data: Vec<Tile>,
    pub size: Vector2<i32>,
    pub metadata: HashMap<String, String>,

    pub player_position: Vector2<i32>,
    pub crate_positions: HashSet<Vector2<i32>>,
    pub target_positions: HashSet<Vector2<i32>>,
}

impl Hash for Level {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

#[derive(Debug)]
pub enum Error {
    MoreThanOnePlayer,
    NoPlayer,
    MismatchBetweenCratesAndTargets,
    InvalidCharacter(char),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MoreThanOnePlayer => write!(f, "more than one player"),
            Error::NoPlayer => write!(f, "no player"),
            Error::MismatchBetweenCratesAndTargets => {
                write!(f, "mismatch between number of crates and targets")
            }
            Error::InvalidCharacter(c) => write!(f, "invalid character: {}", c),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

impl Level {
    pub fn new(
        map: Vec<String>,
        size: Vector2<i32>,
        metadata: HashMap<String, String>,
    ) -> Result<Self> {
        let mut data = vec![Tile::Void; (size.x * size.y) as usize];
        let mut player_position: Option<Vector2<i32>> = None;
        let mut crate_positions = HashSet::<Vector2<i32>>::new();
        let mut target_positions = HashSet::<Vector2<i32>>::new();

        for (y, line) in map.iter().enumerate() {
            for (x, char) in line.chars().enumerate() {
                let position = Vector2::<i32>::new(x as i32, y as i32);
                data[(y * size.x as usize + x) as usize] = match char {
                    ' ' | '-' | '_' => Tile::Void,
                    '#' => Tile::Wall,
                    '$' => {
                        crate_positions.insert(position);
                        Tile::Crate
                    }
                    '.' => {
                        target_positions.insert(position);
                        Tile::Target
                    }
                    '@' => {
                        if player_position.is_some() {
                            return Err(Error::MoreThanOnePlayer);
                        }
                        player_position = Some(position);
                        Tile::Player
                    }
                    '*' => {
                        crate_positions.insert(position);
                        target_positions.insert(position);
                        Tile::Crate | Tile::Target
                    }
                    '+' => {
                        if player_position.is_some() {
                            return Err(Error::MoreThanOnePlayer);
                        }
                        player_position = Some(position);
                        target_positions.insert(position);
                        Tile::Player | Tile::Target
                    }
                    _ => return Err(Error::InvalidCharacter(char)),
                };
            }
        }
        if player_position.is_none() {
            return Err(Error::NoPlayer);
        }
        if crate_positions.len() != target_positions.len() {
            return Err(Error::MismatchBetweenCratesAndTargets);
        }

        let mut instance = Self {
            data,
            size,
            metadata,
            player_position: player_position.unwrap(),
            crate_positions,
            target_positions,
        };
        instance.flood_fill(&instance.player_position.clone(), Tile::Floor, Tile::Wall);
        Ok(instance)
    }

    pub fn load_from_memory(buffer: String) -> Result<Vec<Level>> {
        let buffer = buffer.replace("\r", "") + "\n";

        let mut levels = Vec::new();

        let mut map_data = Vec::<String>::new();
        let mut map_size = Vector2::<i32>::zeros();
        let mut metadata = HashMap::<String, String>::new();

        let mut in_comment_block = false;
        for line in buffer.split(&['\n', '|']) {
            let trimmed_line = line.trim();

            // comment
            if in_comment_block {
                if trimmed_line.to_lowercase().starts_with("comment-end")
                    || trimmed_line.to_lowercase().starts_with("comment_end")
                {
                    in_comment_block = false;
                }
                continue;
            }
            if trimmed_line.starts_with(";") {
                continue;
            }

            if trimmed_line.is_empty() {
                // multiple empty lines
                if map_data.is_empty() {
                    continue;
                }

                levels.push(Level::new(map_data.clone(), map_size, metadata.clone())?);
                map_data.clear();
                map_size = Vector2::<i32>::zeros();
                metadata.clear();
                continue;
            }

            // metadata
            if trimmed_line.contains(":") {
                let (key, value) = trimmed_line.split_once(":").unwrap();
                let key = key.trim().to_lowercase();

                if key == "comment" {
                    in_comment_block = true;
                    continue;
                }

                metadata.insert(key, value.trim().to_string());
                continue;
            }

            // if line contains numbers, perform RLE decoding
            if line.chars().any(|c| c.is_digit(10)) {
                map_data.push(rle_decode(line));
            } else {
                map_data.push(line.to_string());
            }

            map_size.x = std::cmp::max(line.len() as i32, map_size.x);
            map_size.y += 1;
        }

        Ok(levels)
    }

    pub fn load_from_file(file_path: &Path) -> Result<Vec<Level>> {
        Self::load_from_memory(fs::read_to_string(file_path).unwrap())
    }

    pub fn at(&self, position: &Vector2<i32>) -> Option<Tile> {
        if self.in_bounds(position) {
            Some(self.data[(position.y * self.size.x + position.x) as usize])
        } else {
            None
        }
    }

    pub fn at_mut(&mut self, position: &Vector2<i32>) -> Option<&mut Tile> {
        if self.in_bounds(position) {
            Some(&mut self.data[(position.y * self.size.x + position.x) as usize])
        } else {
            None
        }
    }

    pub fn at_unchecked(&self, position: &Vector2<i32>) -> Tile {
        debug_assert!(self.in_bounds(position));
        self.data[(position.y * self.size.x + position.x) as usize]
    }

    pub fn at_unchecked_mut(&mut self, position: &Vector2<i32>) -> &mut Tile {
        debug_assert!(self.in_bounds(position));
        &mut self.data[(position.y * self.size.x + position.x) as usize]
    }

    pub fn export_map(&self) -> String {
        let mut result = String::new();
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let tiles = self.at_unchecked(&Vector2::<i32>::new(x, y));
                if tiles.contains(Tile::Crate | Tile::Target) {
                    result.push('*');
                } else if tiles.contains(Tile::Player | Tile::Target) {
                    result.push('+');
                } else if tiles.contains(Tile::Wall) {
                    result.push('#');
                } else if tiles.contains(Tile::Crate) {
                    result.push('$');
                } else if tiles.contains(Tile::Target) {
                    result.push('.');
                } else if tiles.contains(Tile::Player) {
                    result.push('@');
                } else {
                    result.push(' ');
                }
            }
            result.push('\n');
        }
        result
    }

    pub fn export_metadata(&self) -> String {
        let mut result = String::new();
        for (key, value) in self.metadata.iter() {
            result.push_str(&format!("{}: {}\n", key, value));
        }
        result
    }

    pub fn normalize(&mut self) {
        assert!(self
            .at_unchecked(&self.player_position)
            .contains(Tile::Floor));
        self.clear(Tile::Wall);
        self.clear(Tile::Void);
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let position = Vector2::<i32>::new(x, y);
                if self.at_unchecked(&position).intersects(Tile::Floor) {
                    let directions = [
                        Vector2::<i32>::y(),
                        -Vector2::<i32>::y(),
                        Vector2::<i32>::x(),
                        -Vector2::<i32>::x(),
                        Vector2::<i32>::new(1, 1),
                        Vector2::<i32>::new(-1, -1),
                        Vector2::<i32>::new(1, -1),
                        Vector2::<i32>::new(-1, 1),
                    ];
                    for direction in directions {
                        let neighbor_position = position + direction;
                        if !self.at_unchecked(&neighbor_position).contains(Tile::Floor) {
                            self.at_unchecked_mut(&neighbor_position).insert(Tile::Wall);
                        }
                    }
                }
            }
        }

        let mut min_hash = u64::MAX;
        for i in 1..=8 {
            self.rotate();
            if i == 5 {
                self.flip();
            }

            self.set_player_position(&normalized_area(&self.reachable_area(
                &self.player_position,
                |position| {
                    self.at_unchecked(position)
                        .intersects(Tile::Wall | Tile::Crate)
                },
            )));

            let mut hasher = SipHasher24::new();
            self.hash(&mut hasher);
            let hash = hasher.finish();

            min_hash = std::cmp::min(min_hash, hash);

            // dbg!(hash);
            // print!("{}", self.export_map());
        }

        for i in 1..=8 {
            self.rotate();
            if i == 5 {
                self.flip();
            }

            self.set_player_position(&normalized_area(&self.reachable_area(
                &self.player_position,
                |position| {
                    self.at_unchecked(position)
                        .intersects(Tile::Wall | Tile::Crate)
                },
            )));

            let mut hasher = SipHasher24::new();
            self.hash(&mut hasher);
            let hash = hasher.finish();

            if hash == min_hash {
                return;
            }
        }
        unreachable!();
    }

    pub fn reachable_area(
        &self,
        player_position: &Vector2<i32>,
        is_block: impl Fn(&Vector2<i32>) -> bool,
    ) -> HashSet<Vector2<i32>> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::<Vector2<i32>>::new();
        queue.push_back(*player_position);

        while let Some(position) = queue.pop_front() {
            if !reachable.insert(position) {
                continue;
            }

            for direction in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ] {
                let next_position = position + direction.to_vector();
                if is_block(&next_position) {
                    continue;
                }
                queue.push_back(next_position);
            }
        }

        reachable
    }

    pub fn crate_reachable_path(
        &self,
        crate_position: &Vector2<i32>,
    ) -> HashMap<Vector2<i32>, Vector2<i32>> {
        assert!(self.crate_positions.contains(crate_position));

        self.crate_reachable_path_recursive(
            crate_position,
            &self.player_position,
            &self.crate_positions,
            &mut HashMap::new(),
            &mut HashSet::new(),
        )
    }

    fn crate_reachable_path_recursive(
        &self,
        crate_position: &Vector2<i32>,
        player_position: &Vector2<i32>,
        crate_positions: &HashSet<Vector2<i32>>,
        cost: &mut HashMap<Node, i32>,
        visited: &mut HashSet<Node>,
    ) -> HashMap<Vector2<i32>, Vector2<i32>> {
        let mut came_from = HashMap::new();

        let player_reachable_area = self.reachable_area(&player_position, |position| {
            self.at_unchecked(position).intersects(Tile::Wall) || crate_positions.contains(position)
        });
        for push_direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let new_crate_position = crate_position + push_direction.to_vector();
            let new_player_position = crate_position - push_direction.to_vector();

            if self
                .at_unchecked(&new_player_position)
                .intersects(Tile::Wall)
                || !player_reachable_area.contains(&new_player_position)
            {
                continue;
            }

            if self
                .at_unchecked(&new_crate_position)
                .intersects(Tile::Wall)
                || crate_positions.contains(&new_crate_position)
            {
                continue;
            }

            let mut new_crate_positions = crate_positions.clone();
            new_crate_positions.remove(&crate_position);
            new_crate_positions.insert(new_crate_position);

            let node = Node {
                crate_position: new_crate_position,
            };

            if !visited.insert(node.clone()) {
                continue;
            }

            // FIXME: came_from 可能产生死循环
            // 因为 came_from 中每个位置只能有一个父位置, 但对于需要走回头路的情况来说, 可能重复经过一个位置, 因此一个位置可能有多个父位置
            let new_cost = cost.get(&node).unwrap_or(&0) + 1;
            if let Some(old_cost) = cost.get(&node) {
                if new_cost < *old_cost {
                    came_from.insert(new_crate_position, *crate_position);
                    cost.insert(node, new_cost);
                }
            } else {
                came_from.insert(new_crate_position, *crate_position);
                cost.insert(node, new_cost);
            }

            came_from.extend(self.crate_reachable_path_recursive(
                &new_crate_position,
                &new_player_position,
                &new_crate_positions,
                cost,
                visited,
            ));
        }

        came_from
    }

    fn set_player_position(&mut self, position: &Vector2<i32>) {
        self.at_unchecked_mut(&self.player_position.clone())
            .remove(Tile::Player);
        self.player_position = *position;
        self.at_unchecked_mut(&self.player_position.clone())
            .insert(Tile::Player);
    }

    fn rotate(&mut self) {
        let rotate_position =
            |position: &Vector2<i32>| Vector2::new(position.y, self.size.x - 1 - position.x);

        let mut rotated_data = vec![Tile::Void; (self.size.x * self.size.y) as usize];
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let position = Vector2::new(x, y);
                let rotated_position = rotate_position(&position);
                rotated_data[(rotated_position.x + rotated_position.y * self.size.y) as usize] =
                    self.at_unchecked(&position);
            }
        }

        self.data = rotated_data;
        self.player_position = rotate_position(&self.player_position);
        self.crate_positions = self
            .crate_positions
            .iter()
            .map(|x| rotate_position(x))
            .collect();
        self.size = self.size.yx();
    }

    fn flip(&mut self) {
        let flip_position =
            |position: &Vector2<i32>| Vector2::new(self.size.x - 1 - position.x, position.y);

        let mut flipped_data = vec![Tile::Void; (self.size.x * self.size.y) as usize];
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let position = Vector2::new(x, y);
                let flipped_position = flip_position(&position);
                flipped_data[(flipped_position.x + flipped_position.y * self.size.x) as usize] =
                    self.at_unchecked(&position);
            }
        }

        self.data = flipped_data;
        self.player_position = flip_position(&self.player_position);
        self.crate_positions = self
            .crate_positions
            .iter()
            .map(|x| flip_position(x))
            .collect();
    }

    pub fn in_bounds(&self, position: &Vector2<i32>) -> bool {
        0 <= position.x && position.x < self.size.x && 0 <= position.y && position.y < self.size.y
    }

    pub fn clear(&mut self, value: Tile) {
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                self.at_unchecked_mut(&Vector2::<i32>::new(x, y))
                    .remove(value);
            }
        }
    }

    fn flood_fill(&mut self, position: &Vector2<i32>, value: Tile, border: Tile) {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if !self.in_bounds(position) {
            return;
        }

        queue.push_back(*position);
        while let Some(curr_pos) = queue.pop_front() {
            if visited.contains(&curr_pos) {
                continue;
            }
            visited.insert(curr_pos);

            let curr_tiles = self.at_unchecked(&curr_pos);
            if curr_tiles.contains(value) {
                continue;
            }
            self.at_unchecked_mut(&curr_pos).insert(value);

            let directions = [
                Vector2::<i32>::y(),
                -Vector2::<i32>::y(),
                Vector2::<i32>::x(),
                -Vector2::<i32>::x(),
            ];
            for direction in directions {
                let neighbor_position = curr_pos + direction;
                if !self.in_bounds(position) {
                    continue;
                }

                if self
                    .at_unchecked(&neighbor_position)
                    .intersects(value | border)
                    || visited.contains(&neighbor_position)
                {
                    continue;
                }

                queue.push_back(neighbor_position);
            }
        }
    }
}

#[allow(dead_code)]
fn rle_encode(data: &str) -> String {
    let mut result = String::new();
    let mut chars = data.chars().peekable();
    let mut count = 0;
    while let Some(char) = chars.next() {
        count += 1;
        if chars.peek() != Some(&char) {
            if count > 1 {
                result.push_str(&count.to_string())
            }
            result.push(char);
            count = 0;
        }
    }
    result
}

fn rle_decode(data: &str) -> String {
    let mut result = String::new();
    let mut length_str = String::new();

    let mut iter = data.chars();
    while let Some(char) = iter.next() {
        if char.is_digit(10) {
            length_str.push(char);
            continue;
        }
        let mut token = String::new();
        if char == '(' {
            let mut nesting_level = 0;
            while let Some(char) = iter.next() {
                if char == '(' {
                    nesting_level += 1;
                } else if char == ')' {
                    if nesting_level == 0 {
                        break;
                    }
                    nesting_level -= 1;
                }
                token.push(char);
            }
        } else {
            token = char.to_string();
        }
        let length = length_str.parse().unwrap_or(1);
        result += &token.repeat(length);
        length_str.clear();
    }

    if result.contains("(") {
        return rle_decode(&result);
    }
    result
}

pub fn normalized_area(area: &HashSet<Vector2<i32>>) -> Vector2<i32> {
    *area
        .iter()
        .min_by(|a, b| a.x.cmp(&b.x).then_with(|| a.y.cmp(&b.y)))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_encode() {
        assert_eq!(rle_encode(""), "");
        assert_eq!(rle_encode("aaabbbcdd"), "3a3bc2d");
    }

    #[test]
    fn test_rle_decode() {
        assert_eq!(rle_decode(""), "");
        assert_eq!(rle_decode("-#$.*+@"), "-#$.*+@");
        assert_eq!(rle_decode("3-##3(.$2(+*))-#"), "---##.$+*+*.$+*+*.$+*+*-#");
    }
}
