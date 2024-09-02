use std::collections::{HashMap, HashSet};

use crate::{
    ecs::{
        ecs::{DeleteComponentOrder, DeleteEntityOrder, Delta, IndexedData, ECS},
        entity::{take_component_from_owned, take_component_from_refs},
        event::{self, propagate_event, EventResponse, EventType, InteractionEvent},
        system::{ComponentQuery, System},
    },
    game::{
        archetype,
        components::{
            attributes::{get_xp_to_next, Attributes},
            core::*,
        }, responses,
    },
    map::{gamemap::GameMap, utils::Coordinate},
    utils::{logger, pathfinding},
};

use super::components::combat::Health;

#[derive(Default)]
pub struct UnitCull {}

impl System for UnitCull {
    fn get_requirements(&self) -> ComponentQuery {
        ComponentQuery {
            required: vec![ComponentType::Health],
            optional: vec![],
        }
    }

    fn run_next(&mut self, components: &[&Component], ecs: &ECS, _map: &GameMap) -> Vec<Delta> {
        if let (Some(Component::Health(health)), _) =
            take_component_from_refs(ComponentType::Health, components)
        {
            if health.data.current <= 0 {
                let event = InteractionEvent {
                    event_type: EventType::Death,
                    attack: None,
                    payload: vec![],
                };
                let entity_id = ecs.get_entity_id_from_component_id(health.index).unwrap();
                let mut event_results = event::propagate_event(&event, entity_id, ecs);
                event_results.push(Delta::DeleteEntity(DeleteEntityOrder::new_from_entity(
                    entity_id,
                )));
                return event_results;
            }
        }
        vec![]
    }
}

pub type NavigationGrid = HashMap<Coordinate, Coordinate>;
#[derive(Default)]
pub struct MonsterTurns {
    safe_nav_grid: NavigationGrid,
    hazard_nav_grid: NavigationGrid,
}

impl System for MonsterTurns {
    fn get_requirements(&self) -> ComponentQuery {
        archetype::TURNTAKER.with(|query| query.clone())
    }

    fn run_pre_loop(&mut self, ecs: &ECS, map: &GameMap) {
        let Some(player_report) = ecs.get_player_report() else {
            return;
        };
        let player_position = player_report.position.data;
        let heuristic = |_| 0;
        let ignore_units = true;
        let ignore_doors = false;
        let ignore_hazards = true;

        self.hazard_nav_grid = pathfinding::calculate_pathing_grid(
            player_position,
            player_position,
            map,
            ecs,
            heuristic,
            ignore_units,
            ignore_doors,
            ignore_hazards,
        );
        
        let ignore_hazards = false;

        self.safe_nav_grid = pathfinding::calculate_pathing_grid(
            player_position,
            player_position,
            map,
            ecs,
            heuristic,
            ignore_units,
            ignore_doors,
            ignore_hazards,
        );
    }

    fn run_next(&mut self, components: &[&Component], ecs: &ECS, map: &GameMap) -> Vec<Delta> {
        if let (Some(Component::Turn(data)), _) =
            take_component_from_refs(ComponentType::Turn, components)
        {
            data.data.process_turn(components, ecs, map, &self.safe_nav_grid, &self.hazard_nav_grid)
        } else {
            vec![]
        }
    }
}

#[derive(Default)]
pub struct PlayerCheck {}

impl System for PlayerCheck {
    fn get_requirements(&self) -> ComponentQuery {
        archetype::PLAYER.with(|query| query.clone())
    }

    fn run_next(&mut self, components: &[&Component], _ecs: &ECS, _map: &GameMap) -> Vec<Delta> {
        let Some(self_report) = archetype::make_unit_report(components) else {
            return vec![];
        };
        if let Some(stats) = self_report.stats {
            if stats.data.xp >= get_xp_to_next(&stats.data) {
                let new_level = stats.data.level + 1;
                logger::log_message(&format!("You have reached level {}!", new_level));
                return vec![Delta::Change(Component::Attributes(stats.make_change(
                    Attributes {
                        level_pending: true,
                        ..Default::default()
                    },
                )))];
            }
        }
        vec![]
    }
}

