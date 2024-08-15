use std::cmp::Reverse;

use crate::ecs::component::attributes::Attributes;
use crate::ecs::component::combat::Health;
use crate::ecs::component::{combat, Component, ComponentType};
use crate::ecs::system::{
    ComponentQuery, Exploration, MonsterTurns, PlayerCheck, SystemManager, UnitCull,
};
use crate::ecs::{component, take_component_from_refs, Delta, IndexedData, ECS};
use crate::event::{propagate_event, InteractionEvent};
use crate::map::mapbuilder::MapBuilder;
use crate::map::{Coordinate, GameMap};
use crate::MessageLog;
use crate::{logger, los};

/* TODOS:
   performance
       make it so only "awake" enemies take their turn

*/

pub struct Game {
    pub ecs: ECS,
    pub systems: SystemManager,
    pub map: GameMap,
    pub log: MessageLog,
}

impl Game {
    pub fn new(size_x: usize, size_y: usize) -> Game {
        let mut game = Game {
            ecs: ECS::new(),
            systems: SystemManager::new(),
            map: MapBuilder::generate_new(size_x, size_y, 1),
            log: MessageLog::new(),
        };

        game.ecs.spawn_all_entities(&game.map);
        game.add_default_systems();
        game.explore_first_room();
        game
    }

    pub fn wait_command(&mut self) {
        self.end_turn();
    }

    pub fn shoot_command(&mut self, coord: Coordinate) {
        let player_report = match self.ecs.get_player_report() {
            Some(report) => report,
            _ => return,
        };

        let maybe_target = self.ecs.get_blocking_entity(coord);
        let target = match maybe_target {
            Some(entity) => entity,
            None => return,
        };
        let event = player_report.shoot;
        let distance = coord.distance(player_report.position.data) as usize;
        let range = event.attack.unwrap().range.unwrap();
        let line_of_sight =
            los::line_of_sight(player_report.position.data, coord, &self.map, &self.ecs);

        if !line_of_sight {
            logger::log_message("Target is out of sight.");
            return;
        }
        if distance > range {
            logger::log_message("Target is out of range.");
            return;
        }
        self.propagate_and_apply_event(&event, target);
        self.end_turn();
    }

    pub fn target_command(&mut self, coord: Coordinate) {
        let player_id = self.ecs.get_player_id();
        let components = self.ecs.get_components_from_entity(player_id);

        let (maybe_position, _) = take_component_from_refs(ComponentType::Position, &components);
        let position = match maybe_position {
            Some(Component::Position(data)) => data,
            _ => panic!("Player found without a position component."),
        };

        if coord == position.data {
            self.wait_command();
        } else if coord.distance(position.data) <= 1.05 {
            // clicked adjacent <=> wasd command
            let direction = Coordinate {
                x: coord.x - position.data.x,
                y: coord.y - position.data.y,
            };
            self.step_command(direction);
        }
    }

    pub fn step_command(&mut self, direction: Coordinate) {
        let player_report = match self.ecs.get_player_report() {
            Some(report) => report,
            _ => return,
        };

        let coord = player_report.position.data + direction;
        if !self.map.is_tile_passable(coord) {
            return;
        }

        let event = player_report.bump;
        if let Some(entity_id) = self.ecs.get_blocking_entity(coord) {
            self.propagate_and_apply_event(&event, entity_id);
        } else {
            let entities = self.ecs.get_all_entities_in_tile(coord);
            for entity_id in entities {
                self.propagate_and_apply_event(&event, entity_id);
            }
            self.move_player(direction);
        }
        self.end_turn();
    }

    pub fn close_doors_command(&mut self) {
        let Some(player_position) = self.ecs.get_player_position() else {
            return;
        };

        let adjacent = vec![
            Coordinate { x: 1, y: 0 },
            Coordinate { x: -1, y: 0 },
            Coordinate { x: 0, y: 1 },
            Coordinate { x: 0, y: -1 },
        ];

        let neighbors = adjacent.iter().map(|dir| player_position + *dir);

        let doors: Vec<usize> = neighbors
            .into_iter()
            .map(|pos| self.ecs.get_all_entities_in_tile(pos))
            .flatten()
            .filter(|entity_id| {
                self.ecs
                    .entity_has_component(*entity_id, ComponentType::Door)
            })
            .collect();

        // make the correct event
        let event = &InteractionEvent {
            event_type: crate::event::EventType::Bump,
            attack: None,
            payload: vec![]
        };

        for door in doors {
            let delta = propagate_event(event, door, &self.ecs);
            self.ecs.apply_changes(delta);
        }
        self.end_turn();
    }

