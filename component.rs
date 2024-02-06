use std::mem::discriminant;

use crate::map::Coordinate;

// Defines which types of components exist.
#[derive(Debug, Clone)]
pub enum ComponentType {
    Player,
    Image(i32),
    Position(Coordinate),
    Health(Health),
    Movement(Movement),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Health {
    pub current: isize,
    pub max: isize,
}

impl Health {
    pub fn reset_to_full(&mut self) {
        self.current = self.max;
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pos: Coordinate,
}

#[derive(Debug, Clone, Default)]
pub struct Movement {
    neighbors: Vec<Coordinate>,
    steps: usize,
}

trait System {
    type ComponentRequirements;
    /*
    we want:
        run function -> game state diff
        entity filter based on components
     */
    fn run(&self, entities: &mut Vec<Self::ComponentRequirements>);
}
