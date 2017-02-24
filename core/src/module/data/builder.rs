use super::{DataModule, DataComponent};
use state::StateBuilder;
use Context;

pub struct DataModuleBuilder<'a, Cx: 'a + Context> {
    state_builder: &'a mut StateBuilder<Cx>,
    data_module: DataModule,
}

impl<'a, Cx: 'a + Context> DataModuleBuilder<'a, Cx> {
    pub fn new(state_builder: &'a mut StateBuilder<Cx>) -> Self {
        DataModuleBuilder {
            state_builder: state_builder,
            data_module: DataModule::new(),
        }
    }

    pub fn register<D: DataComponent>(&mut self) -> &mut Self {
        self.state_builder.component::<D>();
        self.data_module.register::<D>(D::Storage::default());
        self
    }

    pub fn build(self) -> &'a mut StateBuilder<Cx> {
        self.state_builder.module(self.data_module);

        self.state_builder
    }
}
