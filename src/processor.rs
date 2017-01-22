use ecs::state::{State, Commit};
use ecs::module::ComponentType;
use std::any::Any;
use daggy::{self, Dag, Walker};
use std::sync::atomic::{AtomicUsize, Ordering};
use parking_lot::Mutex;
use rayon;
use context::Context;
use fnv::FnvHashMap;

pub type ComponentTypes = [ComponentType];

pub trait Model {
    fn from_state(state: &State, commit: Commit) -> Self;

    fn writes() -> &'static ComponentTypes;
    fn reads() -> &'static ComponentTypes;
}

pub trait Processor: Send + Any {
    fn writes(&self) -> &'static ComponentTypes;
    fn reads(&self) -> &'static ComponentTypes;

    fn update(&mut self, _state: &State, _commit: Commit, _context: &Context, _delta: f32) {}
    fn fixed_update(&mut self, _state: &State, _commit: Commit, _context: &Context) {}
}

type ProcessorIndex = usize;
type TakeableProcessor = Mutex<Option<Box<Processor>>>;

struct Processors {
    processors: Vec<TakeableProcessor>,
}

impl Processors {
    pub fn new() -> Self {
        Processors { processors: Vec::new() }
    }

    pub fn push<F>(&mut self, processor: Box<Processor>, mut handler: F) -> ProcessorIndex
        where F: FnMut(ProcessorIndex, &Processor)
    {
        let index = self.processors.len();
        handler(index, &*processor);
        self.processors.push(Mutex::new(Some(processor)));

        index
    }

    pub fn take(&self, index: ProcessorIndex) -> Option<Box<Processor>> {
        let mut processor_opt = self.processors[index].lock();
        processor_opt.take()
    }

    pub fn put(&self, index: ProcessorIndex, processor: Box<Processor>) {
        let mut processor_opt = self.processors[index].lock();
        *processor_opt = Some(processor)
    }

    pub fn shrink_to_fit(&mut self) {
        self.processors.shrink_to_fit();
    }
}

type Index = u32;
type NodeIndex = daggy::NodeIndex<Index>;

#[derive(Debug)]
enum LinkType {
    Read,
    Write,
}

struct Slot {
    processor: ProcessorIndex,
    dependencies_counter: AtomicUsize,
    dependencies_count: usize,
}

impl Slot {
    fn new(processor: ProcessorIndex) -> Self {
        Slot {
            processor: processor,
            dependencies_counter: AtomicUsize::new(0),
            dependencies_count: 0,
        }
    }

    #[inline]
    fn set_dependencies_count(&mut self, count: usize) {
        self.dependencies_count = count;
    }

    #[inline]
    fn acknowledge_dependency_resolved(&self) -> bool {
        let old = self.dependencies_counter.fetch_add(1, Ordering::SeqCst);

        if (old + 1) == self.dependencies_count {
            self.dependencies_counter.store(0, Ordering::SeqCst);

            true
        } else {
            false
        }
    }
}

struct ActionGraphBuilder {
    heads: Vec<NodeIndex>,
    execution_dag: Dag<Slot, LinkType, Index>,
    writes: FnvHashMap<ComponentType, NodeIndex>,
    reads: FnvHashMap<ComponentType, Vec<NodeIndex>>,
}

impl ActionGraphBuilder {
    pub fn new() -> Self {
        ActionGraphBuilder {
            execution_dag: Dag::new(),
            writes: FnvHashMap::default(),
            reads: FnvHashMap::default(),
            heads: Vec::new(),
        }
    }

    pub fn register(&mut self,
                    processor_index: ProcessorIndex,
                    reads: &ComponentTypes,
                    writes: &ComponentTypes) {
        let node = self.execution_dag.add_node(Slot::new(processor_index));

        let read_dependencies = self.add_read_dependencies(node, reads);
        let write_dependencies = self.add_write_dependencies(node, writes);

        if read_dependencies == 0 && write_dependencies == 0 {
            self.heads.push(node);
        } else {
            self.execution_dag[node].set_dependencies_count(read_dependencies + write_dependencies);
        }

        self.register_reads(node, reads);
    }

    fn add_write_dependencies(&mut self,
                              processor_node: NodeIndex,
                              writes: &ComponentTypes)
                              -> usize {
        use std::collections::hash_map::Entry;

        let mut dependencies_count = 0;

        for &write in writes {
            match self.writes.entry(write) {
                Entry::Occupied(mut old_writer) => {
                    dependencies_count += 1;

                    self.execution_dag
                        .add_edge(*old_writer.get(), processor_node, LinkType::Write)
                        .expect("cyclic dependency");

                    old_writer.insert(processor_node);
                }
                Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(processor_node);
                }
            }

            let read_nodes = self.reads.entry(write).or_insert(Vec::new());
            for &read in &*read_nodes {
                dependencies_count += 1;
                self.execution_dag
                    .add_edge(read, processor_node, LinkType::Write)
                    .expect("cyclic dependency");
            }
            read_nodes.clear();
        }