#[derive(Default)]
pub struct Exploration {
    open_doors: HashSet<usize>,
}

impl System for Exploration {
    fn get_requirements(&self) -> ComponentQuery {
        ComponentQuery {
            required: vec![ComponentType::Door, ComponentType::Collision],
            optional: vec![],
        }
    }

    fn run_next(&mut self, components: &[&Component], ecs: &ECS, map: &GameMap) -> Vec<Delta> {
        let (maybe_position, components) =
            take_component_from_refs(ComponentType::Position, components);
        let (maybe_collision, _components) =
            take_component_from_refs(ComponentType::Collision, &components);

        let Some(Component::Position(pos_data)) = maybe_position else {
            return vec![];
        };

        let Some(Component::Collision(col_data)) = maybe_collision else {
            return vec![];
        };

        let Some(door_id) = ecs.get_entity_id_from_component_id(col_data.index) else {
            return vec![];
        };

        if col_data.data == Collision::Walkable && !self.open_doors.contains(&door_id) {
            map.explore_room(pos_data.data);
            // the floodfill covers hallways and miss-generated areas
            map.explore_flood_fill(pos_data.data, ecs);
            self.open_doors.insert(door_id);
        }
        return vec![];
    }

    fn new_floor_update(&mut self, _ecs: &ECS, _map: &GameMap) {
        self.open_doors.clear();
    }
}

#[derive(Default)]
pub struct Fire {}
impl System for Fire {
    fn get_requirements(&self) -> ComponentQuery {
        ComponentQuery {
            required: vec![ComponentType::DurationEffect],
            optional: vec![],
        }
    }

    fn run_next(&mut self, components: &[&Component], ecs: &ECS, _map: &GameMap) -> Vec<Delta> {
        let (maybe_burning, components) =
            take_component_from_refs(ComponentType::DurationEffect, components);
        let Some(Component::DurationEffect(indexed_effect)) = maybe_burning else {
            return vec![];
        };

        let DurationEffect(_, effect) = indexed_effect.data;
        let EffectType::Burning = effect else {
            return vec![];
        };

        let mut delta = vec![];
        let event = InteractionEvent {
            event_type: EventType::Fire,
            payload: vec![],
            attack: None,
        };

        // Do burning
        if let Some(entity_id) = ecs.get_entity_id_from_component_id(indexed_effect.index) {
            let event_delta = propagate_event(&event, entity_id, ecs);
            delta = event_delta;
        };

        // Spread to adjacent
        let (maybe_position, _components) =
            take_component_from_refs(ComponentType::Position, &components);
        if let Some(Component::Position(position)) = maybe_position {
            let adjacents = ecs.get_all_adjacent_entities(position.data);
            for entity_id in adjacents {
                let event_delta = propagate_event(&event, entity_id, ecs);
                delta.extend(event_delta)
            }
        }
        delta
    }
}

#[derive(Default)]
pub struct Acid {}
impl System for Acid {
    fn get_requirements(&self) -> ComponentQuery {
        ComponentQuery {
            required: vec![ComponentType::DurationEffect],
            optional: vec![],
        }
    }

    fn run_next(&mut self, components: &[&Component], ecs: &ECS, _map: &GameMap) -> Vec<Delta> {
        let (maybe_acid, _components) =
            take_component_from_refs(ComponentType::DurationEffect, components);
        let Some(Component::DurationEffect(indexed_effect)) = maybe_acid else {
            return vec![];
        };

        let DurationEffect(_, effect) = indexed_effect.data;
        let EffectType::Acid = effect else {
            return vec![];
        };

        let mut delta = vec![];

        // Do acid damage
        if let Some(entity_id) = ecs.get_entity_id_from_component_id(indexed_effect.index) {
            let maybe_health = ecs.get_component_from_entity_id(entity_id, ComponentType::Health);
            let maybe_name = ecs.get_component_from_entity_id(entity_id, ComponentType::Name);
            if let Some(Component::Health(health)) = maybe_health {
                let damage_data = Health {current: -2, ..Default::default()};

                if let Some(Component::Name(name_data)) = maybe_name {
                    logger::log_message(&[&name_data.data.raw, "is burned by acid."].join(" "));
                };

                delta.push(
                    Delta::Change(Component::Health(health.make_change(damage_data)))
                );
            }
        };
        delta
    }
}

