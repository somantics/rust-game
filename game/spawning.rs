use std::collections::HashMap;

use phf::phf_map;
use rand::{thread_rng, Rng};

use crate::{
    ecs::ecs::{IndexedData, ECS},
    ecs::event::EventResponse,
    game::components::attributes::Attributes,
    game::components::behavior::TurnTaker,
    game::components::combat::{Attack, Combat, Health},
    game::components::core::*,
    game::components::inventory::Inventory,
    game::responses,
    map::utils::Coordinate,
};

use super::{responses::{retaliate_response, spikes_response, spread_acid_response, spread_fire_response}, spelldefinitions::{self, SPELL_REGISTRY}};

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
    "Spikes" => make_spikes,
    "Fire" => make_flame,
    "Acid pool" => make_acid,
    "Fungus" => make_mushroom,
    "Rat" => make_rat,
    "Critters" => make_critter,
    "Bat" => make_bat,
);

pub fn make_player(ecs: &mut ECS, start: Coordinate, _depth: usize) {
    let player_combat = Combat::new(
        Some(Attack::new_melee(1, 7)),
        Some(Attack::new_ranged(2, 0)),
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

    let player_image = ImageHandle::new(ImageData {
        id: 3,
        depth: 5,
    });

    let take_damage = EventResponse::new_with(responses::take_damage_response);
    let flammable = EventResponse::new_with(responses::default_burn_response);

    let components = vec![
        Component::Player(IndexedData::new_with(())),
        Component::Name(IndexedData::new_with(Name::new("Bartholomew"))),
        Component::Image(IndexedData::new_with(player_image)),
        Component::Position(IndexedData::new_with(start)),
        Component::Combat(IndexedData::new_with(player_combat)),
        Component::Health(IndexedData::new_with(player_health)),
        Component::Inventory(IndexedData::new_with(player_inventory)),
        Component::Collision(IndexedData::new_with(Collision::Blocking)),
        Component::Attributes(IndexedData::new_with(player_stats)),
        Component::BumpResponse(IndexedData::new_with(take_damage.clone())),
        Component::ShotResponse(IndexedData::new_with(take_damage)),
        Component::FireResponse(IndexedData::new_with(flammable)),
        Component::Spell(IndexedData::new_with(spelldefinitions::FLAMES.with(|spell| spell.clone()))),
        Component::Spell(IndexedData::new_with(spelldefinitions::BRITTLE.with(|spell| spell.clone()))),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_doggo(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let melee = match depth {
        0 => Attack::new_melee(1, 1),
        0..=4 => Attack::new_melee(1, 2),
        5..=9 => Attack::new_melee(2, 3),
        10..=14 => Attack::new_melee(3, 3),
        _ => Attack::new_melee(4, 4),
    };
    let combat = Combat::new(Some(melee), None);
    let depth = depth as f64;
    let health =
        (thread_rng().gen_range(6..=9) as f64 * (1.0 + ENEMY_HP_INCREASE * (depth - 1.0))) as isize;
    let health = Health::new(health);
    let image = ImageData { id: 6, depth: 5 };

    let take_damage = EventResponse::new_with(responses::take_damage_response);
    let flammable = EventResponse::new_with(responses::default_burn_response);

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
        Component::FireResponse(IndexedData::new_with(flammable)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_melee(true))),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_bat(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let melee = match depth {
        0 => Attack::new_melee(1, 1),
        0..=4 => Attack::new_melee(1, 1),
        5..=9 => Attack::new_melee(2, 1),
        10..=14 => Attack::new_melee(2, 2),
        _ => Attack::new_melee(3, 2),
    };
    let combat = Combat::new(Some(melee), None);
    let depth = depth as f64;
    let health =
        (thread_rng().gen_range(4..=6) as f64 * (1.0 + ENEMY_HP_INCREASE * (depth - 1.0))) as isize;
    let health = Health::new(health);
    let image = ImageData { id: 23, depth: 5 };

    let take_damage = EventResponse::new_with(responses::take_damage_response);
    let flammable = EventResponse::new_with(responses::default_burn_response);

    let components = vec![
        Component::Monster(IndexedData::new_with(())),
        Component::Name(IndexedData::new_with(Name::new("Bat"))),
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Position(IndexedData::new_with(start)),
        Component::Combat(IndexedData::new_with(combat)),
        Component::Health(IndexedData::new_with(health)),
        Component::Collision(IndexedData::new_with(Collision::Blocking)),
        Component::BumpResponse(IndexedData::new_with(take_damage.clone())),
        Component::ShotResponse(IndexedData::new_with(take_damage)),
        Component::FireResponse(IndexedData::new_with(flammable)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_fast_melee(false))),
        Component::DurationEffect(IndexedData::new_with(DurationEffect(-1, EffectType::Levitate))),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_heavy(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let melee = match depth {
        0..=4 => Attack::new_melee(3, 3),
        5..=9 => Attack::new_melee(4, 4),
        10..=14 => Attack::new_melee(5, 5),
        _ => Attack::new_melee(6, 6),
    };
    let combat = Combat::new(Some(melee), None);
    let depth = depth as f64;
    let health = (thread_rng().gen_range(13..=15) as f64
        * (1.0 + ENEMY_HP_INCREASE * (depth - 1.0))) as isize;
    let health = Health::new(health);
    let image = ImageData { id: 11, depth: 5 };
    let take_damage = EventResponse::new_with(responses::take_damage_response);
    let flammable = EventResponse::new_with(responses::default_burn_response);

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
        Component::FireResponse(IndexedData::new_with(flammable)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_slow_melee(true))),
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
    let flammable = EventResponse::new_with(responses::default_burn_response);

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
        Component::FireResponse(IndexedData::new_with(flammable)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_melee(false))),
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
        max_range: 3.0,
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
    let flammable = EventResponse::new_with(responses::default_burn_response);

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
        Component::FireResponse(IndexedData::new_with(flammable)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_mage(true))),
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
    let health = Health::new(6);

    let event_response = EventResponse::new_with(responses::open_door_response);
    let flammable = EventResponse::new_with(responses::default_burn_response);

    let components = vec![
        Component::Door(IndexedData::new_with(())),
        Component::Image(IndexedData::new_with(images)),
        Component::Position(IndexedData::new_with(start)),
        Component::Health(IndexedData::new_with(health)),
        Component::Collision(IndexedData::new_with(Collision::Blocking)),
        Component::LineOfSight(IndexedData::new_with(LoSBlocking::Blocking)),
        Component::BumpResponse(IndexedData::new_with(event_response)),
        Component::FireResponse(IndexedData::new_with(flammable)),
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
    let health = Health::new(5);

    let depth = depth as f64;
    let coins = (thread_rng().gen_range(25..=52) as f64 * (1.0 + GOLD_INCREASE * depth)) as isize;
    let inventory = Inventory { coins };
    let event_response = EventResponse::new_with(responses::open_chest_response);
    let drop_coins = EventResponse::new_with(responses::drop_inventory_response);
    let flammable = EventResponse::new_with(responses::default_burn_response);

    let components = vec![
        Component::Image(IndexedData::new_with(images)),
        Component::Position(IndexedData::new_with(start)),
        Component::Health(IndexedData::new_with(health)),
        Component::Inventory(IndexedData::new_with(inventory)),
        Component::Collision(IndexedData::new_with(Collision::Blocking)),
        Component::LineOfSight(IndexedData::new_with(LoSBlocking::Partial)),
        Component::BumpResponse(IndexedData::new_with(event_response)),
        Component::DeathResponse(IndexedData::new_with(drop_coins)),
        Component::FireResponse(IndexedData::new_with(flammable)),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_lootable_body(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let image = ImageData { id: 14, depth: 6 };
    let depth = depth as f64;
    let health = Health::new(2);
    let coins = (thread_rng().gen_range(5..=18) as f64 * (1.0 + GOLD_INCREASE * depth)) as isize;
    let inventory = Inventory { coins };
    let award_coins = EventResponse::new_with(responses::pickup_loot_response);
    let flammable = EventResponse::new_with(responses::default_burn_response);

    let components = vec![
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Position(IndexedData::new_with(start)),
        Component::Collision(IndexedData::new_with(Collision::Walkable)),
        Component::Inventory(IndexedData::new_with(inventory)),
        Component::Health(IndexedData::new_with(health)),
        Component::BumpResponse(IndexedData::new_with(award_coins)),
        Component::FireResponse(IndexedData::new_with(flammable)),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_spikes(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let image = ImageData { id: 17, depth: 6 };

    let melee = match depth {
        0..=4 => Attack::new_melee(3, 1),
        5..=9 => Attack::new_melee(4, 2),
        10..=14 => Attack::new_melee(5, 3),
        _ => Attack::new_melee(6, 4),
    };
    let combat = Combat::new(Some(melee), None);
    let spikes = EventResponse::new_with(spikes_response);

    let components = vec![
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Name(IndexedData::new_with(Name::new("Spikes"))),
        Component::Position(IndexedData::new_with(start)),
        Component::Combat(IndexedData::new_with(combat)),
        Component::Collision(IndexedData::new_with(Collision::Hazard)),
        Component::BumpResponse(IndexedData::new_with(spikes)),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_flame(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let image = ImageData { id: 18, depth: 6 };
    let spread_fire = EventResponse::new_with(spread_fire_response);

    let components = vec![
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Name(IndexedData::new_with(Name::new("Flame"))),
        Component::Position(IndexedData::new_with(start)),
        Component::Collision(IndexedData::new_with(Collision::Hazard)),
        Component::BumpResponse(IndexedData::new_with(spread_fire)),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_acid(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let image = ImageData { id: 24, depth: 6 };
    let spread_acid = EventResponse::new_with(spread_acid_response);

    let components = vec![
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Name(IndexedData::new_with(Name::new("Acid pool"))),
        Component::Position(IndexedData::new_with(start)),
        Component::Collision(IndexedData::new_with(Collision::Hazard)),
        Component::BumpResponse(IndexedData::new_with(spread_acid)),
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
        Component::Collision(IndexedData::new_with(Collision::Walkable)),
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
        Component::Collision(IndexedData::new_with(Collision::Walkable)),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_mushroom(ecs: &mut ECS, start: Coordinate, _depth: usize) {
    let image = ImageData { id: 22, depth: 6 };
    let health = Health::new(4);
    let flammable = EventResponse::new_with(responses::default_burn_response);

    let components = vec![
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Position(IndexedData::new_with(start)),
        Component::Health(IndexedData::new_with(health)),
        Component::Collision(IndexedData::new_with(Collision::Blocking)),
        Component::LineOfSight(IndexedData::new_with(LoSBlocking::Partial)),
        Component::FireResponse(IndexedData::new_with(flammable)),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_critter(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let health = Health::new(2);
    let image = ImageData { id: 21, depth: 6 };

    let take_damage = EventResponse::new_with(responses::take_damage_response);

    let components = vec![
        Component::Name(IndexedData::new_with(Name::new("Critters"))),
        Component::Combat(IndexedData::new_with(Combat::default())),
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Position(IndexedData::new_with(start)),
        Component::Health(IndexedData::new_with(health)),
        Component::Collision(IndexedData::new_with(Collision::Walkable)),
        Component::ShotResponse(IndexedData::new_with(take_damage)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_wander(3))),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_rat(ecs: &mut ECS, start: Coordinate, depth: usize) {
    let health = Health::new(2);
    let image = ImageData { id: 20, depth: 6 };

    let take_damage = EventResponse::new_with(responses::take_damage_response);

    let components = vec![
        Component::Name(IndexedData::new_with(Name::new("Rat"))),
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Combat(IndexedData::new_with(Combat::default())),
        Component::Position(IndexedData::new_with(start)),
        Component::Health(IndexedData::new_with(health)),
        Component::Collision(IndexedData::new_with(Collision::Walkable)),
        Component::ShotResponse(IndexedData::new_with(take_damage)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_wander(4))),
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}