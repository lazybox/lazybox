use entity::Accessor;
use policy::Id;
use bit_set::{self,  BitSet};
use fnv::FnvHashMap;
use std::ops::{Index, IndexMut};

use super::component::ComponentType;

pub struct ChangeSet {
    entities: BitSet,
}

impl ChangeSet {
    pub fn new() -> Self {
        ChangeSet {
            entities: BitSet::new()
        }
    }

    pub fn mark(&mut self, entity: Accessor) {
        self.entities.insert(entity.id() as usize);
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn iter(&self) -> Iter {
        Iter {
            inner: self.entities.iter()
        }
    }
}

impl<'a> IntoIterator for &'a ChangeSet {
    type Item = Id;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a> {
    inner: bit_set::Iter<'a, u32>
}

impl<'a> Iterator for Iter<'a> {
    type Item = Id;

    #[inline]
    fn next(&mut self) -> Option<Id> {
        self.inner.next().map(|id| id as Id)
    }
}

pub struct ChangeSetMap {
    changesets: FnvHashMap<ComponentType, ChangeSet>
}

impl ChangeSetMap {
    pub fn new() -> Self {
        ChangeSetMap {
            changesets: FnvHashMap::default()
        }
    }

    pub fn insert(&mut self, component_type: ComponentType, changeset: ChangeSet) -> Option<ChangeSet> {
        self.changesets.insert(component_type, changeset)
    }

    pub fn get(&self, component_type: ComponentType) -> Option<&ChangeSet> {
        self.changesets.get(&component_type)
    }

    pub fn get_mut(&mut self, component_type: ComponentType) -> Option<&mut ChangeSet> {
        self.changesets.get_mut(&component_type)
    }

    pub fn take(&mut self, component_type: ComponentType) -> Option<ChangeSet> {
        self.changesets.remove(&component_type)
    }
}

impl Index<ComponentType> for ChangeSetMap {
    type Output = ChangeSet;

    #[inline]
    fn index(&self, component_type: ComponentType) -> &Self::Output {
        self.get(component_type).unwrap()
    }
}

impl IndexMut<ComponentType> for ChangeSetMap {

    #[inline]
    fn index_mut(&mut self, component_type: ComponentType) -> &mut Self::Output {
        self.get_mut(component_type).unwrap()
    }
}

