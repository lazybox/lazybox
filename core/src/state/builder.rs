use {State, Context, Interfaces};
use state::update_queue::UpdateQueues;
use {Module, Modules, Component};
use group::{Groups, GroupType, GroupToken};

pub struct StateBuilder<Cx: Context> {
    update_queues: UpdateQueues,
    interfaces: Interfaces,
    groups: Groups,
    modules: Modules<Cx::ForModules>,
}

impl<Cx: Context> StateBuilder<Cx> {
    pub fn new() -> Self {
        StateBuilder {
            update_queues: UpdateQueues::new(),
            interfaces: Interfaces::new(),
            groups: Groups::new(),
            modules: Modules::new(),
        }
    }

    pub fn register_component<C: Component>(&mut self) -> &mut Self {
        self.update_queues.register::<C>();
        self
    }

    pub fn register_module<M: Module<Cx::ForModules>>(&mut self, module: M) -> &mut Self {
        self.modules.insert(Box::new(module));
        self
    }

    pub fn register_group<G: GroupToken>(&mut self) -> &mut Self {
        self.groups.insert_empty(GroupType::of::<G>());
        self
    }

    pub fn build(self) -> State<Cx> {
        State::new(self.modules,
                   self.interfaces,
                   self.groups,
                   self.update_queues)
    }
}
