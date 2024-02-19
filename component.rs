use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::vec;

use serde::de::IgnoredAny;

use crate::combat::{Combat, Health};
use crate::gamestate::{get_position_from, has_components, Diff, GameState, Match};
use crate::map::Coordinate;

pub trait Diffable {
    fn apply_diff(&mut self, other: &Self);
}

// Defines which types of components exist. Components without data represent tags.
#[derive(Clone, Debug)]
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
    Turn(TurnTaker),
    Collision(bool),
    Interaction(Interact),
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
            (Self::Combat(data), Self::Combat(other_data)) => {
                data.borrow_mut().apply_diff(&other_data.borrow())
            }
            (Self::Inventory(data), Self::Inventory(other_data)) => {
                data.borrow_mut().apply_diff(&other_data.borrow())
            }
            // Overwrite types
            (Self::Image(data), Self::Image(other_data)) => *data = *other_data,
            (Self::Collision(data), Self::Collision(other_data)) => *data = *other_data,
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

impl Movement {
    fn new() -> Self {
        Self {
            neighbors: vec![
                Coordinate { x: 1, y: 0 },
                Coordinate { x: -1, y: 0 },
                Coordinate { x: 0, y: 1 },
                Coordinate { x: 0, y: -1 },
            ],
            steps_left: 0,
            steps_max: 0,
        }
    }
}

impl Diffable for Movement {
    // neighbor directions currently not diffable
    fn apply_diff(&mut self, other: &Self) {
        self.steps_left += other.steps_left;
        self.steps_max += other.steps_max;
    }
}

pub trait Response: Debug {
    fn process(&self, own_id: usize, other_id: usize, other: Match) -> Vec<Diff>;
}

#[derive(Debug, Clone)]
pub struct Interact {
    pub response: Option<fn(&Interact, own_id: usize, other_id: usize, other: Match) -> Vec<Diff>>,
}

impl Interact {
    pub fn new_door() -> Self {
        Interact { response: Some(open_door) }
    }
}

impl Response for Interact {
    fn process(&self, own_id: usize, other_id: usize, other: Match) -> Vec<Diff> {
        if let Some(response_func) = self.response {
            response_func(self, own_id, other_id, other)
        } else {
            vec![]
        }
    }
}

pub fn open_door(owner: &Interact, own_id: usize, other_id: usize, other: Match) -> Vec<Diff> {
    vec![
        // set image to open door and turn off collision (which will turn off bumps)
        (
            own_id,
            vec![ComponentType::Image(10), ComponentType::Collision(false)],
        ),
    ]
}

#[derive(Debug, Clone)]
pub struct Inventory {
    pub coins: isize,
    // items go here
    pub response: Option<fn(&Inventory, own_id: usize, other_id: usize, other: Match) -> Vec<Diff>>,
}

impl Inventory {
    pub fn new(coins: isize) -> Self {
        Inventory {
            coins,
            ..Default::default()
        }
    }

    fn inverse(&self) -> Self {
        Inventory {
            coins: -self.coins,
            ..Default::default()
        }
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory {
            coins: 0,
            response: Some(award_inventory),
        }
    }
}

impl Diffable for Inventory {
    fn apply_diff(&mut self, other: &Self) {
        self.coins += other.coins;
        self.response = other.response;
    }
}

impl Response for Inventory {
    fn process(&self, own_id: usize, other_id: usize, other: Match) -> Vec<Diff> {
        if let Some(response_func) = self.response {
            response_func(self, own_id, other_id, other)
        } else {
            vec![]
        }
    }
}

pub fn award_inventory(
    owner: &Inventory,
    own_id: usize,
    other_id: usize,
    other: Match,
) -> Vec<Diff> {
    let requirements = vec![ComponentType::Inventory(Rc::new(RefCell::new(
        Inventory::default(),
    )))];
    if has_components(other, &requirements) {
        let empty_inventory = Inventory {
            coins: -owner.coins,
            response: None,
        };
        let award_inventory = Inventory {
            coins: owner.coins,
            response: None,
        };

        println!("Giving player {:?}", award_inventory.coins);
        vec![
            (
                own_id,
                vec![
                    ComponentType::Inventory(Rc::new(RefCell::new(empty_inventory))),
                    ComponentType::Image(8),
                ],
            ),
            (
                other_id,
                vec![ComponentType::Inventory(Rc::new(RefCell::new(
                    award_inventory,
                )))],
            ),
        ]
    } else {
        println!("Other can't accept my inventory.");
        vec![]
    }
}

enum AIAction {
    Approach,
    Attack,
    Flee,
    Wander,
    Sleep,
    //Ability(usize),
}

#[derive(Debug, Clone)]
pub struct TurnTaker {
    pub movement: Movement,
    behavior: Rc<RefCell<dyn Behavior>>,
}

impl TurnTaker {
    pub fn new() -> Self {
        Self {
            movement: Movement::new(),
            ..Default::default()
        }
    }
    pub fn process_turn(&self, game: &GameState, own_id: usize) -> Vec<Diff> {
        let (pl_id, pl_entity) = game.get_player();
        let my_entity = game.get_entity_of(own_id).unwrap();
        let pl_pos = get_position_from(pl_entity).unwrap();
        let my_pos = get_position_from(my_entity.clone()).unwrap();

        match self.behavior.borrow().select_action(game, own_id) {
            AIAction::Approach => {
                return self.approach_player(game, own_id, my_pos, pl_pos);
            }
            AIAction::Attack => {
                println!("Doggo bite.");
                return game.propagate_bump_to(own_id, my_entity, pl_id);
            }
            _ => {
                return vec![];
            }
        }
    }

    fn approach_player(
        &self,
        game: &GameState,
        own_id: usize,
        my_pos: Coordinate,
        pl_pos: Coordinate,
    ) -> Vec<Diff> {
        let direction = self
            .movement
            .neighbors
            .iter()
            .filter_map(|&dir| {
                // gather free directions into (distance_to_player, dir) pairs
                let dest = dir + my_pos;
                if let Some(_) = game.get_tile_occupant_id(dest) {
                    None
                } else {
                    if game.is_tile_passable(dest) {
                        Some((dest.distance(pl_pos), dir))
                    } else {
                        None
                    }
                }
            })
            .reduce(|(distance_a, data_a), (distance_b, data_b)| {
                // get best pair
                if distance_a < distance_b {
                    (distance_a, data_a)
                } else {
                    (distance_b, data_b)
                }
            })
            .map(|(_, data)| data);

        if let Some(coord) = direction {
            let diff = vec![ComponentType::Position(coord)];
            vec![(own_id, diff)]
        } else {
            vec![]
        }
    }
}

impl Default for TurnTaker {
    fn default() -> Self {
        TurnTaker {
            movement: Movement::default(),
            behavior: Rc::new(RefCell::new(DogBehavior::default())),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct DogBehavior {
    // some internal state here
}

impl Behavior for DogBehavior {
    fn select_action(&self, game: &GameState, own_id: usize) -> AIAction {
        let my_pos = game.get_position_of(own_id).unwrap();
        let pl_pos = game.get_player_position().unwrap();
        if my_pos.distance(pl_pos) > 1.0 {
            AIAction::Approach
        } else {
            AIAction::Attack
        }
    }
}

trait Behavior: Debug {
    fn select_action(&self, game: &GameState, own_id: usize) -> AIAction;
}
