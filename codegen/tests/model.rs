#![feature(custom_attribute)]
#![allow(dead_code)]

extern crate lazybox;
#[macro_use]
extern crate lazybox_codegen;

#[derive(Debug, Clone)]
pub struct Health {
    count: u32
}

impl lazybox::modules::data::DataComponent for Health {
    type Storage = lazybox::modules::data::storages::PackedStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct Armor {
    percent: f32
}

impl lazybox::modules::data::DataComponent for Armor {
    type Storage = lazybox::modules::data::storages::PackedStorage<Self>;
}

#[derive(Model)]
#[name = "Model"]
pub struct _Model {
    #[read] armor: Armor,
    #[write] health: Health,
}
