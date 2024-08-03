use std::io::prelude::*;
use std::path::Path;
use std::fs::File;

use game::Game;
use serde_json;
use serde_json::Result;

use map::{Coordinate, GameMap, GameMapSerializable};
use logger::MessageLog;
use logger::LOG;

mod ecs;
mod event;
mod game;
mod los;
mod map;
mod logger;

slint::include_modules!();

const GRID_WIDTH: usize = (16.0 * 2.0) as usize;
const GRID_HEIGHT: usize = (9.0 * 2.0) as usize;
const TILESET_SIZE: f32 = 32.0;


fn main() {
    let game = Game::new(GRID_WIDTH, GRID_HEIGHT);

    let main_window = initialize_main_window();
    update_game_info(&game, &main_window);
    update_tile_map(&game, &main_window);
    set_up_input(game, &main_window);
    main_window.run().unwrap();
}

fn initialize_main_window() -> MainWindow {
    let window = MainWindow::new().unwrap();
    window.set_tile_size(TILESET_SIZE);
    window.set_grid_width(GRID_WIDTH as i32);
    window.set_grid_height(GRID_HEIGHT as i32);
    window
}

fn set_up_input(mut game: Game, window: &MainWindow) {
    let weak_window = window.as_weak();
    window.on_received_input(move |command, x, y| {
        // Main game loop
        match command {
            InputCommand::Direction => {
                game.step_command(Coordinate { x, y });
            }
            InputCommand::Position => {
                game.target_command(Coordinate { x, y });
            }
            InputCommand::Shoot => {
                game.shoot_command(Coordinate { x, y });
            }
            InputCommand::Descend => {
                game.descend_command();
            }
            InputCommand::Wait => {
                game.wait_command();
            }
            InputCommand::LevelUp => {
                let (stat, amount) = (x, y);
                game.level_up_command(stat, amount);
            }
            InputCommand::Quit => {
                close_window(&weak_window.unwrap());
            }
            _ => {}
        }
        update_game_info(&game, &weak_window.unwrap());
        LOG.with(|log| 
            display_messages(&log, &weak_window.unwrap())
        );
        display_popup(&game, &weak_window.unwrap());
        update_tile_map(&game, &weak_window.unwrap());
    });
}

fn display_popup(game: &Game, window: &MainWindow) {
    if !game.is_player_alive() {
        window.invoke_display_death_popup();
    }
    if game.is_player_ready_for_level() {
        window.invoke_display_level_up_popup();
    }
}

fn display_messages(message_log: &MessageLog, window: &MainWindow) {
    while let Some(msg) = message_log.next_message() {
        window.invoke_display_message(msg.into());
    }
}

fn close_window(window: &MainWindow) {
    window.window().hide().unwrap();
}

fn update_game_info(game: &Game, window: &MainWindow) {
    let (
        name, 
        level, 
        coins, 
        xp_current, 
        xp_goal, 
        hp_curent, 
        hp_max, 
        strength, 
        dexterity, 
        cunning,
        melee_damage,
        melee_crit,
        ranged_damage,
        ranged_crit,
    ) = game.get_player_info();

    let depth = game.get_map_info();

    window.set_depth(depth);
    window.set_character_name(name.into());
    window.set_player_level(level);
    window.set_player_coins(coins);
    window.set_player_xp_current(xp_current);
    window.set_player_xp_goal(xp_goal);
    window.set_player_health_current(hp_curent);
    window.set_player_health_max(hp_max);
    window.set_player_strength(strength);
    window.set_player_dexterity(dexterity);
    window.set_player_cunning(cunning);
    window.set_player_melee_damage(melee_damage.into());
    window.set_player_melee_crit(melee_crit);
    window.set_player_ranged_damage(ranged_damage.into());
    window.set_player_ranged_crit(ranged_crit);
}

fn update_tile_map(game: &Game, window: &MainWindow) {
    // Updates frontend's internal data for tiles, which triggers redraw.
    let tiles: Vec<TileGraphics> = game
        .get_image_ids_for_map()
        .into_iter()
        .map(|vec| std::rc::Rc::new(slint::VecModel::from(vec)))
        .map(|vec_model| TileGraphics {
            image_ids: vec_model.into(),
        })
        .collect();

    let tiles = std::rc::Rc::new(slint::VecModel::from(tiles));

    window.set_memory_tiles(tiles.into());
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

