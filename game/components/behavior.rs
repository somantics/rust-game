use core::fmt::Debug;
use std::{borrow::BorrowMut, cell::Cell};
use rand::thread_rng;
use rand::seq::SliceRandom;

use crate::{
    ecs::{
        component::Diffable,
        ecs::{Delta, IndexedData, ECS},
        event::{propagate_event, InteractionEvent},
    },
    game::{
        archetype::{make_unit_report, UnitReport},
        components::core::*,
        system::NavigationGrid,
    },
    map::{self, gamemap::GameMap, utils::{Coordinate, Euclidian}},
    utils::{logger, los::line_of_sight},
};

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
    ) -> Vec<AIAction>;
}

#[derive(Debug, Clone)]
pub struct TurnTaker {
    pub(crate) behavior: Box<dyn Behavior>,
    pub(crate) state: AIState,
    pub(crate) avoid_hazards: bool,
}

impl TurnTaker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_slow_melee(avoid_hazards: bool) -> Self {
        Self {
            behavior: Box::new(SlowBehavior::default()),
            avoid_hazards,
            ..Default::default()
        }
    }

    pub fn new_fast_melee(avoid_hazards: bool) -> Self {
        Self {
            behavior: Box::new(FastMeleeBehavior::default()),
            avoid_hazards,
            ..Default::default()
        }
    }

    pub fn new_melee(avoid_hazards: bool) -> Self {
        Self {
            behavior: Box::new(MeleeBehavior::default()),
            avoid_hazards,
            ..Default::default()
        }
    }

    pub fn new_archer() -> Self {
        Self {
            behavior: Box::new(ArcherBehavior::default()),
            ..Default::default()
        }
    }

    pub fn new_mage(avoid_hazards: bool) -> Self {
        Self {
            behavior: Box::new(TrueSightArcherBehavior::default()),
            avoid_hazards,
            ..Default::default()
        }
    }

    pub fn new_wander(delay: usize) -> Self {
        Self {
            behavior: Box::new(WanderBehavior::new(delay)),
            avoid_hazards: true,
            ..Default::default()
        }
    }

    pub fn process_turn(
        &self,
        components: &[&Component],
        ecs: &ECS,
        map: &GameMap,
        safe_grid: &NavigationGrid,
        hazard_grid: &NavigationGrid,
    ) -> Vec<Delta> {
        let Some(player_report) = ecs.get_player_report() else {
            return vec![];
        };
        let Some(mut self_report) = make_unit_report(components) else {
            return vec![];
        };
        let grid = match self.avoid_hazards {
            true => safe_grid,
            false => hazard_grid,
        };
        let player_index = ecs.get_player_id();

        let mut output: Vec<Delta> = Vec::new(); 
        let actions = self.behavior.select_action(&self_report, &player_report, self.state, map, ecs);
        for action in actions {
            let deltas = match action {
                AIAction::Approach => {
                    let (deltas, dir) = approach_player(&self_report.position, &self_report.bump, ecs, grid);
                    self_report.position.data += dir;
                    deltas
                }
                AIAction::Flee => {
                    let (deltas, dir) = flee(&self_report.position, &self_report.bump, ecs, map, grid);
                    self_report.position.data += dir;
                    deltas
                }
                AIAction::Attack => {
                    propagate_event(&self_report.bump, player_index, ecs)
                }
                AIAction::Shoot => {
                    propagate_event(&self_report.shoot, player_index, ecs)
                }
                AIAction::Awake => {
                    wake_up(&self_report.position, ecs)
                }
                AIAction::Wander => {
                    let (deltas, dir) = wander(&self_report.position, ecs, map);
                    self_report.position.data += dir;
                    deltas
                }
                AIAction::Sleep => {
                    sleep(&self_report.position, ecs)
                }
                _ => {
                    vec![]
                }
            };
            output.extend(deltas.into_iter());
        }
        output
    }
}

