use std::{borrow::BorrowMut, collections::HashMap, ops::{Add, AddAssign}};
use strum_macros::EnumDiscriminants;

use crate::{
    ecs::{component::Diffable, ecs::IndexedData, event::EventResponse},
    game::components::{
        attributes::Attributes,
        behavior::TurnTaker,
        combat::{Combat, Health},
        inventory::Inventory,
    },
    map::utils::Coordinate,
};

use super::spells::Spell;

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
#[strum_discriminants(name(ComponentType))]
pub enum Component {
    Player(IndexedData<()>),
    Monster(IndexedData<()>),
    Door(IndexedData<()>),
    Stairs(IndexedData<()>),
    Name(IndexedData<Name>),
    Spell(IndexedData<Spell>),
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
    FireResponse(IndexedData<EventResponse>),
    DurationEffect(IndexedData<DurationEffect>),
}

impl Component {
    pub fn set_id(&mut self, id: usize) {
        let stored_id = match self {
            Component::Player(data) => data.index.borrow_mut(),
            Component::Monster(data) => data.index.borrow_mut(),
            Component::Door(data) => data.index.borrow_mut(),
            Component::Stairs(data) => data.index.borrow_mut(),
            Component::Name(data) => data.index.borrow_mut(),
            Component::Spell(data) => data.index.borrow_mut(),
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
            Component::FireResponse(data) => data.index.borrow_mut(),
            Component::DurationEffect(data) => data.index.borrow_mut(),
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
            Component::Spell(data) => data.index,
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
            Component::FireResponse(data) => data.index,
            Component::DurationEffect(data) => data.index,
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
                data.data.apply_diff(&other_data.data);
            }
            (Self::Attributes(data), Self::Attributes(other_data)) => {
                data.data.apply_diff(&other_data.data);
            }
            (Self::Combat(data), Self::Combat(other_data)) => {
                data.data.apply_diff(&other_data.data);
            }
            (Self::Inventory(data), Self::Inventory(other_data)) => {
                data.data.apply_diff(&other_data.data);
            }
            (Self::Position(data), Self::Position(other_data)) => {
                data.data.apply_diff(&other_data.data);
            }
            (Self::DurationEffect(data), Self::DurationEffect(other_data)) => {
                data.data += other_data.data;
            }
            (Self::Image(data), Self::Image(other_data)) => data.data.apply_diff(&other_data.data),
            // Clone overwrite types
            (Self::Name(data), Self::Name(other_data)) => data.data = other_data.data.clone(),
            (Self::Turn(data), Self::Turn(other_data)) => data.data = other_data.data.clone(),
            (Self::Spell(data), Self::Spell(other_data)) => data.data = other_data.data.clone(),
            // Copy overwrite types
            (Self::Collision(data), Self::Collision(other_data)) => data.data = other_data.data,
            (Self::LineOfSight(data), Self::LineOfSight(other_data)) => data.data = other_data.data,

            (Self::BumpResponse(data), Self::BumpResponse(other_data)) => {
                data.data = other_data.data;
            }
            (Self::ShotResponse(data), Self::ShotResponse(other_data)) => {
                data.data = other_data.data;
            }
            (Self::DeathResponse(data), Self::DeathResponse(other_data)) => {
                data.data = other_data.data;
            }
            (Self::FireResponse(data), Self::FireResponse(other_data)) => {
                data.data = other_data.data;
            }
            _ => {}
        };
    }
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

    pub fn new_spell(available: i32, cooldown: i32) -> Self {
        let states = HashMap::from([
            ("available", ImageData::new(available)),
            ("cooldown", ImageData::new(cooldown)),
        ]);
        Self { 
            current: ImageData::new(available), 
            states, 
        }
    }

    pub fn change_state(&self, new_state: &str) -> Self {
        let current = *self.states.get(new_state).unwrap_or(&self.current);
        Self {
            current,
            ..self.clone()
        }
    }
}

impl Diffable for ImageHandle {
    fn apply_diff(&mut self, other: &Self) {
        self.current = other.current;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ImageData {
    pub id: i32,
    pub depth: i32,
}

impl ImageData {
    pub fn new(id: i32) -> Self{
        Self { id, depth: 5 }
    }
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
    Hazard,
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


#[derive(Debug, Default, Clone, Copy)]
pub struct DurationEffect(pub isize, pub EffectType);

impl Add<DurationEffect> for DurationEffect {
    type Output = DurationEffect;
    fn add(self, rhs: DurationEffect) -> Self::Output {
        DurationEffect(self.0 + rhs.0, self.1)
    }

}

impl AddAssign<DurationEffect> for DurationEffect {
    fn add_assign(&mut self, rhs: DurationEffect) {
        *self = *self + rhs;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum EffectType {
    #[default]
    None,
    Burning,
    Invisible,
    Levitate,
    Stoneskin,
    Acid,
}