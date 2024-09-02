use std::vec;

use crate::{
    ecs::{
        ecs::{
            DeleteComponentOrder, DeleteEntityOrder, Delta, EntityIdentifier, IndexedData,
            MakeComponentOrder, MakeEntityOrder, ECS,
        },
        entity::{take_component_from_owned, take_component_from_refs},
        event::{propagate_event, EventResponse, EventType, InteractionEvent},
    },
    game::components::{attributes::Attributes, combat::Health, core::*},
    utils::logger,
};

use super::components::combat::{self, calculate_melee_attack, default_take_damage, default_take_double_damage, default_take_half_damage};

pub fn take_damage_response(event: &InteractionEvent, own_components: &[&Component], _ecs: &ECS) -> Vec<Delta> {
    let Some(attack) = event.attack else {
        return vec![];
    };

    let (maybe_health, own_components) =
        take_component_from_refs(ComponentType::Health, own_components);
    let Some(Component::Health(health)) = maybe_health else {
        return vec![];
    };

    let (maybe_stats, own_components) =
        take_component_from_refs(ComponentType::Attributes, &own_components);
    let maybe_stats = match maybe_stats {
        Some(Component::Attributes(stats)) => Some(stats),
        _ => None,
    };

    let (maybe_items, _) = take_component_from_refs(ComponentType::Inventory, &own_components);
    let maybe_items = match maybe_items {
        Some(Component::Inventory(items)) => Some(items),
        _ => None,
    };

    let (delta, damage_taken) = default_take_damage(&attack, health, maybe_stats, maybe_items);

    let (maybe_my_name, _own_components) =
        take_component_from_refs(ComponentType::Name, &own_components);
    let (maybe_their_name, _) =
        take_component_from_owned(ComponentType::Name, event.payload.clone());
    if let (Some(Component::Name(my_name)), Some(Component::Name(their_name))) =
        (maybe_my_name, maybe_their_name)
    {
        let msg = logger::generate_attack_message(
            &their_name.data,
            &my_name.data,
            attack.hit_message,
            damage_taken,
        );
        logger::log_message(&msg);
    } else if let Some(Component::Name(my_name)) = maybe_my_name {
        let msg = logger::generate_take_damage_message(&my_name.data, damage_taken);
        logger::log_message(&msg);
    }
    delta
}

pub fn take_half_damage_response(
    event: &InteractionEvent,
    own_components: &[&Component],
    ecs: &ECS,
) -> Vec<Delta> {
    let Some(attack) = event.attack else {
        return vec![];
    };

    let (maybe_health, own_components) =
        take_component_from_refs(ComponentType::Health, own_components);
    let Some(Component::Health(health)) = maybe_health else {
        return vec![];
    };

    let (maybe_stats, own_components) =
        take_component_from_refs(ComponentType::Attributes, &own_components);
    let maybe_stats = match maybe_stats {
        Some(Component::Attributes(stats)) => Some(stats),
        _ => None,
    };

    let (maybe_items, _) = take_component_from_refs(ComponentType::Inventory, &own_components);
    let maybe_items = match maybe_items {
        Some(Component::Inventory(items)) => Some(items),
        _ => None,
    };

    let (delta, damage_taken) = default_take_half_damage(&attack, health, maybe_stats, maybe_items);

    let (maybe_my_name, _own_components) =
        take_component_from_refs(ComponentType::Name, &own_components);
    let (maybe_their_name, _) =
        take_component_from_owned(ComponentType::Name, event.payload.clone());
    if let (Some(Component::Name(my_name)), Some(Component::Name(their_name))) =
        (maybe_my_name, maybe_their_name)
    {
        let msg = logger::generate_attack_message(
            &their_name.data,
            &my_name.data,
            attack.hit_message,
            damage_taken,
        );
        logger::log_message(&msg);
    } else if let Some(Component::Name(my_name)) = maybe_my_name {
        let msg = logger::generate_take_damage_message(&my_name.data, damage_taken);
        logger::log_message(&msg);
    }
    delta
}

