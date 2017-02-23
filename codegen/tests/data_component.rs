#![feature(custom_attribute)]
#![allow(dead_code)]

extern crate lazybox;
#[macro_use]
extern crate lazybox_codegen;

use lazybox::core::module::data::SparseStorage;

#[derive(DataComponent, Debug)]
pub struct Health(u32);

#[derive(DataComponent, Debug)]
#[storage(SparseStorage)]
pub struct Position(f32, f32);
