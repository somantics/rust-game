use crate::ecs::component::Diffable;

#[derive(Debug, Clone, Copy, Default)]
pub struct Attributes {
    pub strength: isize,
    pub dexterity: isize,
    pub level: isize,
    pub xp: isize,
    pub level_pending: bool,
}

impl Diffable for Attributes {
    fn apply_diff(&mut self, other: &Self) {
        // raw attributes
        self.strength += other.strength;
        self.dexterity += other.dexterity;
        // leveling up
        self.level += other.level;
        self.xp += other.xp;
        self.level_pending = other.level_pending;
    }
}

pub fn get_xp_to_next(attr: &Attributes) -> isize {
    100 * attr.level
}
