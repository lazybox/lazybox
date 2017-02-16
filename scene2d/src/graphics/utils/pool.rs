use std::mem;
use std::ops::*;
use std::sync::{Mutex, MutexGuard};

pub struct Pool<T: 'static + Sized> {
    elements: Mutex<Vec<T>>,
}

impl<T: Default> Pool<T> {
    pub fn new() -> Self {
        Pool { elements: Mutex::new(Vec::new()) }
    }

    pub fn get(&self) -> Guard<T> {
        let mut elements = self.elements.lock().unwrap();
        let element = elements.pop().unwrap_or_default();

        Guard {
            element: element,
            pool: self,
        }
    }

    fn put(&self, element: T) {
        let mut elements = self.elements.lock().unwrap();
        elements.push(element);
    }

    /// Return all objects currently available in the pool.
    ///
    /// This method blocks the pool until the guard goes out of scope.
    pub fn availables(&self) -> MutexGuard<Vec<T>> {
        self.elements.lock().unwrap()
    }
}

pub struct Guard<'a, T: 'static + Default> {
    element: T,
    pool: &'a Pool<T>,
}

impl<'a, T: 'static + Default> Deref for Guard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

impl<'a, T: 'static + Default> DerefMut for Guard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.element
    }
}

impl<'a, T: 'static + Default> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        let element = mem::replace(&mut self.element, T::default());
        self.pool.put(element);
    }
}