pub fn take_double_damage_response(
    event: &InteractionEvent,
    own_components: &[&Component],
    ecs: &ECS,
) -> Vec<Delta> {
    let Some(attack) = event.attack else {
        return vec![];
    };

    let (maybe_health, own_components) =
        take_component_from_refs(ComponentType::Health, own_components);
    let Some(Component::Health(health)) = maybe_health else {
        return vec![];
    };

    let (maybe_stats, own_components) =
        take_component_from_refs(ComponentType::Attributes, &own_components);
    let maybe_stats = match maybe_stats {
        Some(Component::Attributes(stats)) => Some(stats),
        _ => None,
    };

    let (maybe_items, _) = take_component_from_refs(ComponentType::Inventory, &own_components);
    let maybe_items = match maybe_items {
        Some(Component::Inventory(items)) => Some(items),
        _ => None,
    };

    let (delta, damage_taken) = default_take_double_damage(&attack, health, maybe_stats, maybe_items);

    let (maybe_my_name, _own_components) =
        take_component_from_refs(ComponentType::Name, &own_components);
    let (maybe_their_name, _) =
        take_component_from_owned(ComponentType::Name, event.payload.clone());
    if let (Some(Component::Name(my_name)), Some(Component::Name(their_name))) =
        (maybe_my_name, maybe_their_name)
    {
        let msg = logger::generate_attack_message(
            &their_name.data,
            &my_name.data,
            attack.hit_message,
            damage_taken,
        );
        logger::log_message(&msg);
    } else if let Some(Component::Name(my_name)) = maybe_my_name {
        let msg = logger::generate_take_damage_message(&my_name.data, damage_taken);
        logger::log_message(&msg);
    }
    delta
}

pub fn award_inventory_response(
    event: &InteractionEvent,
    own_components: &[&Component],
    ecs: &ECS,
) -> Vec<Delta> {
    let (maybe_player, components) =
        take_component_from_owned(ComponentType::Player, event.payload.clone());
    let (maybe_inventory, components) =
        take_component_from_owned(ComponentType::Inventory, components);
    let (maybe_stats, _) = take_component_from_owned(ComponentType::Attributes, components);
    let (maybe_my_inventory, _) =
        take_component_from_refs(ComponentType::Inventory, own_components);

    if let (
        Some(Component::Player(_)),
        Some(Component::Inventory(their_items)),
        Some(Component::Inventory(my_items)),
        Some(Component::Attributes(their_stats)),
    ) = (
        maybe_player,
        maybe_inventory,
        maybe_my_inventory,
        maybe_stats,
    ) {
        let my_change = my_items.data.inverse();
        let their_change = my_items.data.clone();
        let their_xp_change = Attributes {
            xp: their_change.coins,
            ..Default::default()
        };
        let msg = logger::generate_receive_gold_message(their_change.coins);
        logger::log_message(&msg);

        vec![
            Delta::Change(Component::Inventory(their_items.make_change(their_change))),
            Delta::Change(Component::Inventory(my_items.make_change(my_change))),
            Delta::Change(Component::Attributes(
                their_stats.make_change(their_xp_change),
            )),
        ]
    } else {
        vec![]
    }
}

pub fn drop_inventory_response(
    event: &InteractionEvent,
    own_components: &[&Component], 
    ecs: &ECS,
) -> Vec<Delta> {
    let (maybe_my_inventory, own_components) =
        take_component_from_refs(ComponentType::Inventory, own_components);
    let (maybe_my_position, _) = take_component_from_refs(ComponentType::Position, &own_components);

    if let (Some(Component::Inventory(my_items)), Some(Component::Position(my_position))) =
        (maybe_my_inventory, maybe_my_position)
    {
        let image = ImageData { id: 15, depth: 6 };
        let response = EventResponse::new_with(pickup_loot_response);

        let new_components = vec![
            Component::Collision(IndexedData::new_with(Collision::Walkable)),
            Component::Image(IndexedData::new_with(ImageHandle::new(image))),
            Component::Position(IndexedData::new_with(my_position.data)),
            Component::Inventory(IndexedData::new_with(my_items.data.clone())),
            Component::BumpResponse(IndexedData::new_with(response)),
        ];
        vec![Delta::MakeEntity(MakeEntityOrder {
            components: new_components,
        })]
    } else {
        vec![]
    }
}

