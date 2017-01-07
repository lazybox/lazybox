pub mod iter;

use policy::{Id, Version};
use std::sync::atomic::{AtomicUsize, Ordering};
use crossbeam::sync::SegQueue;
use std::marker::PhantomData;
use policy;
use vec_map::{self, VecMap};

/// Represents an unique entity in the world.
/// There is no data associated to it.
#[derive(Copy, Clone, Debug, Hash)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
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

    fn next_version(self) -> Self {
        Entity(self.0, self.1.wrapping_add(1))
    }
}


/// A reference that keeps track of a particular entity
///
/// It is used to keep track of an entity across frames.
/// You can use the method `updgrade_entity_ref` on the state to get an accessor from it.
#[derive(Copy, Clone, Debug, Hash)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityRef(Entity);

impl EntityRef {
    #[inline]
    pub fn from_entity(entity: Entity) -> Self {
        EntityRef(entity)
    }
}

/// An entity accessor.
/// It is the key to access and modify entity components
///
/// It is guaranted that the entity is alive while you have an accessor to it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
            counter: AtomicUsize::new(0),
            availables: SegQueue::new(),
            to_be_recycled: SegQueue::new(),
        }
    }

    pub fn acquire(&self) -> Entity {
        match self.availables.try_pop() {
            Some(entity) => entity.next_version(),
            None => {
                assert!(self.counter.load(Ordering::Relaxed) != policy::max_entity_count(),
                        "max entity count reached");

                let next = self.counter.fetch_add(1, Ordering::Relaxed);
                Entity(next as Id, 0)
            }
        }
    }

    pub fn free(&self, entity: Entity) {
        self.to_be_recycled.push(entity);
    }

    pub fn push_freed<F>(&self, mut hook: F)
        where F: FnMut(Entity) -> bool
    {
        while let Some(entity) = self.to_be_recycled.try_pop() {
            if hook(entity) {
                self.availables.push(entity);
            }
        }
    }
}


pub struct Entities {
    pool: Pool,
    versions: VecMap<Version>,
    spawns: SegQueue<Entity>,
}

impl Entities {
    pub fn new() -> Self {
        Entities {
            pool: Pool::new(),
            versions: VecMap::new(),
            spawns: SegQueue::new(),
        }
    }

    /// Creates an entity
    ///
    /// The entity is not considered alive until `spawn` is called
    pub fn create(&self) -> Entity {
        self.pool.acquire()
    }

    /// Spawns an entity when the state will be commited.
    pub fn spawn_later(&self, entity: Entity) -> EntityRef {
        self.spawns.push(entity);

        EntityRef(entity)
    }

    /// Spawns an entity making it alive
    pub fn spawn(&mut self, entity: Entity) -> EntityRef {
        assert!(!self.versions.contains_key(entity.index()));

        self.versions.insert(entity.index(), entity.version());

        EntityRef(entity)
    }

    /// Schedule the removal of an entity.
    ///
    /// The entity will be removed at the next state commit
    pub fn remove_later<'a>(&self, accessor: Accessor<'a>) {
        let entity = self.entity_from_accessor(accessor);
        self.pool.free(entity);
    }

    /// Creates a new entity iterator
    pub fn iter(&self) -> Iter {
        Iter { inner: self.versions.iter() }
    }

    /// Creates an entity reference
    pub fn entity_ref<'a>(&self, accessor: Accessor<'a>) -> EntityRef {
        let index = accessor.id as usize;

        EntityRef(Entity(accessor.id, self.versions[index]))
    }

    /// Uprades an `EntityRef` to an `Accessor`
    ///
    /// Returns None if the `Entity` has been killed
    pub fn upgrade(&self, entity_ref: EntityRef) -> Option<Accessor> {
        let version = self.versions
            .get(entity_ref.0.index())
            .unwrap_or_else(|| panic!("invalid entity index, this is a bug"));

        if &entity_ref.0.version() == version {
            Some(unsafe { Accessor::new_unchecked(entity_ref.0.id()) })
        } else {
            None
        }
    }

    /// Get an entity from an accessor.
    fn entity_from_accessor<'a>(&self, accessor: Accessor<'a>) -> Entity {
        Entity(accessor.id, self.versions[accessor.id as usize])
    }

    /// Removes all killed entities and returns them.
    ///
    /// The entities returned might be recycled in the future and need
    /// to be used with care.
    pub fn push_removes(&mut self) -> Vec<Entity> {
        let mut removed = Vec::new();

        let (pool, versions) = (&mut self.pool, &mut self.versions);

        pool.push_freed(|entity| {
            if versions.remove(entity.index()).is_some() {
                removed.push(entity);
                true
            } else {
                false
            }
        });

        removed
    }

    /// Commit the entities changes.
    pub fn commit(&mut self) {
        while let Some(entity) = self.spawns.try_pop() {
            self.spawn(entity);
        }
    }
}


