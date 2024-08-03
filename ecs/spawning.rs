
use phf::phf_map;
use rand::{thread_rng, Rng};
use behavior::TurnTaker;

use crate::{ecs::component::*, event::EventResponse};
use crate::ecs::*;
use crate::map::Coordinate;

use self::combat::{Attack, Health};

pub static OBJECT_SPAWN_NAMES: phf::Map<&'static str, fn(&mut ECS, Coordinate)> = phf_map!(
    "Doggo" => make_doggo,
    "Heavy" => make_heavy,
    "Pewpew" => make_cultist,
    "Pewpewpet" => make_skelly,
    "Player" => make_player,
    "Chest" => make_chest,
    "Door" => make_door,
    "StairsDown" => make_stairs_down,
    "Corpse" => make_lootable_body,
);

pub fn make_player(ecs: &mut ECS, start: Coordinate) {
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
        cunning: 5,
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
        Component::ShotResponse(IndexedData::new_with(take_damage))
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_doggo(ecs: &mut ECS, start: Coordinate) {
    let combat = Combat::new(
        Some(Attack::new_melee(1, 2)),
        None,
    );
    let health = Health::new(7);
    let stats = Attributes {
        strength: 5,
        dexterity: 5,
        cunning: 5,
        level: 1,
        ..Default::default()
    };
    let image = ImageData {
        id: 6,
        depth: 5,
    };

    let take_damage = EventResponse::new_with(component::responses::take_damage_response);

    let components = vec![
        Component::Monster(IndexedData::new_with(())),
        Component::Name(IndexedData::new_with(Name::new("Doggo"))),
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Position(IndexedData::new_with(start)),
        Component::Combat(IndexedData::new_with(combat)),
        Component::Health(IndexedData::new_with(health)),
        Component::Collision(IndexedData::new_with(Collision::Blocking)),
        Component::Attributes(IndexedData::new_with(stats)),
        Component::BumpResponse(IndexedData::new_with(take_damage.clone())),
        Component::ShotResponse(IndexedData::new_with(take_damage)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_melee()))
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_heavy(ecs: &mut ECS, start: Coordinate) {
    let combat = Combat::new(
        Some(Attack::new_melee(2, 3)),
        None,
    );
    let health = Health::new(14);
    let stats = Attributes {
        strength: 5,
        dexterity: 5,
        cunning: 8,
        level: 3,
        ..Default::default()
    };
    let image = ImageData {
        id: 11,
        depth: 5,
    };
    let take_damage = EventResponse::new_with(component::responses::take_damage_response);

    let components = vec![
        Component::Monster(IndexedData::new_with(())),
        Component::Name(IndexedData::new_with(Name::new("Boar"))),
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Position(IndexedData::new_with(start)),
        Component::Combat(IndexedData::new_with(combat)),
        Component::Health(IndexedData::new_with(health)),
        Component::Collision(IndexedData::new_with(Collision::Blocking)),
        Component::Attributes(IndexedData::new_with(stats)),
        Component::BumpResponse(IndexedData::new_with(take_damage.clone())),
        Component::ShotResponse(IndexedData::new_with(take_damage)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_slow_melee()))
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_skelly(ecs: &mut ECS, start: Coordinate) {
    let combat = Combat::new(
        Some(Attack::new_melee(2, 1)),
        None,
    );
    let health = Health::new(9);
    let stats = Attributes {
        strength: 5,
        dexterity: 5,
        cunning: 3,
        level: 1,
        ..Default::default()
    };
    let image = ImageData {
        id: 13,
        depth: 5,
    };
    let coins = thread_rng().gen_range(0..=4);
    let inventory = Inventory { coins };

    let take_damage = EventResponse::new_with(responses::take_damage_response);
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
        Component::Attributes(IndexedData::new_with(stats)),
        Component::BumpResponse(IndexedData::new_with(take_damage.clone())),
        Component::ShotResponse(IndexedData::new_with(take_damage)),
        Component::DeathResponse(IndexedData::new_with(drop_coins)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_melee()))
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_cultist(ecs: &mut ECS, start: Coordinate) {
    let combat = Combat::new(
        Some(Attack::new_melee(1, 2)),
        None
    );
    let health = Health::new(12);
    let stats = Attributes {
        strength: 5,
        dexterity: 5,
        cunning: 8,
        level: 5,
        ..Default::default()
    };
    let image = ImageData {
        id: 12,
        depth: 5,
    };
    let coins = thread_rng().gen_range(15..=32);
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
        Component::Attributes(IndexedData::new_with(stats)),
        Component::BumpResponse(IndexedData::new_with(take_damage.clone())),
        Component::ShotResponse(IndexedData::new_with(take_damage)),
        Component::DeathResponse(IndexedData::new_with(drop_coins)),
        Component::Turn(IndexedData::new_with(TurnTaker::new_melee()))
    ];

    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}

pub fn make_door(ecs: &mut ECS, start: Coordinate) {
    let open_image = ImageData{id: 10, depth: 7};
    let closed_image = ImageData{id: 9, depth: 7};
    let images = ImageHandle {
        current: closed_image.to_owned(),
        states: HashMap::from([
            ("open", open_image),
            ("closed", closed_image)])
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

pub fn make_chest(ecs: &mut ECS, start: Coordinate) {
    let open_image = ImageData{id: 8, depth: 7};
    let closed_image = ImageData{id: 7, depth: 7};
    let images = ImageHandle {
        current: closed_image.to_owned(),
        states: HashMap::from([
            ("open", open_image),
            ("closed", closed_image)])
    };
    let coins = thread_rng().gen_range(15..=32);
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

pub fn make_lootable_body(ecs: &mut ECS, start: Coordinate) {
    let image = ImageData {
        id: 14,
        depth: 6,
    };
    
    let coins = thread_rng().gen_range(15..=32);
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

pub fn make_gold_pile(ecs: &mut ECS, start: Coordinate) {
    let image = ImageData {
        id: 15,
        depth: 6,
    };
    
    let coins = thread_rng().gen_range(15..=32);
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

pub fn make_stairs_down(ecs: &mut ECS, start: Coordinate) {
    let image = ImageData {
        id: 16,
        depth: 7,
    };

    let components = vec![
        Component::Image(IndexedData::new_with(ImageHandle::new(image))),
        Component::Stairs(IndexedData::new_with(())),
        Component::Position(IndexedData::new_with(start)),
        Component::Collision(IndexedData::new_with(component::Collision::Walkable)),
    ];
    
    let new_id = ecs.create_entity();
    ecs.add_components_to_entity(new_id, components);
}