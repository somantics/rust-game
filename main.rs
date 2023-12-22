use std::io::prelude::*;
use std::path::Path;
use std::fs::File;

use map::GameMapSerializable;
use serde_json::Result;
use serde_json;

use crate::{map::{GameMap, Coordinate, GameUnit}, gamestate::GameState};

mod map;
mod gamestate;

slint::include_modules!();

fn main() { 

  const GRID_WIDTH: u32 = 16 * 2 ;
  const GRID_HEIGHT: u32 = 9 * 2 ;
  const TILESET_SIZE: f32 = 32.0;
  const STARTING_POSITION: Coordinate = Coordinate(5,6);

  // let map_path = Path::new("manualmap.json");
  // let level_map = load_map(&map_path);
  let level_map = GameMap::create_map(GRID_WIDTH, GRID_HEIGHT);

  let game_state = GameState::create_new(level_map, STARTING_POSITION);

  let main_window = MainWindow::new().unwrap();

  main_window.set_tile_size(TILESET_SIZE);
  main_window.set_grid_width(GRID_WIDTH as i32);
  main_window.set_grid_height(GRID_HEIGHT as i32);

  let tiles: Vec<TileGraphics> = game_state.get_image_ids_for_map()
  .into_iter()
  .map (|vec| {
    std::rc::Rc::new(slint::VecModel::from(vec))
  })
  .map(|vec_model| TileGraphics{image_ids: vec_model.into()})
  .collect();

  let tiles = std::rc::Rc::new(slint::VecModel::from(tiles));

  main_window.set_memory_tiles(tiles.into());

  main_window.on_tile_clicked(move |x, y| {});
  main_window.run().unwrap();

  //save_map(&level_map);
  
}



fn save_map(map: &GameMap) {
  let json_map = serde_json::to_string_pretty(&map.to_serializable());
  let json_map = match json_map {
    Ok(json) => json,
    Err(reason) => panic!("Could not be serialized: {}", reason),
  };

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

fn load_map(path: &Path) -> GameMap {
  let path_to_map = Path::new(path);
  let path_name = path_to_map.display();

  let save_file = match File::open(&path_to_map) {
    Ok(file) => file,
    Err(reason) => panic!("Could not open {}: {}", path_name, reason),
  };

  let deserialized: Result<GameMapSerializable> = serde_json::from_reader(save_file);
  let deserialized = match deserialized {
    Ok(map) => map,
    Err(reason) => panic!("Error parsing map from file {}: {}", path_name, reason),
  };
  
  GameMap::from_serializable(deserialized)
}
