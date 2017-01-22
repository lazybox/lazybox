pub mod component;

pub use self::component::{Component, Template, ComponentType};
pub use self::component::storage::{StorageLock, StorageReadGuard, StorageWriteGuard};


use std::any::TypeId;
use std::collections::hash_map;
use fnv::FnvHashMap;
use ecs::state::CommitArgs;
use mopa;
use ::context::Context;

pub trait Module: mopa::Any + Send + Sync {
    fn get_type(&self) -> ModuleType { ModuleType(TypeId::of::<Self>()) }
    fn commit(&mut self, args: &CommitArgs, context: &mut Context);
}
mopafy!(Module);

pub trait HasComponent<C: ?Sized + Component> {
    type Storage: ?Sized;

    fn read(&self) -> StorageReadGuard<Self::Storage>;
    fn write(&self) -> StorageWriteGuard<Self::Storage>;
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ModuleType(TypeId);

impl ModuleType {
    pub fn of<M: Module>() -> Self {
        ModuleType(TypeId::of::<M>())
    }
}

pub struct Modules {
    modules: FnvHashMap<ModuleType, Box<Module>>,
}

impl Modules {
    pub fn new() -> Self {
        Modules { modules: FnvHashMap::default() }
    }

    pub fn insert(&mut self, module: Box<Module>) {
        self.modules.insert(module.get_type(), module);
    }

    pub fn get<M: Module>(&self) -> Option<&M> {
        self.modules
            .get(&ModuleType::of::<M>())
            .and_then(|module| module.downcast_ref())
    }

    pub fn commit(&mut self, args: &CommitArgs, cx: &mut Context) {
        for (_, module) in &mut self.modules {
            module.commit(args, cx);
        }
    }

    pub fn iter(&self) -> Iter {
        Iter { inner: self.modules.iter() }
    }

    pub fn iter_mut(&mut self) -> IterMut {
        IterMut { inner: self.modules.iter_mut() }
    }
}

pub struct Iter<'a> {
    inner: hash_map::Iter<'a, ModuleType, Box<Module>>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (ModuleType, &'a Module);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(&module_type, module)| (module_type, &**module))
    }
}

impl<'a> IntoIterator for &'a Modules {
    type Item = (ModuleType, &'a Module);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}


pub struct IterMut<'a> {
    inner: hash_map::IterMut<'a, ModuleType, Box<Module>>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = (ModuleType, &'a mut Module);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(&module_type, module)| (module_type, &mut **module))
    }
}

impl<'a> IntoIterator for &'a mut Modules {
    type Item = (ModuleType, &'a mut Module);
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
