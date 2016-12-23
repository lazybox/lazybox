pub mod component;
pub mod changeset;

use self::component::Component;
use self::component::storage::{StorageReadGuard, StorageWriteGuard};
use self::changeset::ChangeSetMap;

use std::any::{Any, TypeId};
use rayon::Scope;
use fnv::FnvHashMap;

pub trait Module<Cx: Send>: Any + Sync {
    fn get_type(&self) -> ModuleType { ModuleType(TypeId::of::<Self>()) }

    fn update<'a: 'scope, 'scope>(&'a mut self, scope: &Scope<'scope>, context: &'a mut Cx) {}
    fn changesets(&self) -> &ChangeSetMap;
}

impl<Cx: Send> Module<Cx> {
    #[inline]
    pub fn is<M: Module<Cx>>(&self) -> bool {
        ModuleType::of::<M, Cx>() == self.get_type()
    }

    #[inline]
    pub fn downcast_ref<M: Module<Cx>>(&self) -> Option<&M> {
        if self.is::<M>() {
            unsafe {
                Some(self.downcast_ref_unchecked())
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn downcast_mut<M: Module<Cx>>(&mut self) -> Option<&mut M> {
        if self.is::<M>() {
            unsafe {
                Some(self.downcast_mut_unchecked())
            }
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<M: Module<Cx>>(&self) -> &M {
        &*(self as *const Self as *const M)
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<M: Module<Cx>>(&mut self) -> &mut M {
        &mut *(self as *mut Self as *mut M)
    }
}

pub trait HasComponent<C: ?Sized + Component> {
    type Storage: ?Sized;

    fn read(&self) -> StorageReadGuard<Self::Storage>;
    fn write(&self) -> StorageWriteGuard<Self::Storage>;
}


#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ModuleType(TypeId);

impl ModuleType {
    pub fn of<M: Module<Cx>, Cx: Send>() -> Self {
        ModuleType(TypeId::of::<M>())
    }
}

pub struct Modules<Cx: Send> {
    modules: FnvHashMap<ModuleType, Box<Module<Cx>>>,
}

impl<Cx: Send> Modules<Cx> {
    pub fn new() -> Self {
        Modules {
            modules: FnvHashMap::default()
        }
    }

    pub fn insert(&mut self, module: Box<Module<Cx>>) {
        self.modules.insert(module.get_type(), module);
    }

    pub fn get<M: Module<Cx>>(&self) -> Option<&M> {
        self.modules.get(&ModuleType::of::<M, Cx>())
                    .and_then(|module| module.downcast_ref())
    }
}