use std::mem::discriminant;
use std::{collections::HashMap, vec};

use crate::component::{Diffable, System, TestSystem};
use crate::{
    component::ComponentType as Component,
    map::{Coordinate, GameMap},
};

pub struct GameState {
    current_level: GameMap,
    current_entities: HashMap<usize, Vec<Component>>,
    turn_systems: Vec<Box<dyn System>>,
}

impl GameState {
    pub fn create_new(level: GameMap, start: Coordinate) -> GameState {
        let mut game = GameState {
            current_level: level,
            current_entities: HashMap::<usize, Vec<Component>>::default(),
            turn_systems: Vec::<Box<dyn System>>::default(),
        };
        game.create_unit(
            0,
            vec![
                Component::Player,
                Component::Image(3),
                Component::Position(start),
            ],
        );
        game.create_unit(
            1,
            vec![
                Component::Image(6),
                Component::Position(Coordinate { x: 7, y: 7 }),
            ],
        );

        game.create_unit(
            15,
            vec![
                Component::Image(6),
                Component::Position(Coordinate { x: 12, y: 9 }),
            ],
        );
        game
    }

    pub fn get_image_ids_for_map(&self) -> Vec<Vec<i32>> {
        // Collects everything on the map that needs to be drawn.
        let mut tile_images = self.current_level.get_tile_image_ids();
        self.add_entity_images(&mut tile_images);
        tile_images
    }

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

    fn create_unit(&mut self, id: usize, components: Vec<Component>) {
        self.current_entities.insert(id, components);
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

    fn run_system(&mut self, system: &Box<dyn System>) -> Vec<(usize, Vec<Component>)> {
        let requirements = system.get_component_requirements();
        let matches = self.get_entities_with(requirements);
        system.run(matches)
    }

    fn apply_diffs(&mut self, diffs: Vec<(usize, Vec<Component>)>) {
        for (index, changes) in diffs {
            if let Some(mut data) = self.current_entities.get_mut(&index) {
                for mut component in data {
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

        if GameState::has_components(&matches, requirements) {
            Some((index, matches))
        } else {
            None
        }
    }

    fn has_components(entity: &Vec<&Component>, requirements: &Vec<Component>) -> bool {
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
}
