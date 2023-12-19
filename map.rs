use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rand::Rng;

pub struct GameMap {
  map: HashMap<Coordinate, GameTile>,
  width: u32,
  height: u32,
}

impl GameMap {
  pub fn get_tile_graphics(&self) -> Vec<u32> {
    let mut vec =Vec::<u32>::new();
    for i in 0..self.width*self.height {
      let coord = Coordinate(i % self.width, i / self.width);
      match self.map.get(&coord) {
        Some(tile) => {
          vec.push(tile.get_image_id());
        },
        None => {},
      }
    }
    return vec;
  }

  pub fn create_map(width: u32, height: u32) -> GameMap {
    let mut map = HashMap::<Coordinate, GameTile>::new();
    let mut rng = rand::thread_rng();

    for i in 0..width*height {
      let x = i % width;
      let y =  i / width;
      let coord = Coordinate(x, y);
      let tile: GameTile = GameTile { root: RootTile { image_id: rng.gen_range(0..3) }, unit: None };
      map.insert (coord, tile);
    }

    GameMap {map, width, height}
  }

  pub fn get_serializable(&self) -> Vec<(String, &GameTile)> {

    let serializable_kv_pairs: Vec<(String, &GameTile)> = self.map
    .iter()
    .map(|(coord, tile)| { (coord.to_string(), tile) })
    .collect();

    return serializable_kv_pairs;
  }
}

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coordinate(u32, u32);

impl Coordinate {
  pub fn to_string(&self) -> String {
    let mut out_string = self.0.to_string();
    out_string.push(':');
    out_string.push_str(&self.1.to_string());
    return out_string;
  }
}

#[derive(Default, Serialize, Deserialize)]
pub struct GameTile {
  root: RootTile,
  unit: Option<GameUnit>,
}

impl GameTile {
  pub fn get_image_id(&self) -> u32 {
    match &self.unit {
      Some(unit) => {
        unit.image_id
      },
      None => {
        self.root.image_id
      },
    } 
  }
}

#[derive(Serialize, Deserialize)]
struct RootTile {
  image_id: u32,
}

impl Default for RootTile {
  fn default() -> Self {
    RootTile { 
      image_id: 0, 
    }
  }
}

#[derive(Serialize, Deserialize)]
struct GameUnit {
  image_id: u32,
}

impl Default for GameUnit {
  fn default() -> Self {
    GameUnit { 
      image_id: 0
    }
  }
}