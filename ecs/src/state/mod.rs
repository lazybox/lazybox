use entity::Entities;
use module::Modules;
use spawn::SpawnQueue;

pub struct State<Cx: Send> {
    entities: Entities,
    modules: Modules<Cx>,
    spawn_queue: SpawnQueue
}

impl<Cx: Send> State<Cx> {
    pub fn new() -> Self {
        State {
            entities: Entities::new(),
            modules: Modules::new(),
            spawn_queue: SpawnQueue::new()
        }
    }
}
