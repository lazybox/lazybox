use schema::SchemaBuilder;
use state::State;
use spawn::{SpawnQueue, PrototypeToken};
use entity::Entities;
use std::marker::PhantomData;

pub struct StateBuilder<Cx: Send> {
    schema: SchemaBuilder,
    context: PhantomData<Cx>
}

impl<Cx: Send> StateBuilder<Cx> {
    pub fn new() -> Self {
        StateBuilder {
            schema: SchemaBuilder::new(),
            context: PhantomData
        }
    }

    pub fn register_prototype<P: PrototypeToken>(&mut self) -> &mut Self {
        self.schema.register_prototype::<P>();
        self
    }

    pub fn build(mut self) -> State<Cx> {
        State::new(self.schema.build())
    }
}