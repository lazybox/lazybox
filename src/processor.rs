use context::{self, processors};
use core;

pub type Context = processors::Context;
pub type State = State<context::Context>;
pub type Commit = Commit<context::Context>;

pub trait Processor<'a>: Send + Any {
    type Access: core::processor::StateAccess<'a, Context>;

    fn update_type(&self) -> UpdateType;

    fn update(&mut self, _: &State, _: Commit, _: &Context, _: f32) {}
    fn fixed_update(&mut self, _: &State, _: Commit, _: &Context) {}
}

impl<'a, T> core::processor::Processor<'a, context::Context> for T
    where T: Processor<'a>
{
    type Access = <Self as Processor<'a>>::Access;

    fn update_type(&self) -> UpdateType {
        <Self as Processor<'a>>::update_type(self)
    }

    fn update(&mut self, s: &State, c: Commit, cx: &Context, dt: f32) {
        <Self as Processor<'a>>::update(self, s, c, cx, dt)
    }
    fn fixed_update(&mut self, s: &State, c: Commit, cx: &Context) {
        <Self as Processor<'a>>::fixed_update(self, s, c, cx, dt)
    }
}
