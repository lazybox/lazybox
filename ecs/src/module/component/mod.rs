pub mod storage;

use std::any::{Any, TypeId};
use std::fmt::Debug;
use super::HasComponent;

pub use self::storage::{StorageLock, StorageWriteGuard, StorageReadGuard};

pub trait Component: Any {
    type Module: HasComponent<Self>;
    type Template: Template;
}

pub trait Template: Any + Send + Sync + Debug + Clone {
    fn name() -> &'static str where Self: Sized;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ComponentType(TypeId);

impl ComponentType {
    pub fn of<C: Component>() -> Self {
        ComponentType(TypeId::of::<C>())
    }
}