impl Default for TurnTaker {
    fn default() -> Self {
        TurnTaker {
            behavior: Box::new(MeleeBehavior::default()),
            state: AIState::default(),
            avoid_hazards: false,
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
struct MeleeBehavior { }

impl Behavior for MeleeBehavior {
    fn select_action(
        &self,
        self_report: &UnitReport,
        player_report: &UnitReport,
        state: AIState,
        map: &GameMap,
        ecs: &ECS,
    ) -> Vec<AIAction> {
        let (my_pos, pl_pos) = (self_report.position.data, player_report.position.data);
        let distance = my_pos.distance(pl_pos);

        if let Some(Component::DurationEffect(indexed_data)) = ecs.get_component_from_entity_id(ecs.get_player_id(), ComponentType::DurationEffect) {
            if let DurationEffect(_, EffectType::Invisible) = indexed_data.data {
                if line_of_sight(my_pos, pl_pos, map, ecs) {
                    return vec![AIAction::Wander]
                } else {
                    return vec![AIAction::Sleep]
                }
            }
        }

        if let Some(action) = handle_sleep(state) {
            return vec![action];
        }

        if distance > 1.1 {
            vec![AIAction::Approach]
        } else {
            vec![AIAction::Attack]
        }
    }
}

#[derive(Debug, Clone, Default)]
struct FastMeleeBehavior { }

impl Behavior for FastMeleeBehavior {
    fn select_action(
        &self,
        self_report: &UnitReport,
        player_report: &UnitReport,
        state: AIState,
        map: &GameMap,
        ecs: &ECS,
    ) -> Vec<AIAction> {
        let (my_pos, pl_pos) = (self_report.position.data, player_report.position.data);
        let distance = my_pos.distance(pl_pos);

        if let Some(Component::DurationEffect(indexed_data)) = ecs.get_component_from_entity_id(ecs.get_player_id(), ComponentType::DurationEffect) {
            if let DurationEffect(_, EffectType::Invisible) = indexed_data.data {
                if line_of_sight(my_pos, pl_pos, map, ecs) {
                    return vec![AIAction::Wander]
                } else {
                    return vec![AIAction::Sleep]
                }
            }
        }

        if let Some(action) = handle_sleep(state) {
            return vec![action];
        }

        if distance > 2.1 {
            vec![AIAction::Approach, AIAction::Approach]
        } else if distance > 1.1 {
            vec![AIAction::Approach, AIAction::Attack]
        } else {
            vec![AIAction::Attack, AIAction::Wander]
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
    ) -> Vec<AIAction> {
        let (my_pos, pl_pos) = (self_report.position.data, player_report.position.data);
        let my_range = self_report.combat.data.ranged.unwrap().max_range;

        if let Some(Component::DurationEffect(indexed_data)) = ecs.get_component_from_entity_id(ecs.get_player_id(), ComponentType::DurationEffect) {
            if let DurationEffect(_, EffectType::Invisible) = indexed_data.data {
                if line_of_sight(my_pos, pl_pos, map, ecs) {
                    return vec![AIAction::Wander]
                } else {
                    return vec![AIAction::Sleep]
                }
            }
        }

        if let Some(action) = handle_sleep(state) {
            return vec![action];
        }

        if my_pos.distance(pl_pos) > my_range as f32 {
            vec![AIAction::Approach]
        } else if !line_of_sight(my_pos, pl_pos, map, ecs) {
            vec![AIAction::Approach]
        } else if my_pos.distance(pl_pos) <= 1.0 {
            vec![AIAction::Attack]
        } else {
            vec![AIAction::Shoot]
        }
    }
}

#[derive(Debug, Clone, Default)]
struct TrueSightArcherBehavior {}

impl Behavior for TrueSightArcherBehavior {
    fn select_action(
        &self,
        self_report: &UnitReport,
        player_report: &UnitReport,
        state: AIState,
        map: &GameMap,
        ecs: &ECS,
    ) -> Vec<AIAction> {
        let (my_pos, pl_pos) = (self_report.position.data, player_report.position.data);
        let my_range = self_report.combat.data.ranged.unwrap().max_range;

        if let Some(action) = handle_sleep(state) {
            return vec![action];
        }

        if my_pos.distance(pl_pos) > my_range as f32 {
            vec![AIAction::Approach]
        } else if !line_of_sight(my_pos, pl_pos, map, ecs) {
            vec![AIAction::Approach]
        } else if my_pos.distance(pl_pos) <= 1.0 {
            vec![AIAction::Attack]
        } else {
            vec![AIAction::Shoot]
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
        map: &GameMap,
        ecs: &ECS,
    ) -> Vec<AIAction> {
        let (my_pos, pl_pos) = (self_report.position.data, player_report.position.data);
        let distance = my_pos.distance(pl_pos);

        if let Some(Component::DurationEffect(indexed_data)) = ecs.get_component_from_entity_id(ecs.get_player_id(), ComponentType::DurationEffect) {
            if let DurationEffect(_, EffectType::Invisible) = indexed_data.data {
                if line_of_sight(my_pos, pl_pos, map, ecs) {
                    return vec![AIAction::Wander]
                } else {
                    return vec![AIAction::Sleep]
                }
            }
        }

        if let Some(action) = handle_sleep(state) {
            return vec![action];
        }

        if self.acted_last_turn.get() {
            self.acted_last_turn.set(false);
            logger::log_message(&format!(
                "{} is reeling from impact.",
                self_report.name.as_ref().unwrap().data.raw
            ));
            vec![AIAction::Sleep]
        } else if distance > 1.1 {
            vec![AIAction::Approach]
        } else {
            self.acted_last_turn.set(true);
            vec![AIAction::Attack]
        }
    }
}

#[derive(Debug, Clone)]
struct WanderBehavior {
    wander_counter: Cell<usize>,
    wander_threshold: usize,
}

impl WanderBehavior {
    fn new(delay: usize) -> Self {
        Self { wander_counter: Cell::new(0), wander_threshold: delay }
    }
}

impl Behavior for WanderBehavior {
    fn select_action(
        &self,
        self_report: &UnitReport,
        player_report: &UnitReport,
        state: AIState,
        _map: &GameMap,
        _ecs: &ECS,
    ) -> Vec<AIAction> {
        if let Some(action) = handle_sleep(state) {
            return vec![action];
        }
        let (my_pos, pl_pos) = (self_report.position.data, player_report.position.data);
        if my_pos.distance(pl_pos) < 2.0 {
            return vec![AIAction::Flee];
        }

        if self.wander_counter.get() >= self.wander_threshold {
            self.wander_counter.replace(0);
            vec![AIAction::Wander]
        } else {
            self.wander_counter.replace(self.wander_counter.get() + 1);
            vec![AIAction::Sleep]
        }
    }
}

impl Default for WanderBehavior {
    fn default() -> Self {
        Self { 
            wander_counter: Cell::new(0), 
            wander_threshold: 3 
        }
    }
}

fn approach_player(
    my_pos: &IndexedData<Coordinate>,
    my_bump: &InteractionEvent,
    ecs: &ECS,
    grid: &NavigationGrid,
) -> (Vec<Delta>, Coordinate) {
    let direction = grid.get(&my_pos.data);

    if let Some(&dir) = direction {
        if !ecs.is_blocked_by_entity(my_pos.data + dir) {
            // propagate bump event to everything on the space
            let entities = ecs.get_all_entities_in_tile(my_pos.data + dir);
            let mut deltas: Vec<Delta> = entities
                .into_iter()
                .map(|entity_id| propagate_event(my_bump, entity_id, ecs))
                .flatten()
                .collect();
            deltas.push(
                Delta::Change(Component::Position(my_pos.make_change(dir)))
            );
            (deltas, dir)
        } else {
            (vec![], Coordinate::default())
        }
    } else {
        (vec![], Coordinate::default())
    }
}

fn flee(
    my_pos: &IndexedData<Coordinate>,
    my_bump: &InteractionEvent,
    ecs: &ECS,
    map: &GameMap,
    grid: &NavigationGrid,
) -> (Vec<Delta>, Coordinate) {
    let direction = grid.get(&my_pos.data);

    if let Some(&dir) = direction {
        let dir = map::utils::reverse_direction(&dir);
        let destination = my_pos.data + dir;
        if !ecs.is_blocked_by_entity(destination) 
            && map.is_tile_passable(destination)
        {
            // propagate bump event to everything on the space without attacking
            let bump = InteractionEvent {
                attack: None,
                ..my_bump.clone()
            };
            let entities = ecs.get_all_entities_in_tile(destination);
            let mut deltas: Vec<Delta> = entities
                .into_iter()
                .map(|entity_id| propagate_event(&bump, entity_id, ecs))
                .flatten()
                .collect();

            // Order the move
            deltas.push(
                Delta::Change(Component::Position(my_pos.make_change(dir)))
            );
            (deltas, dir)
        } else {
            (vec![], Coordinate::default())
        }
    } else {
        (vec![], Coordinate::default())
    }
}

fn wake_up(my_pos: &IndexedData<Coordinate>, ecs: &ECS) -> Vec<Delta> {
    let entity_id = ecs.get_entity_id_from_component_id(my_pos.index).unwrap();
    let Some(Component::Turn(data)) = ecs.get_component_from_entity_id(entity_id, ComponentType::Turn)
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

fn wander(
    my_pos: &IndexedData<Coordinate>,
    ecs: &ECS,
    map: &GameMap,
) -> (Vec<Delta>, Coordinate) {
    let direction = [
        map::utils::UP, 
        map::utils::DOWN,
        map::utils::LEFT,
        map::utils::RIGHT,
    ].choose(thread_rng().borrow_mut());

    if let Some(&dir) = direction {
        let destination = my_pos.data + dir;
        if !ecs.is_blocked_by_entity(destination) 
            && map.is_tile_passable(destination)
        {
            (vec![Delta::Change(Component::Position(my_pos.make_change(dir)))], dir)
        } else {
            (vec![], Coordinate::default())
        }
    } else {
        (vec![], Coordinate::default())
    }
}

fn sleep(my_pos: &IndexedData<Coordinate>, ecs: &ECS) -> Vec<Delta> {
    let entity_id = ecs.get_entity_id_from_component_id(my_pos.index).unwrap();
    let Some(Component::Turn(data)) = ecs.get_component_from_entity_id(entity_id, ComponentType::Turn)
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
