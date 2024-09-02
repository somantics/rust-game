use crate::ecs::component::Diffable;

#[derive(Debug, Clone)]
pub struct Inventory {
    pub coins: isize,
}

impl Inventory {
    pub fn new(coins: isize) -> Self {
        Inventory {
            coins,
            ..Default::default()
        }
    }

    pub fn inverse(&self) -> Self {
        Inventory {
            coins: -self.coins,
            ..Default::default()
        }
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory { coins: 0 }
    }
}

impl Diffable for Inventory {
    fn apply_diff(&mut self, other: &Self) {
        self.coins += other.coins;
    }
}
