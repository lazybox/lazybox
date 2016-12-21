use state::State;
use component::ComponentType;
use entity::Entities;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use daggy::{self, petgraph, Dag, Walker};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;
use rayon;

pub trait Model {
    fn from_state(state: State) -> Self;
    fn merge_with(self, state: State) -> State;
}

pub trait Processor: Send + Any {
    type Model: Model;

    fn get_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn writes(&self) -> &'static [ComponentType];
    fn reads(&self) -> &'static [ComponentType];
}

pub trait AnyProcessor: Processor {}

impl<T: ?Sized + Processor> AnyProcessor for T {}

type Index = u32;
type NodeIndex = daggy::NodeIndex<Index>;

enum LinkType {
    Read,
    Write,
}

struct Slot<P: ?Sized + AnyProcessor> {
    processor: Mutex<Option<Box<P>>>,
    dependencies_counter: AtomicUsize,
    dependencies_count: usize,
}

impl<P: ?Sized + AnyProcessor> Slot<P> {
    fn new(processor: Box<P>) -> Self {
        Slot {
            processor: Mutex::new(Some(processor)),
            dependencies_counter: AtomicUsize::new(0),
            dependencies_count: 0
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

pub struct ExecutionGraphBuilder<P: ?Sized + AnyProcessor> {
    execution_dag: Dag<Slot<P>, LinkType, Index>,
    writes: HashMap<ComponentType, NodeIndex>,
    reads: HashMap<ComponentType, Vec<NodeIndex>>,
    heads: Vec<NodeIndex>,
}

impl<P: ?Sized + AnyProcessor> ExecutionGraphBuilder<P> {
    pub fn new() -> Self {
        ExecutionGraphBuilder {
            execution_dag: Dag::new(),
            writes: HashMap::new(),
            reads: HashMap::new(),
            heads: Vec::new(),
        }
    }

    pub fn register(mut self, processor: Box<P>) -> Self {
        let type_id = Processor::get_type_id(&*processor);

        let writes = processor.writes();
        let reads = processor.reads();

        let node = self.execution_dag.add_node(Slot::new(processor));

        let read_dependencies = self.add_read_dependencies(node, reads);
        let write_dependencies = self.add_write_dependencies(node, writes);

        if read_dependencies == 0 && write_dependencies == 0 {
            self.heads.push(node);
        } else {
            self.execution_dag[node]
                .set_dependencies_count(read_dependencies + write_dependencies);
        }

        self.register_reads(node, reads);

        self
    }

    fn add_write_dependencies(&mut self,
                              processor_node: NodeIndex,
                              writes: &[ComponentType])
                              -> usize {
        use std::collections::hash_map::Entry;

        let mut dependencies_count = 0;

        for &write in writes {
            match self.writes.entry(write) {
                Entry::Occupied(mut old_writer) => {
                    dependencies_count += 1;

                    self.execution_dag.add_edge(*old_writer.get(), processor_node, LinkType::Write);
                    old_writer.insert(processor_node);
                }
                Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(processor_node);
                }
            }

            let read_nodes = self.reads.entry(write).or_insert(Vec::new());
            for &read in &*read_nodes {
                dependencies_count += 1;
                self.execution_dag.add_edge(read, processor_node, LinkType::Write);
            }
            read_nodes.clear();
        }

        dependencies_count
    }

    fn add_read_dependencies(&mut self,
                             processor_node: NodeIndex,
                             reads: &[ComponentType])
                             -> usize {

        let mut dependencies_count = 0;

        for read in reads {
            if let Some(&writer) = self.writes.get(read) {
                dependencies_count += 1;
                self.execution_dag.add_edge(writer, processor_node, LinkType::Read);
            }
        }

        dependencies_count
    }

    fn register_reads(&mut self, processor_node: NodeIndex, reads: &[ComponentType]) {
        for &read in reads {
            let read_nodes = self.reads.entry(read).or_insert(Vec::new());
            read_nodes.push(processor_node);
        }
    }

    fn build(self) -> Scheduler<P> {
        Scheduler {
            heads: self.heads,
            execution_dag: self.execution_dag,
        }
    }
}

pub struct Scheduler<P: ?Sized + AnyProcessor> {
    heads: Vec<NodeIndex>,
    execution_dag: Dag<Slot<P>, LinkType, Index>,
}

impl<P: ?Sized + AnyProcessor> Scheduler<P> {
    pub fn par_for_each_mut<F>(&self, state: &mut State, f: F)
        where F: Fn(&State, &mut P) + Sync + Send
    {
        let f = &f;
        let state = &*state;

        rayon::scope(|scope| {
            for &head in &self.heads {
                scope.spawn(move |scope| self.run_process_mut(scope, head, state, f));
            }
        });
    }

    fn run_process_mut<'b: 'scope, 'scope, F>(&'b self, scope: &rayon::Scope<'scope>, node: NodeIndex, state: &'b State, f: &'b F)
        where F: Fn(&State, &mut P) + Sync + Send
    {
        let mut process = self.take_process(node);
        f(state, &mut *process);
        self.put_process(node, process);

        let mut children_walker = self.execution_dag.children(node);
        while let Some((_, child)) = children_walker.next(&self.execution_dag) {
            let child_slot = &self.execution_dag[child];

            if child_slot.acknowledge_dependency_resolved() {
                scope.spawn(move |scope| self.run_process_mut(scope, child, state, f));
            }
        }
    }

    #[inline]
    fn take_process(&self, node: NodeIndex) -> Box<P> {
        let mut slot_opt = self.execution_dag[node].processor.lock();
        slot_opt.take().unwrap()
    }

    #[inline]
    fn put_process(&self, node: NodeIndex, process: Box<P>) {
        let mut slot_opt = self.execution_dag[node].processor.lock();
        *slot_opt = Some(process);
    }
}
