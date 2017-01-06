use std::borrow::Borrow;
use std::cmp::Eq;
use std::hash::Hash;
use std::collections::hash_map;
use fnv::FnvHashMap;

#[derive(Debug)]
pub struct NameMap<T: Hash + Eq + Clone + Copy> {
    name_to: FnvHashMap<String, T>,
    to_name: FnvHashMap<T, String>,
}

impl<T: Hash + Eq + Clone + Copy> NameMap<T> {
    pub fn new() -> Self {
        NameMap {
            name_to: FnvHashMap::default(),
            to_name: FnvHashMap::default(),
        }
    }

    pub fn insert(&mut self, t: T, name: String) {
        self.name_to.insert(name.clone(), t);
        self.to_name.insert(t, name);
    }

    pub fn name_of(&self, t: &T) -> Option<&str> {
        self.to_name.get(&t).map(|s| s as &str)
    }

    pub fn of_name<S: Borrow<str>>(&self, name: S) -> Option<&T> {
        self.name_to.get(name.borrow())
    }
}

pub type Iter<'a, T> = hash_map::Iter<'a, T, String>;