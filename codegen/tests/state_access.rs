#![feature(custom_attribute)]
#![allow(dead_code)]

extern crate lazybox;
#[macro_use]
extern crate lazybox_codegen;

use lazybox::core::module::data::DataComponent;
use lazybox::core::module::data::PackedStorage;

#[derive(Debug, Clone)]
pub struct Health {
    count: u32,
}

impl DataComponent for Health {
    type Storage = PackedStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct Armor {
    percent: f32,
}

impl DataComponent for Armor {
    type Storage = PackedStorage<Self>;
}

#[derive(StateAccess)]
#[name(Access)]
pub struct AccessInfo {
    #[read]
    armor: Armor,
    #[write]
    health: Health,
}
