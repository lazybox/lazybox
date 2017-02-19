pub mod storages;
pub mod builder;

pub use self::storages::Storage;
pub use self::storages::packed::Packed as PackedStorage;
pub use self::builder::DataModuleBuilder;

use state::CommitArgs;
use {Context, Module, HasComponent};
use component::{Component, Template, ComponentType};
use {StorageReadGuard, StorageWriteGuard};
use fnv::FnvHashMap;
use rayon;
use std::fmt::Debug;
use std::any::Any;

use self::storages::{StorageHandler, Handler};


pub trait DataComponent: Any + Clone + Debug + Send + Sync {
    type Storage: Storage;
}

impl<C: DataComponent> Component for C {
    type Template = Self;
    type Module = DataModule;
}

impl<C: DataComponent> Template for C {}

pub struct DataModule {
    handlers: FnvHashMap<ComponentType, Box<Handler>>,
}

impl DataModule {
    pub fn new() -> Self {
        DataModule { handlers: FnvHashMap::default() }
    }

    pub fn register<D: DataComponent>(&mut self, storage: D::Storage) {
        let handler = StorageHandler::new(storage);
        self.handlers.insert(ComponentType::of::<D>(), Box::new(handler));
    }

    pub fn read<D: DataComponent>(&self) -> Option<StorageReadGuard<D::Storage>> {
        self.handlers
            .get(&ComponentType::of::<D>())
            .and_then(|handler| handler.downcast_ref::<StorageHandler<D::Storage>>())
            .map(|handler| handler.storage.read())
    }

    pub fn write<D: DataComponent>(&self) -> Option<StorageWriteGuard<D::Storage>> {
        self.handlers
            .get(&ComponentType::of::<D>())
            .and_then(|handler| handler.downcast_ref::<StorageHandler<D::Storage>>())
            .map(|handler| handler.storage.write())
    }
}

impl<Cx> Module<Cx> for DataModule {
    fn commit(&mut self, args: &CommitArgs, _context: &mut Cx) {
        rayon::scope(|scope| for (_, handler) in &mut self.handlers {
            scope.spawn(move |_| handler.commit(args));
        });
    }
}

impl<C: DataComponent + Component> HasComponent<C> for DataModule {
    type Storage = C::Storage;

    fn read(&self) -> StorageReadGuard<Self::Storage> {
        self.read::<C>()
            .expect("the data component has not been registered")
    }

    fn write(&self) -> StorageWriteGuard<Self::Storage> {
        self.write::<C>()
            .expect("the data component has not been registered")
    }
}
