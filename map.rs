use rand_distr::num_traits::Pow;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse, collections::{hash_set, BinaryHeap, HashMap, HashSet}, ops::{Add, Sub}
};
use priority_queue::PriorityQueue;

use crate::{
    component::Diffable,
    gamestate::GameState,
    tile::{GameTile, TILE_NOT_FOUND, TILE_REGISTRY},
};

// Stores all game tiles for a particular map. This contains information about
// visual tiles to use, passability, and in the future interactable objects
// like doors and chests. Does not store creatures or players.
#[derive(Debug)]
pub struct GameMap {
    map: HashMap<Coordinate, GameTile>,
    pub width: u32,
    pub height: u32,
}

impl GameMap {
    pub fn get_tile_image_ids(&self) -> Vec<Vec<i32>> {
        // go over coordinates in sorted order
        (0..self.width * self.height)
            .into_iter()
            .map(|i| {
                let coord = Coordinate {
                    x: (i % self.width) as i32,
                    y: (i / self.width) as i32,
                };

                // assemble image ID data
                match self.map.get(&coord) {
                    Some(tile) => tile.get_image_ids(),
                    None => {
                        let im_id = TILE_REGISTRY[&TILE_NOT_FOUND.index].image_id as i32;
                        vec![im_id]
                    }
                }
            })
            .collect()
    }

    pub fn is_tile_passable(&self, coord: Coordinate) -> bool {
        match self.map.get(&coord) {
            Some(tile) => tile.is_empty(),
            None => false,
        }
    }

    pub fn set_game_tile(&mut self, coord: Coordinate, tile: GameTile) {
        self.map.insert(coord, tile);
    }

    pub fn get_game_tile(&self, coord: Coordinate) -> Option<&GameTile> {
        self.map.get(&coord)
    }

    pub fn create_empty(width: u32, height: u32) -> GameMap {
        let map = HashMap::<Coordinate, GameTile>::new();

        GameMap { map, width, height }
    }

    pub fn coordinate_to_index(&self, coord: Coordinate) -> usize {
        (coord.y * self.width as i32 + coord.x) as usize
    }

    pub fn to_serializable(&self) -> GameMapSerializable {
        let serializable_kv_pairs: Vec<(Coordinate, GameTile)> = self
            .map
            .iter()
            .map(|(coord, tile)| (*coord, tile.clone()))
            .collect();

        GameMapSerializable {
            vector_map: serializable_kv_pairs,
            width: self.width,
            height: self.height,
        }
    }

    pub fn from_serializable(other: GameMapSerializable) -> GameMap {
        let hash_map: HashMap<Coordinate, GameTile> = other
            .vector_map
            .iter()
            .map(|(coord, tile)| (*coord, tile.clone()))
            .collect();

        GameMap {
            map: hash_map,
            width: other.width,
            height: other.height,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameMapSerializable {
    vector_map: Vec<(Coordinate, GameTile)>,
    width: u32,
    height: u32,
}

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct ImageId {
    index: i32, // for compatibility with slint
}

#[derive(
    Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Default, Ord, PartialOrd,
)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

impl Coordinate {
    pub fn distance(&self, other: Coordinate) -> f32 {
        let delta_x = self.x - other.position().x;
        let delta_y = self.y - other.position().y;

        ((delta_x.pow(2) + delta_y.abs().pow(2)) as f32).sqrt()
    }
}

impl Diffable for Coordinate {
    fn apply_diff(&mut self, other: &Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Euclidian for Coordinate {
    fn distance_to<T>(&self, other: T) -> f32
    where
        T: Euclidian,
    {
        self.distance(other.position())
    }

    fn position(&self) -> Coordinate {
        *self
    }
}

impl Add for Coordinate {
    type Output = Coordinate;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Coordinate {
    type Output = Coordinate;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

pub trait Euclidian {
    fn distance_to<T>(&self, other: T) -> f32
    where
        T: Euclidian;

    fn position(&self) -> Coordinate;
}

#[derive(Debug, Hash, Clone, Copy)]
struct NodeData {
    distance: usize,
    parent: Option<Coordinate>,
}

impl NodeData {
    fn origin() -> Self {
        NodeData {
            distance: 0,
            parent: None,
        }
    }
}

impl Ord for NodeData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance.cmp(&other.distance)
    }
}

impl PartialOrd for NodeData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl PartialEq for NodeData {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for NodeData {}

pub fn pathfind(origin: Coordinate, destination: Coordinate, game: &GameState) -> Option<Vec<Coordinate>> {
    let neighbors = vec![
        Coordinate { x: 0, y: 1 },
        Coordinate { x: 0, y: -1 },
        Coordinate { x: 1, y: 0 },
        Coordinate { x: -1, y: 0 },
    ];

    let mut open = PriorityQueue::new();
    let mut closed: HashMap<Coordinate, NodeData> = HashMap::new();
    let mut last_node: (Coordinate, NodeData) = (origin, NodeData::origin());

    open.push(origin, Reverse(NodeData::origin()));

    while let Some((visited_coord, Reverse(visited_data))) = open.pop() {
        // add visited node to closed
        closed.insert(visited_coord, visited_data);
        last_node = (visited_coord, visited_data);

        if visited_coord == destination {
            break;
        }

        // filter out impassable neighbors
        let passable_neighbors = neighbors
            .iter()
            .map(|&dir| visited_coord + dir)
            .filter(|&coord| game.is_tile_passable(coord) && !game.is_blocked_by_entity(coord));

        for neighbor_coord in passable_neighbors {
            // neighbor already visited
            if closed.contains_key(&neighbor_coord) {
                continue;
            }

            let distance_through_here = visited_data.distance + 1;
            // neighbor in open set already
            if let Some(Reverse(neigbor_data)) = open.get_priority(&neighbor_coord) {
                if neigbor_data.distance > distance_through_here {
                    open.change_priority(
                        &neighbor_coord, 
                        Reverse(NodeData { 
                            distance: distance_through_here, 
                            parent: Some(visited_coord)
                    }));
                }
            // add neighbor to open set
            } else {
                open.push(neighbor_coord, Reverse(NodeData { 
                    distance: distance_through_here, 
                    parent: Some(visited_coord)
                }));
            }
        }
    }
    // check if we have a solution
    if last_node.0 != destination {
        return None;
    }
    // do the backtracking and return sequence of move instructions
    let mut sequence: Vec<Coordinate> = Vec::new();

    while let Some(parent) = last_node.1.parent {
        let current = last_node.0;
        let delta = current - parent;
        sequence.push(delta);

        if parent == origin {
            break;
        }

        last_node = (parent, *closed.get(&parent).expect("Failed to find note data."));
    }

    Some(sequence)
}