        dependencies_count
    }

    fn add_read_dependencies(&mut self,
                             processor_node: NodeIndex,
                             reads: &ComponentTypes)
                             -> usize {

        let mut dependencies_count = 0;

        for read in reads {
            if let Some(&writer) = self.writes.get(read) {
                dependencies_count += 1;
                self.execution_dag
                    .add_edge(writer, processor_node, LinkType::Read)
                    .expect("cyclic dependency");
            }
        }

        dependencies_count
    }

    fn register_reads(&mut self, processor_node: NodeIndex, reads: &ComponentTypes) {
        for &read in reads {
            let read_nodes = self.reads.entry(read).or_insert(Vec::new());
            read_nodes.push(processor_node);
        }
    }

    pub fn build(self) -> ActionGraph {
        ActionGraph {
            heads: self.heads,
            execution_dag: self.execution_dag,
        }
    }
}


pub struct ActionGraph {
    heads: Vec<NodeIndex>,
    execution_dag: Dag<Slot, LinkType, Index>,
}

impl ActionGraph {
    fn par_for_each_mut<F>(&self,
                           processors: &Processors,
                           state: &State,
                           commit: Commit,
                           cx: &Context,
                           f: F)
        where F: Fn(&State, Commit, &Context, &mut Processor) + Sync + Send
    {
        let f = &f;
        rayon::scope(|scope| {
            for &head in &self.heads {
                scope.spawn(move |scope| {
                    self.run_process_mut(processors, scope, head, state, commit, cx, f)
                });
            }
        });
    }

    fn run_process_mut<'a: 's, 's, F: 'a>(&'a self,
                                          processors: &'a Processors,
                                          scope: &rayon::Scope<'s>,
                                          node: NodeIndex,
                                          state: &'a State,
                                          commit: Commit<'a>,
                                          cx: &'a Context,
                                          f: &'a F)
        where F: Fn(&'a State, Commit<'a>, &'a Context, &mut Processor) + Sync + Send
    {
        let slot = &self.execution_dag[node];
        let mut processor = processors.take(slot.processor).unwrap();

        f(state, commit, cx, &mut *processor);

        processors.put(slot.processor, processor);

        let mut children_walker = self.execution_dag.children(node);
        while let Some((_, child)) = children_walker.next(&self.execution_dag) {
            let child_slot = &self.execution_dag[child];

            if child_slot.acknowledge_dependency_resolved() {
                scope.spawn(move |scope| {
                    self.run_process_mut(processors, scope, child, state, commit, cx, f)
                });
            }
        }
    }
}

pub enum UpdateType {
    Frame,
    Fixed,
    Both,
}

pub struct SchedulerBuilder {
    processors: Processors,
    updates: ActionGraphBuilder,
    fixed_updates: ActionGraphBuilder,
}

impl SchedulerBuilder {
    pub fn new() -> Self {
        SchedulerBuilder {
            processors: Processors::new(),
            updates: ActionGraphBuilder::new(),
            fixed_updates: ActionGraphBuilder::new()
        }
    }

    pub fn register<P: Processor>(&mut self, processor: P, update_type: UpdateType) -> &mut Self {
        {
            let &mut SchedulerBuilder { ref mut processors, ref mut updates, ref mut fixed_updates } = self;
            processors.push(Box::new(processor), |index, processor| {
                let reads = processor.reads();
                let writes = processor.writes();

                match update_type {
                    UpdateType::Frame => { updates.register(index, reads, writes); },
                    UpdateType::Fixed => { fixed_updates.register(index, reads, writes); }
                    UpdateType::Both => {
                        updates.register(index, reads, writes);
                        fixed_updates.register(index, reads, writes);
                    }
                }
            });
        }

        self
    }

    pub fn build(mut self) -> Scheduler {
        self.processors.shrink_to_fit();

        Scheduler {
            processors: self.processors,
            updates: self.updates.build(),
            fixed_updates: self.fixed_updates.build(),
        }
    }
}

pub struct Scheduler {
    processors: Processors,
    updates: ActionGraph,
    fixed_updates: ActionGraph,
}

impl Scheduler {
    pub fn update(&mut self, state: &mut State, context: &mut Context, delta: f32) {
        let mut update = state.update();

        update.commit(context, |state, commit, context| {
            self.updates.par_for_each_mut(&self.processors, state, commit, context, |state, commit, context, processor| {
                processor.update(state, commit, context, delta);
            });
        });
    }

    pub fn fixed_update(&mut self, state: &mut State, context: &mut Context) {
        let mut update = state.update();

        update.commit(context, |state, commit, context| {
            self.fixed_updates.par_for_each_mut(&self.processors, state, commit, context, |state, commit, context, processor| {
                processor.fixed_update(state, commit, context);
            });
        });
    }
}
