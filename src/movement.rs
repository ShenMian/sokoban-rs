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
