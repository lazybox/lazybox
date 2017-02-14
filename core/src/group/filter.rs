//! The filter module
//!

use {Component, ComponentType};
use fnv::FnvHashSet;

/// Represents a filter of entities
///
/// It is defined by components that an entity must and must not have
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Filter {
    /// Components that are required
    pub require: FnvHashSet<ComponentType>,
    /// Components that are rejected
    pub reject: FnvHashSet<ComponentType>,
}

impl Filter {
    /// Creates a new empty `Filter`
    pub fn new() -> Self {
        Filter {
            require: FnvHashSet::default(),
            reject: FnvHashSet::default(),
        }
    }

    /// Adds the component `C` as a *must have* constraint
    pub fn require<C: Component>(mut self) -> Self {
        self.require.insert(ComponentType::of::<C>());
        self
    }

    /// Adds the component `C` as a *must not have* constraint
    pub fn reject<C: Component>(mut self) -> Self {
        self.reject.insert(ComponentType::of::<C>());
        self
    }
}
