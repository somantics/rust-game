use std::cell::RefCell;
use std::collections::VecDeque;

use crate::game::components::core::Name;

thread_local!(
    pub static LOG: MessageLog = MessageLog::new();
);

pub struct MessageLog {
    message_queue: RefCell<VecDeque<String>>,
}

impl MessageLog {
    pub fn new() -> Self {
        MessageLog {
            message_queue: RefCell::new(VecDeque::new()),
        }
    }

    pub fn queue_message(&self, msg: &str) {
        self.message_queue.borrow_mut().push_back(msg.to_string());
    }

    pub fn next_message(&self) -> Option<String> {
        self.message_queue.borrow_mut().pop_front()
    }
}

pub fn log_message(msg: &str) {
    LOG.with(|log| log.queue_message(msg));
}

pub fn generate_attack_message(
    attacker: &Name,
    defender: &Name,
    hit_message: &str,
    damage_taken: isize,
) -> String {
    vec![
        &attacker.raw,
        hit_message,
        &defender.raw,
        "for",
        &damage_taken.to_string(),
    ]
    .join(" ")
}

pub fn generate_take_damage_message(defender: &Name, damage_taken: isize) -> String {
    vec![&defender.raw, "took", &damage_taken.to_string(), "damage."].join(" ")
}

pub fn generate_receive_gold_message(amount: isize) -> String {
    vec!["You found", &amount.to_string(), "gold!"].join(" ")
}

pub fn generate_is_burning_message(defender: &Name, damage_taken: isize) -> String {
    vec![&defender.raw, "is burning! Took", &damage_taken.to_string(), "damage."].join(" ")
}

pub fn generate_on_fire_message(defender: &Name) -> String {
    vec![&defender.raw, "catches on fire!"].join(" ")
}