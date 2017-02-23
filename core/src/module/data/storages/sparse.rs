//! The `Sparse` storage module
//!
use std::ops::{Index, IndexMut};
use vec_map::{self, VecMap};

use super::{Storage, DataComponent};
use Accessor;
use policy::Id;

#[derive(Debug)]
pub struct Sparse<V> {
    inner: VecMap<V>,
}

impl<V> Sparse<V> {
    pub fn new() -> Self {
        Sparse { inner: VecMap::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Sparse { inner: VecMap::with_capacity(capacity) }
    }

    /// Associate a new Component V to the entity
    #[inline]
    pub fn insert<'a>(&mut self, key: Accessor<'a>, component: V) -> bool {
        self.inner.insert(key.index(), component).is_none()
    }

    /// Detach a Component V from the entity
    #[inline]
    pub fn remove<'a>(&mut self, key: Accessor<'a>) {
        self.inner.remove(key.index());
    }

    /// Returns true if a component is associated to the entity
    #[inline]
    pub fn contains<'a>(&self, key: Accessor<'a>) -> bool {
        self.inner.contains_key(key.index())
    }

    /// Returns a immutable access to the associated component
    #[inline]
    pub fn get<'a>(&self, key: Accessor<'a>) -> Option<&V> {
        self.inner.get(key.index())
    }

    /// Returns a mutable access to the associated component
    #[inline]
    pub fn get_mut<'a>(&mut self, key: Accessor<'a>) -> Option<&mut V> {
        self.inner.get_mut(key.index())
    }

    /// An iterator visiting all component-entity pairs in arbitrary order.
    pub fn iter(&self) -> Iter<V> {
        Iter { inner: self.inner.iter() }
    }

    /// An iterator visiting all component-entity pairs in arbitrary order.
    pub fn iter_mut(&mut self) -> IterMut<V> {
        IterMut { inner: self.inner.iter_mut() }
    }
}

impl<V> Storage for Sparse<V>
    where V: DataComponent
{
    type Component = V;

    fn insert<'a>(&mut self, key: Accessor<'a>, component: V) -> bool {
        Sparse::<V>::insert(self, key, component)
    }

    fn remove<'a>(&mut self, key: Accessor<'a>) {
        Sparse::<V>::remove(self, key);
    }
}

impl<V> Default for Sparse<V> {
    fn default() -> Self {
        Sparse::new()
    }
}

impl<'a, V> Index<Accessor<'a>> for Sparse<V> {
    type Output = V;

    #[inline]
    fn index(&self, key: Accessor<'a>) -> &V {
        self.get(key).unwrap()
    }
}

impl<'a, V> IndexMut<Accessor<'a>> for Sparse<V> {
    #[inline]
    fn index_mut(&mut self, key: Accessor<'a>) -> &mut V {
        self.get_mut(key).unwrap()
    }
}


pub struct Iter<'a, V: 'a> {
    inner: vec_map::Iter<'a, V>,
}

impl<'a, V: 'a> Iterator for Iter<'a, V> {
    type Item = (Accessor<'a>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some((i, component)) => Some((unsafe { Accessor::new_unchecked(i as Id) }, component)),
            None => None,
        }
    }
}

impl<'a, V: 'a> IntoIterator for &'a Sparse<V> {
    type Item = (Accessor<'a>, &'a V);
    type IntoIter = Iter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IterMut<'a, V: 'a> {
    inner: vec_map::IterMut<'a, V>,
}

impl<'a, V: 'a> Iterator for IterMut<'a, V> {
    type Item = (Accessor<'a>, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some((i, component)) => Some((unsafe { Accessor::new_unchecked(i as Id) }, component)),
            None => None,
        }
    }
}

impl<'a, V: 'a> IntoIterator for &'a mut Sparse<V> {
    type Item = (Accessor<'a>, &'a mut V);
    type IntoIter = IterMut<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use module::data::DataComponent;
    use entity::Accessor;
    use policy::Id;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Dummy(usize);

    impl DataComponent for Dummy {
        type Storage = Sparse<Self>;
    }

    #[test]
    fn test_insert() {
        let mut sparse = Sparse::new();
        let entity = unsafe { Accessor::new_unchecked(0) };

        sparse.insert(entity, Dummy(0));

        assert_eq!(sparse.get(entity), Some(&Dummy(0)));
        assert_eq!(&sparse[entity], &Dummy(0));

        assert_eq!(sparse.get_mut(entity), Some(&mut Dummy(0)));
        assert_eq!(&mut sparse[entity], &mut Dummy(0));
    }

    #[test]
    fn test_insert_with_old_component() {
        let mut sparse = Sparse::new();
        let entity = unsafe { Accessor::new_unchecked(0) };

        assert_eq!(sparse.insert(entity, Dummy(0)), true);
        assert_eq!(sparse.insert(entity, Dummy(1)), false);
    }

    #[test]
    fn test_get_nonexistent() {
        let mut sparse: Sparse<Dummy> = Sparse::new();
        let non_existent = unsafe { Accessor::new_unchecked(0) };

        assert_eq!(sparse.get(non_existent), None);
        assert_eq!(sparse.get_mut(non_existent), None);
    }

    #[test]
    #[should_panic]
    fn test_indexing_nonexistent() {
        let mut sparse: Sparse<Dummy> = Sparse::new();
        let non_existent = unsafe { Accessor::new_unchecked(0) };

        assert_eq!(&sparse[non_existent], &Dummy(0));
        assert_eq!(&mut sparse[non_existent], &mut Dummy(0));
    }

    #[test]
    fn test_remove() {
        let mut sparse = Sparse::new();
        let entity = unsafe { Accessor::new_unchecked(0) };

        sparse.insert(entity, Dummy(0));
        sparse.remove(entity);

        assert_eq!(sparse.get(entity), None);
        assert_eq!(sparse.get_mut(entity), None);
    }

    #[test]
    #[should_panic]
    fn test_indexing_removed() {
        let mut sparse = Sparse::new();
        let entity = unsafe { Accessor::new_unchecked(0) };

        sparse.insert(entity, Dummy(0));
        sparse.remove(entity);

        assert_eq!(&sparse[entity], &Dummy(0));
        assert_eq!(&mut sparse[entity], &mut Dummy(0));
    }

    #[test]
    fn test_iter() {
        let mut sparse = Sparse::new();

        let entity1 = insert_for_entity(&mut sparse, 0, Dummy(0));
        let entity2 = insert_for_entity(&mut sparse, 1, Dummy(1));

        {
            let mut iter = sparse.iter();
            assert_eq!(iter.next(), Some((entity1, &Dummy(0))));
            assert_eq!(iter.next(), Some((entity2, &Dummy(1))));
        }

        {
            let mut iter_mut = sparse.iter_mut();
            assert_eq!(iter_mut.next(), Some((entity1, &mut Dummy(0))));
            assert_eq!(iter_mut.next(), Some((entity2, &mut Dummy(1))));
        }
    }

    fn insert_for_entity<'a, V: DataComponent>(sparse: &mut Sparse<V>,
                                               entity: Id,
                                               component: V)
                                               -> Accessor<'a> {
        let entity = unsafe { Accessor::new_unchecked(entity) };
        sparse.insert(entity, component);

        entity
    }
}
