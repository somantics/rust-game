use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::tile::GameTile;


#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct Coordinate {
  pub x: i32,
  pub y: i32
}

impl Coordinate {

  pub fn distance(&self, other: Coordinate) -> f32 {

    let delta_x = self.x - other.position().x;
    let delta_y = self.y - other.position().x;

    ((delta_x as f32).powf(2.0) + (delta_y as f32).powf(2.0)).sqrt()
  }
}

impl Euclidian for Coordinate {

  fn distance_to<T>(&self, other: T) -> f32
    where 
      T: Euclidian {

      self.distance(other.position())
  }

  fn position(&self) -> Coordinate {
      *self
  }
}

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct ImageId {
  index: i32, // for compatibility with slint
}

#[derive(Serialize, Deserialize)]
pub struct GameMapSerializable {
  vector_map: Vec<(Coordinate, GameTile)>,
  width: u32,
  height: u32,
}

pub struct GameMap {
  map: HashMap<Coordinate, GameTile>,
  pub width: u32,
  pub height: u32,
}

impl GameMap {

  pub fn get_tile_image_ids(&self) -> Vec<Vec<i32>> {

    // go over coordinates in sorted order
    (0..self.width*self.height)
      .into_iter()
      .map(|i|  {
        let coord = Coordinate{x: (i % self.width) as i32, y: (i / self.width) as i32};

        // assemble image ID data
        match self.map.get(&coord) {
          Some(tile) => {
            tile.get_image_ids()
          },
          None => {
            //println!("Could not find game tile data at {:?}.", coord);
            vec![4]
          }
        }
      })
      .collect()
  }

  pub fn create_random(width: u32, height: u32) -> GameMap {

    let mut map = HashMap::<Coordinate, GameTile>::new();
    let mut rng = rand::thread_rng();

    for i in 0..width*height {
      let x = (i % width) as i32;
      let y =  (i / width) as i32;
      let coord = Coordinate{x: x, y: y };
      let tile: GameTile = GameTile::new_random(&mut rng);
      map.insert (coord, tile);
    }

    GameMap {map, width, height}
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

    GameMap {map, width, height}
  }

  pub fn to_serializable(&self) -> GameMapSerializable {

    let serializable_kv_pairs: Vec<(Coordinate, GameTile)> = self.map
      .iter()
      .map(|(coord, tile)| (*coord, tile.clone()))
      .collect();

    GameMapSerializable {vector_map: serializable_kv_pairs, width: self.width, height: self.height }
  }

  pub fn from_serializable(other: GameMapSerializable) -> GameMap {

    let hash_map: HashMap<Coordinate, GameTile> = other.vector_map
      .iter()
      .map(|(coord, tile)| (*coord, tile.clone()))
      .collect();

    GameMap {map: hash_map, width: other.width, height: other.height }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameUnit {
  pub image_id: u32,
  pub position: Coordinate,
}

impl Default for GameUnit {

  fn default() -> Self {

    GameUnit { 
      image_id: 3,
      position: Coordinate::default(),
    }
  }
}

pub trait Euclidian {
  
  fn distance_to<T>(&self, other: T) -> f32
  where 
    T: Euclidian;
  
  fn position(&self) -> Coordinate;
}