use std::{cell::RefCell, collections::HashMap, mem::discriminant, rc::Rc, vec};

use crate::{
    combat::{Attack, Combat, Health},
    component::{ComponentType as Component, Diffable, Interact, Inventory, TurnTaker},
    map::{Coordinate, GameMap},
    system::{MonsterTurns, System, UnitCull},
};

pub type Entity = Vec<Component>;
pub type Match<'a> = Vec<&'a Component>;
pub type Diff = (usize, Vec<Component>);

pub struct GameState {
    current_level: GameMap,
    current_entities: HashMap<usize, Entity>,
    turn_systems: Vec<Box<dyn System>>,
}

impl GameState {
    // TOP LEVEL COMMANDS
    pub fn create_new(level: GameMap, start: Coordinate) -> GameState {
        let mut game = GameState {
            current_level: level,
            current_entities: HashMap::<usize, Entity>::default(),
            turn_systems: Vec::<Box<dyn System>>::default(),
        };
        game.turn_systems.push(Box::new(UnitCull::default()));
        game.turn_systems.push(Box::new(MonsterTurns::default()));

        let player_combat = Rc::new(RefCell::new(Combat::new(
            Health {
                current: 10,
                max: 10,
            },
            Attack { damage: 3 },
        )));
        let player_inventory = Rc::new(RefCell::new(Inventory {
            coins: 0,
            response: None,
        }));

        game.create_unit(
            0,
            vec![
                Component::Player,
                Component::Image(3),
                Component::Position(start),
                Component::Combat(player_combat.clone()),
                Component::Inventory(player_inventory),
                Component::Bump(player_combat),
                Component::Collision(true),
            ],
        );

        let dog_combat = Rc::new(RefCell::new(Combat::new(
            Health { current: 5, max: 5 },
            Attack { damage: 3 },
        )));
        game.create_unit(
            1,
            vec![
                Component::Image(6),
                Component::Position(Coordinate { x: 7, y: 7 }),
                Component::Combat(dog_combat.clone()),
                Component::Bump(dog_combat),
                Component::Turn(TurnTaker::new()),
                Component::Collision(true),
            ],
        );

        let dog_combat = Rc::new(RefCell::new(Combat::new(
            Health { current: 5, max: 5 },
            Attack { damage: 3 },
        )));
        game.create_unit(
            15,
            vec![
                Component::Image(6),
                Component::Position(Coordinate { x: 12, y: 9 }),
                Component::Combat(dog_combat.clone()),
                Component::Bump(dog_combat),
                Component::Turn(TurnTaker::new()),
                Component::Collision(true),
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
                Component::Collision(true),
            ],
        );
        let interaction = Rc::new(RefCell::new(Interact::new_door()));
        game.create_unit(
            4,
            vec![
                Component::Image(9),
                Component::Position(Coordinate { x: 5, y: 5 }),
                Component::Bump(interaction),
                Component::Collision(true),
            ],
        );
        game
    }

    pub fn create_unit(&mut self, id: usize, components: Entity) {
        self.current_entities.insert(id, components);
    }

