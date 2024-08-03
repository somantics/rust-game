pub mod archetypes;
pub mod component;
pub mod spawning;
pub mod system;

use std::collections::{HashMap, HashSet};

use combat::Attack;
use component::combat::Combat;
use component::*;

use crate::ecs::archetypes::{make_unit_report, UnitReport};
use crate::map::{Coordinate, GameMap};

use self::attributes::Attributes;
use self::inventory::Inventory;
use self::system::ComponentQuery;

pub type Entity = IndexedData<HashSet<usize>>;

#[derive(Debug, Clone, Copy)]
pub struct DeleteEntityOrder{
    entity: EntityIdentifier
}
impl DeleteEntityOrder {
    fn new_from_component(component_id: usize) -> Self {
        DeleteEntityOrder {
            entity: EntityIdentifier::new_from_component(component_id)
        }
    }

    fn new_from_entity(entity_id: usize) -> Self {
        DeleteEntityOrder {
            entity: EntityIdentifier::new_from_entity(entity_id)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeleteComponentOrder{
    component_id: usize, 
    entity_id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct MakeComponentOrder{
    component: Component, 
    entity: EntityIdentifier
}

#[derive(Debug, Clone)]
pub struct MakeEntityOrder{
    components: Vec<Component>
}

#[derive(Debug, Clone, Copy)]
pub struct EntityIdentifier {
    owned_component_id: Option<usize>, 
    entity_id: Option<usize>
}

impl EntityIdentifier {
    fn new_from_component(component_id: usize) -> Self {
        Self {
            owned_component_id: Some(component_id),
            entity_id: None
        }
    }

    fn new_from_entity(entity_id: usize) -> Self {
        Self {
            owned_component_id: None,
            entity_id: Some(entity_id)
        }
    }
}

#[derive(Debug, Clone)]
pub enum Delta {
    Change(Component),
    DeleteComponent(DeleteComponentOrder),
    DeleteEntity(DeleteEntityOrder), //this is component index, not entity index
    MakeComponent(MakeComponentOrder),
    MakeEntity(MakeEntityOrder)
}

/* left to do:
improve pathfinding code
*/

pub struct ECS {
    component_storage: ComponentManager,
    entity_storage: EntityManager,
}

impl ECS {
    pub fn new() -> Self {
        ECS {
            component_storage: ComponentManager::new(),
            entity_storage: EntityManager::new(),
        }
    }

    pub fn spawn_all_entities(&mut self, map: &GameMap) {
        for room in map.graph.node_weights() {
            room.spawn_entities(self);
        }
    }

    fn attatch_component(&mut self, entity_id: usize, component: &mut Component) {
        match component {
            Component::BumpResponse(ref mut response)
            | Component::ShotResponse(ref mut response)
            | Component::DeathResponse(ref mut response) => {
                response.data.own_entity = entity_id;
            }
            _ => {}
        };

        self.entity_storage
            .add_component(entity_id, component.get_id());
    }

    pub fn add_component_to_entity(&mut self, entity_id: usize, mut component: Component) {

        if component.is_of_type(&ComponentType::Player)
        {
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

    pub fn copy_entity_from_other(&mut self, other: &ECS, entity_id: usize) {
        let old_components = other.get_components_from_entity(entity_id);
        let new_components = old_components.into_iter().cloned().collect();
        let new_entity_id = self.create_entity();
        self.add_components_to_entity(new_entity_id, new_components)

    }

    pub fn remove_component(&mut self, entity_id: usize, component_id: usize) {
        self.entity_storage.remove_component(entity_id, component_id);
        self.component_storage.remove_component(component_id);
    }

    
    fn remove_component_list(&mut self, vec: Vec<usize>) {
        for id in vec {
            self.component_storage.remove_component(id);
        };
    }

    pub fn get_entities_matching_query(&self, query: ComponentQuery) -> Vec<&Entity> {
        let required = query.required;
        let optional = query.optional;
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

    pub fn get_entity_id_from_component_id(&self, component_id: usize) -> usize {
        self.entity_storage.get_entity_from_component(component_id).expect("Component has no entity.")
    }

    fn get_ids_from_components(&self, components: Vec<&Component>) -> Vec<usize> {
        components
            .iter()
            .map(|comp| comp.get_id())
            .collect()
    }
    
    pub fn get_component(&self, component_id: usize) -> Option<&Component> {
        self.component_storage.get_component(&component_id)
    }

    pub fn get_components_from_entity(&self, entity_id: usize) -> Vec<&Component> {
        if let Some(entity) = self.get_entity(entity_id){
            self.component_storage.get_components(entity)
        } else {
            vec![]
        }
    }

    pub fn get_component_from_entity(&self, entity_id: usize, comp_type: ComponentType) -> Option<&Component> {
        let components = self.get_components_from_entity(entity_id);
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
        let player_id = self.entity_storage.player_id;
        self.get_entity(player_id).is_some()
    }

    pub fn is_blocked_by_monster(&self, coord: Coordinate) -> bool {
        match self.get_blocking_entity(coord) {
            Some(entity_id) => {
                let components = self.get_components_from_entity(entity_id);
                if let (Some(Component::Monster(_)), _) = take_component_from_refs(ComponentType::Monster, &components) {
                    true
                } else {
                    false
                }
            },
            None => false,
        }
    }

    pub fn is_blocked_by_entity(&self, coord: Coordinate) -> bool {
        match self.get_blocking_entity(coord) {
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

    pub fn position_has_stairs(&self, coord: Coordinate) -> bool {
        self.get_all_entities_in_tile(coord)
            .iter()
            .find(|&&entity_id| self.entity_has_component(entity_id, ComponentType::Stairs))
            .is_some()
    }

    pub fn get_all_entities_in_tile(&self, coord: Coordinate) -> Vec<usize> {
        self.entity_storage
            .get_entities_at_position(coord, &self.component_storage)
            .iter()
            .map(|entity| entity.index)
            .collect()
    }

    pub fn get_player_attacks(&self) -> (Option<Attack>, Option<Attack>) {
        let player_entity = self.entity_storage.get_player_entity().unwrap();
        let player_components = self.get_components_from_entity(player_entity.index);
        if let (
            Some(Component::Combat(combat)), 
            _ 
        ) = take_component_from_refs(ComponentType::Combat, &player_components) 
        {
            (combat.data.melee, combat.data.ranged)
        } else {
            (None, None)
        }
    }

    pub fn get_player_report(&self) -> Option<UnitReport> {
        if let Some(player_entity) = self.entity_storage.get_player_entity() {
            let player_components = self.get_components_from_entity(player_entity.index);
            Some(make_unit_report(&player_components))
        } else {
            None
        }
    }

    pub fn get_player_id(&self) -> usize {
        self.entity_storage.player_id
    }

    pub fn get_player_position(&self) -> Option<Coordinate> {
        let player_entity = self.entity_storage.get_player_entity().unwrap();
        let player_components = self.get_components_from_entity(player_entity.index);

        let (maybe_position, _) = take_component_from_refs(ComponentType::Position, &player_components);
        match maybe_position {
            Some(Component::Position(data)) => Some(data.data),
            _ => None
        }
    }

    pub fn set_player_position(&mut self, coord: Coordinate) {
        let player_entity = self.entity_storage.get_player_entity().unwrap();
        let player_components = self.get_components_from_entity(player_entity.index);

        let (maybe_position, _) = take_component_from_refs(ComponentType::Position, &player_components);
        if let Some(Component::Position(data)) = maybe_position.clone() {
            let change = data.make_change(coord - data.data);
            let delta = Delta::Change(Component::Position(change));
            self.apply_change(delta);
        }
    }

    pub fn apply_change(&mut self, change: Delta) {
        match change {
            Delta::Change(component) => {
                self.component_storage.apply_change(component);
            },
            Delta::DeleteComponent(DeleteComponentOrder { component_id, entity_id }) => {
                self.delete_component(component_id, entity_id);
            },
            Delta::MakeComponent(MakeComponentOrder{ component, entity  }) => {
                self.add_component(component, entity);
            },
            Delta::DeleteEntity(DeleteEntityOrder { entity }) => {
                self.delete_entity(entity);
            },
            Delta::MakeEntity(MakeEntityOrder { components }) => {
                let entity_id = self.create_entity();
                self.add_components_to_entity(entity_id, components);
            },
        }
    }

    pub fn apply_changes(&mut self, change_list: Vec<Delta>) {
        for change in change_list {
            self.apply_change(change);
        }
    }

    fn delete_entity(&mut self, entity: EntityIdentifier) {
        if let Some(entity_id) = self.get_entity_id_from_identifier(entity) {
            let comps = self.get_components_from_entity(entity_id);
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
        if let Some (entity_id) = self.get_entity_id_from_identifier(entity) {
            self.add_component_to_entity(entity_id, component);
        }
        dbg!("Entity to be added to cannot be found", entity);
    }

    fn get_entity_id_from_identifier(&self, entity: EntityIdentifier) -> Option<usize> {
        let EntityIdentifier{owned_component_id, entity_id} = entity;
        if let Some(id) = entity_id {
            Some(id)
        } else if let Some(id) = owned_component_id {
            Some(self.get_entity_id_from_component_id(id))
        } else {
            None
        }
    }

    pub fn entity_has_component(&self, entity_id: usize, comp_type: ComponentType) -> bool {
        self.get_component_from_entity(entity_id, comp_type).is_some()
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

    pub fn make_change(&self, data: T)  -> Self {
        Self { index: self.index, data }
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

#[derive(Debug, Default)]
pub struct ComponentManager {
    next_id: usize,
    components: HashMap<usize, Component>,
}

impl ComponentManager {
    fn new() -> Self {
        ComponentManager::default()
    }

    fn assign_id(&mut self, component: &mut Component) {
        component.set_id(self.next_id);
        self.next_id += 1;

    }

    fn register_new(&mut self, component: Component) {
        self.components.insert(component.get_id(), component.clone());
    }

    fn get_component(&self, id: &usize) -> Option<&Component> {
        self.components.get(&id)
    }

    fn get_component_mut(&mut self, id: &usize) -> Option<&mut Component> {
        self.components.get_mut(&id)
    }

    fn remove_component(&mut self, id: usize) {
        self.components.remove_entry(&id);
    }

    fn get_components(&self, entity: &Entity) -> Vec<&Component> {
        entity
            .data
            .iter()
            .filter_map(|id| self.get_component(id))
            .collect()
    }

    fn apply_change(&mut self, change: Component) {
        if let Some(component) = self.get_component_mut(&change.get_id()) {
            component.apply_diff(&change);
        }
    }

    fn apply_changes(&mut self, change_list: Vec<Component>) {
        for change in change_list {
            self.apply_change(change);
        }
    }
}

#[derive(Debug, Default)]
pub struct EntityManager {
    entities: Vec<Entity>,
    ids_to_reuse: Vec<usize>,
    player_id: usize, // TODO: refactor as option type
}

impl EntityManager {
    fn new() -> Self {
        Self { player_id: usize::MAX, ..Default::default() }
    }

    fn get_entity_from_component(&self, component_id: usize) -> Option<usize> {
        self
            .entities
            .iter()
            .find(|entity| entity.data.contains(&component_id))
            .map(|entity| entity.index)
    }

    fn get_all_entities(&self) -> Vec<&Entity> {
        self.entities.iter().map(|elem| elem).collect()
    }

    fn get_entity(&self, id: usize) -> Option<&Entity> {
        self.entities.get(id)
    }

    fn get_entities_at_position(
        &self,
        position: Coordinate,
        component_manager: &ComponentManager,
    ) -> Vec<&Entity> {
        self.entities
            .iter()
            .filter(|entity| {
                entity
                    .data
                    .iter()
                    .find(|comp_id| {
                        if let Some(Component::Position(data)) = component_manager.get_component(comp_id) {
                            data.data == position
                        } else {
                            false
                        }
                    })
                    .is_some()
            })
            .collect()
    }

    fn add_component(&mut self, id: usize, component_id: usize) {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.data.insert(component_id);
        }
    }

    fn remove_component(&mut self, id: usize, component_id: usize) {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.data.remove(&component_id);
        }
    }

    fn get_player_entity(&self) -> Option<&Entity> {
        self.get_entity(self.player_id)
    }

    fn set_new_player(&mut self, id: usize) {
        self.player_id = id;
    }

    fn remove_entity(&mut self, id: usize) {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.data = HashSet::new();
            self.ids_to_reuse.push(entity.index);
        }
    }

    fn register_new(&mut self, mut entity: Entity) -> usize {
        entity.index = self.next_id();
        let index = entity.index;
        if let Some(old) = self.entities.get(index) {
            assert!(
                old.data.is_empty(),
                "Provided memory index has not been emptied."
            );
            self.entities[index] = entity;
        } else {
            assert!(
                index == self.entities.len(),
                "Provided index is not in use nor immediately next."
            );
            self.entities.push(entity);
        };
        index
    }

    fn next_id(&mut self) -> usize {
        self.entities.len()
        // todo! make use of recycling system in ids_to_reuse
    }
}

// impl Default for EntityManager {
//     fn default() -> Self {
//         Self { entities: Default::default(), ids_to_reuse: Default::default(), player_id: usize::MAX }
//     }
// }

pub fn get_matching_components<'a>(
    components: Vec<&'a Component>,
    requirements: &Vec<ComponentType>,
) -> (Vec<&'a Component>, Vec<&'a Component>) {
    let mut requested_left: HashSet<&ComponentType> = requirements.into_iter().collect();
    let mut matched: Vec<&Component> = Vec::new();
    let mut missed: Vec<&Component> = Vec::new();

    let mut collection = components.into_iter();
    while let Some(component) = collection.next() {
        let comp_type: ComponentType = component.into();
        if requested_left.contains(&comp_type) {
            requested_left.remove(&comp_type);
            matched.push(component);
        } else {
            missed.push(component);
        }
    }
    (matched, missed)
}

pub fn take_component_from_refs<'a>(
    comp_type: ComponentType,
    vec: &Vec<&'a Component>,
) -> (Option<&'a Component>, Vec<&'a Component>) {
    let mut matching_component: Option<&Component> = None;
    let mut remaining_components: Vec<&Component> = Vec::new();

    for component in vec {
        if component.is_of_type(&comp_type) {
            matching_component = Some(component);
        } else {
            remaining_components.push(component);
        }
    }

    (matching_component, remaining_components)
}

pub fn take_component_from_owned(
    comp_type: ComponentType,
    vec: Vec<Component>,
) -> (Option<Component>, Vec<Component>) {
    let mut matching_component: Option<Component> = None;
    let mut remaining_components: Vec<Component> = Vec::new();

    for component in vec {
        if component.is_of_type(&comp_type) {
            matching_component = Some(component);
        } else {
            remaining_components.push(component);
        }
    }

    (matching_component, remaining_components)
}
