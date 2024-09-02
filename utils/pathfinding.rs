use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::collections::HashMap;

use crate::{
    ecs::ecs::ECS,
    game::components::core::ComponentType,
    map::{gamemap::GameMap, utils::Coordinate},
};

#[derive(Debug, Hash, Clone, Copy)]
struct NodeData {
    distance: usize,
    h_value: usize,
    parent: Option<Coordinate>,
}

impl NodeData {
    fn emtpy() -> Self {
        NodeData {
            distance: 0,
            h_value: 0,
            parent: None,
        }
    }
    fn new(h_value: usize) -> Self {
        NodeData {
            distance: 0,
            h_value,
            parent: None,
        }
    }
    fn get_comparable(&self) -> usize {
        self.distance + self.h_value
    }
}

impl Ord for NodeData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_comparable().cmp(&other.get_comparable())
    }
}

impl PartialOrd for NodeData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_comparable().partial_cmp(&other.get_comparable())
    }
}

impl PartialEq for NodeData {
    fn eq(&self, other: &Self) -> bool {
        self.get_comparable() == other.get_comparable()
    }
}

impl Eq for NodeData {}

pub fn pathfind<F>(
    origin: Coordinate,
    destination: Coordinate,
    map: &GameMap,
    ecs: &ECS,
    heuristic: F,
    ignore_units: bool,
    ignore_doors: bool,
    ignore_hazards: bool,
) -> Option<Vec<Coordinate>>
where
    F: Fn(Coordinate) -> usize,
{
    let return_early = true;
    let origin_h_value = heuristic(origin);

    let neighbors = vec![
        Coordinate { x: 0, y: 1 },
        Coordinate { x: 0, y: -1 },
        Coordinate { x: 1, y: 0 },
        Coordinate { x: -1, y: 0 },
    ];

    let mut open = PriorityQueue::new();
    let mut closed: HashMap<Coordinate, NodeData> = HashMap::new();
    let mut last_node: (Coordinate, NodeData) = (origin, NodeData::new(origin_h_value));

    open.push(origin, Reverse(NodeData::new(origin_h_value)));

    (last_node, closed) = fill_path_map(
        open,
        closed,
        last_node,
        &neighbors,
        &destination,
        heuristic,
        return_early,
        ignore_units,
        ignore_doors,
        ignore_hazards,
        map,
        ecs,
    );

    // check if we have a solution
    if last_node.0 != destination {
        return None;
    }
    calculate_sequence(last_node, closed, origin)
}

fn calculate_sequence(
    mut last_node: (Coordinate, NodeData),
    closed: HashMap<Coordinate, NodeData>,
    origin: Coordinate,
) -> Option<Vec<Coordinate>> {
    let mut sequence: Vec<Coordinate> = Vec::new();

    while let Some(parent) = last_node.1.parent {
        let current = last_node.0;
        let delta = current - parent;
        sequence.push(delta);

        if parent == origin {
            break;
        }

        last_node = (
            parent,
            *closed.get(&parent).expect("Failed to find note data."),
        );
    }

    Some(sequence)
}

fn get_passable(
    neighbors: &[Coordinate],
    visited_coord: &Coordinate,
    destination: &Coordinate,
    ignore_units: bool,
    ignore_doors: bool,
    map: &GameMap,
    ecs: &ECS,
) -> Vec<Coordinate> {
    neighbors
        .iter()
        .map(|dir| *visited_coord + *dir)
        .filter(|&coord| {
            let blocking_entity = ecs.get_blocking_entity(coord);
            (
                map.is_tile_passable(coord) &&
                (
                    blocking_entity.is_none()
                    || ignore_units && ecs.entity_id_has_component(blocking_entity.unwrap(), ComponentType::Monster) // only one blocking entity, so if it's monster ignore
                    || ignore_doors && ecs.entity_id_has_component(blocking_entity.unwrap(), ComponentType::Door)
                )
            )
                || coord == *destination
        }).collect()
}

