use crate::ecs::component::Diffable;

pub const UP: Coordinate = Coordinate{x: 0, y:-1 };
pub const DOWN: Coordinate = Coordinate{x: 0, y:1 };
pub const LEFT: Coordinate = Coordinate{x: 1, y:0 };
pub const RIGHT: Coordinate = Coordinate{x: -1, y:0 };

pub fn reverse_direction(direction: &Coordinate) -> Coordinate {
    Coordinate {
        x: -direction.x,
        y: -direction.y,
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Default, Ord, PartialOrd)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

impl Coordinate {
    pub fn distance(&self, other: Coordinate) -> f32 {
        let delta_x = self.x - other.position().x;
        let delta_y = self.y - other.position().y;

        ((delta_x.pow(2) + delta_y.pow(2)) as f32).sqrt()
    }
}

impl Diffable for Coordinate {
    fn apply_diff(&mut self, other: &Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Euclidian for Coordinate {
    fn distance_to<T>(&self, other: T) -> f32
    where
        T: Euclidian,
    {
        self.distance(other.position())
    }

    fn position(&self) -> Coordinate {
        *self
    }
}

impl std::ops::AddAssign for Coordinate {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Add for Coordinate {
    type Output = Coordinate;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Coordinate {
    type Output = Coordinate;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<i32> for Coordinate {
    type Output = Coordinate;
    fn mul(self, rhs: i32) -> Self::Output {
        Coordinate {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

pub trait Euclidian {
    fn distance_to<T>(&self, other: T) -> f32
    where
        T: Euclidian;

    fn position(&self) -> Coordinate;
}