pub fn open_image_response(_event: &InteractionEvent, own_components: &[&Component], _ecs: &ECS) -> Vec<Delta> {
    let (maybe_image, _own_components) =
        take_component_from_refs(ComponentType::Image, own_components);
    let image = match maybe_image {
        Some(Component::Image(data)) => data,
        _ => {
            return vec![];
        }
    };
    let new_image_data = match image.data.states.get("open") {
        Some(data) => data,
        _ => {
            return vec![];
        }
    };
    vec![Delta::Change(Component::Image(
        image.make_change(ImageHandle::new(*new_image_data)),
    ))]
}

pub fn close_image_response(
    _event: &InteractionEvent,
    own_components: &[&Component],
    ecs: &ECS,
) -> Vec<Delta> {
    let (maybe_image, _own_components) =
        take_component_from_refs(ComponentType::Image, own_components);
    let image = match maybe_image {
        Some(Component::Image(data)) => data,
        _ => {
            return vec![];
        }
    };
    let new_image_data = match image.data.states.get("closed") {
        Some(data) => data,
        _ => {
            return vec![];
        }
    };
    vec![Delta::Change(Component::Image(
        image.make_change(ImageHandle::new(*new_image_data)),
    ))]
}

pub fn open_collision_response(
    event: &InteractionEvent,
    own_components: &[&Component],
    ecs: &ECS,
) -> Vec<Delta> {
    let (maybe_collision, own_components) =
        take_component_from_refs(ComponentType::Collision, own_components);
    let collision = match maybe_collision {
        Some(Component::Collision(data)) => data,
        _ => {
            return vec![];
        }
    };

    vec![Delta::Change(Component::Collision(
        collision.make_change(Collision::Walkable),
    ))]
}

pub fn close_collision_response(
    event: &InteractionEvent,
    own_components: &[&Component],
    ecs: &ECS,
) -> Vec<Delta> {
    let (maybe_collision, own_components) =
        take_component_from_refs(ComponentType::Collision, own_components);
    let collision = match maybe_collision {
        Some(Component::Collision(data)) => data,
        _ => {
            return vec![];
        }
    };

    vec![Delta::Change(Component::Collision(
        collision.make_change(Collision::Blocking),
    ))]
}

pub fn open_los_blocking_response(
    _event: &InteractionEvent,
    own_components: &[&Component],
    ecs: &ECS,
) -> Vec<Delta> {
    let (maybe_collision, _) = take_component_from_refs(ComponentType::LineOfSight, own_components);
    let blocking = match maybe_collision {
        Some(Component::LineOfSight(data)) => data,
        _ => {
            return vec![];
        }
    };

    vec![Delta::Change(Component::LineOfSight(
        blocking.make_change(LoSBlocking::None),
    ))]
}

pub fn close_los_blocking_response(
    _event: &InteractionEvent,
    own_components: &[&Component],
    ecs: &ECS,
) -> Vec<Delta> {
    let (maybe_collision, _) = take_component_from_refs(ComponentType::LineOfSight, own_components);
    let blocking = match maybe_collision {
        Some(Component::LineOfSight(data)) => data,
        _ => {
            return vec![];
        }
    };

    vec![Delta::Change(Component::LineOfSight(
        blocking.make_change(LoSBlocking::Blocking),
    ))]
}

pub fn set_open_door_bump_response(
    event: &InteractionEvent,
    own_components: &[&Component],
    ecs: &ECS,
) -> Vec<Delta> {
    let (maybe_response, _) = take_component_from_refs(ComponentType::BumpResponse, own_components);
    let old_response = match maybe_response {
        Some(Component::BumpResponse(data)) => data,
        _ => {
            return vec![];
        }
    };
    let new_response = EventResponse {
        own_entity: old_response.data.own_entity,
        response_function: open_door_response,
    };

    vec![Delta::Change(Component::BumpResponse(
        old_response.make_change(new_response),
    ))]
}