#[derive(Default)]
pub struct Stoneskin {}
impl System for Stoneskin {
    fn get_requirements(&self) -> ComponentQuery {
        ComponentQuery {
            required: vec![ComponentType::DurationEffect],
            optional: vec![],
        }
    }

    fn run_next(&mut self, components: &[&Component], _ecs: &ECS, _map: &GameMap) -> Vec<Delta> {
        let (maybe_burning, components) =
            take_component_from_refs(ComponentType::DurationEffect, components);
        let Some(Component::DurationEffect(indexed_effect)) = maybe_burning else {
            return vec![];
        };

        let DurationEffect(duration, effect) = indexed_effect.data;
        let EffectType::Stoneskin = effect else {
            return vec![];
        };

        if duration == 0 {
            let (Some(Component::BumpResponse(melee_response)), components) = take_component_from_refs(ComponentType::BumpResponse, &components) else {
                return vec![];
            };
            let (Some(Component::ShotResponse(ranged_response)), _components) = take_component_from_refs(ComponentType::ShotResponse, &components) else {
                return vec![];
            };
            let full_damage = EventResponse::new_with(responses::take_damage_response);
            vec![
                Delta::Change(Component::BumpResponse(melee_response.make_change(full_damage))),
                Delta::Change(Component::ShotResponse(ranged_response.make_change(full_damage.clone()))),
            ]
        } else {
            vec![]
        }
    }
}

#[derive(Default)]
pub struct Duration {}
impl System for Duration {
    fn get_requirements(&self) -> ComponentQuery {
        ComponentQuery {
            required: vec![ComponentType::DurationEffect],
            optional: vec![],
        }
    }

    fn run_next(&mut self, components: &[&Component], _ecs: &ECS, _map: &GameMap) -> Vec<Delta> {
        let (maybe_effect, _components) =
            take_component_from_refs(ComponentType::DurationEffect, components);
        let Some(Component::DurationEffect(indexed_effect)) = maybe_effect else {
            return vec![];
        };
        let (maybe_name, _components) =
            take_component_from_refs(ComponentType::Name, components);

        let DurationEffect(duration, effect) = indexed_effect.data;

        if duration == 0 {
            let action = match effect {
                EffectType::Burning => {
                    "stops burning."
                },
                EffectType::Levitate => {
                    "stops levitating."
                },
                EffectType::Invisible => {
                    "is no longer invisible."
                },
                EffectType::Stoneskin => {
                    "lost stoneskin."
                },
                _ => {"lost an effect."}
            };
            match maybe_name {
                Some(Component::Name(name)) => logger::log_message(&[&name.data.raw, action].join(" ")),
                _ => {}
            };
            vec![Delta::DeleteComponent(DeleteComponentOrder{component_id: indexed_effect.index, entity_id: None})]
        } else {
            vec![Delta::Change(Component::DurationEffect(indexed_effect.make_change(DurationEffect(-1, effect))))]
        }

    }
}

#[derive(Default)]
pub struct Cooldowns {}
impl System for Cooldowns {
    fn get_requirements(&self) -> ComponentQuery {
        archetype::CASTER.with(|query| query.clone())
    }

    fn run_next(&mut self, components: &[&Component], _ecs: &ECS, _map: &GameMap) -> Vec<Delta> {
        components
            .into_iter()
            .filter_map(|component| {
                if let Component::Spell(spell_index) = component {
                    Some(Delta::Change(Component::Spell(spell_index.make_change(spell_index.data.off_cooldown()))))
                } else {
                    return None;
                }
            })
            .collect()
    }
}