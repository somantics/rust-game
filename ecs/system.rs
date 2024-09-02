use crate::ecs::ecs::*;
use crate::ecs::entity::Entity;
use crate::game::components::core::*;
use crate::map::gamemap::GameMap;

#[derive(Debug, Clone)]
pub struct ComponentQuery {
    pub required: Vec<ComponentType>,
    pub optional: Vec<ComponentType>,
}

impl ComponentQuery {
    pub fn new_single(requirement: ComponentType) -> Self {
        Self { required: vec![requirement], optional: vec![] }
    }
}

impl Default for ComponentQuery {
    fn default() -> Self {
        Self {
            required: vec![],
            optional: vec![],
        }
    }
}

pub trait System {
    fn get_requirements(&self) -> ComponentQuery;
    fn run_next(&mut self, components: &[&Component], ecs: &ECS, map: &GameMap) -> Vec<Delta>;

    fn run_pre_loop(&mut self, _ecs: &ECS, _map: &GameMap) {}
    fn new_floor_update(&mut self, _ecs: &ECS, _map: &GameMap) {}
}

#[derive(Default)]
pub struct SystemManager {
    turn_systems: Vec<Box<dyn System>>,
    descend_systems: Vec<Box<dyn System>>,
}

impl SystemManager {
    pub fn new() -> Self {
        SystemManager::default()
    }

    pub fn run_system(system: &mut Box<dyn System>, ecs: &mut ECS, map: &GameMap) {
        let query = system.get_requirements();
        let matches: Vec<Entity> = ecs
            .get_entities_matching_query(&query)
            .iter()
            .map(|&data| data.to_owned())
            .collect();

        system.run_pre_loop(ecs, map);
        for entity in matches {
            let component_list = ecs.get_components_from_entity_id(entity.index);
            let changes = system.run_next(&component_list, ecs, map);
            ecs.apply_changes(changes);
        }
    }

    pub fn run_descend_systems(&mut self, ecs: &mut ECS, map: &GameMap) {
        for system in self.descend_systems.iter_mut() {
            Self::run_system(system, ecs, map);
        }
    }

    pub fn run_turn_systems(&mut self, ecs: &mut ECS, map: &GameMap) {
        for system in self.turn_systems.iter_mut() {
            Self::run_system(system, ecs, map);
        }
    }

    pub fn update_systems(&mut self, ecs: &ECS, map: &GameMap) {
        for system in self.turn_systems.iter_mut() {
            system.new_floor_update(ecs, map);
        }
    }

    pub fn add_turn_system(&mut self, system: Box<dyn System>) {
        self.turn_systems.push(system);
    }

    pub fn add_descend_system(&mut self, system: Box<dyn System>) {
        self.descend_systems.push(system);
    }
}
