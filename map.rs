use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    component::Diffable,
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

    pub fn is_tile_empty(&self, coord: Coordinate) -> bool {
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
        let delta_y = self.y - other.position().x;

        ((delta_x as f32).powf(2.0) + (delta_y as f32).powf(2.0)).sqrt()
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

pub trait Euclidian {
    fn distance_to<T>(&self, other: T) -> f32
    where
        T: Euclidian;

    fn position(&self) -> Coordinate;
}
