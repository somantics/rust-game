use serde::{Deserialize, Serialize};
use rand::{Rng, rngs::ThreadRng};
use ::phf::{Map, phf_map};


static TILE_REGISTRY: Map<u32, &'static RootTile> = phf_map!(
  0u32 => &RootTile {image_id: 0, passable: true},
  1u32 => &RootTile {image_id: 1, passable: true},
  2u32 => &RootTile {image_id: 2, passable: false},
);

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct TileID {
  pub index: u32,
}

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct ObjectID {
  index: u32,
}

#[derive(Default, Serialize, Deserialize, Clone )]
pub struct GameTile {
  pub root_tile: TileID,
  // room for containers and stuff here
}

impl GameTile {
  pub fn get_image_ids(&self) -> Vec<i32> {
    let mut ids = vec![];
    
    let tile = self.get_root_tile();

    match tile {
      Some(root) => ids.push(root.image_id as i32),
      None => (),
    };

    let ids = ids;
    ids
  }

  pub fn is_empty(&self) -> bool {
    let tile = self.get_root_tile().expect("No root tile found.");
    if !tile.passable {return false};

    true
  }

  pub fn new_random(rng: &mut ThreadRng) -> GameTile {
    GameTile {root_tile: TileID { index: rng.gen_range(0..3) } }
  }

  fn get_root_tile(&self) -> Option<&RootTile> {
    TILE_REGISTRY.get(&self.root_tile.index).copied()
  }
}

#[derive(Serialize, Deserialize, Clone)]
struct RootTile {
  image_id: u32,
  passable: bool,
}

impl Default for RootTile {
  fn default() -> Self {
    RootTile { 
      image_id: 0, 
      passable: true,
    }
  }
}