/// An Iterator other entities.
pub struct Iter<'a> {
    inner: vec_map::Iter<'a, Version>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Accessor<'a>;

    #[inline]
    fn next(&mut self) -> Option<Accessor<'a>> {
        self.inner.next().map(&|(index, _)| unsafe { Accessor::new_unchecked(index as Id) })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::Entities;

    use policy::{self, Id};

    #[test]
    fn test_create_entity() {
        let entities = Entities::new();

        for i in 0..policy::max_entity_count() {
            let entity = entities.create();
            assert_eq!(entity, Entity(i as Id, 0));
        }
    }

    #[test]
    #[should_panic]
    fn test_create_above_limit() {
        let entities = Entities::new();

        for _ in 0..(policy::max_entity_count() + 1) {
            let _ = entities.create();
        }
    }

    #[test]
    fn test_spawn() {
        let mut entities = Entities::new();

        let entity = entities.create();
        let entity_ref = entities.spawn(entity);

        let expected_accessor = unsafe { Accessor::new_unchecked(entity_ref.0.id()) };
        assert_eq!(Some(expected_accessor), entities.upgrade(entity_ref));
    }

    #[test]
    #[should_panic]
    fn test_spawn_twice() {
        let mut entities = Entities::new();

        let entity = entities.create();
        entities.spawn(entity);
        entities.spawn(entity);
    }

    #[test]
    fn test_remove_later() {
        let mut entities = Entities::new();

        let entity_ref = remove_later_one_entity(&mut entities);
        assert_eq!(true, entities.upgrade(entity_ref).is_some());
    }

    #[test]
    fn test_remove() {
        let mut entities = Entities::new();

        let entity_ref = remove_later_one_entity(&mut entities);

        let removed_entities = entities.push_removes();
        let mut iter = removed_entities.into_iter();

        assert_eq!(Some(entity_ref.0), iter.next());
        assert_eq!(None, iter.next());

    }

    #[test]
    fn test_remove_twice() {
        let mut entities = Entities::new();

        let entity_ref = remove_later_one_entity(&mut entities);
        {
            let accessor = entities.upgrade(entity_ref).unwrap();
            entities.remove_later(accessor);
        }

        let removed_entities = entities.push_removes();
        let mut iter = removed_entities.into_iter();

        assert_eq!(Some(entity_ref.0), iter.next());
        assert_eq!(None, iter.next());

        // This check that we are not recycling the entity twice
        let recycled_entity = entities.create();
        let new_entity = entities.create();
        assert!(recycled_entity != new_entity);
    }

    #[test]
    fn test_recycling() {
        let mut entities = Entities::new();

        let removed = remove_later_one_entity(&mut entities);
        entities.push_removes();

        let recycled_entity = entities.create();
        assert_eq!(recycled_entity.0, removed.0.id());

        let expected_version = removed.0.version() + 1;
        assert_eq!(recycled_entity.1, expected_version);
    }

    #[test]
    fn test_remove_later_not_now() {
        // It test that a remove_later on a entity does not put
        // it in the available entities for recycling.
        let mut entities = Entities::new();

        let removed = remove_later_one_entity(&mut entities);

        let new_entity = entities.create();
        assert!(EntityRef(new_entity) != removed);
    }

    fn remove_later_one_entity(entities: &mut Entities) -> EntityRef {
        let entity = entities.create();
        let entity_ref = entities.spawn(entity);
        let accessor = entities.upgrade(entity_ref).unwrap();

        entities.remove_later(accessor);

        entity_ref
    }
}
