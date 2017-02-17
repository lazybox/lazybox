mod filter;
pub use self::filter::Filter;

use policy::IdSet;
use entity;
use state::UpdateMonitors;
use fnv::FnvHashMap;
use std::any::{Any, TypeId};
use rayon::prelude::*;

pub struct Interface {
    filter: Filter,
    entities: IdSet,
}

impl Interface {
    pub fn new(filter: Filter) -> Self {
        Interface {
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
        let modified_iter = self.filter.require.iter().chain(self.filter.reject.iter());

        for &component_type in modified_iter {
            if monitors.monitor(component_type).modified() {
                return true;
            }
        }

        return false;
    }
}

pub trait InterfaceToken: Any + Send + Sync {
    fn name() -> &'static str;
    fn filter() -> Filter;
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct InterfaceType(TypeId);

impl InterfaceType {
    pub fn of<T: InterfaceToken>() -> Self {
        InterfaceType(TypeId::of::<T>())
    }
}

type InterfaceIndex = usize;

pub struct Interfaces {
    interfaces: Vec<Interface>,
    type_to_index: FnvHashMap<InterfaceType, InterfaceIndex>,
}

impl Interfaces {
    pub fn new() -> Self {
        Interfaces {
            interfaces: Vec::new(),
            type_to_index: FnvHashMap::default(),
        }
    }

    pub fn insert(&mut self, interface_type: InterfaceType, interface: Interface) {
        use std::collections::hash_map::Entry;

        match self.type_to_index.entry(interface_type) {
            Entry::Vacant(vacant) => {
                vacant.insert(self.interfaces.len());
                self.interfaces.push(interface);
            }
            Entry::Occupied(occupied_entry) => {
                self.interfaces[*occupied_entry.get()] = interface;
            }
        }
    }

    pub fn get(&self, interface_type: InterfaceType) -> Option<&Interface> {
        self.type_to_index
            .get(&interface_type)
            .and_then(|&index| self.interfaces.get(index))
    }

    pub fn commit(&mut self, monitors: &UpdateMonitors) {
        self.interfaces
            .par_iter_mut()
            .for_each(|interface| interface.commit(monitors));
    }
}
