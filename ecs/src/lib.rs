#![feature(pub_restricted, associated_consts)]

extern crate parking_lot;
extern crate crossbeam;
extern crate vec_map;
extern crate bit_set;
extern crate daggy;
extern crate rayon;
extern crate fnv;
#[macro_use]
extern crate mopa;

pub mod policy;
pub mod entity;
pub mod processor;
pub mod state;
pub mod spawn;
pub mod module;
pub mod data;
pub mod group;
mod schema;