use {Context, ComponentType};
use state::{State, Commit};
use sync::Mutex;
use daggy::{self, Dag, Walker};
use std::sync::atomic::{AtomicUsize, Ordering};
use rayon;
use fnv::FnvHashMap;

use super::{ProcessorExt, ComponentTypes};

type ProcessorIndex = usize;
type TakeableProcessor<Cx> = Mutex<Option<Box<ProcessorExt<Cx>>>>;

pub struct Processors<Cx: Context> {
    processors: Vec<TakeableProcessor<Cx>>,
}

impl<Cx: Context> Processors<Cx> {
    pub fn new() -> Self {
        Processors { processors: Vec::new() }
    }

    pub fn push<F>(&mut self, processor: Box<ProcessorExt<Cx>>, mut handler: F) -> ProcessorIndex
        where F: FnMut(ProcessorIndex, &ProcessorExt<Cx>)
    {
        let index = self.processors.len();
        handler(index, &*processor);
        self.processors.push(Mutex::new(Some(processor)));

        index
    }

    pub fn take(&self, index: ProcessorIndex) -> Option<Box<ProcessorExt<Cx>>> {
        let mut processor_opt = self.processors[index].lock();
        processor_opt.take()
    }

    pub fn put(&self, index: ProcessorIndex, processor: Box<ProcessorExt<Cx>>) {
        let mut processor_opt = self.processors[index].lock();
        *processor_opt = Some(processor)
    }

    pub fn shrink_to_fit(&mut self) {
        self.processors.shrink_to_fit();
    }
}

#[derive(Debug)]
enum LinkType {
    Read,
    Write,
}

pub struct Slot {
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

type Index = u32;
type NodeIndex = daggy::NodeIndex<Index>;

pub struct ActionGraph {
    heads: Vec<NodeIndex>,
    execution_dag: Dag<Slot, LinkType, Index>,
}

impl ActionGraph {
    pub fn par_for_each_mut<F, Cx: Context>(&self,
                                            processors: &Processors<Cx>,
                                            state: &State<Cx>,
                                            commit: Commit<Cx>,
                                            cx: &Cx::ForProcessors,
                                            f: F)
        where F: Fn(&State<Cx>,
                    Commit<Cx>,
                    &Cx::ForProcessors,
                    &mut ProcessorExt<Cx>) + Sync + Send
    {
        let f = &f;
        rayon::scope(|scope| for &head in &self.heads {
            scope.spawn(move |scope| {
                self.run_process_mut(processors, scope, head, state, commit, cx, f)
            });
        });
    }

    pub fn run_process_mut<'a: 's, 's, F: 'a, Cx: Context>(&'a self,
                                                           processors: &'a Processors<Cx>,
                                                           scope: &rayon::Scope<'s>,
                                                           node: NodeIndex,
                                                           state: &'a State<Cx>,
                                                           commit: Commit<'a, Cx>,
                                                           cx: &'a Cx::ForProcessors,
                                                           f: &'a F)
        where F: Fn(&'a State<Cx>,
                    Commit<'a, Cx>,
                    &'a Cx::ForProcessors,
                    &mut ProcessorExt<Cx>) + Sync + Send
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

pub struct ActionGraphBuilder {
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