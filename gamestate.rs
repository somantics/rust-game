use std::cell::RefCell;
use std::mem::discriminant;
use std::rc::Rc;
use std::{collections::HashMap, vec};

use crate::combat::{
    Attack, AttackReport, Combat, Health
};
use crate::component::{Diffable, Inventory};
use crate::system::{get_component_from_entity, System, UnitCull};
use crate::{
    component::ComponentType as Component,
    map::{Coordinate, GameMap},
};

// Top level commands
// move player
// run turn systems
// new, create, load, save
// get image ids
// create unit?

// for systems
// is tile empty
// get bump stim
// propagate bump stim
// get entities with
// get components
// get player
// get entity position
// get tile occupant
// get entity

//internal

pub struct GameState {
    current_level: GameMap,
    current_entities: HashMap<usize, Vec<Component>>,
    turn_systems: Vec<Box<dyn System>>,
}

impl GameState {
    // TOP LEVEL COMMANDS
    pub fn create_new(level: GameMap, start: Coordinate) -> GameState {
        let mut game = GameState {
            current_level: level,
            current_entities: HashMap::<usize, Vec<Component>>::default(),
            turn_systems: Vec::<Box<dyn System>>::default(),
        };
        game.turn_systems.push(Box::new(UnitCull::default()));

        let player_combat = Rc::new(RefCell::new(Combat::new(
            Health { current: 10, max: 10 },
            Attack { damage: 3 },)));
        let player_inventory = Rc::new(RefCell::new(Inventory { coins: 0, response: None }));
        
        game.create_unit(
            0,
            vec![
                Component::Player,
                Component::Image(3),
                Component::Position(start),
                Component::Combat(player_combat.clone()),
                Component::Inventory(player_inventory),
                Component::Bump(player_combat),
            ],
        );

        
        let dog_combat = Rc::new(RefCell::new(Combat::new(
            Health { current: 5, max: 5 },
            Attack { damage: 3 },)));
        game.create_unit(
            1,
            vec![
                Component::Image(6),
                Component::Position(Coordinate { x: 7, y: 7 }),
                Component::Combat(dog_combat.clone()),
                Component::Bump(dog_combat),
            ],
        );

        let dog_combat = Rc::new(RefCell::new(Combat::new(
            Health { current: 5, max: 5 },
            Attack { damage: 3 },)));
        game.create_unit(
            15,
            vec![
                Component::Image(6),
                Component::Position(Coordinate { x: 12, y: 9 }),
                Component::Combat(dog_combat.clone()),
                Component::Bump(dog_combat),
            ],
        );

        let chest_inventory = Rc::new(RefCell::new(Inventory::new(5)));
        game.create_unit(
            3, 
            vec![
                Component::Image(7),
                Component::Position(Coordinate { x: 12, y: 12 }),
                Component::Inventory(chest_inventory.clone()),
                Component::Bump(chest_inventory),
            ]
        );
        game
    }

    pub fn create_unit(&mut self, id: usize, components: Vec<Component>) {
        self.current_entities.insert(id, components);
    }

    pub fn step_command(&mut self, direction: Coordinate) {
        let (player_id, player_comps) = self.get_player();
        if let Some(position) = GameState::get_entity_position(&(player_id, player_comps)) {
            let destination = Coordinate {
                x: position.x + direction.x,
                y: position.y + direction.y,
            };
            match self.get_tile_occupant(destination) {
                Some(id) => {
                    let diffs = self.propagate_bump_to(id);
                    self.apply_diffs(diffs);
                }
                None => {
                    if self.is_tile_passable(destination) {
                        self.move_player(direction)
                    }
                }
            }
        } else {
            println!("Player position not found.");
        }
    }

    pub fn get_image_ids_for_map(&self) -> Vec<Vec<i32>> {
        // Collects everything on the map that needs to be drawn.
        let mut tile_images = self.current_level.get_tile_image_ids();
        self.add_entity_images(&mut tile_images);
        tile_images
    }

    pub fn run_turn_systems(&mut self) {
        let system_list =
            std::mem::replace(&mut self.turn_systems, Vec::<Box<dyn System>>::default());

        for system in system_list.iter() {
            let diffs = self.run_system(&system);
            self.apply_diffs(diffs);
        }
        self.turn_systems = system_list;
    }

    // PUBLIC FOR SYSTEMS

    pub fn is_tile_passable(&self, coord: Coordinate) -> bool {
        self.current_level.is_tile_passable(coord)
    }

    pub fn get_tile_occupant(&self, coord: Coordinate) -> Option<usize> {
        self.get_entities_with(&vec![Component::Position(Coordinate::default())])
            .iter()
            .find(|&elem| {
                if let Some(pos) = GameState::get_entity_position(elem) {
                    pos == coord
                } else {
                    false
                }
            })
            .map(|elem| match elem {
                (index, _) => *index,
            })
    }

    pub fn get_player(&self) -> (usize, Vec<&Component>) {
        let reqs = vec![Component::Player];
        let players = self.get_entities_with(&reqs);
        if let Some((player_index, _)) = players.get(0) {
            self.get_entity(*player_index)
                .expect("Incorrect player ID.")
        } else {
            panic!("No player found.");
        }
    }

