use std::path::Path;
use core::module::data;
use core::{self, processor};
use context::Context;

pub type SchedulerBuilder = processor::SchedulerBuilder<Context>;
pub type DataModuleBuilder<'a> = data::DataModuleBuilder<'a, Context>;
pub type StateBuilder = core::StateBuilder<Context>;

pub trait Game {
    fn config_path(&self) -> &Path;

    fn data_components(&self, _builder: &mut DataModuleBuilder) {}
    fn processes(&self, _builder: &mut SchedulerBuilder) {}
    fn interfaces(&self, _builder: &mut StateBuilder) {}
    fn modules(&self, _builder: &mut StateBuilder) {}
}
