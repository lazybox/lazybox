use std::any::Any;
use mopa;
use entity::Accessor;
use module::CommitArgs;
use module::component::{Component, StorageLock, StorageReadGuard, StorageWriteGuard};
use module::changeset::ChangeSet;
use super::DataComponent;
use std::fmt::Debug;

/// Defines any `DataComponent` storage that can be used.
///
/// If you want to define a special storage for a `DataComponent` you need to implement this trait.
pub trait Storage: Any + Debug + Send + Sync {
    /// The component that the storage is holding.
    type Component: DataComponent;

    fn insert<'a>(&mut self, accessor: Accessor<'a>, component: Self::Component) -> bool;
    fn remove<'a>(&mut self, accessor: Accessor<'a>);
}

/// Represents a storage Handler.
///
/// This is used internally to abstract component storages.
pub trait Handler: mopa::Any + Send + Debug + Sync {
    fn commit(&mut self, args: &CommitArgs);
}
mopafy!(Handler);

#[derive(Debug)]
pub struct StorageHandler<S: Storage> {
    pub storage: StorageLock<S>,
}

impl<S: Storage> StorageHandler<S> {
    pub fn new(storage: S) -> Self {
        StorageHandler { storage: StorageLock::new(storage) }
    }
}

impl<S: Storage> Handler for StorageHandler<S> {
    fn commit(&mut self, args: &CommitArgs) {
        let mut storage = self.storage.write();

        for entity in args.world_removes {
            let accessor = unsafe { Accessor::new_unchecked(entity.id()) };
            storage.remove(accessor);
        }

        for request in args.requests {
            let accessor = unsafe { Accessor::new_unchecked(request.entity().id()) };

            if let Some(component) = request.get::<S::Component>(args.prototypes) {
                storage.insert(accessor, component);
            }
        }
    }
}