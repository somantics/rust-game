use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rand::{Rng, seq::IteratorRandom};

#[derive(Serialize, Deserialize)]
pub struct GameMapSerializable {
  vector_map: Vec<(Coordinate, GameTile)>,
  width: u32,
  height: u32,
}

pub struct GameMap {
  map: HashMap<Coordinate, GameTile>,
  width: u32,
  height: u32,
}

impl GameMap {

  pub fn add_unit(&mut self, unit: GameUnit, position: &Coordinate) {
    match self.map.get_mut(position) {
      Some(tile) => {
        tile.conditional_insert_unit(unit);
      },
      None => println!("No tile at coordinate {:?}", position)
    };
  }

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
      let tile: GameTile = GameTile { root: RootTile { image_id: rng.gen_range(0..3) }, unit: None };
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
}

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Coordinate(pub u32, pub u32);

#[derive(Default, Serialize, Deserialize, Clone )]
pub struct GameTile {
  root: RootTile,
  unit: Option<GameUnit>,
}

impl GameTile {
  pub fn get_image_ids(&self) -> Vec<i32> {
    // we always have a root tile
    let mut ids = vec![
      self.root.image_id as i32,
    ];

    // add unit image if present
    match &self.unit {
      Some(unit) => ids.push(unit.image_id as i32),
      None => {println!("no unit")},
    };

    let ids = ids;
    println!("Image id vec: {:?}", ids);
    return ids;
  }
  

  pub fn conditional_insert_unit(&mut self, unit: GameUnit) {
    match &self.unit {
      Some(_) => {},
      None => self.unit = Some(unit)
    };
  }
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameUnit {
  image_id: u32,
}

impl Default for GameUnit {
  fn default() -> Self {
    GameUnit { 
      image_id: 3
    }
  }
}