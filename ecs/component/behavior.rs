use crate::{
    ecs::{
        archetypes::{make_unit_report, UnitReport}, system::NavigationGrid, Delta, IndexedData, ECS
    }, event::propagate_event, logger, los::line_of_sight, map::{pathfinding::{self, pathfind}, Coordinate, GameMap}
};

use super::Component;

use core::fmt::Debug;
use std::cell::Cell;

#[derive(Debug, Clone, Copy)]
enum AIAction {
    Approach,
    Attack,
    Shoot,
    Flee,
    Wander,
    Sleep,
    Awake,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum AIState {
    #[default]
    Alert,
    Sleeping,
}

trait Behavior: Debug + CloneBehavior {
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
    behavior: Box<dyn Behavior>,
    state: AIState,
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

    pub fn process_turn(&self, components: &Vec<&Component>, ecs: &ECS, map: &GameMap, grid: &NavigationGrid) -> Vec<Delta> {
        let player_report = ecs.get_player_report().unwrap();
        let player_index = ecs.get_player_id();

        let self_report = make_unit_report(components);

        match self.behavior.select_action(&self_report, &player_report, self.state, map, ecs) {
            AIAction::Approach => {
                return approach_player(
                    &self_report.position,
                    player_report.position.data,
                    ecs,
                    map,
                    grid
                );
            }
            AIAction::Attack => {
                return propagate_event(&self_report.bump, player_index, ecs);
            }
            AIAction::Shoot => {
                return propagate_event(&self_report.shoot, player_index, ecs);
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

#[derive(Debug, Clone, Default)]
struct MeleeBehavior { }

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
        
        if state == AIState::Sleeping {
            AIAction::Sleep
        } else if distance > 1.1 {
            AIAction::Approach
        } else {
            AIAction::Attack
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ArcherBehavior { }

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

        if state == AIState::Sleeping {
            AIAction::Sleep
        } else if my_pos.distance(pl_pos) > my_range as f32 {
            AIAction::Approach
        } else if !line_of_sight(my_pos, pl_pos, map, ecs) {
            AIAction::Sleep
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

            if state == AIState::Sleeping {
                AIAction::Sleep
            } else if self.acted_last_turn.get() {
                self.acted_last_turn.set(false);
                logger::log_message(&format!("{} is reeling from impact.", self_report.name.as_ref().unwrap().data.raw));
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
    pl_pos: Coordinate,
    ecs: &ECS,
    map: &GameMap,
    grid: &NavigationGrid
) -> Vec<Delta> {

    let direction = grid.get(&my_pos.data);

    // let path = pathfind(
    //     my_pos.data, 
    //     pl_pos, 
    //     map, 
    //     ecs, 
    //     pathfinding::astar_heuristic_factory(pl_pos), 
    //     true, 
    //     true
    // );

    // let direction = match path {
    //     Some(mut vec) => vec.pop(),
    //     _ => None,
    // };

    if let Some(&coord) = direction {
        if !ecs.is_blocked_by_entity(my_pos.data + coord) {
            vec![Delta::Change(Component::Position(my_pos.make_change(coord)))]
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

trait CloneBehavior {
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