pub fn set_close_door_bump_response(
    event: &InteractionEvent,
    own_components: &[&Component],
    ecs: &ECS,
) -> Vec<Delta> {
    let (maybe_response, _) = take_component_from_refs(ComponentType::BumpResponse, own_components);
    let old_response = match maybe_response {
        Some(Component::BumpResponse(data)) => data,
        _ => {
            return vec![];
        }
    };
    let new_response = EventResponse {
        own_entity: old_response.data.own_entity,
        response_function: close_door_response,
    };
    vec![Delta::Change(Component::BumpResponse(
        old_response.make_change(new_response),
    ))]
}

pub fn clear_bump_response(event: &InteractionEvent, own_components: &[&Component], ecs: &ECS,) -> Vec<Delta> {
    let (maybe_response, _) = take_component_from_refs(ComponentType::BumpResponse, own_components);
    let response = match maybe_response {
        Some(Component::BumpResponse(data)) => data,
        _ => {
            return vec![];
        }
    };

    vec![Delta::DeleteComponent(DeleteComponentOrder {
        component_id: response.index,
        entity_id: Some(response.data.own_entity),
    })]
}

pub fn delete_self_response(event: &InteractionEvent, own_components: &[&Component], ecs: &ECS,) -> Vec<Delta> {
    let id = own_components[0].get_id();
    vec![Delta::DeleteEntity(DeleteEntityOrder::new_from_component(
        id,
    ))]
}