    pub fn step_command(&mut self, direction: Coordinate) {
        let (player_id, player_comps) = self.get_player();
        if let Some(position) = get_position_from(player_comps.clone()) {
            let destination = Coordinate {
                x: position.x + direction.x,
                y: position.y + direction.y,
            };
            match self.get_tile_occupant_id(destination) {
                Some(target_id) => {
                    if self.get_passable_of(target_id) {
                        self.move_player(direction)
                    } else {
                        let diffs = self.propagate_bump_to(player_id, player_comps, target_id);
                        self.apply_diffs(diffs);
                    }
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

    pub fn get_passable_of(&self, id: usize) -> bool {
        let entity = self.get_entity_of(id).expect("Entity with ID does not exist.");
        let collision = get_component_from(Component::Collision(true), entity);
        match collision {
            Some(Component::Collision(true)) => false,
            Some(Component::Collision(false)) => true,
            _ => true
        }
    }

    pub fn get_tile_occupant_id(&self, coord: Coordinate) -> Option<usize> {
        self.get_matches_with(&vec![Component::Position(Coordinate::default())])
            .iter()
            .find(|(_, data)| {
                if let Some(pos) = get_position_from(data.to_owned()) {
                    pos == coord
                } else {
                    false
                }
            })
            .map(|elem| match elem {
                (index, _) => *index,
            })
    }

    pub fn get_player(&self) -> (usize, Match) {
        let reqs = vec![Component::Player];
        let players = self.get_matches_with(&reqs);
        if let Some((player_id, _)) = players.get(0) {
            // Getting full player entity
            if let Some(components) = self.get_entity_of(*player_id) {
                (*player_id, components)
            } else {
                panic!("Incorrect player ID.");
            }
        } else {
            panic!("No player found.");
        }
    }

    pub fn get_player_position(&self) -> Option<Coordinate> {
        let (_, player) = self.get_player();
        get_position_from(player)
    }

    pub fn get_position_of(&self, id: usize) -> Option<Coordinate> {
        if let Some(entity) = GameState::get_entity_of(self, id) {
            get_position_from(entity)
        } else {
            None
        }
    }

    pub fn get_combat_of(&self, id: usize) -> Option<Rc<RefCell<Combat>>> {
        if let Some(entity) = GameState::get_entity_of(self, id) {
            get_combat_from(entity)
        } else {
            None
        }
    }

    pub fn get_image_of(&self, id: usize) -> Option<i32> {
        if let Some(entity) = GameState::get_entity_of(self, id) {
            get_image_from(entity)
        } else {
            None
        }
    }

    pub fn get_entity_of<'a>(&'a self, id: usize) -> Option<Match> {
        if let Some(entity) = self.current_entities.get(&id) {
            let comp_references: Vec<&Component> = entity.iter().collect();
            Some(comp_references)
        } else {
            None
        }
    }

    pub fn get_component_of<'a>(
        &'a self,
        comp_type: Component,
        id: usize,
    ) -> Option<&'a Component> {
        if let Some(entity) = GameState::get_entity_of(self, id) {
            get_component_from(comp_type, entity)
        } else {
            None
        }
    }

    pub fn get_components_of<'a>(
        &'a self,
        requirements: &Vec<Component>,
        id: usize,
    ) -> Option<Match<'a>> {
        if let Some(entity) = self.get_entity_of(id) {
            let component_matches: Vec<&Component> = entity
                .iter()
                .filter(|&&elem| {
                    requirements
                        .to_owned()
                        .iter()
                        .any(|req| discriminant(elem) == discriminant(req))
                })
                .map(|&elem| elem)
                .collect();

            if has_components(component_matches.clone(), requirements) {
                Some(component_matches)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn propagate_bump_to(&self, source_id: usize,  source: Match, id: usize) -> Vec<Diff> {
        if let Some(entity) = self.get_entity_of(id) {
            let comp_type = Component::Bump(Rc::new(RefCell::new(Combat::default())));
            if let Some(Component::Bump(data)) = get_component_from(comp_type, entity.clone()) {
                data.borrow().process(id, source_id, source)
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    // INTERNAL ONLY

    fn add_entity_images(&self, image_data: &mut Vec<Vec<i32>>) {
        let requirements = vec![
            Component::Image(i32::default()),
            Component::Position(Coordinate::default()),
        ];
        let image_entities = self.get_matches_with(&requirements);

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
        let (player_id, player) = self.get_player();
        let position = Component::Position(Coordinate::default());
        if let Some(Component::Position(_)) = get_component_from(position, player) {
            let diffs = vec![Component::Position(direction)];
            self.apply_diffs(vec![(player_id, diffs)]);
        }
    }

    fn run_system(&mut self, system: &Box<dyn System>) -> Vec<Diff> {
        let requirements = system.get_component_requirements();
        let entities = self.get_matches_with(&requirements);
        system.run(self, entities)
    }

    fn apply_diffs(&mut self, diffs: Vec<Diff>) {
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

    fn get_matches_with(&self, components: &Vec<Component>) -> Vec<(usize, Match)> {
        let matched_entities: Vec<(usize, Match)> = self
            .current_entities
            .iter()
            .filter_map(|(&index, data)| {
                get_components_from(data, components).map(|data| (index, data))
            })
            .collect();

        matched_entities
    }
}

// DATA OPERATIONS ON ENTITIES

pub fn get_component_from<'a>(comp_type: Component, entity: Match<'a>) -> Option<&'a Component> {
    entity
        .iter()
        .find(|&&component| discriminant(component) == discriminant(&comp_type))
        .map(|&val| val)
}

pub fn get_components_from<'a>(
    entity: &'a Entity,
    requirements: &Vec<Component>,
) -> Option<Match<'a>> {
    let component_matches: Vec<&Component> = entity
        .iter()
        .filter(|&elem| {
            requirements
                .to_owned()
                .iter()
                .any(|req| discriminant(elem) == discriminant(req))
        })
        .map(|elem| elem)
        .collect();

    if has_components(component_matches.clone(), requirements) {
        Some(component_matches)
    } else {
        None
    }
}

pub fn get_position_from(entity: Match) -> Option<Coordinate> {
    let position = get_component_from(Component::Position(Coordinate::default()), entity);
    match position {
        Some(Component::Position(coord)) => Some(*coord),
        _ => None,
    }
}

pub fn get_image_from(entity: Match) -> Option<i32> {
    let requirement = Component::Image(i32::default());
    if let Some(Component::Image(image_id)) = get_component_from(requirement, entity) {
        Some(*image_id)
    } else {
        None
    }
}

pub fn get_combat_from(entity: Match) -> Option<Rc<RefCell<Combat>>> {
    let requirement = Component::Combat(Rc::new(RefCell::new(Combat::default())));
    if let Some(Component::Combat(data)) = get_component_from(requirement, entity) {
        Some(data.clone())
    } else {
        None
    }
}

pub fn has_components(entity: Match, requirements: &Entity) -> bool {
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
