pub mod component;
pub mod changeset;

use self::component::Component;
use self::component::storage::{StorageReadGuard, StorageWriteGuard};
use self::changeset::ChangeSetMap;

use std::any::{Any, TypeId};
use std::collections::hash_map;
use rayon::Scope;
use fnv::FnvHashMap;


pub trait Module<Cx: Send>: Any + Sync {
    fn get_type(&self) -> ModuleType { ModuleType(TypeId::of::<Self>()) }

    fn update<'a: 'scope, 'scope>(&'a mut self, _scope: &Scope<'scope>, _context: &'a mut Cx) {}
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


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

    pub fn iter(&self) -> Iter<Cx> {
        Iter {
            inner: self.modules.iter()
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<Cx> {
        IterMut {
            inner: self.modules.iter_mut()
        }
    }
}

pub struct Iter<'a, Cx: Send + 'a> {
    inner: hash_map::Iter<'a, ModuleType, Box<Module<Cx>>>,
}

impl<'a, Cx: Send + 'a> Iterator for Iter<'a, Cx> {
    type Item = (ModuleType, &'a Module<Cx>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(&module_type, module)| (module_type, &**module))
    }
}

impl<'a, Cx: Send + 'a> IntoIterator for &'a Modules<Cx> {
    type Item = (ModuleType, &'a Module<Cx>);
    type IntoIter = Iter<'a, Cx>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}


pub struct IterMut<'a, Cx: Send + 'a> {
    inner: hash_map::IterMut<'a, ModuleType, Box<Module<Cx>>>
}

impl<'a, Cx: Send + 'a> Iterator for IterMut<'a, Cx> {
    type Item = (ModuleType, &'a mut Module<Cx>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(&module_type, module)| (module_type, &mut **module))
    }
}

impl<'a, Cx: Send> IntoIterator for &'a mut Modules<Cx> {
    type Item = (ModuleType, &'a mut Module<Cx>);
    type IntoIter = IterMut<'a, Cx>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}