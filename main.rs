use std::collections::HashMap;
use rand::distributions::Uniform;
use rand::Rng;

slint::slint! {
  struct TileGraphics {
    floor_tile: int,
  }

  component MapTile inherits Rectangle {
    in property <image> icon;
    in property <length> size;
    width: size;
    height: size;
    background: pink;

    Image {
        source: icon;
        width: parent.width;
        height: parent.height;
    }
  }

  export component MainWindow inherits Window {
    width: 1280px;
    height: 720px;

    in property <length> tile_size;
    in property <int> grid_width;
    in property <int> grid_height;

    in property <[TileGraphics]> memory_tiles;

    property <[image]> images_by_index: [
      @image-url("icons/tile008.png"),
      @image-url("icons/tile011.png"),
      @image-url("icons/tile017.png"),
    ];

    for tile[i] in memory_tiles : MapTile {
      x: mod(i, grid_width) * tile_size;
      y: floor(i / grid_width) * tile_size;

      icon: images_by_index[tile.floor_tile];
      size: tile_size;
    }
  }
}

fn main() { 

  const GRID_WIDTH: u32 = 16 * 2 ;
  const GRID_HEIGHT: u32 = 9 * 2 ;
  const TILESET_SIZE: f32 = 32.0;

  let level_map = GameMap::create_map(GRID_WIDTH, GRID_HEIGHT);

  let main_window = MainWindow::new().unwrap();

  main_window.set_tile_size(TILESET_SIZE);
  main_window.set_grid_width(GRID_WIDTH as i32);
  main_window.set_grid_height(GRID_HEIGHT as i32);

  let tiles: Vec<TileGraphics> = level_map
    .get_tile_graphics()
    .into_iter()
    .map(|id| TileGraphics {floor_tile: id as i32})
    .collect();

  let tiles = std::rc::Rc::new(slint::VecModel::from(tiles));

  main_window.set_memory_tiles(tiles.into());
  main_window.run().unwrap();

}

struct GameMap {
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
}

#[derive(Hash, PartialEq, Eq)]
struct Coordinate(u32, u32);

#[derive(Default)]
struct GameTile {
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