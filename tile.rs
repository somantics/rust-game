use ::phf::{phf_map, Map};
use serde::{Deserialize, Serialize};

pub static TILE_REGISTRY: Map<u32, &'static RootTile> = phf_map!(
  0u32 => &RootTile {image_id: 0, passable: true},
  1u32 => &RootTile {image_id: 1, passable: true},
  2u32 => &RootTile {image_id: 2, passable: false},
  3u32 => &RootTile {image_id: 5, passable: true},
  4u32 => &RootTile {image_id: 4, passable: false},
  5u32 => &RootTile {image_id: 6, passable: false},
);

pub const FLOOR_TILE_ID: TileID = TileID { index: 0 };
pub const WALL_TILE_ID: TileID = TileID { index: 2 };
pub const PATH_TEST_TILE: TileID = TileID { index: 3 };
pub const TILE_NOT_FOUND: TileID = TileID { index: 4 };

// GameTile gathers IDs for assets and objects to be referenced in the game map.
// Does not reference units or creatures.
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
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
        if !tile.passable {
            return false;
        };

        true
    }

    fn get_root_tile(&self) -> Option<&RootTile> {
        TILE_REGISTRY.get(&self.root_tile.index).copied()
    }
}

// Represent floors, walls, pillars. Features part of a tile that is drawn first and is non-interactable.
#[derive(Serialize, Deserialize, Clone)]
pub struct RootTile {
    pub image_id: u32,
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

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct TileID {
    pub index: u32,
}

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct ObjectID {
    index: u32,
}
