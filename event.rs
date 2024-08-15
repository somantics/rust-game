use crate::ecs::{
    self,
    component::{combat::AttackReport, Component, Diffable},
    Delta, ECS,
};

#[derive(Debug, Clone, Default, Copy)]
pub enum EventType {
    #[default]
    Bump,
    Shot,
    Death,
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
    pub response_function: fn(&InteractionEvent, &Vec<&Component>) -> Vec<Delta>,
}

impl EventResponse {
    fn process_event(&self, event: &InteractionEvent, ecs: &ECS) -> Vec<Delta> {
        (self.response_function)(event, &ecs.get_components_from_entity(self.own_entity))
    }

    pub fn new_with(
        response_function: fn(&InteractionEvent, &Vec<&Component>) -> Vec<Delta>,
    ) -> Self {
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
            response_function: |_, _| vec![],
        }
    }
}

impl Diffable for EventResponse {
    fn apply_diff(&mut self, other: &Self) {
        self.response_function = other.response_function;
    }
}

fn event_to_reponse_type(event_type: EventType) -> ecs::component::ComponentType {
    match event_type {
        EventType::Bump => ecs::component::ComponentType::BumpResponse,
        EventType::Shot => ecs::component::ComponentType::ShotResponse,
        EventType::Death => ecs::component::ComponentType::DeathResponse,
    }
}

pub fn propagate_event(event: &InteractionEvent, entity_id: usize, ecs: &ECS) -> Vec<Delta> {
    let response_type = event_to_reponse_type(event.event_type);
    if let Some(Component::BumpResponse(comp))
    | Some(Component::ShotResponse(comp))
    | Some(Component::DeathResponse(comp)) =
        ecs.get_component_from_entity(entity_id, response_type)
    {
        comp.data.process_event(event, ecs)
    } else {
        vec![]
    }
}
