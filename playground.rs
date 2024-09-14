use std::any::Any;

use crate::ecs::ecs::ECS;
use crate::ecs::event::InteractionEvent;
use crate::game::components::spells::Spell;

fn get_comp_clone<T: Component + Clone + 'static>(comps: &'static [&'static dyn Component]) -> Option<T> {
    comps
        .into_iter()
        .find_map(|comp| {
            let any = &comp as &dyn Any;
            any.downcast_ref::<T>().cloned()
        })
}

fn get_comp_copy<T: Component + Copy + 'static>(comps:  &'static [&'static dyn Component]) -> Option<T> {
    comps
        .into_iter()
        .find_map(|comp| {
            let any = &comp as &dyn Any;
            any.downcast_ref::<T>().copied()
        })
}

// delta struct and trait can be macro generated, maybe not the type enum though
// this way we only use one enum, not two
// this way we define merging behavior by the type, not in one big blob by the enum
trait Component {
    fn apply_diff(&mut self, other: &dyn AnyDelta) {
        if self.get_type() == other.get_type() {
            self.apply_diff_inner(other);
        }
    }

    fn apply_diff_inner(&mut self, other: &dyn AnyDelta);
    
    fn get_type(&self) -> ComponentType;
}

trait Delta {
    fn get_type(&self) -> ComponentType;
}

trait AnyDelta: Delta + Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Delta + Any> AnyDelta for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComponentType {
    Health,
    Response,
    Spellbook,
}


struct Health {
    pub current: i32,
    pub max: i32,
}

struct HealthDelta {
    pub current: Option<i32>,
    pub max: Option<i32>,
}

impl Delta for HealthDelta {
    fn get_type(&self) -> ComponentType {
        ComponentType::Health
    }
}

impl Component for Health {
    fn apply_diff_inner(&mut self, other: &dyn AnyDelta) {
        match other.as_any().downcast_ref::<HealthDelta>() {
            Some(delta) => {
                if let Some(value) = delta.current {
                    self.current += value;
                }
                if let Some(value) = delta.max {
                    self.max += value;
                }
            }
            None => {}
        }
    }
    fn get_type(&self) -> ComponentType {
        ComponentType::Health
    }
}

struct BumpResponse {
    pub response_function: fn(&InteractionEvent, &[&dyn Component], &ECS) -> Vec<Box<dyn AnyDelta>>,
}

struct ResponseDelta {
    pub response_function: Option<fn(&InteractionEvent, &[&dyn Component], &ECS) -> Vec<Box<dyn AnyDelta>>>,
}

impl Delta for ResponseDelta {
    fn get_type(&self) -> ComponentType {
        ComponentType::Response
    }
}

impl Component for BumpResponse {
    fn apply_diff_inner(&mut self, other: &dyn AnyDelta) {
        match other.as_any().downcast_ref::<ResponseDelta>() {
            Some(delta) => {
                if let Some(value) = delta.response_function {
                    self.response_function = value;
                }
            }
            None => {}
        }
    }
    fn get_type(&self) -> ComponentType {
        ComponentType::Health
    }
}


struct Spellbook {
    spells: Vec<Spell>,
}

struct SpellbookDelta {
    spells: Vec<Option<Spell>>,
}

impl Delta for SpellbookDelta {
    fn get_type(&self) -> ComponentType {
        ComponentType::Spellbook
    }
}

impl Component for Spellbook {
    fn apply_diff_inner(&mut self, other: &dyn AnyDelta) {
        match other.as_any().downcast_ref::<SpellbookDelta>() {
            Some(delta) => {
                if self.spells.len() != delta.spells.len() {
                    return;
                }
                
                delta.spells.iter().enumerate().for_each(|(index, maybe_spell)| {
                    if let Some(value) = maybe_spell {
                        let _ = self.spells.get_mut(index).insert(&mut value.clone());
                    }
                })
            }
            None => {}
        };
    }

    fn get_type(&self) -> ComponentType {
        ComponentType::Spellbook
    }
}