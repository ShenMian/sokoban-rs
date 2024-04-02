use std::ops::{Deref, DerefMut};

use crate::direction::Direction;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Movement {
    Move(Direction),
    Push(Direction),
}

impl Movement {
    pub fn direction(&self) -> Direction {
        match self {
            Movement::Move(direction) => *direction,
            Movement::Push(direction) => *direction,
        }
    }

    pub fn is_push(&self) -> bool {
        matches!(&self, Movement::Push(_))
    }
}

impl From<char> for Movement {
    fn from(ch: char) -> Self {
        let direction = match ch.to_ascii_lowercase() {
            'u' => Direction::Up,
            'd' => Direction::Down,
            'l' => Direction::Left,
            'r' => Direction::Right,
            _ => panic!("invalid character"),
        };
        if ch.is_uppercase() {
            Movement::Push(direction)
        } else {
            Movement::Move(direction)
        }
    }
}

impl From<Movement> for char {
    fn from(movement: Movement) -> Self {
        let char = match movement.direction() {
            Direction::Up => 'u',
            Direction::Down => 'd',
            Direction::Left => 'l',
            Direction::Right => 'r',
        };
        if movement.is_push() {
            char.to_ascii_uppercase()
        } else {
            char
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Movements(pub Vec<Movement>);

impl Movements {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_lurd(lurd: &str) -> Self {
        let mut instance = Self::new();
        for ch in lurd.chars() {
            instance.push(Movement::from(ch));
        }
        instance
    }

    pub fn move_count(&self) -> usize {
        self.len()
    }

    pub fn push_count(&self) -> usize {
        self.iter().filter(|x| x.is_push()).count()
    }

    pub fn lurd(&self) -> String {
        self.iter().map(|x| Into::<char>::into(x.clone())).collect()
    }
}

impl Deref for Movements {
    type Target = Vec<Movement>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Movements {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
