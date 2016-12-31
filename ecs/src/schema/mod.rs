mod name_map;

use std::sync::Arc;
use module::component::{Component, Template, ComponentType};
use self::name_map::NameMap;

#[derive(Debug)]
struct SchemaData {
    components: NameMap<ComponentType>
}

impl SchemaData {
    pub fn new() -> Self {
        SchemaData {
            components: NameMap::new()
        }
    }
}

pub struct SchemaBuilder(SchemaData);

impl SchemaBuilder {
    pub fn new() -> Self {
        SchemaBuilder(SchemaData::new())
    }

    pub fn register_component<C: Component>(&mut self) -> &mut Self {
        self.0.components.insert(ComponentType::of::<C>(), C::Template::name().into());
        self
    }

    pub fn build(self) -> Schema {
        Schema(Arc::new(self.0))
    }
}

#[derive(Clone, Debug)]
pub struct Schema(Arc<SchemaData>);

impl Schema {
    pub fn component_name(&self, component_type: ComponentType) -> Option<&str> {
        self.0.components.name_of(&component_type)
    }

    pub fn component_type(&self, name: &str) -> Option<ComponentType> {
        self.0.components.of_name(name)
                         .cloned()
    }
}
