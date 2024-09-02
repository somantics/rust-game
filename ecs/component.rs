use std::fmt::Debug;
use std::{any::Any, collections::HashMap};

use crate::game::components::core::ComponentType;
use crate::{ecs::entity::Entity, game::components::core::Component};

#[derive(Debug, Default)]
pub struct ComponentManager {
    next_id: usize,
    components: HashMap<usize, Component>,
}

impl ComponentManager {
    pub(super) fn new() -> Self {
        ComponentManager {
            components: HashMap::<usize, Component>::with_capacity(800),
            ..Default::default()
        }
    }

    pub(super) fn assign_id(&mut self, component: &mut Component) {
        component.set_id(self.next_id);
        self.next_id += 1;
    }

    pub(super) fn register_new(&mut self, component: Component) {
        self.components
            .insert(component.get_id(), component.clone());
    }

    pub fn get_component(&self, id: &usize) -> Option<&Component> {
        self.components.get(&id)
    }

    pub(super) fn get_component_mut(&mut self, id: &usize) -> Option<&mut Component> {
        self.components.get_mut(&id)
    }

    pub(super) fn remove_component(&mut self, id: usize) {
        self.components.remove_entry(&id);
    }
    pub fn get_all_components(&self, comp_type: &ComponentType) -> Vec<&Component> {
        self.components
            .iter()
            .filter_map(|(_, comp)| {
                match comp.is_of_type(comp_type) {
                    true => Some(comp),
                    false => None
                }
            })
            .collect()
    }

    pub fn get_components(&self, entity: &Entity) -> Vec<&Component> {
        entity
            .data
            .iter()
            .filter_map(|id| self.get_component(id))
            .collect()
    }

    pub(super) fn apply_change(&mut self, change: Component) {
        if let Some(component) = self.get_component_mut(&change.get_id()) {
            component.apply_diff(&change);
        }
    }

    pub(super) fn apply_changes(&mut self, change_list: Vec<Component>) {
        for change in change_list {
            self.apply_change(change);
        }
    }

    pub fn get_component_count(&self) -> usize {
        self.components.len()
    }
}

pub trait Diffable {
    fn apply_diff(&mut self, other: &Self);
}