    fn make_new_map(&mut self, size_x: usize, size_y: usize, depth: usize) {
        let new_map = MapBuilder::generate_new(size_x, size_y, depth);
        let mut new_ecs = ECS::new();

        let player_id = self.ecs.get_player_id();
        new_ecs.copy_entity_from_other(&self.ecs, player_id);
        new_ecs.spawn_all_entities(&new_map);

        self.ecs = new_ecs;
        self.map = new_map;
        self.update_systems();
        self.explore_first_room();
    }

    fn explore_first_room(&mut self) {
        if let Some(player_position) = self.ecs.get_player_position() {
            self.map.explore_room(player_position);
        }
    }

    pub fn descend_command(&mut self) {
        // check if player is on staircase
        if let Some(player_position) = self.ecs.get_player_position() {
            if self.ecs.position_has_stairs(player_position) {
                self.make_new_map(self.map.width, self.map.height, self.map.depth + 1);
            }
        }
    }

    pub fn level_up_command(&mut self, stat: i32, amount: i32) {
        let id = self.ecs.get_player_id();
        let components = self.ecs.get_components_from_entity(id);

        let (maybe_stats, components) =
            take_component_from_refs(ComponentType::Attributes, &components);
        let (maybe_health, components) =
            take_component_from_refs(ComponentType::Health, &components);
        if let (Some(Component::Attributes(stats)), Some(Component::Health(health))) =
            (maybe_stats, maybe_health)
        {
            let mut stat_change = IndexedData::<Attributes>::default();
            match stat {
                0 => {
                    stat_change = stats.make_change(Attributes {
                        strength: amount as isize,
                        ..Default::default()
                    });
                }
                1 => {
                    stat_change = stats.make_change(Attributes {
                        dexterity: amount as isize,
                        ..Default::default()
                    });
                }
                _ => {}
            }

            let xp_change = stats.make_change(Attributes {
                level: 1,
                xp: -stats.data.xp,
                level_pending: false,
                ..Default::default()
            });

            let health_increase = health.make_change(Health {
                current: (health.data.max as f32 * 0.2) as isize,
                max: (health.data.max as f32 * 0.2) as isize,
            });

            let restore_health = health.make_change(health.data.get_health_reset_diff());
            let change_list = vec![
                Delta::Change(Component::Attributes(stat_change)),
                Delta::Change(Component::Attributes(xp_change)),
                Delta::Change(Component::Health(restore_health)),
                Delta::Change(Component::Health(health_increase)),
            ];
            self.ecs.apply_changes(change_list);
        }
    }

    fn move_player(&mut self, direction: Coordinate) {
        let player_report = match self.ecs.get_player_report() {
            Some(report) => report,
            _ => return,
        };
        let position_change = Component::Position(player_report.position.make_change(direction));
        self.ecs.apply_change(Delta::Change(position_change));
    }

    fn end_turn(&mut self) {
        self.run_turn_systems();
    }

    fn propagate_and_apply_event(&mut self, event: &InteractionEvent, entity_id: usize) {
        let change_list = propagate_event(&event, entity_id, &self.ecs);
        self.ecs.apply_changes(change_list);
    }

    fn add_entity_images(&self, mut images: Vec<Vec<Vec<i32>>>) -> Vec<Vec<i32>> {
        let query = ComponentQuery {
            required: vec![ComponentType::Position, ComponentType::Image],
            optional: vec![],
        };
        let matches = self.ecs.get_entities_matching_query(query);
        for entity in matches {
            let component_list = &self.ecs.get_components_from_entity(entity.index);
            let (maybe_position, components) =
                take_component_from_refs(ComponentType::Position, component_list);
            let (maybe_image, _components) =
                take_component_from_refs(ComponentType::Image, &components);
            if let (Some(Component::Position(position)), Some(Component::Image(image))) =
                (maybe_position, maybe_image)
            {
                if !self.map.explored.borrow().contains(&position.data) {
                    continue;
                }

                let (index, image, depth) = (
                    position.data.y as usize * self.map.width + position.data.x as usize, // TODO: get the correct index calculation
                    image.data.current.id,
                    image.data.current.depth,
                );
                images[index].push(vec![image, depth]);
            }
        }
        Game::sort_image_by_depth(images)
    }