    pub fn get_entity_position(entity: &(usize, Vec<&Component>)) -> Option<Coordinate> {
        let position =
            get_component_from_entity(Component::Position(Coordinate::default()), entity);
        match position {
            Some(Component::Position(coord)) => Some(*coord),
            _ => None,
        }
    }

    pub fn get_entity<'a>(&'a self, id: usize) -> Option<(usize, Vec<&'a Component>)> {
        if let Some(entity) = self.current_entities.get(&id) {
            let comp_references: Vec<&Component> = entity.iter().collect();
            Some((id, comp_references))
        } else {
            None
        }
    }

    pub fn get_bump_stim(&self, id: usize) -> Option<AttackReport> {
        if let Some(entity) = self.get_entity(id) {
            let comp_type = Component::Combat(Rc::new(RefCell::new(Combat::default())));
            if let Some(Component::Combat(data)) = get_component_from_entity(comp_type, &entity) {
                Some((data.borrow().bump_stim)(data.clone()))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn propagate_bump_to(
        &self,
        id: usize,
    ) -> Vec<(usize, Vec<Component>)> {
        let diff = 
        if let Some((_, entity)) = self.get_entity(id) {
            let comp_type = Component::Bump(Rc::new(RefCell::new(Combat::default())));
            if let Some(Component::Bump(data)) = get_component_from_entity(comp_type, &(id, entity.clone())) {
                Some(data.borrow().process(entity))
            } else {
                None
            }
        } else {
            None
        };

        if let Some(components) = diff {
            vec![(id, components)]
        } else {
            vec![]
        }
    }

    pub fn has_components(entity: Vec<&Component>, requirements: &Vec<Component>) -> bool {
        let mut requirements = requirements.to_owned();

        for component in entity {
            // do we fullfill a requirement, if so where?
            let index = requirements
                .iter()
                .position(|requirement| discriminant(requirement) == discriminant(component));

            // Requirement is fulfilled, no need to check any further
            if let Some(i) = index {
                requirements.swap_remove(i);
            }

            // No more unfulfilled requirements
            if requirements.is_empty() {
                break;
            }
        }

        requirements.is_empty()
    }

    // INTERNAL ONLY

    fn add_entity_images(&self, image_data: &mut Vec<Vec<i32>>) {
        let requirements = vec![
            Component::Image(i32::default()),
            Component::Position(Coordinate::default()),
        ];
        let image_entities = self.get_entities_with(&requirements);

        for (_, data) in image_entities {
            match data.as_slice() {
                [Component::Image(im_id), Component::Position(coord)] => {
                    let flat_position = self.current_level.coordinate_to_index(*coord);
                    image_data[flat_position].push(*im_id);
                }
                _ => {}
            }
        }
    }

    fn move_player(&mut self, direction: Coordinate) {
        // TODO: reseve a unique ID for player so this becomes lookup
        let player = self.get_player();
        let position = Component::Position(Coordinate::default());
        if let Some(Component::Position(_)) = get_component_from_entity(position, &player) {
            let diffs = vec![Component::Position(direction)];
            self.apply_diffs(vec![(player.0, diffs)]);
        }
    }

    fn run_system(&mut self, system: &Box<dyn System>) -> Vec<(usize, Vec<Component>)> {
        let requirements = system.get_component_requirements();
        let matches = self.get_entities_with(&requirements);
        system.run(matches)
    }

    fn apply_diffs(&mut self, diffs: Vec<(usize, Vec<Component>)>) {
        for (index, changes) in diffs {
            if let Some(data) = self.current_entities.get_mut(&index) {
                let marked_for_dead = changes
                    .iter()
                    .find(|&change| discriminant(&Component::Delete) == discriminant(change));

                if let Some(_) = marked_for_dead {
                    self.current_entities.remove(&index);
                    continue;
                }

                for component in data {
                    let diff = changes
                        .iter()
                        .find(|&change| discriminant(component) == discriminant(change));

                    match diff {
                        Some(change) => component.apply_diff(change),

                        None => {}
                    };
                }
            }
        }
    }

    fn get_entities_with(&self, components: &Vec<Component>) -> Vec<(usize, Vec<&Component>)> {
        let matched_entities: Vec<(usize, Vec<&Component>)> = self
            .current_entities
            .iter()
            .filter_map(|(&index, data)| GameState::get_components((index, data), components))
            .collect();

        matched_entities
    }

    fn get_components<'a>(
        entity: (usize, &'a Vec<Component>),
        requirements: &Vec<Component>,
    ) -> Option<(usize, Vec<&'a Component>)> {
        let (index, components) = entity;
        let matches: Vec<&Component> = components
            .iter()
            .filter(|&elem| {
                requirements
                    .to_owned()
                    .iter()
                    .any(|req| discriminant(elem) == discriminant(req))
            })
            .map(|elem| elem)
            .collect();

        if GameState::has_components(matches.clone(), requirements) {
            Some((index, matches))
        } else {
            None
        }
    }

}