pub fn default_burn_response(
    _event: &InteractionEvent,
    own_components: &[&Component],
    _ecs: &ECS,
) -> Vec<Delta> {
    let (maybe_burn, components) = take_component_from_refs(ComponentType::DurationEffect, own_components);
    let (maybe_health, components) = take_component_from_refs(ComponentType::Health, &components);
    let (maybe_name, _components) = take_component_from_refs(ComponentType::Name, &components);
    match (maybe_burn, maybe_health) {
        (
            Some(Component::DurationEffect(IndexedData{index, data: DurationEffect(duration, EffectType::Burning)})),
            Some(Component::Health(health_data)),
        ) => {
            if *duration == 0 {
                return vec![Delta::DeleteComponent(DeleteComponentOrder{
                    component_id: *index,
                    entity_id: None,
                })];
            }

            if let Some(Component::Name(name_data)) = maybe_name {
                let msg = logger::generate_is_burning_message(&name_data.data, 1);
                logger::log_message(&msg);
            }
            let damage_taken = Health {
                current: -1,
                ..Default::default()
            };
            vec![Delta::Change(Component::Health(
                health_data.make_change(damage_taken),
            ))]
        }
        (
            None,
            _,
        ) => {
            if let Some(component) = own_components.first() {
                if let Some(Component::Name(name_data)) = maybe_name {
                    let msg = logger::generate_on_fire_message(&name_data.data);
                    logger::log_message(&msg);
                }
                vec![Delta::MakeComponent(MakeComponentOrder {
                    component: Component::DurationEffect(IndexedData::new_with(DurationEffect(4, EffectType::Burning))),
                    entity: EntityIdentifier::new_from_component(component.get_id()),
                })]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

pub fn empty_response(_event: &InteractionEvent, _own_components: &[&Component], ecs: &ECS,) -> Vec<Delta> {
    vec![]
}

pub fn spread_acid_response(event: &InteractionEvent, _own_components: &[&Component], ecs: &ECS,) -> Vec<Delta> {
    let Some(payload_component) = event.payload.first() else {
        return vec![];
    };
    let Some(entity_id) = ecs.get_entity_id_from_component_id(payload_component.get_id()) else {
        return vec![];
    };
    vec![
        Delta::MakeComponent(MakeComponentOrder {
            component: Component::DurationEffect(IndexedData::new_with(DurationEffect(3, EffectType::Acid))),
            entity: EntityIdentifier::new_from_entity(entity_id),
        })
    ]
}

pub fn spread_fire_response(event: &InteractionEvent, _own_components: &[&Component], ecs: &ECS,) -> Vec<Delta> {
    let EventType::Bump = event.event_type else {
        return vec![];
    };
    let Some(payload_component) = event.payload.first() else {
        return vec![];
    };
    let Some(entity_id) = ecs.get_entity_id_from_component_id(payload_component.get_id()) else {
        return vec![];
    };

    let event = InteractionEvent {
        event_type: EventType::Fire,
        payload: vec![],
        attack: None,
    };

    propagate_event(&event, entity_id, ecs)
}


pub fn spread_if_on_fire_response(event: &InteractionEvent, own_components: &[&Component], ecs: &ECS,) -> Vec<Delta> {
    
    let (Some(Component::DurationEffect(IndexedData { index:_, data:DurationEffect(_, EffectType::Burning) })), _) = take_component_from_refs(ComponentType::DurationEffect, own_components) else {
        return vec![];
    };

    spread_fire_response(event, own_components, ecs)
}

pub fn spikes_response(event: &InteractionEvent, own_components: &[&Component], ecs: &ECS,) -> Vec<Delta> {
    let (other_levitate, _) = take_component_from_owned(ComponentType::DurationEffect, event.payload.clone());
    if let Some(Component::DurationEffect(IndexedData { index:_, data:DurationEffect(_, EffectType::Levitate) })) = other_levitate {
        vec![]
    } else {
        retaliate_response(event, own_components, ecs)
    }
}

pub fn retaliate_response(event: &InteractionEvent, own_components: &[&Component], ecs: &ECS,) -> Vec<Delta> {
    let EventType::Bump = event.event_type else {
        return vec![];
    };
    let Some(payload_component) = event.payload.first() else {
        return vec![];
    };
    let Some(entity_id) = ecs.get_entity_id_from_component_id(payload_component.get_id()) else {
        return vec![];
    };
    let (Some(Component::Combat(combat)), _) = take_component_from_refs(ComponentType::Combat, own_components) else {
        return vec![];
    };

    let event = InteractionEvent {
        event_type: EventType::Bump,
        payload: own_components
            .into_iter()
            .map(|comp| comp.to_owned().clone())
            .collect(),
        attack: combat::calculate_melee_attack(&combat.data, None),
    };

    propagate_event(&event, entity_id, ecs)
}

pub fn open_chest_response(event: &InteractionEvent, own_components: &[&Component], ecs: &ECS,) -> Vec<Delta> {
    let image_delta = open_image_response(event, own_components, ecs);
    let inventory_delta = award_inventory_response(event, own_components, ecs);
    let bump_delta = clear_bump_response(event, own_components, ecs);
    let burn_delta = spread_if_on_fire_response(event, own_components, ecs);

    vec![image_delta, inventory_delta, bump_delta, burn_delta].concat()
}

pub fn open_door_response(event: &InteractionEvent, own_components: &[&Component], ecs: &ECS,) -> Vec<Delta> {
    let image_delta = open_image_response(event, own_components, ecs);
    let collision_delta = open_collision_response(event, own_components, ecs);
    let los_delta = open_los_blocking_response(event, own_components, ecs);
    let bump_delta = set_close_door_bump_response(event, own_components, ecs);
    let burn_delta = spread_if_on_fire_response(event, own_components, ecs);
    
    vec![image_delta, collision_delta, bump_delta, los_delta, burn_delta].concat()
}

pub fn close_door_response(event: &InteractionEvent, own_components: &[&Component], ecs: &ECS,) -> Vec<Delta> {
    let (other_pos, _) = take_component_from_owned(ComponentType::Position, event.payload.clone());
    if let Some(_) = other_pos {
        // only react to close door command, which does not send a position
        return vec![];
    }

    let image_delta = close_image_response(event, own_components, ecs);
    let collision_delta = close_collision_response(event, own_components, ecs);
    let los_delta = close_los_blocking_response(event, own_components, ecs);
    let bump_delta = set_open_door_bump_response(event, own_components, ecs);

    vec![image_delta, collision_delta, bump_delta, los_delta].concat()
}

pub fn pickup_loot_response(event: &InteractionEvent, own_components: &[&Component], ecs: &ECS,) -> Vec<Delta> {
    let inventory_changes = award_inventory_response(event, own_components, ecs);
    if inventory_changes.is_empty() {
        return vec![];
    }
    let despawning = delete_self_response(event, own_components, ecs);

    vec![inventory_changes, despawning].concat()
}
