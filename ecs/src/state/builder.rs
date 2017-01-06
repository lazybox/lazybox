use schema::SchemaBuilder;
use state::State;
use state::update_queue::UpdateQueues;
use group::Groups;
use module::{Module, Modules};
use module::component::Component;

pub struct StateBuilder<Cx: Send> {
    schema: SchemaBuilder,
    update_queues: UpdateQueues,
    groups: Groups,
    modules: Modules<Cx>,
}

impl<Cx: Send> StateBuilder<Cx> {
    pub fn new() -> Self {
        StateBuilder {
            schema: SchemaBuilder::new(),
            update_queues: UpdateQueues::new(),
            groups: Groups::new(),
            modules: Modules::new(),
        }
    }

    pub fn register_component<C: Component>(&mut self) -> &mut Self {
        self.schema.register_component::<C>();
        self.update_queues.register::<C>();
        self
    }

    pub fn register_module<M: Module<Cx>>(&mut self, module: M) -> &mut Self {
        self.modules.insert(Box::new(module));
        self
    }

    pub fn build(self) -> State<Cx> {
        State::new(self.schema.build(), self.modules, self.groups, self.update_queues)
    }
}
