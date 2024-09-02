use std::collections::HashSet;

use crate::ecs::component::*;
use crate::ecs::entity::*;
use crate::game::archetype::make_unit_report;
use crate::game::archetype::UnitReport;
use crate::game::components::combat::Attack;
use crate::game::components::core::*;
use crate::game::components::spells::Spell;
use crate::map;
use crate::map::boxextends::Room;
use crate::map::gamemap::GameMap;
use crate::map::mapbuilder::RoomGraph;
use crate::map::utils::Coordinate;
use super::system::ComponentQuery;

#[derive(Debug, Clone, Copy)]
pub struct DeleteEntityOrder {
    pub entity: EntityIdentifier,
}
impl DeleteEntityOrder {
    pub fn new_from_component(component_id: usize) -> Self {
        DeleteEntityOrder {
            entity: EntityIdentifier::new_from_component(component_id),
        }
    }

    pub fn new_from_entity(entity_id: usize) -> Self {
        DeleteEntityOrder {
            entity: EntityIdentifier::new_from_entity(entity_id),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeleteComponentOrder {
    pub component_id: usize,
    pub entity_id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct MakeComponentOrder {
    pub component: Component,
    pub entity: EntityIdentifier,
}

#[derive(Debug, Clone)]
pub struct MakeEntityOrder {
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EntityIdentifier {
    owned_component_id: Option<usize>,
    entity_id: Option<usize>,
}

impl EntityIdentifier {
    pub fn new_from_component(component_id: usize) -> Self {
        Self {
            owned_component_id: Some(component_id),
            entity_id: None,
        }
    }

    pub fn new_from_entity(entity_id: usize) -> Self {
        Self {
            owned_component_id: None,
            entity_id: Some(entity_id),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Delta {
    Change(Component),
    DeleteComponent(DeleteComponentOrder),
    DeleteEntity(DeleteEntityOrder),
    MakeComponent(MakeComponentOrder),
    MakeEntity(MakeEntityOrder),
}

pub struct ECS {
    component_storage: ComponentManager,
    entity_storage: EntityManager,
}

impl ECS {
    pub fn new(bsp_graph: RoomGraph) -> Self {
        ECS {
            component_storage: ComponentManager::new(),
            entity_storage: EntityManager::new(bsp_graph),
        }
    }

    pub fn print_counts(&self) {
        println!(
            "Components: {}",
            self.component_storage.get_component_count()
        );
        println!("Entities: {}", self.entity_storage.get_entity_count());
    }

    pub fn spawn_all_entities(&mut self, map: &GameMap) {
        for room in map.graph.node_weights() {
            room.spawn_entities(self, map.depth);
        }
        self.print_counts();
    }

    fn attatch_component(&mut self, entity_id: usize, component: &mut Component) {
        match component {
            Component::BumpResponse(ref mut response)
            | Component::ShotResponse(ref mut response)
            | Component::DeathResponse(ref mut response)
            | Component::FireResponse(ref mut response) => {
                response.data.own_entity = entity_id;
            }
            _ => {}
        };
        if let Component::Position(position) = component {
            let new_position = position.data;
            let old_position = match self.get_component_from_entity_id(entity_id, ComponentType::Position) {
                Some(Component::Position(indexed_data)) => Some(indexed_data.data),
                _ => None,
            };

            self.entity_storage.set_entity_position(entity_id, new_position, old_position);
        }

        self.entity_storage
            .add_component(entity_id, component.get_id());
    }

    pub fn add_component_to_entity(&mut self, entity_id: usize, mut component: Component) {
        if component.is_of_type(&ComponentType::Player) {
            if let Some(old_player) = self.entity_storage.get_player_entity() {
                for component_id in &old_player.data {
                    self.component_storage.remove_component(*component_id)
                }
                self.entity_storage.remove_entity(old_player.index);
            }
            self.entity_storage.set_new_player(entity_id);
        }

        self.component_storage.assign_id(&mut component);
        self.attatch_component(entity_id, &mut component);
        self.component_storage.register_new(component);
    }

    pub fn add_components_to_entity(&mut self, entity_id: usize, components: Vec<Component>) {
        for component in components {
            self.add_component_to_entity(entity_id, component);
        }
    }

    pub fn create_entity(&mut self) -> usize {
        let entity: IndexedData<HashSet<usize>> = IndexedData::new();
        self.entity_storage.register_new(entity)
    }

    pub fn remove_entity(&mut self, entity_id: usize) {
        if let Some(entity) = self.entity_storage.get_entity(entity_id) {
            for component in entity.data.to_owned() {
                self.component_storage.remove_component(component);
            }
            self.entity_storage.remove_entity(entity_id);
        }
    }

    pub fn update_entity_position(&mut self, indexed_change: &IndexedData<Coordinate>) {
        let Some(Component::Position(indexed_old)) = self.get_component(indexed_change.index) else {
            return;
        };
        let Some(entity_id) = self.get_entity_id_from_component_id(indexed_change.index) else {
            return;
        };
        let (old_pos, new_pos) = (indexed_old.data, indexed_old.data + indexed_change.data);

        self.entity_storage.set_entity_position(entity_id, new_pos, Some(old_pos));
    }

    pub fn copy_entity_from_other(&mut self, other: &ECS, entity_id: usize) {
        let old_components = other.get_components_from_entity_id(entity_id);
        let new_components = old_components.into_iter().cloned().collect();
        let new_entity_id = self.create_entity();
        self.add_components_to_entity(new_entity_id, new_components)
    }

    pub fn remove_component(&mut self, entity_id: usize, component_id: usize) {
        self.entity_storage
            .remove_component(entity_id, component_id);
        self.component_storage.remove_component(component_id);
    }

    fn remove_component_list(&mut self, vec: Vec<usize>) {
        for id in vec {
            self.component_storage.remove_component(id);
        }
    }

    pub fn get_entities_matching_query(&self, query: &ComponentQuery) -> Vec<&Entity> {
        let required = &query.required;
        let optional = &query.optional;
        let mut output: Vec<&Entity> = Vec::new();

        for entity in self.entity_storage.get_all_entities() {
            let components = self.component_storage.get_components(entity);
            let (mut required_matches, missed) = get_matching_components(components, &required);
            if required_matches.len() == required.len() {
                let (mut optional_matches, _) = get_matching_components(missed, &optional);
                required_matches.append(&mut optional_matches);
                output.push(entity);
            }
        }
        output
    }

    pub fn get_entity(&self, entity_id: usize) -> Option<&Entity> {
        self.entity_storage.get_entity(entity_id)
    }

    pub fn get_entities_in_room(&self, position: Coordinate) -> Vec<&Entity> {
        let room = self.entity_storage.get_room_at_coordinate(position);
        room.entities
            .iter()
            .filter_map(|entity_id| self.entity_storage.get_entity(*entity_id))
            .collect()
    }

    pub fn get_entity_id_from_component_id(&self, component_id: usize) -> Option<usize> {
        self.entity_storage.get_entity_from_component(component_id)
    }

    fn get_ids_from_components(&self, components: Vec<&Component>) -> Vec<usize> {
        components.iter().map(|comp| comp.get_id()).collect()
    }

    pub fn get_component(&self, component_id: usize) -> Option<&Component> {
        self.component_storage.get_component(&component_id)
    }

    pub fn get_components_from_entity_id(&self, entity_id: usize) -> Vec<&Component> {
        if let Some(entity) = self.get_entity(entity_id) {
            self.component_storage.get_components(entity)
        } else {
            vec![]
        }
    }

    pub fn get_components_from_entity(&self, entity: &Entity) -> Vec<&Component> {
        self.component_storage.get_components(entity)
    }

    pub fn get_component_from_entity_id(
        &self,
        entity_id: usize,
        comp_type: ComponentType,
    ) -> Option<&Component> {
        let components = self.get_components_from_entity_id(entity_id);
        components
            .iter()
            .find(|comp| comp.is_of_type(&comp_type))
            .copied()
    }

    pub fn get_component_from_entity(
        &self,
        entity: &Entity,
        comp_type: ComponentType,
    ) -> Option<&Component> {
        let components = self.get_components_from_entity(entity);
        components
            .iter()
            .find(|comp| comp.is_of_type(&comp_type))
            .copied()
    }

    pub fn is_los_blocked_by_entity(&self, coord: Coordinate) -> bool {
        match self.get_los_blocking_entity(coord) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn get_los_blocking_entity(&self, coord: Coordinate) -> Option<usize> {
        self.entity_storage
            .get_entities_at_position(coord, &self.component_storage)
            .iter()
            .find_map(|entity| {
                self.component_storage
                    .get_components(entity)
                    .iter()
                    .find(|comp| {
                        if let Component::LineOfSight(data) = comp {
                            data.data == LoSBlocking::Blocking
                        } else {
                            false
                        }
                    })
                    .map(|_| entity.index)
            })
    }

    pub fn has_player(&self) -> bool {
        let player_id = self.get_player_id();
        self.get_entity(player_id).is_some()
    }

    pub fn is_blocked_by_monster(&self, coord: Coordinate) -> bool {
        match self.get_blocking_entity(coord) {
            Some(entity_id) => {
                let components = self.get_components_from_entity_id(entity_id);
                if let (Some(Component::Monster(_)), _) =
                    take_component_from_refs(ComponentType::Monster, &components)
                {
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    pub fn is_blocked_by_door(&self, coord: Coordinate) -> bool {
        match self.get_blocking_entity(coord) {
            Some(entity_id) => {
                let components = self.get_components_from_entity_id(entity_id);
                if let (Some(Component::Door(_)), _) =
                    take_component_from_refs(ComponentType::Door, &components)
                {
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    pub fn is_blocked_by_entity(&self, coord: Coordinate) -> bool {
        match self.get_blocking_entity(coord) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn has_hazard(&self, coord: Coordinate) -> bool {
        match self.get_hazard_entity(coord) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn get_blocking_entity(&self, coord: Coordinate) -> Option<usize> {
        self.entity_storage
            .get_entities_at_position(coord, &self.component_storage)
            .iter()
            .find_map(|entity| {
                self.component_storage
                    .get_components(entity)
                    .iter()
                    .find(|comp| {
                        if let Component::Collision(data) = comp {
                            data.data == Collision::Blocking
                        } else {
                            false
                        }
                    })
                    .map(|_| entity.index)
            })
    }

    pub fn get_hazard_entity(&self, coord: Coordinate) -> Option<usize> {
        self.entity_storage
            .get_entities_at_position(coord, &self.component_storage)
            .iter()
            .find_map(|entity| {
                self.component_storage
                    .get_components(entity)
                    .iter()
                    .find(|comp| {
                        if let Component::Collision(data) = comp {
                            data.data == Collision::Hazard
                        } else {
                            false
                        }
                    })
                    .map(|_| entity.index)
            })
    }

    pub fn position_has_stairs(&self, coord: Coordinate) -> bool {
        self.get_all_entities_in_tile(coord)
            .iter()
            .find(|&&entity_id| self.entity_id_has_component(entity_id, ComponentType::Stairs))
            .is_some()
    }

    pub fn get_all_entities_in_tile(&self, coord: Coordinate) -> Vec<usize> {
        self.entity_storage
            .get_entities_at_position(coord, &self.component_storage)
            .iter()
            .map(|entity| entity.index)
            .collect()
    }

    pub fn get_all_adjacent_entities(&self, coord: Coordinate) -> Vec<usize> {
        let adjacent = [
            coord + map::utils::UP,
            coord + map::utils::DOWN,
            coord + map::utils::LEFT,
            coord + map::utils::RIGHT,
        ];
        adjacent
            .iter()
            .map(|pos| self.get_all_entities_in_tile(*pos))
            .flatten()
            .collect()
    }

    pub fn get_all_components(&self, comp_type: &ComponentType) -> Vec<&Component> {
        self.component_storage.get_all_components(comp_type)
    }

    pub fn get_player_attacks(&self) -> (Option<Attack>, Option<Attack>) {
        let player_entity = self.entity_storage.get_player_entity().unwrap();
        let player_components = self.get_components_from_entity_id(player_entity.index);
        if let (Some(Component::Combat(combat)), _) =
            take_component_from_refs(ComponentType::Combat, &player_components)
        {
            (combat.data.melee, combat.data.ranged)
        } else {
            (None, None)
        }
    }

    pub fn get_player_spells(&self) -> Vec<&IndexedData<Spell>> {
        // gets ALL spells right now, not just player spells (since no one else has spells)
        self.get_all_components(&ComponentType::Spell)
            .iter()
            .filter_map(|spell_comp| {
                if let Component::Spell(index_data) = spell_comp {
                    Some(index_data)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_player_report(&self) -> Option<UnitReport> {
        if let Some(player_entity) = self.entity_storage.get_player_entity() {
            let player_components = self.get_components_from_entity_id(player_entity.index);
            make_unit_report(&player_components)
        } else {
            None
        }
    }

    pub fn get_player_id(&self) -> usize {
        self.entity_storage.get_player_id()
    }

    pub fn get_player_position(&self) -> Option<Coordinate> {
        let player_entity = self.entity_storage.get_player_entity().unwrap();
        let player_components = self.get_components_from_entity_id(player_entity.index);

        let (maybe_position, _) =
            take_component_from_refs(ComponentType::Position, &player_components);
        match maybe_position {
            Some(Component::Position(data)) => Some(data.data),
            _ => None,
        }
    }

    pub fn set_player_position(&mut self, coord: Coordinate) {
        let player_entity = self.entity_storage.get_player_entity().unwrap();
        let player_components = self.get_components_from_entity_id(player_entity.index);

        let (maybe_position, _) =
            take_component_from_refs(ComponentType::Position, &player_components);
        if let Some(Component::Position(data)) = maybe_position.clone() {
            let change = data.make_change(coord - data.data);
            let delta = Delta::Change(Component::Position(change));
            self.apply_change(delta);
        }
    }

    pub fn apply_change(&mut self, change: Delta) {
        match change {
            Delta::Change(component) => {
                if let Component::Position(indexed_position) = &component {
                    self.update_entity_position(indexed_position);
                }
                self.component_storage.apply_change(component);
            }
            Delta::DeleteComponent(DeleteComponentOrder {
                component_id,
                entity_id,
            }) => {
                self.delete_component(component_id, entity_id);
            }
            Delta::MakeComponent(MakeComponentOrder { component, entity }) => {
                self.add_component(component, entity);
            }
            Delta::DeleteEntity(DeleteEntityOrder { entity }) => {
                self.delete_entity(entity);
            }
            Delta::MakeEntity(MakeEntityOrder { components }) => {
                let entity_id = self.create_entity();
                self.add_components_to_entity(entity_id, components);
            }
        }
    }

    pub fn apply_changes(&mut self, change_list: Vec<Delta>) {
        for change in change_list {
            self.apply_change(change);
        }
    }

    fn delete_entity(&mut self, entity: EntityIdentifier) {
        if let Some(entity_id) = self.get_entity_id_from_identifier(entity) {
            let comps = self.get_components_from_entity_id(entity_id);
            let comps = self.get_ids_from_components(comps);
            self.remove_component_list(comps);
            self.remove_entity(entity_id);
        } else {
            dbg!("Entity to delete to cannot be found", entity);
        }
    }

    fn delete_component(&mut self, component_id: usize, entity_id: Option<usize>) {
        if let Some(entity) = entity_id {
            self.remove_component(entity, component_id);
        } else if let Some(entity) = self.entity_storage.get_entity_from_component(component_id) {
            self.remove_component(entity, component_id);
        } else {
            dbg!("Component to be removed cannot be found", component_id);
        }
    }

    fn add_component(&mut self, component: Component, entity: EntityIdentifier) {
        if let Some(entity_id) = self.get_entity_id_from_identifier(entity) {
            self.add_component_to_entity(entity_id, component);
        } else {
            dbg!("Cannot find entity while adding component", entity);
        }
    }

    fn get_entity_id_from_identifier(&self, entity: EntityIdentifier) -> Option<usize> {
        let EntityIdentifier {
            owned_component_id,
            entity_id,
        } = entity;
        if let Some(id) = entity_id {
            Some(id)
        } else if let Some(id) = owned_component_id {
            self.get_entity_id_from_component_id(id)
        } else {
            None
        }
    }

    pub fn entity_id_has_component(&self, entity_id: usize, comp_type: ComponentType) -> bool {
        self.get_component_from_entity_id(entity_id, comp_type)
            .is_some()
    }

    pub fn entity_has_component(&self, entity: &Entity, comp_type: ComponentType) -> bool {
        self.get_component_from_entity(entity, comp_type)
            .is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct IndexedData<T: Default> {
    pub index: usize,
    pub data: T,
}

impl<T: Default> IndexedData<T> {
    pub fn unwrap_data(option: Option<&IndexedData<T>>) -> Option<&T> {
        match option {
            Some(data) => Some(&data.data),
            None => None,
        }
    }
}

impl<T: Default> IndexedData<T> {
    pub fn new_with(data: T) -> Self {
        Self { index: 0, data }
    }

    pub fn make_change(&self, data: T) -> Self {
        Self {
            index: self.index,
            data,
        }
    }
}

impl IndexedData<HashSet<usize>> {
    fn new() -> Self {
        Self {
            index: 0,
            data: HashSet::new(),
        }
    }
}
