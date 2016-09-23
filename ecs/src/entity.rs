use policy::{Id, Version};
use std::sync::atomic::{AtomicUsize, Ordering};
use crossbeam::sync::SegQueue;

/// Represents an unique entity in the world.
/// There is no data associated to it.
#[derive(Copy, Clone, Debug, Hash, PartialEq, PartialOrd)]
pub struct Entity(Id, Version);

impl Entity {
    pub fn index(&self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn id(&self) -> Id {
        self.0
    }

    #[inline]
    pub fn version(&self) -> Version {
        self.1
    }

    #[inline]
    pub unsafe fn accessor(&self) -> Accessor {
        Accessor::new_unchecked(self.0)
    }
}


/// A reference that keeps track of a particular entity
///
/// It is used to keep track of an entity across frames.
/// You can use the method `updgrade_entity_ref` on the state to get an accessor from it.
#[derive(Copy, Clone, Debug, Hash, PartialEq, PartialOrd)]
pub struct EntityRef(Entity);

/// An entity accessor.
/// It is the key to access and modify entity components
///
/// It is guarented that the entity is alive while you have an accessor to it.
#[derive(Copy, Clone, Debug)]
pub struct Accessor<'a> {
    id: Id,
    bound_lifetime: PhantomData<&'a ()>,
}

impl<'a> Accessor<'a> {
    #[inline]
    pub unsafe fn new_unchecked(id: Id) -> Self {
        Accessor {
            id: id,
            bound_lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.id as usize
    }

    #[inline]
    pub fn id(&self) -> Id {
        self.id
    }
}

#[derive(Debug)]
struct Pool {
    counter: AtomicUsize,
    availables: SegQueue<Entity>,
    to_be_recycled: SegQueue<Entity>,
}

impl Pool {
    pub fn new() -> Self {
        Pool {
            counter: AtomicUsize::new(),
            availables: SegQueue::new(),
            to_be_recycled: SegQueue::new(),
        }
    }

    pub fn acquire(&self) -> Entity {
        match self.availables.try_pop() {
            Some(entity) => entity,
            None => {
                debug_assert!(self.counter.load(Ordering::Relaxed) != policy::max_entity_count(),
                             "max entity count reached");

                let next = self.counter.fetch_add(1, Ordering::Relaxed);
                Entity(next as Id, 0)
            }
        }
    }
}