    fn sort_image_by_depth(images: Vec<Vec<Vec<i32>>>) -> Vec<Vec<i32>> {
        images
            .into_iter()
            .map(|mut img_vec| {
                img_vec.sort_by_key(|vec| Reverse(vec[1]));
                img_vec
                    .into_iter()
                    .filter_map(|vec| vec.first().copied())
                    .collect()
            })
            .collect()
    }

    pub fn get_image_ids_for_map(&self) -> Vec<Vec<i32>> {
        let tile_images = self.map.get_tile_image_ids();
        self.add_entity_images(tile_images)
    }

    pub fn get_map_info(&self) -> i32 {
        self.map.depth as i32
    }

    pub fn get_player_info(
        &self,
    ) -> (
        String,   // name
        i32,      // level
        i32,      // coins
        i32,      // current xp
        i32,      // level up xp
        i32,      // current hp
        i32,      // max hp
        i32,      // strength
        i32,      // dexterity
        [i32; 2], // melee damage
        f32,      // melee crit chance
        [i32; 2], // ranged damage
        f32,      // ranged crit chance
    ) {
        let report = match self.ecs.get_player_report() {
            Some(report) => report,
            _ => {
                return (
                    "None".to_string(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    [0, 0],
                    0.0,
                    [0, 0],
                    0.0,
                )
            }
        };

        let name = report.name.unwrap().data;
        let health = report.health.unwrap().data;
        let stats = report.stats.unwrap().data;
        let items = report.items.unwrap().data;

        let mut melee_damage = [0, 0];
        let mut melee_crit = 0.0;
        let mut ranged_damage = [0, 0];
        let mut ranged_crit = 0.0;

        let (melee, ranged) = self.ecs.get_player_attacks();

        if let Some(attack) = melee {
            let bonus_damage = combat::get_bonus_dmg(&stats, &attack);
            melee_damage = [
                (attack.damage_base + bonus_damage.0) as i32,
                (attack.damage_base + bonus_damage.1 + attack.damage_spread) as i32,
            ];
            melee_crit = combat::BASE_CRIT_CHANCE + attack.crit_chance_bonus;
        }

        if let Some(attack) = ranged {
            let bonus_damage = combat::get_bonus_dmg(&stats, &attack);
            ranged_damage = [
                (attack.damage_base + bonus_damage.0) as i32,
                (attack.damage_base + bonus_damage.1 + attack.damage_spread) as i32,
            ];
            ranged_crit = combat::BASE_CRIT_CHANCE + attack.crit_chance_bonus;
        }

        // frontend requires i32:s
        (
            name.raw,
            stats.level as i32,
            items.coins as i32,
            stats.xp as i32,
            component::attributes::get_xp_to_next(&stats) as i32,
            health.current as i32,
            health.max as i32,
            stats.strength as i32,
            stats.dexterity as i32,
            melee_damage,
            melee_crit as f32,
            ranged_damage,
            ranged_crit as f32,
        )
    }

    pub fn is_player_alive(&self) -> bool {
        let components = &self
            .ecs
            .get_components_from_entity(self.ecs.get_player_id());
        let (maybe_health, _components) =
            take_component_from_refs(ComponentType::Health, components);
        let health = match maybe_health {
            Some(Component::Health(data)) => data.data,
            _ => return false,
        };
        health.current >= 0
    }

    pub fn is_player_ready_for_level(&self) -> bool {
        let components = &self
            .ecs
            .get_components_from_entity(self.ecs.get_player_id());
        let (stats, _) = take_component_from_refs(ComponentType::Attributes, components);
        let stats = match stats {
            Some(Component::Attributes(data)) => data.data,
            _ => return false,
        };
        stats.xp >= component::attributes::get_xp_to_next(&stats)
    }

    pub fn add_default_systems(&mut self) {
        self.systems
            .add_turn_system(Box::new(Exploration::default()));
        self.systems.add_turn_system(Box::new(UnitCull::default()));
        self.systems
            .add_turn_system(Box::new(PlayerCheck::default()));
        self.systems
            .add_turn_system(Box::new(MonsterTurns::default()));
    }

    pub fn run_turn_systems(&mut self) {
        self.systems.run_turn_systems(&mut self.ecs, &self.map);
    }

    pub fn update_systems(&mut self) {
        self.systems.update_systems(&self.ecs, &self.map);

    }
}