fn fill_path_map<F>(
    mut open: PriorityQueue<Coordinate, Reverse<NodeData>>,
    mut closed: HashMap<Coordinate, NodeData>,
    mut last_node: (Coordinate, NodeData),
    neighbors: &[Coordinate],
    destination: &Coordinate,
    heuristic: F,
    return_early: bool,
    ignore_units: bool,
    ignore_doors: bool,
    ignore_hazards: bool,
    map: &GameMap,
    ecs: &ECS,
) -> ((Coordinate, NodeData), HashMap<Coordinate, NodeData>)
where
    F: Fn(Coordinate) -> usize,
{
    let hazard_cost = match ignore_hazards {
        true => 1,
        false => 3,
    };

    while let Some((visited_coord, Reverse(visited_data))) = open.pop() {
        // add visited node to closed
        closed.insert(visited_coord, visited_data);
        last_node = (visited_coord, visited_data);

        if visited_coord == *destination && return_early {
            break;
        }

        let passable_neighbors = get_passable(
            neighbors,
            &visited_coord,
            &destination,
            ignore_units,
            ignore_doors,
            map,
            ecs,
        );

        for neighbor_coord in passable_neighbors {
            // neighbor already visited
            if closed.contains_key(&neighbor_coord) {
                continue;
            }
            let cost = match ecs.has_hazard(neighbor_coord) {
                true => hazard_cost,
                false => 1,
            };
            let distance_through_here = visited_data.distance + cost;
            // neighbor in open set already
            if let Some(Reverse(neigbor_data)) = open.get_priority(&neighbor_coord) {
                if neigbor_data.distance > distance_through_here {
                    open.change_priority(
                        &neighbor_coord,
                        Reverse(NodeData {
                            distance: distance_through_here,
                            h_value: heuristic(neighbor_coord),
                            parent: Some(visited_coord),
                        }),
                    );
                }
            // add neighbor to open set
            } else {
                open.push(
                    neighbor_coord,
                    Reverse(NodeData {
                        distance: distance_through_here,
                        h_value: heuristic(neighbor_coord),
                        parent: Some(visited_coord),
                    }),
                );
            }
        }
    }
    return (last_node, closed);
}

pub fn calculate_pathing_grid<F>(
    origin: Coordinate,
    destination: Coordinate,
    map: &GameMap,
    ecs: &ECS,
    heuristic: F,
    ignore_units: bool,
    ignore_doors: bool,
    ignore_hazards: bool,
) -> HashMap<Coordinate, Coordinate>
where
    F: Fn(Coordinate) -> usize,
{
    let return_early = false;
    let origin_h_value = heuristic(origin);

    let neighbors = vec![
        Coordinate { x: 0, y: 1 },
        Coordinate { x: 0, y: -1 },
        Coordinate { x: 1, y: 0 },
        Coordinate { x: -1, y: 0 },
    ];

    let mut open = PriorityQueue::new();
    let mut closed: HashMap<Coordinate, NodeData> = HashMap::new();
    let last_node: (Coordinate, NodeData) = (origin, NodeData::new(origin_h_value));

    open.push(origin, Reverse(NodeData::new(origin_h_value)));

    (_, closed) = fill_path_map(
        open,
        closed,
        last_node,
        &neighbors,
        &destination,
        heuristic,
        return_early,
        ignore_units,
        ignore_doors,
        ignore_hazards,
        map,
        ecs,
    );

    closed
        .into_iter()
        .filter_map(
            |(
                coord,
                NodeData {
                    distance,
                    h_value,
                    parent,
                },
            )| {
                if let Some(parent) = parent {
                    Some((coord, parent - coord))
                } else {
                    None
                }
            },
        )
        .collect()
}

pub fn astar_heuristic_factory(pl_pos: Coordinate) -> impl Fn(Coordinate) -> usize {
    move |coordinate: Coordinate| {
        ((coordinate.x - pl_pos.x).abs() + (coordinate.y - pl_pos.y).abs()) as usize
    }
}
