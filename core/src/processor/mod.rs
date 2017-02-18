mod graph;

use {Context, ComponentType};
use state::{State, Commit};
use std::any::Any;
use sync::Mutex;
use self::graph::{ActionGraphBuilder, ActionGraph, Processors};

pub type ComponentTypes = Vec<ComponentType>;

pub trait StateAccess<'a, Cx: Context> {
    fn from_state(state: &'a State<Cx>) -> Self;

    fn writes() -> ComponentTypes;
    fn reads() -> ComponentTypes;
}

pub enum UpdateType {
    Frame,
    Fixed,
    Both,
}

pub trait ProcessorExt<Cx: Context>: Send + Any {
    fn writes(&self) -> ComponentTypes;
    fn reads(&self) -> ComponentTypes;
    fn update_type(&self) -> UpdateType;

    fn update(&mut self, _state: &State<Cx>, _commit: Commit<Cx>, _context: &Cx, _delta: f32) {}
    fn fixed_update(&mut self, _state: &State<Cx>, _commit: Commit<Cx>, _context: &Cx) {}
}

pub trait Processor<'a, Cx: Context>: Send + Any {
    type Access: StateAccess<'a, Cx>;

    fn update_type(&self) -> UpdateType;

    fn update(&mut self, _state: &State<Cx>, _commit: Commit<Cx>, _context: &Cx, _delta: f32) {}
    fn fixed_update(&mut self, _state: &State<Cx>, _commit: Commit<Cx>, _context: &Cx) {}
}

impl<'a, Cx: Context, P> ProcessorExt<Cx> for P
    where P: Processor<'a, Cx>
{
    fn writes(&self) -> ComponentTypes {
        P::Access::writes()
    }
    fn reads(&self) -> ComponentTypes {
        P::Access::reads()
    }
    fn update_type(&self) -> UpdateType {
        <P as Processor<'a, Cx>>::update_type(self)
    }

    fn update(&mut self, state: &State<Cx>, commit: Commit<Cx>, context: &Cx, delta: f32) {
        <P as Processor<'a, Cx>>::update(self, state, commit, context, delta);
    }

    fn fixed_update(&mut self, state: &State<Cx>, commit: Commit<Cx>, context: &Cx) {
        <P as Processor<'a, Cx>>::fixed_update(self, state, commit, context);
    }
}

pub struct SchedulerBuilder<Cx: Context> {
    processors: Processors<Cx>,
    updates: ActionGraphBuilder,
    fixed_updates: ActionGraphBuilder,
}

impl<Cx: Context> SchedulerBuilder<Cx> {
    pub fn new() -> Self {
        SchedulerBuilder {
            processors: Processors::new(),
            updates: ActionGraphBuilder::new(),
            fixed_updates: ActionGraphBuilder::new(),
        }
    }

    pub fn register<P: ProcessorExt<Cx>>(&mut self,
                                         processor: P,
                                         update_type: UpdateType)
                                         -> &mut Self {
        {
            let &mut SchedulerBuilder { ref mut processors,
                                        ref mut updates,
                                        ref mut fixed_updates } = self;
            processors.push(Box::new(processor), |index, processor| {
                let reads = &processor.reads();
                let writes = &processor.writes();

                match update_type {
                    UpdateType::Frame => {
                        updates.register(index, reads, writes);
                    }
                    UpdateType::Fixed => {
                        fixed_updates.register(index, reads, writes);
                    }
                    UpdateType::Both => {
                        updates.register(index, reads, writes);
                        fixed_updates.register(index, reads, writes);
                    }
                }
            });
        }

        self
    }

    pub fn build(mut self) -> Scheduler<Cx> {
        self.processors.shrink_to_fit();

        Scheduler {
            processors: self.processors,
            updates: self.updates.build(),
            fixed_updates: self.fixed_updates.build(),
        }
    }
}

pub struct Scheduler<Cx: Context> {
    processors: Processors<Cx>,
    updates: ActionGraph,
    fixed_updates: ActionGraph,
}

impl<Cx: Context> Scheduler<Cx> {
    pub fn update(&mut self, state: &mut State<Cx>, context: &mut Cx, delta: f32) {
        let mut update = state.update();

        update.commit(context, |state, commit, context| {
            self.updates.par_for_each_mut(&self.processors,
                                          state,
                                          commit,
                                          context,
                                          |state, commit, context, processor| {
                                              processor.update(state, commit, context, delta);
                                          });
        });
    }

    pub fn fixed_update(&mut self, state: &mut State<Cx>, context: &mut Cx) {
        let mut update = state.update();

        update.commit(context, |state, commit, context| {
            self.fixed_updates.par_for_each_mut(&self.processors,
                                                state,
                                                commit,
                                                context,
                                                |state, commit, context, processor| {
                                                    processor.fixed_update(state, commit, context);
                                                });
        });
    }
}
