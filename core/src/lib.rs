pub extern crate winit;
pub extern crate rayon;
pub extern crate alga;
pub extern crate nalgebra;
extern crate parking_lot;
extern crate crossbeam;
extern crate vec_map;
extern crate bit_set;
extern crate daggy;
extern crate fnv;
#[macro_use]
extern crate mopa;
extern crate yaml_rust;
#[macro_use]
extern crate error_chain;

pub mod policy;
pub mod entity;
pub mod component;
#[macro_use]
pub mod module;
pub mod state;
pub mod spawn;
pub mod interface;
pub mod processor;
pub mod sync;
pub mod event;
pub mod settings;
pub mod assets;
pub mod inputs;
pub mod time;
mod tag;
mod group;

pub use entity::{Entity, EntityRef, Accessor, Entities};
pub use component::{Component, ComponentType};
pub use component::storage::{StorageLock, StorageReadGuard, StorageWriteGuard};
pub use module::{Module, HasComponent, ModuleType, Modules};
pub use state::{StateBuilder, State, Commit, Context};
pub use spawn::SpawnRequest;
pub use interface::{Interface, InterfaceToken, Filter, InterfaceType, Interfaces};
pub use processor::{Processor, StateAccess, UpdateType};
pub use tag::Tag;
pub use group::GroupToken;
pub use module::data::{DataComponent, DataModuleBuilder};
pub use settings::Settings;
