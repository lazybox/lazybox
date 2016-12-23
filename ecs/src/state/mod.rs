use entity::Entities;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct State {
    entities: Entities,
}

impl State {
    pub fn new() -> Self {
        State {
            entities: Entities::new(),
        }
    }
}
