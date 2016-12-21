use std::any::{Any, TypeId};
use crossbeam::sync::SegQueue;
use policy::Id;
use utils::AssociativeVec;
use std::mem;
use mopa;

pub trait Storage: Any + Sync + Send + Clone {}

trait AnyStorage: mopa::Any + Sync + Send {}
mopafy!(AnyStorage);

impl<T: Storage> AnyStorage for T {}

pub trait Template {
    const NAME: &'static str;
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ComponentType(TypeId);

impl ComponentType {
    pub fn of<C: Component>() -> Self {
        ComponentType(TypeId::of::<C>())
    }
}

pub trait Component: 'static {
    type Template: Template;
    type Storage: Storage;
}

type InsertEntry<T> = (Id, T);

pub struct ChangeQueue<C: Component> {
    to_remove: SegQueue<Id>,
    to_insert: SegQueue<InsertEntry<C::Template>>,
}

impl<C: Component> ChangeQueue<C> {
    pub fn new() -> Self {
        ChangeQueue {
            to_remove: SegQueue::new(),
            to_insert: SegQueue::new(),
        }
    }
}

pub struct Components {
    storages: AssociativeVec<ComponentType, Box<AnyStorage>>,
}

impl Components {
    pub fn new() -> Self {
        Components { storages: AssociativeVec::new() }
    }

    pub fn insert<C: Component>(&mut self, mut storage: C::Storage) {
        if let Some(old_storage) = self.storages.get_mut(&ComponentType::of::<C>()) {
            let old_storage = old_storage.downcast_mut::<C::Storage>().unwrap();
            mem::swap(old_storage, &mut storage);

            return;
        }

        self.storages.insert(ComponentType::of::<C>(), Box::new(storage));
    }

    pub fn put<C: Component>(&mut self, storage: Box<C::Storage>) {
        self.storages.insert(ComponentType::of::<C>(), storage);
    }

    pub fn take<C: Component>(&mut self) -> Option<Box<C::Storage>> {
        self.storages
            .remove(&ComponentType::of::<C>())
            .map(|storage| storage.downcast())
            .and_then(Result::ok)
    }

    pub fn clone<C: Component>(&mut self) -> Option<C::Storage> {
        self.storages
            .get(&ComponentType::of::<C>())
            .and_then(|storage| storage.downcast_ref::<C::Storage>())
            .map(|storage| storage.clone())
    }
}
