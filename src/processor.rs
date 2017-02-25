use context::{self, processors};
use core;
use std::any::Any;

pub use core::UpdateType;

pub type Context = processors::Context;
pub type State = core::State<context::Context>;
pub type Commit<'a> = core::Commit<'a, context::Context>;
