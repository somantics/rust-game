
use std::collections::{HashMap, HashSet};

use crate::{ecs::{component::ComponentType, EntityIdentifier}, event, logger, map::{pathfinding, Coordinate, GameMap}};


use super::{
    archetypes::{self, make_unit_report}, attributes::{get_xp_to_next, Attributes}, component, take_component_from_refs, Component, DeleteEntityOrder, Delta, Entity, ECS
};

#[derive(Debug, Clone)]
pub struct ComponentQuery {
    pub required: Vec<ComponentType>,
    pub optional: Vec<ComponentType>,
}

pub trait System {
    fn get_requirements(&self) -> ComponentQuery;
    fn run_pre_loop(&mut self, ecs: &ECS, map: &GameMap);
    fn run_next(&mut self, components: &Vec<&Component>, ecs: &ECS, map: &GameMap) -> Vec<Delta>;
}

#[derive(Default)]
pub struct SystemManager {
    turn_systems: Vec<Box<dyn System>>,
}

impl SystemManager {
    pub fn new() -> Self {
        SystemManager::default()
    }

    pub fn run_system(system: &mut Box<dyn System>, ecs: &mut ECS, map: &GameMap) {
        let query = system.get_requirements();
        let matches: Vec<Entity> = ecs.get_entities_matching_query(query)
            .iter()
            .map(|&data| data.to_owned())
            .collect();
        
        system.run_pre_loop(ecs, map);
        for entity in matches {
            let component_list = ecs.get_components_from_entity(entity.index);
            let changes = system.run_next(&component_list, ecs, map); 
            ecs.apply_changes(changes);
        }
    }

    pub fn run_turn_systems(&mut self, ecs: &mut ECS, map: &GameMap) {
        for system in self.turn_systems.iter_mut() {
            Self::run_system(system, ecs, map);
        }
    }

    pub fn add_turn_system(&mut self, system: Box<dyn System>) {
        self.turn_systems.push(system);
    }
}

#[derive(Default)]
pub struct UnitCull {}

impl System for UnitCull {
    fn get_requirements(&self) -> ComponentQuery {
        ComponentQuery {
            required: vec![ComponentType::Health],
            optional: vec![],
        }
    }

    fn run_pre_loop(&mut self, ecs: &ECS, map: &GameMap) { }
    fn run_next(&mut self, components: &Vec<&Component>, ecs: &ECS, _map: &GameMap) -> Vec<Delta> {
        if let (Some(Component::Health(health)), _) =
            take_component_from_refs(ComponentType::Health, components)
        {
            if health.data.current <= 0 {
                let event = event::InteractionEvent { 
                    event_type: event::EventType::Death, 
                    attack: None, 
                    payload: vec![] 
                };
                let entity_id = ecs.get_entity_id_from_component_id(health.index);
                let mut event_results = event::propagate_event(&event, entity_id, ecs);
                event_results.push(Delta::DeleteEntity(DeleteEntityOrder::new_from_entity(entity_id)));
                return event_results;
            }
        }
        vec![]
    }
}

pub type NavigationGrid = HashMap<Coordinate, Coordinate>;
#[derive(Default)]
pub struct MonsterTurns {
    nav_grid: NavigationGrid
}

impl System for MonsterTurns {
    fn get_requirements(&self) -> ComponentQuery {
        archetypes::TURNTAKER.with(|query| query.clone())
    }

    fn run_pre_loop(&mut self, ecs: &ECS, map: &GameMap) {
        let player_report = ecs.get_player_report().unwrap();
        let player_position = player_report.position.data;
        let heuristic = |_| 0;
        let ignore_units = true;
        let ignore_doors = false;

        self.nav_grid = pathfinding::calculate_pathing_grid(
            player_position, 
            player_position, 
            map, 
            ecs, 
            heuristic, 
            ignore_units,
            ignore_doors
        );
    }

    fn run_next(&mut self, components: &Vec<&Component>, ecs: &ECS, map: &GameMap) -> Vec<Delta> {

        if let (Some(Component::Turn(data)), _) =
            take_component_from_refs(component::ComponentType::Turn, components)
        {
            data.data.process_turn(components, ecs, map, &self.nav_grid)
        } else {
            vec![]
        }
    }
}

#[derive(Default)]
pub struct PlayerCheck {}

impl System for PlayerCheck {
    fn get_requirements(&self) -> ComponentQuery {
        archetypes::PLAYER.with(|query| query.clone())
    }

    fn run_pre_loop(&mut self, _ecs: &ECS, _map: &GameMap) { }

    fn run_next(&mut self, components: &Vec<&Component>, _ecs: &ECS, _map: &GameMap) -> Vec<Delta> {
        let self_report = make_unit_report(components);
        if let Some(stats) = self_report.stats {
            if stats.data.xp >= get_xp_to_next(&stats.data) {
                let new_level = stats.data.level + 1;
                logger::log_message(&format!("You have reached level {}!", new_level));
                return vec![
                    Delta::Change(Component::Attributes(stats.make_change(
                        Attributes {
                            level_pending: true,
                            ..Default::default()
                        },
                    )))];
            }
        }
        vec![]
    }
}
