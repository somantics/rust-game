use std::cell::RefCell;
use std::rc::Rc;


use crate::combat::{Combat, Health};
use crate::gamestate::GameState;
use crate::map::Coordinate;

pub trait Diffable {
    fn apply_diff(&mut self, other: &Self);
}

// Defines which types of components exist. Components without data represent tags.
#[derive(Clone)]
pub enum ComponentType {
    Player,
    Delete,
    Bump(Rc<RefCell<dyn Response>>),
    Image(i32),
    Position(Coordinate),
    Health(Health),
    Movement(Movement),
    Combat(Rc<RefCell<Combat>>),
    Inventory(Rc<RefCell<Inventory>>),
}
// make macro for this later
impl Diffable for ComponentType {
    fn apply_diff(&mut self, other: &Self) {
        match (self, other) {
            // Single owner types
            (Self::Health(data), Self::Health(other_data)) => data.apply_diff(other_data),
            (Self::Movement(data), Self::Movement(other_data)) => data.apply_diff(other_data),
            (Self::Position(data), Self::Position(other_data)) => data.apply_diff(other_data),
            // Multiple reference types, AKA respond capable types
            (Self::Combat(data), Self::Combat(other_data)) => data.borrow_mut().apply_diff(&other_data.borrow()),
            (Self::Inventory(data), Self::Inventory(other_data)) => data.borrow_mut().apply_diff(&other_data.borrow()),
            // Overwrite types
            (Self::Image(data), Self::Image(other_data)) => *data = *other_data,
            (Self::Bump(data), Self::Bump(other_data)) => data.clone_from(other_data),
            _ => {}
        }
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

pub trait Response {
    fn process(&self, other: Vec<&ComponentType>) -> Vec<ComponentType>;
}


#[derive(Debug, Clone)]
pub struct Inventory {
    pub coins: isize,
    // items go here
    pub response: Option<fn(&Inventory, Vec<&ComponentType>) -> Vec<ComponentType>>,
}

impl Inventory {
    pub fn new(coins: isize) -> Self {
        Inventory { coins, ..Default::default()}
    }

    fn inverse(&self) -> Self {
        Inventory { coins: -self.coins, ..Default::default() }
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory { coins: 0, response: Some(award_inventory), }
    }
}

impl Diffable for Inventory {
    fn apply_diff(&mut self, other: &Self) {
        self.coins += other.coins;
        self.response = other.response;
    }
}

impl Response for Inventory {
    fn process(&self, other: Vec<&ComponentType>) -> Vec<ComponentType> {
        if let Some(response_func) = self.response {
            response_func(self, other)
        } else {
            vec![]
        }
    }
}

pub fn award_inventory(owner: &Inventory, other: Vec<&ComponentType>) -> Vec<ComponentType> {
    let requirements = vec![
        ComponentType::Inventory(Rc::new(RefCell::new(Inventory::default())))
        ];
    if GameState::has_components(other, &requirements) {
        let empty_inventory = Inventory {coins: -owner.coins, response: None};
        println!("Current coins: {:?}, delta {:?}.", owner.coins, empty_inventory.coins);
        vec![ComponentType::Inventory(Rc::new(RefCell::new(empty_inventory))),
            ComponentType::Image(8),
        ]
    } else {
        println!("Other can't accept my inventory.");
        vec![]
    }
}