use crate::map::Coordinate;

// Defines which types of components exist. Components without data represent tags.
#[derive(Debug, Clone)]
pub enum ComponentType {
    Player,
    Image(i32),
    Position(Coordinate),
    Health(Health),
    Movement(Movement),
}
// make macro for this later
impl Diffable for ComponentType {
    fn apply_diff(&mut self, other: &Self) {
        match (self, other) {
            (Self::Health(data), Self::Health(other_data)) => data.apply_diff(other_data),
            (Self::Movement(data), Self::Movement(other_data)) => data.apply_diff(other_data),
            (Self::Position(data), Self::Position(other_data)) => data.apply_diff(other_data),
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Health {
    pub current: isize,
    pub max: isize,
}

impl Diffable for Health {
    fn apply_diff(&mut self, other: &Self) {
        self.current += other.current;
        self.max += other.max;
    }
}

impl Health {
    pub fn reset_to_full(&mut self) {
        self.current = self.max;
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }
}

#[derive(Debug, Clone, Default)]
pub struct Movement {
    neighbors: Vec<Coordinate>,
    steps_left: isize,
    steps_max: isize,
}

impl Diffable for Movement {
    // neighbor directions currently not diffable
    fn apply_diff(&mut self, other: &Self) {
        self.steps_left += other.steps_left;
        self.steps_max += other.steps_max;
    }
}

pub trait Diffable {
    fn apply_diff(&mut self, other: &Self);
}

pub struct TestSystem;

impl System for TestSystem {
    fn get_component_requirements(&self) -> &Vec<ComponentType> {
        todo!()
    }

    fn run(&self, entities: Vec<(usize, Vec<&ComponentType>)>) -> Vec<(usize, Vec<ComponentType>)> {
        todo!()
    }
}

pub trait System {
    /*
    we want:
        run function -> game state diff
        entity filter based on components
            the filtering is done in the entity/component manager
     */
    fn get_component_requirements(&self) -> &Vec<ComponentType>;
    fn run(&self, entities: Vec<(usize, Vec<&ComponentType>)>) -> Vec<(usize, Vec<ComponentType>)>;
}
