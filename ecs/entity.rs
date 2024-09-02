use petgraph::{graph::NodeIndex, visit::IntoNodeReferences, Graph};

use super::component::ComponentManager;
use crate::{
    ecs::ecs::*,
    game::components::core::*,
    map::{boxextends::{BoxExtends, Room}, mapbuilder::RoomGraph, utils::Coordinate},
};
use std::collections::HashSet;

pub type Entity = IndexedData<HashSet<usize>>;
pub type StorageGraph = Graph<StorageRoom, (), petgraph::Undirected>;

#[derive(Debug, Default, Clone)]
pub struct StorageRoom {
    pub extends: BoxExtends,
    pub entities: HashSet<usize>,
}

impl From<Room> for StorageRoom {
    fn from(value: Room) -> Self {
        Self { extends: value.extends.to_owned(), entities: HashSet::new() }
    }
}

impl From<&Room> for StorageRoom {
    fn from(value: &Room) -> Self {
        Self { extends: value.extends.to_owned(), entities: HashSet::new() }
    }
}

#[derive(Debug, Default)]
pub struct EntityManager {
    entities: Vec<Entity>,
    ids_to_reuse: Vec<usize>,
    room_graph: StorageGraph,
    player_id: usize, // TODO: refactor as option type
}

impl EntityManager {
    pub fn new(graph: RoomGraph) -> Self {
        let mut room_graph = StorageGraph::new_undirected();
        
        let nodes  = graph
            .node_references()
            .into_iter()
            .map(|(_, room)| room.into());

        for weight in nodes {
            room_graph.add_node(weight);
        }
        for edge in graph.edge_indices() {
            if let Some((a, b)) = graph.edge_endpoints(edge) {
                room_graph.add_edge(a, b, ());
            }
        } 

        Self {
            entities: Vec::<Entity>::with_capacity(90),
            player_id: usize::MAX,
            room_graph, 
            ..Default::default()
        }
    }

    pub fn get_entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn get_entity_from_component(&self, component_id: usize) -> Option<usize> {
        self.entities
            .iter()
            .find(|entity| entity.data.contains(&component_id))
            .map(|entity| entity.index)
    }

    pub fn get_all_entities(&self) -> Vec<&Entity> {
        self.entities.iter().map(|elem| elem).collect()
    }

    pub fn get_player_id(&self) -> usize {
        self.player_id
    }

    pub fn get_entity(&self, id: usize) -> Option<&Entity> {
        self.entities.get(id)
    }

    pub fn get_entities_at_position(
        &self,
        position: Coordinate,
        component_manager: &ComponentManager,
    ) -> Vec<&Entity> {
        let room = self.get_room_at_coordinate(position);
        room.entities
            .iter()
            .filter(|entity_id| {
                self.entities[**entity_id]
                    .data
                    .iter()
                    .find(|comp_id| {
                        if let Some(Component::Position(data)) =
                            component_manager.get_component(comp_id)
                        {
                            data.data == position
                        } else {
                            false
                        }
                    })
                    .is_some()
            })
            .map(|entity_id| &self.entities[*entity_id]) 
            .collect()
    }

    pub(super) fn add_component(&mut self, id: usize, component_id: usize) {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.data.insert(component_id);
        }
    }

    pub(super) fn remove_component(&mut self, id: usize, component_id: usize) {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.data.remove(&component_id);
        }
    }

    pub(super) fn set_entity_position(&mut self, entity_id: usize, new_position: Coordinate, old_position: Option<Coordinate>) {
        if let Some(old_position) = old_position {
            let old_room = self.get_room_at_coordinate_mut(old_position);
            old_room.entities.remove(&entity_id);
        }

        let new_room = self.get_room_at_coordinate_mut(new_position);
        new_room.entities.insert(entity_id);
    }

    pub fn get_room_at_coordinate(&self, coord: Coordinate) -> &StorageRoom {
        let root_index = NodeIndex::<u32>::new(0);
        let last_searched = Self::binary_search_rooms(root_index, root_index, coord, &self.room_graph);

        &self.room_graph[last_searched]
    }

    pub fn get_room_at_coordinate_mut(&mut self, coord: Coordinate) -> &mut StorageRoom {
        let root_index = NodeIndex::<u32>::new(0);
        let last_searched = Self::binary_search_rooms(root_index, root_index, coord, &self.room_graph);

        &mut self.room_graph[last_searched]
    }

    fn binary_search_rooms(index: NodeIndex<u32>, parent: NodeIndex<u32>, coord: Coordinate, graph: &StorageGraph) -> NodeIndex<u32> {
        if graph.edges(index).count() == 1 {
            return index;
        }

        let mut children = graph
            .neighbors(index)
            .filter(|node_index| *node_index != parent);
        let (first_child, second_child) = (children.next().unwrap(), children.next().unwrap());
        let first_room = graph.node_weight(first_child).unwrap().extends;
        
        match first_room.contains_point(coord) {
            true => Self::binary_search_rooms(first_child, index, coord, graph),
            false => Self::binary_search_rooms(second_child, index, coord, graph),
        }
    }

    pub fn get_player_entity(&self) -> Option<&Entity> {
        self.get_entity(self.player_id)
    }

    pub(super) fn set_new_player(&mut self, id: usize) {
        self.player_id = id;
    }

    pub(super) fn remove_entity(&mut self, id: usize) {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.data = HashSet::new();
            self.ids_to_reuse.push(entity.index);
        }
    }

    pub(super) fn register_new(&mut self, mut entity: Entity) -> usize {
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

pub fn get_matching_components<'a>(
    components: Vec<&'a Component>,
    requirements: &[ComponentType],
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
    vec: &[&'a Component],
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
