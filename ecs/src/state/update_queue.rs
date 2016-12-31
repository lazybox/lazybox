use crossbeam::sync::SegQueue;
use module::component::{Component, ComponentType};
use fnv::{FnvHashSet, FnvHashMap};
use parking_lot::{RwLock, RwLockWriteGuard, RwLockReadGuard};
use entity::{Entity, Accessor};
use policy::{Id, IdSet};
use mopa;

pub struct Monitor {
    entities: IdSet,
    modified: bool
}

impl Monitor {
    pub fn new() -> Self {
        Monitor {
            entities: BitSet::new(),
            modified: false
        }
    }

    #[inline]
    pub fn mark(&mut self, entity: Id) {
        self.modified = true;
        self.entities.insert(entity as usize);
    }

    #[inline]
    pub fn unmark(&mut self, entity: Id) {
        self.modified = true;
        self.entities.remove(entity as usize);
    }

    pub fn forget(&mut self, entities: &[Entity]) {
        for entity in entities {
            self.entities.remove(entity.id() as usize);
        }
    }

    fn clear_modified_flag(&mut self) {
        self.modified = false;
    }

    pub fn modified(&self) -> bool {
        self.modified
    }
}

type AttachQueue<T> = SegQueue<(Id, T)>;
type DetachQueue = SegQueue<Id>;

pub struct UpdateQueue<C: Component> {
    monitor: RwLock<Monitor>,
    attach_queue: AttachQueue<C::Template>,
    detach_queue: DetachQueue,
}

impl<C: Component> UpdateQueue<C> {
    pub fn new() -> Self {
        UpdateQueue {
            monitor: RwLock::new(Monitor::new()),
            attach_queue: AttachQueue::new(),
            detach_queue: DetachQueue::new(),
        }
    }

    #[inline]
    pub fn attach<'a>(&self, accessor: Accessor<'a>, template: C::Template) {
        self.attach_queue.push((accessor.id(), template))
    }

    #[inline]
    pub fn detach<'a>(&self, accessor: Accessor<'a>) {
        self.detach_queue.push(accessor.id())
    }

    pub fn process<'a, 'b>(&'a self, world_removes: &'b [Entity]) -> UpdateQueueReader<'a, 'b, C> {
        UpdateQueueReader {
            monitor: self.monitor.write(),
            attach_queue: &self.attach_queue,
            detach_queue: &self.detach_queue,
            world_removes: world_removes
        }
    }
}

pub struct UpdateQueueReader<'a, 'b, C: Component> {
    monitor: RwLockWriteGuard<'a, Monitor>,
    attach_queue: &'a AttachQueue<C::Template>,
    detach_queue: &'a DetachQueue,
    world_removes: &'b [Entity]
}

impl<'a, 'b, C: Component> UpdateQueueReader<'a, 'b, C> {
    pub fn next_attach_query(&mut self) -> Option<(Id, C::Template)> {
        self.attach_queue.try_pop().map(|(entity, template)| {
            self.monitor.mark(entity);
            
            (entity, template)
        })
    }

    pub fn next_detach_query(&mut self) -> Option<Id> {
        self.detach_queue.try_pop().map(|entity| {
            self.monitor.unmark(entity);

            entity
        })
    }
}

impl<'a, 'b, C: Component> Drop for UpdateQueueReader<'a, 'b, C> {
    fn drop(&mut self) {
        self.monitor.forget(self.world_removes);
    }
}

trait AnyUpdateQueue: mopa::Any + Sync + Send {
    fn monitor(&self) -> RwLockReadGuard<Monitor>;
    fn monitor_mut(&mut self) -> RwLockWriteGuard<Monitor>;
}
mopafy!(AnyUpdateQueue);

impl<C: Component> AnyUpdateQueue for UpdateQueue<C> {
    fn monitor(&self) -> RwLockReadGuard<Monitor> {
        self.monitor.read()
    }

    fn monitor_mut(&mut self) -> RwLockWriteGuard<Monitor> {
        self.monitor.write()
    }
}

pub struct UpdateQueues {
    queues: FnvHashMap<ComponentType, Box<AnyUpdateQueue>>
}

impl UpdateQueues {
    pub fn new() -> Self {
        UpdateQueues {
            queues: FnvHashMap::default()
        }
    }

    pub fn register<C: Component>(&mut self) {
        self.queues.insert(ComponentType::of::<C>(), Box::new(UpdateQueue::<C>::new()));
    }

    pub fn get<C: Component>(&self) -> Option<&UpdateQueue<C>> {
        self.queues.get(&ComponentType::of::<C>())
                   .and_then(|queue| queue.downcast_ref())
    }

    pub fn clear_flags(&mut self) {
        for (_, queue) in &mut self.queues {
            queue.monitor_mut().clear_modified_flag()
        }
    }
}
