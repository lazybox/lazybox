pub mod storage;

use std::any::{Any, TypeId};
use std::fmt::Debug;
use super::HasComponent;

pub trait Component: Any {
    type Module: HasComponent<Self>;
    type Template: Template;
}

pub trait Template: Any + Send + Sync + Debug + Clone {
    const NAME: &'static str;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ComponentType(TypeId);

impl ComponentType {
    pub fn of<C: Component>() -> Self {
        ComponentType(TypeId::of::<C>())
    }
}