use std::io::prelude::*;
use std::path::Path;
use std::fs::File;

use serde_json;

use crate::map::GameMap;
mod map;

slint::include_modules!();

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

  save_map(&level_map);
  
}

fn save_map(map: &GameMap) {
  let json_map = serialize_map(&map);

  let path_to_map = Path::new("levelmap.json");
  let path_name = path_to_map.display();

  let mut save_file = match File::create(&path_to_map) {
    Ok(file) => file,
    Err(reason) => panic!("Could not open {}: {}", path_name, reason),
  };

  match save_file.write_all(json_map.as_bytes()) {
    Ok(_) => println!("Successfully saved file {}", path_name),
    Err(reason) => panic!("Could not write to file {}: {}", path_name, reason),
  }
}

fn serialize_map(map: &GameMap) -> String {
  let serializable_map = map.get_serializable();
  match serde_json::to_string_pretty(&serializable_map) {
    Ok(string) => string,
    Err(error) => {println!("Failed to serialize map: {}", error); String::new()} 
  }
}

fn load_map(path: &Path) {

}

fn deserialize_map(map_data: &str)  {
  
}