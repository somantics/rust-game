use std::thread::LocalKey;
use phf::{phf_map, Map};

use crate::ecs::ecs::{Delta, EntityIdentifier, IndexedData, MakeComponentOrder, ECS};
use crate::ecs::entity::Entity;
use crate::ecs::event::{propagate_event, EventResponse, EventType, InteractionEvent};
use crate::ecs::system::ComponentQuery;

use crate::game::components::core::ComponentType;
use crate::game::components::spells::Spell;
use crate::game::components::core::{Component, DurationEffect, EffectType, ImageHandle};

use crate::game::responses;
use crate::utils::logger;


pub static SPELL_REGISTRY: Map<u32, &LocalKey<Spell>> = phf_map!(
    0u32 => &INVISIBILITY,
    1u32 => &LEVITATE,
    2u32 => &HEAL,
    3u32 => &STONESKIN,
    4u32 => &BRITTLE,
    5u32 => &FLAMES,
  );

thread_local! {
    pub static INVISIBILITY: Spell = Spell::new(
        "Invisibility", 
        ImageHandle::new_spell(0, 1), 
        ComponentQuery::new_single(ComponentType::Player), 
        invisible);
    
    pub static LEVITATE: Spell = Spell::new(
        "Levitate", 
        ImageHandle::new_spell(2, 3), 
        ComponentQuery::new_single(ComponentType::Player), 
        levitate);

    pub static HEAL: Spell = Spell::new(
        "Heal", 
        ImageHandle::new_spell(4, 5), 
        ComponentQuery::new_single(ComponentType::Player), 
        heal);

    pub static STONESKIN: Spell = Spell::new(
        "Stoneskin", 
        ImageHandle::new_spell(6, 7), 
        ComponentQuery::new_single(ComponentType::Player), 
        stoneskin);

    pub static BRITTLE: Spell = Spell::new(
        "Brittle", 
        ImageHandle::new_spell(12, 13), 
        ComponentQuery::new_single(ComponentType::Player), 
        brittle);

    pub static FLAMES: Spell = Spell::new(
        "Mass Flames", 
        ImageHandle::new_spell(10, 11), 
        ComponentQuery::new_single(ComponentType::Player), 
        mass_flame);
}


pub fn invisible(entities: &[&Entity], _ecs: &ECS) -> Vec<Delta> {
    logger::log_message("You cast invisibility!");
    entities
        .into_iter()
        .map(|entity| {
            Delta::MakeComponent(MakeComponentOrder {
                component: Component::DurationEffect(IndexedData::new_with(DurationEffect(8, EffectType::Invisible))),
                entity: EntityIdentifier::new_from_entity(entity.index),
            })
        })
        .collect()
}

pub fn levitate(entities: &[&Entity], _ecs: &ECS) -> Vec<Delta> {
    logger::log_message("You cast levitate!");
    entities
        .into_iter()
        .map(|entity| {
            Delta::MakeComponent(MakeComponentOrder {
                component: Component::DurationEffect(IndexedData::new_with(DurationEffect(16, EffectType::Levitate))),
                entity: EntityIdentifier::new_from_entity(entity.index),
            })
        })
        .collect()
}

pub fn stoneskin(entities: &[&Entity], ecs: &ECS) -> Vec<Delta> {
    logger::log_message("You cast stoneskin!");
    entities
        .into_iter()
        .map(|entity| {
            let Some(Component::BumpResponse(melee_response)) = ecs.get_component_from_entity_id(entity.index, ComponentType::BumpResponse) else {
                return vec![];
            };
            let Some(Component::ShotResponse(ranged_response)) = ecs.get_component_from_entity_id(entity.index, ComponentType::ShotResponse) else {
                return vec![];
            };
            let half_damage = EventResponse::new_with(responses::take_half_damage_response);
            vec![
                Delta::Change(Component::BumpResponse(melee_response.make_change(half_damage))),
                Delta::Change(Component::ShotResponse(ranged_response.make_change(half_damage.clone()))),
                Delta::MakeComponent(MakeComponentOrder {
                    component: Component::DurationEffect(IndexedData::new_with(DurationEffect(8, EffectType::Stoneskin))),
                    entity: EntityIdentifier::new_from_entity(entity.index),
                })
            ]
            
        })
        .flatten()
        .collect()
}

pub fn heal(entities: &[&Entity], ecs: &ECS) -> Vec<Delta> {
    logger::log_message("You cast heal!");
    entities
        .into_iter()
        .filter_map(|entity| {
            let Some(Component::Health(health)) = ecs.get_component_from_entity(entity, ComponentType::Health) else {
                return None;
            };
            Some(Delta::Change(Component::Health(health.make_change(health.data.health_reset_diff()))))
        })
        .collect()
}

pub fn brittle(entities: &[&Entity], ecs: &ECS) -> Vec<Delta> {
    logger::log_message("You cast brittle!");
    let entity = entities.first().unwrap();
    let Some(Component::Position(index_pos)) = ecs.get_component_from_entity(entity, ComponentType::Position) else {
        return vec![];
    };
    
    ecs.get_entities_in_room(index_pos.data)
        .into_iter()
        .filter(|entity| ecs.entity_has_component(entity, ComponentType::Monster))
        .map(|entity| {
            let Some(Component::BumpResponse(melee_response)) = ecs.get_component_from_entity_id(entity.index, ComponentType::BumpResponse) else {
                return vec![];
            };
            let Some(Component::ShotResponse(ranged_response)) = ecs.get_component_from_entity_id(entity.index, ComponentType::ShotResponse) else {
                return vec![];
            };
            let Some(Component::Name(name)) = ecs.get_component_from_entity_id(entity.index, ComponentType::Name) else {
                return vec![];
            };
            
            let double_damage = EventResponse {
                own_entity: melee_response.data.own_entity,
                response_function: responses::take_double_damage_response,
            };
            
            logger::log_message(&[&name.data.raw, "shudders."].join(" "));
            vec![
                Delta::Change(Component::BumpResponse(melee_response.make_change(double_damage))),
                Delta::Change(Component::ShotResponse(ranged_response.make_change(double_damage.clone()))),
            ]
        })
        .flatten()
        .collect()
}

pub fn mass_flame(entities: &[&Entity], ecs: &ECS) -> Vec<Delta> {
    logger::log_message("You cast mass flame!");
    let entity = entities.first().unwrap();
    let Some(Component::Position(index_pos)) = ecs.get_component_from_entity(entity, ComponentType::Position) else {
        return vec![];
    };
    
    ecs.get_entities_in_room(index_pos.data)
        .into_iter()
        .filter(|entity| ecs.entity_has_component(entity, ComponentType::Monster))
        .map(|entity| {
            let event = InteractionEvent {
                event_type: EventType::Fire,
                payload: vec![],
                attack: None,
            };

            // Do burning
            propagate_event(&event, entity.index, ecs)
        })
        .flatten()
        .collect()
}

