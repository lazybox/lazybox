#![feature(custom_attribute)]
#![allow(dead_code)]

extern crate lazybox;
#[macro_use]
extern crate lazybox_codegen;

use std::any::Any;
use std::fmt;
use lazybox::core::module::data::DataComponent;
use lazybox::core::module::data::storages::Packed;

#[derive(Debug, Clone)]
pub struct Tag {
    num: u8
}

#[derive(Debug, Clone)]
pub struct Wrap<T> {
    inner: T,
    more: u16
}

impl DataComponent for Tag {
    type Storage = Packed<Self>;
}

impl<T> DataComponent for Wrap<T>
    where T: Any + Send + Sync + Clone + fmt::Debug
{
    type Storage = Packed<Self>;
}

#[derive(Prototype)]
#[batch(TagBatch)]
pub struct TagPrototype {
    tag: Tag
}

#[derive(Prototype)]
#[batch(CompositeBatch)]
pub struct CompositePrototype {
    tag: Tag,
    wrap: Wrap<Tag>
}
