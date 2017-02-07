use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct StorageLock<T: ?Sized>(RwLock<T>);

impl<T> StorageLock<T> {
    pub fn new(storage: T) -> Self {
        StorageLock(RwLock::new(storage))
    }
}

impl<T: ?Sized> StorageLock<T> {
    pub fn read(&self) -> StorageReadGuard<T> {
        StorageReadGuard(self.0.read())
    }

    pub fn write(&self) -> StorageWriteGuard<T> {
        StorageWriteGuard(self.0.write())
    }
}

pub struct StorageReadGuard<'a, T: ?Sized + 'a>(RwLockReadGuard<'a, T>);

impl<'a, T: ?Sized + 'a> Deref for StorageReadGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

pub struct StorageWriteGuard<'a, T: ?Sized + 'a>(RwLockWriteGuard<'a, T>);

impl<'a, T: ?Sized + 'a> Deref for StorageWriteGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'a, T: ?Sized + 'a> DerefMut for StorageWriteGuard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}
