use entity::Entities;
use component::Components;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct State {
    pub entities: Entities,
    pub components: RwLock<Components>
}

impl State {
    pub fn new() -> Self {
        State {
            entities: Entities::new(),
            components: RwLock::new(Components::new())
        }
    }
}
