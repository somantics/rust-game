use ::phf::{phf_map, Map};
use serde::{Deserialize, Serialize};

use crate::game::components::core::ImageData;

pub static TILE_REGISTRY: Map<u32, &'static RootTile> = phf_map!(
  0u32 => &RootTile {image: ImageData { id: 0, depth: 10 }, passable: true, los_blocking: false},
  1u32 => &RootTile {image: ImageData { id: 1, depth: 10 }, passable: true, los_blocking: false},
  2u32 => &RootTile {image: ImageData { id: 2, depth: 10 }, passable: false, los_blocking: true},
  3u32 => &RootTile {image: ImageData { id: 5, depth: 10 }, passable: true, los_blocking: false},
  4u32 => &RootTile {image: ImageData { id: 4, depth: 10 }, passable: false, los_blocking: true},
  5u32 => &RootTile {image: ImageData { id: 6, depth: 10 }, passable: false, los_blocking: true},
);

pub const FLOOR_TILE_ID: TileID = TileID { index: 0 };
pub const WALL_TILE_ID: TileID = TileID { index: 2 };
pub const PATH_TEST_TILE: TileID = TileID { index: 3 };
pub const TILE_NOT_FOUND: TileID = TileID { index: 4 };

// slated for removal
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct GameTile {
    pub root_tile: TileID,
}

impl GameTile {
    pub fn get_image(&self) -> Vec<i32> {
        let tile = self.get_root_tile();

        match tile {
            Some(root) => vec![root.image.id, root.image.depth],
            None => {
                let default = ImageData::default();
                vec![default.id, default.depth]
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        let tile = self.get_root_tile().expect("No root tile found.");
        if !tile.passable {
            return false;
        };

        true
    }

    pub fn is_los_blocking(&self) -> bool {
        let tile = self.get_root_tile().expect("No root tile found.");
        if tile.los_blocking {
            return true;
        };

        false
    }

    fn get_root_tile(&self) -> Option<&RootTile> {
        TILE_REGISTRY.get(&self.root_tile.index).copied()
    }
}

// Represent floors, walls, pillars. Features part of a tile that is drawn first and is non-interactable.
#[derive(Clone)]
pub struct RootTile {
    pub image: ImageData,
    passable: bool,
    los_blocking: bool,
}

impl Default for RootTile {
    fn default() -> Self {
        RootTile {
            image: ImageData::default(),
            passable: true,
            los_blocking: false,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct TileID {
    pub index: u32,
}
