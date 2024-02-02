use nalgebra::Vector2;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Convert to 2D unit vector.
    pub fn to_vector(&self) -> Vector2<i32> {
        match self {
            Direction::Up => -Vector2::<i32>::y(),
            Direction::Down => Vector2::<i32>::y(),
            Direction::Left => -Vector2::<i32>::x(),
            Direction::Right => Vector2::<i32>::x(),
        }
    }

    pub fn from_vector(vector: Vector2<i32>) -> Option<Self> {
        if vector == -Vector2::<i32>::y() {
            Some(Direction::Up)
        } else if vector == Vector2::<i32>::y() {
            Some(Direction::Down)
        } else if vector == -Vector2::<i32>::x() {
            Some(Direction::Left)
        } else if vector == Vector2::<i32>::x() {
            Some(Direction::Right)
        } else {
            None
        }
    }
}
