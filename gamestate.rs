
use crate::map::{GameMap, Coordinate, GameUnit};


pub struct GameState {
  current_level: GameMap,
  creature_list: Vec<(GameUnit, Coordinate)>,
  player_unit: GameUnit,
  player_position: Coordinate,
}

impl GameState {

  pub fn create_new(level: GameMap, start: Coordinate) -> GameState {
    GameState {
      current_level: level,
      creature_list: Vec::<(GameUnit, Coordinate)>::new(),
      player_unit: GameUnit::default(),
      player_position: start,
    }
  }

  pub fn get_image_ids_for_map(&self) -> Vec<Vec<i32>> {
    let mut tile_images = self.current_level.get_tile_image_ids();
    self.add_units_to_draw(&mut tile_images);
    tile_images
  }

  fn attempt_move_to(&mut self, x: i32, y: i32) {
    let coord = Coordinate(x as u32,y as u32);
    println!("{:?}", coord);

    if self.current_level.is_tile_empty(coord) && self.no_creature_at(coord)
    {
      self.player_position = coord;
    } else {
      // tell the player they can't
    }
  }

  fn no_creature_at(&self, coord: Coordinate) -> bool {
    let overlapping_creatures: Vec<&(GameUnit, Coordinate)> = self.creature_list
      .iter()
      .filter(|(_, pos)| coord == *pos)
      .collect();

    overlapping_creatures.is_empty()
  }

  fn add_units_to_draw(&self, tile_image_ids: &mut Vec<Vec<i32>>)  {
    // convert coordinate to location index
    let creatures_by_index = self.creature_list
      .iter()
      .map(|(unit, pos)| {
        let index = (pos.0 * self.current_level.width + pos.1) as usize;
        (unit, index)
      });
    
    // add creatures using index
    for (unit, index) in creatures_by_index {
      let image_id = unit.image_id as i32;
      tile_image_ids[index].push(image_id);
    };

    // same for player
    let player_pos_index = (self.player_position.0 * self.current_level.width + self.player_position.1) as usize;
    let player_image = self.player_unit.image_id as i32;
    tile_image_ids[player_pos_index].push(player_image);
  }
}