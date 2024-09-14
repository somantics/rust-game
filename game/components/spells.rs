
use serde::{Deserialize, Serialize};

use crate::ecs::component::Diffable;
use crate::ecs::ecs::{Delta,ECS};

use crate::ecs::entity::Entity;
use crate::ecs::system::ComponentQuery;
use crate::utils::logger;

use super::core::{ImageData,  ImageHandle};

type EffectFunction = fn(&[&Entity], &ECS) -> Vec<Delta>;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum CooldownState {
    #[default]
    Available,
    Cooldown,
}

#[derive(Debug, Clone)]
pub struct Spell {
    pub name: &'static str,
    pub image: ImageHandle,
    query: ComponentQuery,
    effect: EffectFunction,
    pub castable: CooldownState,
}

impl Spell {
    pub fn new(name: &'static str, image: ImageHandle, query: ComponentQuery, effect: EffectFunction) -> Self {
        Self {name, image, query, effect,  castable: CooldownState::Available}
    }

    pub fn cast(&self, ecs: &ECS) -> Vec<Delta> {
        let CooldownState::Available = self.castable else {
            return vec![];
        };
        let entities = ecs.get_entities_matching_query(&self.query);
        (self.effect)(&entities, ecs)
    }

    pub fn on_cooldown(&self) -> Self {
        Self { 
            castable: CooldownState::Cooldown, 
            image: self.image.change_state( "cooldown"),
            ..self.clone() }
    }

    pub fn off_cooldown(&self) -> Self {
        Self { castable: CooldownState::Available, 
            image: self.image.change_state( "available"),
            ..self.clone() }
    }
}

impl Default for Spell {
    fn default() -> Self {
        Self {name: "Spell", image: ImageHandle::default(), query: ComponentQuery::default(), effect: |_, _| vec![], castable: CooldownState::default() }
    }
}

impl Diffable for Spell {
    fn apply_diff(&mut self, other: &Self) {
        self.query = other.query.clone();
        self.effect = other.effect;
        self.castable = other.castable;
    }
}


// new version under construction
#[derive(Debug, Clone)]
pub struct Spellbook {
    spells: Vec<SpellEntry>
}

#[derive(Debug, Clone)]
pub struct SpellEntry {
    pub id: usize,
    pub image: ImageHandle,
    pub castable: CooldownState,
}

#[derive(Debug, Clone)]
pub struct SpellData {
    name: &'static str,
    icon: ImageData,
    cooldown_icon: ImageData,
    query: ComponentQuery,
    effect: EffectFunction,
}