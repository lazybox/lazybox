pub mod name_map;

use std::sync::Arc;
use spawn::{PrototypeToken, Prototypes};

#[derive(Debug)]
struct SchemaData {
    prototypes: Prototypes
}

impl SchemaData {
    pub fn new() -> Self {
        SchemaData {
            prototypes: Prototypes::new(),
        }
    }
}

pub struct SchemaBuilder(SchemaData);

impl SchemaBuilder {
    pub fn new() -> Self {
        SchemaBuilder(SchemaData::new())
    }

    pub fn register_prototype<P: PrototypeToken>(&mut self) -> &mut Self {
        self.0.prototypes.register::<P>();
        self
    }

    pub fn build(self) -> Schema {
        Schema(Arc::new(self.0))
    }
}

#[derive(Clone, Debug)]
pub struct Schema(Arc<SchemaData>);

impl Schema {
    pub fn prototypes(&self) -> &Prototypes {
        &self.0.prototypes
    }
}
