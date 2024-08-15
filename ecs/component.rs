use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::Debug;

use self::attributes::Attributes;
use self::inventory::Inventory;

use super::IndexedData;
use crate::event::EventResponse;
use crate::map::Coordinate;
use behavior::TurnTaker;
use combat::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;

pub mod attributes;
pub mod behavior;
pub mod combat;
pub mod inventory;
pub mod responses;

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
#[strum_discriminants(name(ComponentType))]
pub enum Component {
    Player(IndexedData<()>),
    Monster(IndexedData<()>),
    Door(IndexedData<()>),
    Stairs(IndexedData<()>),
    Name(IndexedData<Name>),
    Inventory(IndexedData<Inventory>),
    Combat(IndexedData<Combat>),
    Image(IndexedData<ImageHandle>),
    Position(IndexedData<Coordinate>),
    Health(IndexedData<Health>),
    Turn(IndexedData<TurnTaker>),
    Collision(IndexedData<Collision>),
    LineOfSight(IndexedData<LoSBlocking>),
    Attributes(IndexedData<Attributes>),
    BumpResponse(IndexedData<EventResponse>),
    ShotResponse(IndexedData<EventResponse>),
    DeathResponse(IndexedData<EventResponse>),
}

impl Component {
    pub fn set_id(&mut self, id: usize) {
        let stored_id = match self {
            Component::Player(data) => data.index.borrow_mut(),
            Component::Monster(data) => data.index.borrow_mut(),
            Component::Door(data) => data.index.borrow_mut(),
            Component::Stairs(data) => data.index.borrow_mut(),
            Component::Name(data) => data.index.borrow_mut(),
            Component::Inventory(data) => data.index.borrow_mut(),
            Component::Combat(data) => data.index.borrow_mut(),
            Component::Image(data) => data.index.borrow_mut(),
            Component::Position(data) => data.index.borrow_mut(),
            Component::Health(data) => data.index.borrow_mut(),
            Component::Turn(data) => data.index.borrow_mut(),
            Component::Collision(data) => data.index.borrow_mut(),
            Component::LineOfSight(data) => data.index.borrow_mut(),
            Component::Attributes(data) => data.index.borrow_mut(),
            Component::BumpResponse(data) => data.index.borrow_mut(),
            Component::ShotResponse(data) => data.index.borrow_mut(),
            Component::DeathResponse(data) => data.index.borrow_mut(),
        };
        *stored_id = id;
    }

    pub fn get_id(&self) -> usize {
        match self {
            Component::Player(data) => data.index,
            Component::Monster(data) => data.index,
            Component::Door(data) => data.index,
            Component::Stairs(data) => data.index,
            Component::Name(data) => data.index,
            Component::Inventory(data) => data.index,
            Component::Combat(data) => data.index,
            Component::Image(data) => data.index,
            Component::Position(data) => data.index,
            Component::Health(data) => data.index,
            Component::Turn(data) => data.index,
            Component::Collision(data) => data.index,
            Component::LineOfSight(data) => data.index,
            Component::Attributes(data) => data.index,
            Component::BumpResponse(data) => data.index,
            Component::ShotResponse(data) => data.index,
            Component::DeathResponse(data) => data.index,
        }
    }

    pub fn is_of_type(&self, other: &ComponentType) -> bool {
        other.eq(&self.into())
    }
}

impl Diffable for Component {
    fn apply_diff(&mut self, other: &Self) {
        assert!(self.get_id() == other.get_id());

        match (self, other) {
            // Merge types
            (Self::Health(data), Self::Health(other_data)) => {
                data.data.apply_diff(&other_data.data)
            }
            (Self::Attributes(data), Self::Attributes(other_data)) => {
                data.data.apply_diff(&other_data.data)
            }
            (Self::Combat(data), Self::Combat(other_data)) => {
                data.data.apply_diff(&other_data.data)
            }
            (Self::Inventory(data), Self::Inventory(other_data)) => {
                data.data.apply_diff(&other_data.data)
            }
            (Self::Position(data), Self::Position(other_data)) => {
                data.data.apply_diff(&other_data.data)
            }
            (Self::Image(data), Self::Image(other_data)) => data.data.apply_diff(&other_data.data),
            // Clone overwrite types
            (Self::Name(data), Self::Name(other_data)) => data.data = other_data.data.clone(),
            (Self::Turn(data), Self::Turn(other_data)) => data.data = other_data.data.clone(),
            // Copy overwrite types
            (Self::Collision(data), Self::Collision(other_data)) => data.data = other_data.data,
            (Self::LineOfSight(data), Self::LineOfSight(other_data)) => data.data = other_data.data,

            (Self::BumpResponse(data), Self::BumpResponse(other_data)) => {
                data.data = other_data.data
            }
            (Self::ShotResponse(data), Self::ShotResponse(other_data)) => {
                data.data = other_data.data
            }
            (Self::DeathResponse(data), Self::DeathResponse(other_data)) => {
                data.data = other_data.data
            }
            _ => {}
        };
    }
}

pub trait Diffable {
    fn apply_diff(&mut self, other: &Self);
}

// Home of really small components
#[derive(Debug, Clone, Default)]
pub struct ImageHandle {
    pub current: ImageData,
    pub states: HashMap<&'static str, ImageData>,
}

impl ImageHandle {
    pub fn new(current: ImageData) -> Self {
        Self {
            current,
            ..Default::default()
        }
    }
}

impl Diffable for ImageHandle {
    fn apply_diff(&mut self, other: &Self) {
        self.current = other.current;
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ImageData {
    pub id: i32,
    pub depth: i32,
}

#[derive(Debug, Clone, Default)]
pub struct Name {
    pub raw: String,
}

impl Name {
    pub fn new(name: &str) -> Self {
        Self {
            raw: name.to_string(),
        }
    }
}

#[derive(Clone, Debug, Copy, Default, PartialEq, Eq)]
pub enum Collision {
    Blocking,
    Walkable,
    #[default]
    None,
}

#[derive(Clone, Debug, Copy, Default, PartialEq, Eq)]
pub enum LoSBlocking {
    Blocking,
    Partial,
    #[default]
    None,
}

#[derive(Debug, Clone, Default)]
pub struct Movement {
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
