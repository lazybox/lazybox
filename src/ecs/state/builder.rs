use ecs::state::State;
use ecs::state::update_queue::UpdateQueues;
use ecs::group::Groups;
use ecs::module::{Module, Modules,Component};

pub struct StateBuilder {
    update_queues: UpdateQueues,
    groups: Groups,
    modules: Modules,
}

impl StateBuilder {
    pub fn new() -> Self {
        StateBuilder {
            update_queues: UpdateQueues::new(),
            groups: Groups::new(),
            modules: Modules::new(),
        }
    }

    pub fn register_component<C: Component>(&mut self) -> &mut Self {
        self.update_queues.register::<C>();
        self
    }

    pub fn register_module<M: Module>(&mut self, module: M) -> &mut Self {
        self.modules.insert(Box::new(module));
        self
    }

    pub fn build(self) -> State {
        State::new(self.modules,
                   self.groups,
                   self.update_queues)
    }
}
