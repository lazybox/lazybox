//! The `Packed` storage module
//!
use std::ops::{Index, IndexMut};
use std::slice;
use vec_map::VecMap;

use ecs::entity::Accessor;
use ecs::policy::Id;
use super::{DataComponent, Storage};

/// A entry into the storage that associate a component with its link index
#[derive(Clone, Debug)]
struct Entry<V> {
    entity: Id,
    component: V,
}

impl<V> Entry<V> {
    /// Constructs a new entry
    pub fn new(entity: Id, component: V) -> Entry<V> {
        Entry {
            entity: entity,
            component: component,
        }
    }
}

type Link = Id;

/// A `Storage` that hold its values in a contiguous vector.
#[derive(Clone, Debug)]
pub struct Packed<V> {
    components: Vec<Entry<V>>,
    links: VecMap<Link>,
}

impl<V> Packed<V> {
    /// Constructs a new empty Packed<V>` storage.
    pub fn new() -> Packed<V> {
        Packed {
            components: Vec::new(),
            links: VecMap::new(),
        }
    }

    /// Constructs a new empty `Packed<V>` with the given `capacity`
    pub fn with_capacity(capacity: usize) -> Packed<V> {
        Packed {
            components: Vec::with_capacity(capacity),
            links: VecMap::with_capacity(capacity),
        }
    }

    /// Returns true if a component is associated to the entity
    pub fn contains<'a>(&self, key: Accessor<'a>) -> bool {
        self.links.contains_key(key.index())
    }

    /// Returns a immutable access to the associated component
    pub fn get<'a>(&self, key: Accessor<'a>) -> Option<&V> {
        if let Some(&link) = self.links.get(key.index()) {
            return self.components.get(link as usize).map(|entry| &entry.component);
        }
        None
    }
    
    /// Returns a mutable access to the associated component
    pub fn get_mut<'a>(&mut self, key: Accessor<'a>) -> Option<&mut V> {
        if let Some(&link) = self.links.get(key.index()) {
            return self.components.get_mut(link as usize).map(|entry| &mut entry.component);
        }
        None
    }

    /// An iterator visiting all component-entity pairs in arbitrary order.
    pub fn iter(&self) -> Iter<V> {
        Iter { inner: self.components.iter() }
    }

    /// An iterator visiting all component-entity pairs in arbitrary order.
    pub fn iter_mut(&mut self) -> IterMut<V> {
        IterMut { inner: self.components.iter_mut() }
    }
}

impl<V> Storage for Packed<V>
    where V: DataComponent
{
    type Component = V;

    fn insert<'a>(&mut self, key: Accessor<'a>, component: V) -> bool {
        let id = key.id();
        let index = id as usize;

        let entry = Entry::new(id, component);

        if let Some(&link) = self.links.get(index) {
            let link_index = link as usize;

            self.components[link_index] = entry;
            return false;
        }

        let link = self.components.len() as Id;
        self.links.insert(index, link);
        self.components.push(entry);

        true
    }

    fn remove<'a>(&mut self, key: Accessor<'a>) {
        let links = &mut self.links;
        if let Some(link) = links.remove(key.index()) {
            let link_index = link as usize;

            self.components.swap_remove(link_index);
            self.components.get(link_index).map(|entry| links.insert(entry.entity as usize, link));
        }
    }
}

impl<V> Default for Packed<V> {
    fn default() -> Self {
        Packed::new()
    }
}

impl<'a, V> Index<Accessor<'a>> for Packed<V>
    where V: DataComponent
{
    type Output = V;

    #[inline]
    fn index(&self, key: Accessor<'a>) -> &V {
        self.get(key).unwrap()
    }
}

impl<'a, V> IndexMut<Accessor<'a>> for Packed<V>
    where V: DataComponent
{
    #[inline]
    fn index_mut(&mut self, key: Accessor<'a>) -> &mut V {
        self.get_mut(key).unwrap()
    }
}


pub struct Iter<'a, V: 'a> {
    inner: slice::Iter<'a, Entry<V>>,
}

impl<'a, V: 'a> Iterator for Iter<'a, V> {
    type Item = (Accessor<'a>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|entry| {
            let accessor = unsafe { Accessor::new_unchecked(entry.entity) };
            (accessor, &entry.component)
        })
    }
}

