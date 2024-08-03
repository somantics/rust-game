use component::combat;

use crate::{
    ecs::{component, take_component_from_refs},
    ecs::component::ComponentType,
    event::InteractionEvent,
    map::Coordinate,
};

use super::{
    attributes::Attributes,
    combat::{Combat, Health},
    inventory::Inventory,
    system::ComponentQuery,
    Component, IndexedData, Name,
};
thread_local!{
    pub static TURNTAKER: ComponentQuery = ComponentQuery {
        required: vec![
            ComponentType::Name,
            ComponentType::Image,
            ComponentType::Position,
            ComponentType::Turn,
        ],
        optional: vec![
            ComponentType::Attributes,
            ComponentType::Movement,
            ComponentType::Health,
            ComponentType::Combat,
        ],
    };
    
    pub static PLAYER: ComponentQuery = ComponentQuery {
        required: vec![ComponentType::Player],
        optional: vec![],
    };
}

#[derive(Debug, Clone, Default)]
pub struct UnitReport {
    pub position: IndexedData<Coordinate>,
    pub combat: IndexedData<Combat>,
    pub name: Option<IndexedData<Name>>,
    pub stats: Option<IndexedData<Attributes>>,
    pub health: Option<IndexedData<Health>>,
    pub items: Option<IndexedData<Inventory>>,
    pub bump: InteractionEvent,
    pub shoot: InteractionEvent,
}

// todo panics at looking for position on player (who spawned with a position?)
pub fn make_unit_report<'a>(unit_components: &'a Vec<&'a Component>) -> UnitReport {
    let (maybe_position, components) =
        take_component_from_refs(ComponentType::Position, unit_components);
    let position = match maybe_position {
        Some(Component::Position(data)) => data.to_owned(),
        _ => panic!("Unit has no position."),
    };
    let (maybe_combat, components) = take_component_from_refs(ComponentType::Combat, &components);
    let combat = match maybe_combat {
        Some(Component::Combat(data)) => data.to_owned(),
        _ => panic!("Unit has no combat."),
    };

    let (maybe_stats, components) = take_component_from_refs(ComponentType::Attributes, &components);
    let stats = match maybe_stats {
        Some(Component::Attributes(data)) => Some(data.to_owned()),
        _ => None,
    };
    let (maybe_name, components) = take_component_from_refs(ComponentType::Name, &components);
    let name = match maybe_name {
        Some(Component::Name(data)) => Some(data.to_owned()),
        _ => None,
    };
    let (maybe_health, components) = take_component_from_refs(ComponentType::Health, &components);
    let health = match maybe_health {
        Some(Component::Health(data)) => Some(data.to_owned()),
        _ => None,
    };
    let (maybe_items, components) = take_component_from_refs(ComponentType::Inventory, &components);
    let items = match maybe_items {
        Some(Component::Inventory(data)) => Some(data.to_owned()),
        _ => None,
    };

    let payload: Vec<Component> = unit_components.into_iter().map(|&elem| elem.to_owned()).collect();

    let attack = combat::calculate_melee_attack(&combat.data, IndexedData::unwrap_data(stats.as_ref()));
    let bump = InteractionEvent {
        event_type: crate::event::EventType::Bump,
        attack,
        payload: payload.clone(),
    };

    let attack = combat::calculate_ranged_attack(&combat.data, IndexedData::unwrap_data(stats.as_ref()));
    let shoot = InteractionEvent {
        event_type: crate::event::EventType::Shot,
        attack,
        payload,
    };

    UnitReport {
        position,
        combat,
        name,
        bump,
        shoot,
        stats,
        health,
        items,
    }
}
