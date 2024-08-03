use petgraph::Graph;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::HashMap,
    ops::{Add, Sub},
};

use crate::{
    ecs::{component::Diffable, ECS},
    map::{
        boxextends::Room,
        tile::{GameTile, TILE_NOT_FOUND, TILE_REGISTRY},
    },
};

pub mod boxextends;
pub mod mapbuilder;
pub mod tile;
pub mod pathfinding;

#[derive(Debug, Clone)]
pub struct GameMap {
    pub map: HashMap<Coordinate, GameTile>,
    pub graph: Graph<Room, (), petgraph::Undirected>,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
}

impl GameMap {
    pub fn get_tile_image_ids(&self) -> Vec<Vec<Vec<i32>>> {
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
                    Some(tile) => vec![tile.get_image()],
                    None => {
                        let im_id = TILE_REGISTRY[&TILE_NOT_FOUND.index].image.id;
                        let im_depth = TILE_REGISTRY[&TILE_NOT_FOUND.index].image.depth;
                        vec![vec![im_id, im_depth]]
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

    pub fn is_tile_los_blocking(&self, coord: Coordinate) -> bool {
        match self.map.get(&coord) {
            Some(tile) => tile.is_los_blocking(),
            None => false,
        }
    }

    pub fn set_game_tile(&mut self, coord: Coordinate, tile: GameTile) {
        self.map.insert(coord, tile);
    }

    pub fn get_game_tile(&self, coord: Coordinate) -> Option<&GameTile> {
        self.map.get(&coord)
    }

    pub fn create_empty(width: usize, height: usize) -> GameMap {
        let map = HashMap::<Coordinate, GameTile>::new();
        let graph = Graph::default();

        GameMap {
            map,
            width,
            height,
            graph,
            depth: 0,
        }
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
            graph: Graph::default(),
            depth: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameMapSerializable {
    vector_map: Vec<(Coordinate, GameTile)>,
    width: usize,
    height: usize,
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

        ((delta_x.pow(2) + delta_y.pow(2)) as f32).sqrt()
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

