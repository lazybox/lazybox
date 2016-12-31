pub mod filter;

pub use self::filter::Filter;

use bit_set::BitSet;
use policy::{Id};
use std::collections::HashSet;
use entity;
use state::UpdateMonitors;

pub struct Group {
    filter: Filter,
    entities: IdSet,
}

impl Group {
    pub fn new(filter: Filter) -> Self {
        Group {
            filter: Filter,
            entities: IdSet::new(),
        }
    }

    pub fn entities(&self) -> entity::iter::SetIter {
        unsafe { entity::iter::accessors_from_set(&self.entities) }
    }

    pub fn commit(&mut self, monitors: &UpdateMonitors, world_removes: &[Entity]) {
        if !Self::has_been_modified(monitors) {
            return;
        }

        self.update_with(monitors);
        self.forget(world_removes);
    }

    pub fn update_with(&mut self, monitors: &UpdateMonitors) {
        for &component_type in self.filter.required {
            let monitor = monitors.get(component_type);

            self.entities.union_with(monitor.entities());
        }

        for &component_type in self.filter.rejected {
            let monitor = monitors.get(component_type);

            self.entities.difference_with(monitor.entities());
        }
    }

    fn has_been_modified(monitors: &UpdateMonitors) -> bool {
        for &component_type in self.filter.required {
            if monitors.get(component_type).modified() {
                return true;
            }
        }

        for &component_type in self.fiter.rejected {
            if monitors.get(component_type).modified() {
                return true;
            }
        }

        return false;
    }

    fn forget(&mut self, entities: &[Entity]) {
        for entity in entities {
            self.entities.remove(entity.id() as usize)
        }
    }
}