use crate::{
    ecs::{
        archetypes::{make_unit_report, UnitReport},
        system::NavigationGrid,
        take_component_from_refs, Delta, IndexedData, ECS,
    },
    event::propagate_event,
    logger,
    los::line_of_sight,
    map::{Coordinate, GameMap},
};

use super::{Component, Diffable};

use core::fmt::Debug;
use std::cell::Cell;

#[derive(Debug, Clone, Copy)]
pub enum AIAction {
    Approach,
    Attack,
    Shoot,
    Flee,
    Wander,
    Sleep,
    Awake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIState {
    Alert,
    Sleeping(isize),
}

impl Default for AIState {
    fn default() -> Self {
        Self::Alert
    }
}

pub(crate) trait Behavior: Debug + CloneBehavior {
    fn select_action(
        &self,
        self_report: &UnitReport,
        player_report: &UnitReport,
        state: AIState,
        map: &GameMap,
        ecs: &ECS,
    ) -> AIAction;
}

#[derive(Debug, Clone)]
pub struct TurnTaker {
    pub(crate) behavior: Box<dyn Behavior>,
    pub(crate) state: AIState,
}

impl TurnTaker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_slow_melee() -> Self {
        Self {
            behavior: Box::new(SlowBehavior::default()),
            ..Default::default()
        }
    }

    pub fn new_melee() -> Self {
        Self {
            behavior: Box::new(MeleeBehavior::default()),
            ..Default::default()
        }
    }

    pub fn new_archer() -> Self {
        Self {
            behavior: Box::new(ArcherBehavior::default()),
            ..Default::default()
        }
    }

    pub fn process_turn(
        &self,
        components: &Vec<&Component>,
        ecs: &ECS,
        map: &GameMap,
        grid: &NavigationGrid,
    ) -> Vec<Delta> {
        let Some(player_report) = ecs.get_player_report() else {
            return vec![];
        };
        let player_index = ecs.get_player_id();

        let Some(self_report) = make_unit_report(components) else {
            return vec![];
        };

        match self
            .behavior
            .select_action(&self_report, &player_report, self.state, map, ecs)
        {
            AIAction::Approach => {
                return approach_player(&self_report.position, ecs, grid);
            }
            AIAction::Attack => {
                return propagate_event(&self_report.bump, player_index, ecs);
            }
            AIAction::Shoot => {
                return propagate_event(&self_report.shoot, player_index, ecs);
            }
            AIAction::Awake => {
                return wake_up(&self_report.position, ecs);
            }
            AIAction::Sleep => {
                return sleep(&self_report.position, ecs);
            }
            _ => {
                return vec![];
            }
        }
    }
}

impl Default for TurnTaker {
    fn default() -> Self {
        TurnTaker {
            behavior: Box::new(MeleeBehavior::default()),
            state: AIState::default(),
        }
    }
}

impl Diffable for TurnTaker {
    fn apply_diff(&mut self, other: &Self) {
        self.behavior = other.behavior.clone();
        self.state = other.state;
    }
}

#[derive(Debug, Clone, Default)]
struct MeleeBehavior {}

