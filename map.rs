use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use rand::Rng;

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
        let coord = Coordinate(i % self.width, i / self.width);

        // assemble image ID data
        match self.map.get(&coord) {
          Some(tile) => {
            tile.get_image_ids()
          },
          None => panic!("Could not find game tile data at {:?}.", coord)
        }
      })
      .collect()
  }

  pub fn create_map(width: u32, height: u32) -> GameMap {
    let mut map = HashMap::<Coordinate, GameTile>::new();
    let mut rng = rand::thread_rng();

    for i in 0..width*height {
      let x = i % width;
      let y =  i / width;
      let coord = Coordinate(x, y);
      let tile: GameTile = GameTile { root: RootTile { image_id: rng.gen_range(0..3), passable: true } };
      map.insert (coord, tile);
    }

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

  pub fn is_tile_empty(&self, coord: Coordinate) -> bool {
    match self.map.get(&coord) {
      Some(tile) => tile.is_empty(),
      None => false,
    }
  }
}

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Coordinate(pub u32, pub u32);

#[derive(Default, Serialize, Deserialize, Clone )]
pub struct GameTile {
  root: RootTile,
  // room for containers and stuff here
}

impl GameTile {
  pub fn get_image_ids(&self) -> Vec<i32> {
    // we always have a root tile
    let mut ids = vec![
      self.root.image_id as i32,
    ];

    let ids = ids;
    println!("Image id vec: {:?}", ids);
    return ids;
  }

  pub fn is_empty(&self) -> bool {
    if !self.root.is_passable() {return false};

    true
  }
}

#[derive(Serialize, Deserialize, Clone)]
struct RootTile {
  image_id: u32,
  passable: bool,
}

impl RootTile {
  pub fn is_passable(&self) -> bool {
    self.passable
  }
}

impl Default for RootTile {
  fn default() -> Self {
    RootTile { 
      image_id: 0, 
      passable: true,
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameUnit {
  pub image_id: u32,
  pub position: Coordinate,
}

impl GameUnit {
  pub fn coord(&self) -> Coordinate {
    self.position
  }
}

impl Default for GameUnit {
  fn default() -> Self {
    GameUnit { 
      image_id: 3,
      position: Coordinate(0, 0)
    }
  }
}