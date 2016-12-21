#![feature(pub_restricted, associated_consts)]

extern crate parking_lot;
extern crate crossbeam;
extern crate vec_map;
extern crate bit_set;
extern crate daggy;
extern crate rayon;
#[macro_use]
extern crate mopa;

pub mod policy;
pub mod entity;
pub mod processor;
pub mod state;
pub mod component;
mod utils;