impl Behavior for MeleeBehavior {
    fn select_action(
        &self,
        self_report: &UnitReport,
        player_report: &UnitReport,
        state: AIState,
        _map: &GameMap,
        _ecs: &ECS,
    ) -> AIAction {
        let (my_pos, pl_pos) = (self_report.position.data, player_report.position.data);
        let distance = my_pos.distance(pl_pos);

        if let Some(action) = handle_sleep(state) {
            return action;
        }

        if distance > 1.1 {
            AIAction::Approach
        } else {
            AIAction::Attack
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ArcherBehavior {}

impl Behavior for ArcherBehavior {
    fn select_action(
        &self,
        self_report: &UnitReport,
        player_report: &UnitReport,
        state: AIState,
        map: &GameMap,
        ecs: &ECS,
    ) -> AIAction {
        let (my_pos, pl_pos) = (self_report.position.data, player_report.position.data);
        let my_range = self_report.combat.data.ranged.unwrap().max_range;

        if let Some(action) = handle_sleep(state) {
            return action;
        }

        if my_pos.distance(pl_pos) > my_range as f32 {
            AIAction::Approach
        } else if !line_of_sight(my_pos, pl_pos, map, ecs) {
            AIAction::Approach
        } else if my_pos.distance(pl_pos) <= 1.0 {
            AIAction::Attack
        } else {
            AIAction::Shoot
        }
    }
}

#[derive(Debug, Clone, Default)]
struct SlowBehavior {
    acted_last_turn: Cell<bool>,
}

impl Behavior for SlowBehavior {
    fn select_action(
        &self,
        self_report: &UnitReport,
        player_report: &UnitReport,
        state: AIState,
        _map: &GameMap,
        _ecs: &ECS,
    ) -> AIAction {
        let (my_pos, pl_pos) = (self_report.position.data, player_report.position.data);
        let distance = my_pos.distance(pl_pos);

        if let Some(action) = handle_sleep(state) {
            return action;
        }

        if self.acted_last_turn.get() {
            self.acted_last_turn.set(false);
            logger::log_message(&format!(
                "{} is reeling from impact.",
                self_report.name.as_ref().unwrap().data.raw
            ));
            AIAction::Sleep
        } else if distance > 1.1 {
            AIAction::Approach
        } else {
            self.acted_last_turn.set(true);
            AIAction::Attack
        }
    }
}

fn approach_player(
    my_pos: &IndexedData<Coordinate>,
    ecs: &ECS,
    grid: &NavigationGrid,
) -> Vec<Delta> {
    let direction = grid.get(&my_pos.data);

    if let Some(&coord) = direction {
        if !ecs.is_blocked_by_entity(my_pos.data + coord) {
            vec![Delta::Change(Component::Position(
                my_pos.make_change(coord),
            ))]
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

fn wake_up(my_pos: &IndexedData<Coordinate>, ecs: &ECS) -> Vec<Delta> {
    let entity_id = ecs.get_entity_id_from_component_id(my_pos.index).unwrap();
    let Some(Component::Turn(data)) =
        ecs.get_component_from_entity(entity_id, crate::ecs::ComponentType::Turn)
    else {
        println!("Tried to wake up entity and failed");
        return vec![];
    };

    let new_turn = TurnTaker {
        state: AIState::Alert,
        ..data.data.clone()
    };

    vec![Delta::Change(Component::Turn(data.make_change(new_turn)))]
}

fn sleep(my_pos: &IndexedData<Coordinate>, ecs: &ECS) -> Vec<Delta> {
    let entity_id = ecs.get_entity_id_from_component_id(my_pos.index).unwrap();
    let Some(Component::Turn(data)) =
        ecs.get_component_from_entity(entity_id, crate::ecs::ComponentType::Turn)
    else {
        return vec![];
    };
    let AIState::Sleeping(old_duration) = data.data.state else {
        return vec![];
    };
    let new_turn = TurnTaker {
        state: AIState::Sleeping((old_duration - 1).max(-1)),
        ..data.data.clone()
    };

    vec![Delta::Change(Component::Turn(data.make_change(new_turn)))]
}

pub(crate) trait CloneBehavior {
    fn clone_behavior<'a>(&self) -> Box<dyn Behavior>;
}

impl<T> CloneBehavior for T
where
    T: Behavior + Clone + 'static,
{
    fn clone_behavior(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Behavior> {
    fn clone(&self) -> Self {
        self.clone_behavior()
    }
}

fn handle_sleep(state: AIState) -> Option<AIAction> {
    match state {
        AIState::Sleeping(i) if i == 1 => Some(AIAction::Awake),
        AIState::Sleeping(_) => Some(AIAction::Sleep),
        _ => None,
    }
}
