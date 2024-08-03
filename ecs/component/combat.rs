use rand::{thread_rng, Rng};

use crate::ecs::{Component, Delta, IndexedData};

use super::{attributes::Attributes, inventory::Inventory, Diffable};

const DEX_BONUS_DMG_MULTIPLIER: f32 = 0.5;
const STR_BONUS_DMG_MULTIPLIER: f32 = 0.25;
const BONUS_DMG_SCALE: f32 = 0.8;
const BASE_CRIT_CHANCE: f64 = 0.15;
const CUN_CRIT_CHANCE_BONUS: f64 = 0.15;
const BASE_CRIT_MULTIPLIER: f64 = 2.0;

#[derive(Debug, Clone)]
pub struct Combat {
    pub melee: Option<Attack>,
    pub ranged: Option<Attack>,
}

impl Combat {
    pub fn new(melee: Option<Attack>, ranged: Option<Attack>) -> Self {
        Combat {
            melee,
            ranged,
            ..Default::default()
        }
    }
}

impl Default for Combat {
    fn default() -> Self {
        Combat {
            melee: Some(Attack::default()),
            ranged: None,
        }
    }
}

impl Diffable for Combat {
    fn apply_diff(&mut self, other: &Self) {
        self.melee = other.melee;
        self.ranged = other.ranged;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Health {
    pub current: isize,
    pub max: isize,
}

impl Health {
    pub fn new(health: isize)  -> Self {
        Health {
            current: health,
            max: health
        }
    }

    pub fn new_from_stats(stats: &Attributes) -> Self {
        todo!()
    }

    pub fn get_health_reset_diff(&self) -> Self {
        Health {
            current: self.max - self.current,
            ..Default::default()
        }
    }
    pub fn get_healing_diff(&self, amount: isize) -> Self {
        Health {
            current: (amount).min(self.max - self.current),
            ..Default::default()
        }
    }
}

impl Diffable for Health {
    fn apply_diff(&mut self, other: &Self) {
        self.current += other.current;
        self.max += other.max;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum DamageType {
    #[default]
    Phsycial,
    Magical,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HitMessages {
    default: &'static str,
    crit: &'static str,
}

impl HitMessages {
    fn new(default: &'static str, crit: &'static str) -> Self {
        Self { default, crit }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AttackReport {
    pub damage: isize,
    pub damage_type: DamageType,
    pub hit_message: &'static str,
    pub range: Option<usize>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Attack {
    pub damage_base: isize,
    pub damage_spread: isize,
    pub crit_chance_bonus: f64,
    pub crit_multiplier_bonus: f64,
    pub damage_type: DamageType,
    pub hit_messages: HitMessages,
    pub max_range: isize,
}

impl Attack {
    pub fn new_melee(damage_base: isize, damage_spread: isize) -> Self {
        Attack {
            damage_base,
            damage_spread,
            hit_messages: HitMessages::new("hit", "decimated"),
            ..Default::default()
        }
    }

    pub fn new_ranged(damage_base: isize, damage_spread: isize) -> Self {
        Attack {
            damage_base,
            damage_spread,
            hit_messages: HitMessages::new("shot", "sniped"),
            max_range: 6,
            ..Default::default()
        }
    }
}

pub fn get_bonus_dmg(attr: &Attributes, attack: &Attack) -> isize {
    let adj_strength = attr.strength - 5;
    let adj_dexterity = attr.dexterity - 5;
    let str_bonus = adj_strength as f32
        * STR_BONUS_DMG_MULTIPLIER
        * BONUS_DMG_SCALE
        * (attack.damage_base + attack.damage_spread) as f32;
    let dex_bonus = adj_dexterity as f32
        * DEX_BONUS_DMG_MULTIPLIER
        * BONUS_DMG_SCALE
        * attack.damage_base as f32;

    (str_bonus + dex_bonus).ceil() as isize
}

pub fn get_damage_multiplier(critical: bool) -> f64 {
    if critical {
        BASE_CRIT_MULTIPLIER
    } else {
        1.0
    }
}

fn get_damage(attack: &Attack, attributes: Option<&Attributes>, critical: bool) -> isize {
    let rand_factor = thread_rng().gen_range(0..=attack.damage_spread);
    let raw_damage = attack.damage_base + rand_factor;
    if let Some(stats) = attributes {
        let bonus_dmg = get_bonus_dmg(&stats, attack);
        let mut damage_multiplier = get_damage_multiplier(critical);
        if critical {
            damage_multiplier += attack.crit_multiplier_bonus;
        }
        ((raw_damage + bonus_dmg) as f64 * damage_multiplier).max(0.0) as isize
    } else {
        raw_damage
    }
}

pub fn get_crit_chance(attr: &Attributes) -> f64 {
    let adj_cunning = (attr.cunning - 5) as f64;
    (BASE_CRIT_CHANCE + adj_cunning * CUN_CRIT_CHANCE_BONUS).clamp(0.05, 0.95)
}

pub fn crit_roll(attack: &Attack, attributes: Option<&Attributes>) -> bool {
    if let Some(stats) = attributes {
        let crit_chance = get_crit_chance(stats) + attack.crit_chance_bonus;
        thread_rng().gen_bool(crit_chance)
    } else {
        false
    }
}

pub fn calculate_melee_attack(
    combat: &Combat,
    attributes: Option<&Attributes>,
) -> Option<AttackReport> {
    if let Some(attack) = &combat.melee {
        Some(calculate_attack(attack, attributes, None))
    } else {
        None
    }
}

pub fn calculate_ranged_attack(
    combat: &Combat,
    attributes: Option<&Attributes>,
) -> Option<AttackReport> {
    if let Some(attack) = &combat.ranged {
        Some(calculate_attack(
            attack,
            attributes,
            Some(attack.max_range as usize),
        ))
    } else {
        None
    }
}

fn calculate_attack(
    attack: &Attack,
    attributes: Option<&Attributes>,
    range: Option<usize>,
) -> AttackReport {
    let critical: bool = crit_roll(attack, attributes);
    AttackReport {
        damage: get_damage(attack, attributes, critical),
        damage_type: attack.damage_type,
        hit_message: if critical {
            attack.hit_messages.crit
        } else {
            attack.hit_messages.default
        },
        range,
    }
}
pub fn default_calculate_armor(
    _damage_type: DamageType,
    _maybe_stats: Option<&IndexedData<Attributes>>,
    _maybe_items: Option<&IndexedData<Inventory>>,
) -> f32 {
    0.0
}

pub fn default_calculate_reduction(damage: isize, _armor: f32) -> isize {
    damage
}

pub fn default_take_damage(
    attack: &AttackReport,
    health: &IndexedData<Health>,
    maybe_stats: Option<&IndexedData<Attributes>>,
    maybe_items: Option<&IndexedData<Inventory>>,
) -> (Vec<Delta>, isize) {
    let armor = default_calculate_armor(attack.damage_type, maybe_stats, maybe_items);
    let reduced_damage = default_calculate_reduction(attack.damage, armor);
    let damage_taken = Health {
        current: -reduced_damage,
        max: 0,
    };

    (
        vec![Delta::Change(Component::Health(health.make_change(damage_taken)))],
        reduced_damage
    )
}
