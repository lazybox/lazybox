use super::DataModule;
use state::StateBuilder;

pub struct DataModuleBuilder<'a, Cx: Send> {
    state_builder: &'a mut StateBuilder<Cx>
    data_module DataModule
}

impl DataModuleBuilder {
    pub fn new(state_builder: &'a mut StateBuilder) -> Self {
        DataModuleBuilder {
            state_builder: state_builder,
            data_module: DataModule::new()
        }
    }

    pub fn register_data_component<C: DataComponent>(&mut self, storage: C::Storage) -> &mut Self {
        self.state_builder.register_component::<C>();
        self.data_module.register(storage)
    }

    pub fn build(self) -> &'a mut StateBuilder<Cx> {
        self.state_builder.register_module(self.data_module);
        
        self.state_builder
    }
}