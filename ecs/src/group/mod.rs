pub mod filter;

pub use self::filter::Filter;

use policy::IdSet;
use entity;
use state::UpdateMonitors;
use fnv::FnvHashMap;
use std::any::{Any, TypeId};
use rayon::prelude::*;

pub struct Group {
    filter: Filter,
    entities: IdSet,
}

impl Group {
    pub fn new(filter: Filter) -> Self {
        Group {
            filter: filter,
            entities: IdSet::new(),
        }
    }

    pub fn entities(&self) -> entity::iter::SetIter {
        unsafe { entity::iter::accessors_from_set(&self.entities) }
    }

    pub fn commit(&mut self, monitors: &UpdateMonitors) {
        if !self.has_been_modified(monitors) {
            return;
        }

        self.update_with(monitors);
    }

    pub fn update_with(&mut self, monitors: &UpdateMonitors) {
        self.entities.clear();
        for &component_type in &self.filter.require {
            let monitor = monitors.monitor(component_type);

            self.entities.union_with(monitor.entities());
        }

        for &component_type in &self.filter.reject {
            let monitor = monitors.monitor(component_type);

            self.entities.difference_with(monitor.entities());
        }
    }

    fn has_been_modified(&self, monitors: &UpdateMonitors) -> bool {
        for &component_type in &self.filter.require {
            if monitors.monitor(component_type).modified() {
                return true;
            }
        }

        for &component_type in &self.filter.reject {
            if monitors.monitor(component_type).modified() {
                return true;
            }
        }

        return false;
    }
}

pub trait GroupToken: Any + Send + Sync {
    fn name() -> &'static str;
    fn filter() -> Filter;
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct GroupType(TypeId);

impl GroupType {
    pub fn of<T: GroupToken>() -> Self {
        GroupType(TypeId::of::<T>())
    }
}

type GroupIndex = usize;

pub struct Groups {
    groups: Vec<Group>,
    type_to_index: FnvHashMap<GroupType, GroupIndex>
}

impl Groups {
    pub fn new() -> Self {
        Groups {
            groups: Vec::new(),
            type_to_index: FnvHashMap::default()
        }
    }

    pub fn insert(&mut self, group_type: GroupType, group: Group) {
        use std::collections::hash_map::Entry;
        
        match self.type_to_index.entry(group_type) {
            Entry::Vacant(vacant) => {
                vacant.insert(self.groups.len());
                self.groups.push(group);
            }
            Entry::Occupied(occupied_entry) => {
                self.groups[*occupied_entry.get()] = group;
            }
        }
    }

    pub fn get(&self, group_type: GroupType) -> Option<&Group> {
        self.type_to_index.get(&group_type)
                          .and_then(|&index| self.groups.get(index))
    }

    pub fn commit(&mut self, monitors: &UpdateMonitors) {
        self.groups.par_iter_mut()
                   .for_each(|group| group.commit(monitors));
    }
}