impl<'a, V: 'a> IntoIterator for &'a Packed<V> {
    type Item = (Accessor<'a>, &'a V);
    type IntoIter = Iter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IterMut<'a, V: 'a> {
    inner: slice::IterMut<'a, Entry<V>>,
}

impl<'a, V: 'a> Iterator for IterMut<'a, V> {
    type Item = (Accessor<'a>, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|entry| {
            let accessor = unsafe { Accessor::new_unchecked(entry.entity) };
            (accessor, &mut entry.component)
        })
    }
}

impl<'a, V: 'a> IntoIterator for &'a mut Packed<V> {
    type Item = (Accessor<'a>, &'a mut V);
    type IntoIter = IterMut<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use modules::data::DataComponent;
    use ecs::entity::Accessor;
    use ecs::policy::Id;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Dummy(usize);

    impl DataComponent for Dummy {
        type Storage = Packed<Self>;
    }

    #[test]
    fn test_insert() {
        let mut packed = Packed::new();
        let entity = unsafe { Accessor::new_unchecked(0) }; 

        packed.insert(entity, Dummy(0));

        assert_eq!(packed.get(entity), Some(&Dummy(0)));
        assert_eq!(&packed[entity], &Dummy(0));

        assert_eq!(packed.get_mut(entity), Some(&mut Dummy(0)));
        assert_eq!(&mut packed[entity], &mut Dummy(0));
    }

    #[test]
    fn test_insert_with_old_component() {
        let mut packed = Packed::new();
        let entity = unsafe { Accessor::new_unchecked(0) };

        assert_eq!(packed.insert(entity, Dummy(0)), true);
        assert_eq!(packed.insert(entity, Dummy(1)), false);
    }

    #[test]
    fn test_get_nonexistent() {
        let mut packed: Packed<Dummy> = Packed::new();
        let non_existent = unsafe { Accessor::new_unchecked(0) };

        assert_eq!(packed.get(non_existent), None);
        assert_eq!(packed.get_mut(non_existent), None);
    }

    #[test]
    #[should_panic]
    fn test_indexing_nonexistent() {
        let mut packed: Packed<Dummy> = Packed::new();
        let non_existent = unsafe { Accessor::new_unchecked(0) };

        assert_eq!(&packed[non_existent], &Dummy(0));
        assert_eq!(&mut packed[non_existent], &mut Dummy(0));
    }

    #[test]
    fn test_remove() {
        let mut packed = Packed::new();
        let entity = unsafe { Accessor::new_unchecked(0) };

        packed.insert(entity, Dummy(0));
        packed.remove(entity);
    
        assert_eq!(packed.get(entity), None);
        assert_eq!(packed.get_mut(entity), None);
    }

    #[test]
    #[should_panic]
    fn test_indexing_removed() {
        let mut packed = Packed::new();
        let entity = unsafe { Accessor::new_unchecked(0) };

        packed.insert(entity, Dummy(0));
        packed.remove(entity);

        assert_eq!(&packed[entity], &Dummy(0));
        assert_eq!(&mut packed[entity], &mut Dummy(0));
    }

    #[test]
    fn test_iter() {
        let mut packed = Packed::new();

        let entity1 = insert_for_entity(&mut packed, 0, Dummy(0));
        let entity2 = insert_for_entity(&mut packed, 1, Dummy(1));

        {
            let mut iter = packed.iter();
            assert_eq!(iter.next(), Some((entity1, &Dummy(0))));
            assert_eq!(iter.next(), Some((entity2, &Dummy(1))));           
        }

        {
            let mut iter_mut = packed.iter_mut();
            assert_eq!(iter_mut.next(), Some((entity1, &mut Dummy(0))));
            assert_eq!(iter_mut.next(), Some((entity2, &mut Dummy(1))));     
        }
    }

    fn insert_for_entity<'a, V: DataComponent>(packed: &mut Packed<V>, entity: Id, component: V) -> Accessor<'a> {
        let entity = unsafe { Accessor::new_unchecked(entity) };
        packed.insert(entity, component);

        entity
    }
}
