extern crate rayon;
extern crate nalgebra;
extern crate parking_lot;
extern crate crossbeam;
extern crate vec_map;
extern crate bit_set;
extern crate daggy;
extern crate fnv;
#[macro_use]
extern crate mopa;
// #[macro_use]
// extern crate approx;

pub mod policy;
pub mod entity;
pub mod component;
#[macro_use]
pub mod module;
pub mod state;
pub mod spawn;
pub mod group;
pub mod processor;
pub mod math;
pub mod sync;

pub use entity::{Entity, EntityRef, Accessor, Entities};
pub use component::{Component, ComponentType};
pub use component::storage::{StorageLock, StorageReadGuard, StorageWriteGuard};
pub use module::{Module, HasComponent, ModuleType, Modules};
pub use state::{StateBuilder, State, Context};
pub use spawn::SpawnRequest;
pub use group::{Group, GroupToken, GroupType, Groups};
pub use processor::Processor;
