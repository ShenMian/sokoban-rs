use std::ops::{Deref, DerefMut};

use crate::direction::Direction;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Movement {
    pub direction: Direction,
    pub is_push: bool,
}

impl From<char> for Movement {
    fn from(item: char) -> Self {
        let direction = match item.to_ascii_lowercase() {
            'u' => Direction::Up,
            'd' => Direction::Down,
            'l' => Direction::Left,
            'r' => Direction::Right,
            _ => panic!("invalid character"),
        };
        Self {
            direction,
            is_push: item.is_uppercase(),
        }
    }
}

impl Into<char> for Movement {
    fn into(self) -> char {
        let c = match self.direction {
            Direction::Up => 'u',
            Direction::Down => 'd',
            Direction::Left => 'l',
            Direction::Right => 'r',
        };
        if self.is_push {
            c.to_ascii_uppercase()
        } else {
            c
        }
    }
}

impl Movement {
    pub fn with_move(direction: Direction) -> Self {
        Self {
            direction,
            is_push: false,
        }
    }

    pub fn with_push(direction: Direction) -> Self {
        Self {
            direction,
            is_push: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Movements(pub Vec<Movement>);

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

impl Movements {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn move_count(&self) -> usize {
        self.len()
    }

    pub fn push_count(&self) -> usize {
        self.iter().filter(|x| x.is_push).count()
    }

    pub fn lurd(&self) -> String {
        self.iter().map(|x| Into::<char>::into(x.clone())).collect()
    }
}
