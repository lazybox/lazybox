#![feature(custom_attribute)]
#![allow(dead_code)]

extern crate lazybox;
#[macro_use]
extern crate lazybox_codegen;

use std::any::Any;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Tag {
    num: u8
}

#[derive(Debug, Clone)]
pub struct Wrap<T> {
    inner: T,
    more: u16
}

impl lazybox::modules::data::DataComponent for Tag {
    type Storage = lazybox::modules::data::storages::PackedStorage<Self>;
}

impl<T> lazybox::modules::data::DataComponent for Wrap<T>
    where T: Any + Send + Sync + Clone + fmt::Debug
{
    type Storage = lazybox::modules::data::storages::PackedStorage<Self>;
}

#[derive(Prototype)]
#[batch = "TagBatch"]
pub struct TagPrototype {
    tag: Tag
}

#[derive(Prototype)]
#[batch = "CompositeBatch"]
pub struct CompositePrototype {
    tag: Tag,
    wrap: Wrap<Tag>
}
