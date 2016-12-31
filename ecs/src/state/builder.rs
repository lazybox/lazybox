use schema::SchemaBuilder;
use state::State;
use entity::Entities;
use std::marker::PhantomData;
use state::update_queue::UpdateQueues;

pub struct StateBuilder<Cx: Send> {
    schema: SchemaBuilder,
    update_queues: UpdateQueues,
    context: PhantomData<Cx>
}

impl<Cx: Send> StateBuilder<Cx> {
    pub fn new() -> Self {
        StateBuilder {
            schema: SchemaBuilder::new(),
            update_queues: UpdateQueues::new(),
            context: PhantomData
        }
    }

    pub fn build(mut self) -> State<Cx> {
        State::new(self.schema.build(), self.update_queues)
    }
}
