use crate::game::components::{
    combat::AttackReport,
    core::{Component, ComponentType},
};

use crate::ecs::{
    component::Diffable,
    ecs::{Delta, ECS},
};

#[derive(Debug, Clone, Default, Copy)]
pub enum EventType {
    #[default]
    Bump,
    Shot,
    Death,
    Fire,
}

#[derive(Debug, Clone, Default)]
pub struct InteractionEvent {
    pub event_type: EventType,
    pub attack: Option<AttackReport>,
    pub payload: Vec<Component>,
}

#[derive(Debug, Clone, Copy)]
pub struct EventResponse {
    pub own_entity: usize,
    pub response_function: fn(&InteractionEvent, &[&Component], &ECS) -> Vec<Delta>,
}

impl EventResponse {
    fn process_event(&self, event: &InteractionEvent, ecs: &ECS) -> Vec<Delta> {
        (self.response_function)(event, &ecs.get_components_from_entity_id(self.own_entity), ecs)
    }

    pub fn new_with(response_function: fn(&InteractionEvent, &[&Component], &ECS) -> Vec<Delta>) -> Self {
        Self {
            response_function,
            ..Default::default()
        }
    }
}

impl Default for EventResponse {
    fn default() -> Self {
        EventResponse {
            own_entity: 0,
            response_function: |_, _, _| vec![],
        }
    }
}

impl Diffable for EventResponse {
    fn apply_diff(&mut self, other: &Self) {
        self.response_function = other.response_function;
    }
}

fn event_to_reponse_type(event_type: EventType) -> ComponentType {
    match event_type {
        EventType::Bump => ComponentType::BumpResponse,
        EventType::Shot => ComponentType::ShotResponse,
        EventType::Death => ComponentType::DeathResponse,
        EventType::Fire => ComponentType::FireResponse,
    }
}

pub fn propagate_event(event: &InteractionEvent, entity_id: usize, ecs: &ECS) -> Vec<Delta> {
    let response_type = event_to_reponse_type(event.event_type);
    if let Some(Component::BumpResponse(comp))
    | Some(Component::ShotResponse(comp))
    | Some(Component::DeathResponse(comp))
    | Some(Component::FireResponse(comp)) =
        ecs.get_component_from_entity_id(entity_id, response_type)
    {
        comp.data.process_event(event, ecs)
    } else {
        vec![]
    }
}
