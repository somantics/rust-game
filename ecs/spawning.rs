use crate::ecs::*;
use crate::map::Coordinate;
use crate::{ecs::component::*, event::EventResponse};
use behavior::TurnTaker;
use phf::phf_map;
use rand::{thread_rng, Rng};

use self::combat::{Attack, Health};

const ENEMY_HP_INCREASE: f64 = 0.2;
const GOLD_INCREASE: f64 = 0.1;

pub static OBJECT_SPAWN_NAMES: phf::Map<&'static str, fn(&mut ECS, Coordinate, usize)> = phf_map!(
    "Doggo" => make_doggo,
    "Heavy" => make_heavy,
    "Pewpew" => make_cultist,
    "Pewpewpet" => make_skelly,
    "Player" => make_player,
    "Chest" => make_chest,
    "Gold" => make_gold_pile,
    "Door" => make_door,
    "StairsDown" => make_stairs_down,
    "Corpse" => make_lootable_body,
);

pub fn make_player(ecs: &mut ECS, start: Coordinate, _depth: usize) {
    let player_combat = Combat::new(
        Some(Attack::new_melee(1, 7)),
        Some(Attack::new_ranged(2, 1)),
    );
    
    let player_health = Health {
        current: 10,
        max: 10,
    };
    let player_inventory = Inventory { coins: 0 };
    let player_stats = Attributes {
        strength: 5,
        dexterity: 5,
        level: 1,
        ..Default::default()
    };

    let take_damage = EventResponse::new_with(component::responses::take_damage_response);

    let components = vec![
        Component::Player(IndexedData::new_with(())),
        Component::Name(IndexedData::new_with(Name::new("Bartholomew"))),
        Component::Image(IndexedData::new_with(ImageHandle::new(ImageData {
            id: 3,
            depth: 5,
        }))),
        Component::Position(IndexedData::new_with(start)),
        Component::Combat(IndexedData::new_with(player_combat)),
        Component::Health(IndexedData::new_with(player_health)),
        Component::Inventory(IndexedData::new_with(player_inventory)),
        Component::Collision(IndexedData::new_with(Collision::Blocking)),
        Component::Attributes(IndexedData::new_with(player_stats)),
        Component::BumpResponse(IndexedData::new_with(take_damage.clone())),
        Component::ShotResponse(IndexedData::new_with(take_damage)),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_doggo(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let melee = match depth {
        0..=4 => Attack::new_melee(1, 2),
        5..=9 => Attack::new_melee(2, 2),
        10..=14 => Attack::new_melee(3, 3),
        _ => Attack::new_melee(4, 4),
    };
    let combat = Combat::new(Some(melee), None);
    let depth = depth as f64;
    let health =
        (thread_rng().gen_range(6..=9) as f64 * (1.0 + ENEMY_HP_INCREASE * (depth - 1.0))) as isize;
    let health = Health::new(health);
    let image = ImageData { id: 6, depth: 5 };

    let take_damage = EventResponse::new_with(component::responses::take_damage_response);

    let components = vec![
        Component::Monster(IndexedData::new_with(())),
        Component::Name(IndexedData::new_with(Name::new("Doggo"))),
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Position(IndexedData::new_with(start)),
        Component::Combat(IndexedData::new_with(combat)),
        Component::Health(IndexedData::new_with(health)),
        Component::Collision(IndexedData::new_with(Collision::Blocking)),
        Component::BumpResponse(IndexedData::new_with(take_damage.clone())),
        Component::ShotResponse(IndexedData::new_with(take_damage)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_melee())),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_heavy(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let melee = match depth {
        0..=4 => Attack::new_melee(2, 3),
        5..=9 => Attack::new_melee(3, 4),
        10..=14 => Attack::new_melee(4, 5),
        _ => Attack::new_melee(5, 6),
    };
    let combat = Combat::new(Some(melee), None);
    let depth = depth as f64;
    let health = (thread_rng().gen_range(13..=15) as f64
        * (1.0 + ENEMY_HP_INCREASE * (depth - 1.0))) as isize;
    let health = Health::new(health);
    let image = ImageData { id: 11, depth: 5 };
    let take_damage = EventResponse::new_with(component::responses::take_damage_response);

    let components = vec![
        Component::Monster(IndexedData::new_with(())),
        Component::Name(IndexedData::new_with(Name::new("Boar"))),
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Position(IndexedData::new_with(start)),
        Component::Combat(IndexedData::new_with(combat)),
        Component::Health(IndexedData::new_with(health)),
        Component::Collision(IndexedData::new_with(Collision::Blocking)),
        Component::BumpResponse(IndexedData::new_with(take_damage.clone())),
        Component::ShotResponse(IndexedData::new_with(take_damage)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_slow_melee())),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_skelly(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let melee = match depth {
        0..=4 => Attack::new_melee(1, 2),
        5..=9 => Attack::new_melee(2, 2),
        10..=14 => Attack::new_melee(3, 2),
        _ => Attack::new_melee(4, 2),
    };
    let combat = Combat::new(Some(melee), None);
    let depth = depth as f64;
    let health = (thread_rng().gen_range(7..=10) as f64 * (1.0 + ENEMY_HP_INCREASE * (depth - 1.0)))
        as isize;
    let health = Health::new(health);
    let image = ImageData { id: 13, depth: 5 };
    let coins = (thread_rng().gen_range(2..=15) as f64 * (1.0 + GOLD_INCREASE * depth)) as isize;
    let inventory = Inventory { coins };

    let take_damage = EventResponse::new_with(responses::take_damage_response);
    let take_half_damage = EventResponse::new_with(responses::take_half_damage_response);
    let drop_coins = EventResponse::new_with(responses::drop_inventory_response);

    let components = vec![
        Component::Monster(IndexedData::new_with(())),
        Component::Name(IndexedData::new_with(Name::new("Skeleton"))),
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Position(IndexedData::new_with(start)),
        Component::Combat(IndexedData::new_with(combat)),
        Component::Health(IndexedData::new_with(health)),
        Component::Inventory(IndexedData::new_with(inventory)),
        Component::Collision(IndexedData::new_with(Collision::Blocking)),
        Component::BumpResponse(IndexedData::new_with(take_damage)),
        Component::ShotResponse(IndexedData::new_with(take_half_damage)),
        Component::DeathResponse(IndexedData::new_with(drop_coins)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_melee())),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_cultist(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let melee = match depth {
        0..=4 => Attack::new_melee(2, 2),
        5..=9 => Attack::new_melee(3, 2),
        10..=14 => Attack::new_melee(4, 3),
        _ => Attack::new_melee(5, 3),
    };
    let ranged = match depth {
        0..=4 => Attack::new_ranged(1, 2),
        5..=9 => Attack::new_ranged(2, 2),
        10..=14 => Attack::new_ranged(3, 2),
        _ => Attack::new_ranged(4, 2),
    };
    let ranged = Attack {
        max_range: 3,
        ..ranged
    };
    let combat = Combat::new(Some(melee), Some(ranged));
    let depth = depth as f64;
    let health = (thread_rng().gen_range(8..=10) as f64 * (1.0 + ENEMY_HP_INCREASE * (depth - 1.0)))
        as isize;
    let health = Health::new(health);
    let image = ImageData { id: 12, depth: 5 };
    let depth = depth as f64;
    let coins = (thread_rng().gen_range(18..=25) as f64 * (1.0 + GOLD_INCREASE * depth)) as isize;
    let inventory = Inventory { coins };

    let take_damage = EventResponse::new_with(responses::take_damage_response);
    let drop_coins = EventResponse::new_with(responses::drop_inventory_response);

    let components = vec![
        Component::Monster(IndexedData::new_with(())),
        Component::Name(IndexedData::new_with(Name::new("Cultist"))),
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Position(IndexedData::new_with(start)),
        Component::Combat(IndexedData::new_with(combat)),
        Component::Health(IndexedData::new_with(health)),
        Component::Inventory(IndexedData::new_with(inventory)),
        Component::Collision(IndexedData::new_with(Collision::Blocking)),
        Component::BumpResponse(IndexedData::new_with(take_damage.clone())),
        Component::ShotResponse(IndexedData::new_with(take_damage)),
        Component::DeathResponse(IndexedData::new_with(drop_coins)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_archer())),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_door(ecs: &mut ECS, start: Coordinate, _depth: usize) {
    let open_image = ImageData { id: 10, depth: 7 };
    let closed_image = ImageData { id: 9, depth: 7 };
    let images = ImageHandle {
        current: closed_image.to_owned(),
        states: HashMap::from([("open", open_image), ("closed", closed_image)]),
    };

    let event_response = EventResponse::new_with(component::responses::open_door_response);

    let components = vec![
        Component::Door(IndexedData::new_with(())),
        Component::Image(IndexedData::new_with(images)),
        Component::Position(IndexedData::new_with(start)),
        Component::Collision(IndexedData::new_with(component::Collision::Blocking)),
        Component::LineOfSight(IndexedData::new_with(component::LoSBlocking::Blocking)),
        Component::BumpResponse(IndexedData::new_with(event_response)),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_chest(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let open_image = ImageData { id: 8, depth: 7 };
    let closed_image = ImageData { id: 7, depth: 7 };
    let images = ImageHandle {
        current: closed_image.to_owned(),
        states: HashMap::from([("open", open_image), ("closed", closed_image)]),
    };
    let depth = depth as f64;
    let coins = (thread_rng().gen_range(25..=52) as f64 * (1.0 + GOLD_INCREASE * depth)) as isize;
    let inventory = Inventory { coins };
    let event_response = EventResponse::new_with(responses::open_chest_response);

    let components = vec![
        Component::Image(IndexedData::new_with(images)),
        Component::Position(IndexedData::new_with(start)),
        Component::Inventory(IndexedData::new_with(inventory)),
        Component::Collision(IndexedData::new_with(component::Collision::Blocking)),
        Component::LineOfSight(IndexedData::new_with(component::LoSBlocking::Partial)),
        Component::BumpResponse(IndexedData::new_with(event_response)),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_lootable_body(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let image = ImageData { id: 14, depth: 6 };
    let depth = depth as f64;
    let coins = (thread_rng().gen_range(5..=18) as f64 * (1.0 + GOLD_INCREASE * depth)) as isize;
    let inventory = Inventory { coins };
    let award_coins = EventResponse::new_with(responses::pickup_loot_response);

    let components = vec![
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Position(IndexedData::new_with(start)),
        Component::Collision(IndexedData::new_with(component::Collision::Walkable)),
        Component::Inventory(IndexedData::new_with(inventory)),
        Component::BumpResponse(IndexedData::new_with(award_coins)),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_gold_pile(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let image = ImageData { id: 15, depth: 6 };
    let depth = depth as f64;
    let coins = (thread_rng().gen_range(9..=25) as f64 * (1.0 + GOLD_INCREASE * depth)) as isize;
    let inventory = Inventory { coins };
    let award_coins = EventResponse::new_with(responses::pickup_loot_response);

    let components = vec![
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Position(IndexedData::new_with(start)),
        Component::Collision(IndexedData::new_with(component::Collision::Walkable)),
        Component::Inventory(IndexedData::new_with(inventory)),
        Component::BumpResponse(IndexedData::new_with(award_coins)),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_stairs_down(ecs: &mut ECS, start: Coordinate, _depth: usize) {
    let image = ImageData { id: 16, depth: 7 };

    let components = vec![
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Stairs(IndexedData::new_with(())),
        Component::Position(IndexedData::new_with(start)),
        Component::Collision(IndexedData::new_with(component::Collision::Walkable)),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}
