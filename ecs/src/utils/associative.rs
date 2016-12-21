use std::mem;
use std::ops::{Index, IndexMut};
use std::slice;

/// An `AssociativeVec` entry
#[derive(Debug, Clone)]
pub struct Entry<K, V> {
    /// The associated key of the value `V`
    pub key: K,
    /// The actual value
    pub value: V,
}

impl<K, V> Entry<K, V> {
    /// Constructs a new entry with the givven `key` and `value`
    fn new(key: K, value: V) -> Self {
        Entry {
            key: key,
            value: value,
        }
    }
}

/// An associative container that allow iterating over a slice.
///
/// This is useful to perform parrallel operations and keeping a map like
/// behaviour
#[derive(Debug, Clone)]
pub struct AssociativeVec<K: Eq, V> {
    entries: Vec<Entry<K, V>>,
}

impl<K: Eq, V> AssociativeVec<K, V> {
    /// Constructs a new empty `AssociativeVec`
    pub fn new() -> Self {
        AssociativeVec { entries: Vec::new() }
    }

    /// Inserts a new `value` with the corresponding `key`
    ///
    /// If the a value with the same key already exists it will be
    /// replaced and the old value will be returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(old_value) = self.get_mut(&key) {
            return Some(mem::replace(old_value, value));
        }

        self.entries.push(Entry::new(key, value));
        None
    }

    /// Removes the `value` associated with `key`
    ///
    /// If the value is not present, `None` is returned.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let opt_index = self.entries
            .iter()
            .enumerate()
            .find(|&(i, ref entry)| entry.key == *key)
            .map(|(i, _)| i);

        opt_index.map(|index| self.entries.swap_remove(index).value)
    }

    /// Returns the number of values in the container
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns a slice over entries of this container
    pub fn entries(&self) -> &[Entry<K, V>] {
        &self.entries
    }

    /// Returns a mutable slice over entries of this container
    pub fn entries_mut(&mut self) -> &mut [Entry<K, V>] {
        &mut self.entries
    }

    /// Returns a reference to the value associated with the given `key`
    ///
    /// If the value is not found, it returns `None`.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries
            .iter()
            .find(|entry| entry.key == *key)
            .map(|entry| &entry.value)
    }

    /// Returns a mutable reference to the value associated with the given `key`
    ///
    /// If the value is not found, it returns `None`.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.entries
            .iter_mut()
            .find(|entry| entry.key == *key)
            .map(|entry| &mut entry.value)
    }

    /// Returns `true` if a value with the `key` is present in the container.
    pub fn contains_key(&self, key: &K) -> bool {
        self.entries
            .iter()
            .find(|entry| entry.key == *key)
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut associative_vec = AssociativeVec::new();

        let key = 0;
        let value = 0;

        let old_value = associative_vec.insert(key, value);

        assert_eq!(None, old_value);
        assert_eq!(associative_vec.len(), 1);
        assert_eq!(Some(&value), associative_vec.get(&key));
        assert_eq!(Some(&mut value), associative_vec.get_mut(&key));

        let new_value = 1;
        let old_value = associative_vec.insert(key, new_value);

        assert_eq!(Some(value), old_value);
        assert_eq!(associative_vec.len(), 1);
        assert_eq!(Some(&new_value), associative_vec.get(&key));
        assert_eq!(Some(&mut new_value), associative_vec.get_mut(&key));
    }
}
