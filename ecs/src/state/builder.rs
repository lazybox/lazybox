use schema::SchemaBuilder;
use state::State;
use entity::Entities;
use std::marker::PhantomData;
use state::update_queue::UpdateQueues;
use group::Groups;
use module::{Module, Modules};

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
        self.update_queue.register::<C>();
    }

    pub fn register_module<M: Module<Cx>(&mut self, module: M) -> &mut Self {
        self.modules.insert(Box::new(module));
    }

    pub fn build(mut self) -> State<Cx> {
        State::new(self.schema.build(), self.update_queues)
    }
}

pub struct ComponentRegistry<'a> {
    schema: &'a mut SchemaBuilder,
    update_queues: &'a mut UpdateQueues
}

impl<'a> ComponentRegistry<'a> {
    pub fn register<C: Component>(self) {
        schema.register_component::<C>();
        update_queues.register::<C>();
    }
}
