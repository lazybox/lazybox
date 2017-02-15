#![feature(pub_restricted, associated_consts)]
#![allow(dead_code, unused_imports)]

extern crate parking_lot;
extern crate crossbeam;
extern crate vec_map;
extern crate bit_set;
extern crate daggy;
extern crate rayon;
extern crate fnv;
#[macro_use]
extern crate mopa;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate glutin;
extern crate cgmath;
#[macro_use]
extern crate approx;

pub extern crate lazybox_graphics as graphics;
pub extern crate lazybox_settings as settings;
pub extern crate lazybox_inputs as inputs;
pub extern crate lazybox_assets as assets;
pub extern crate lazybox_frameclock as frameclock;

#[macro_use]
pub mod ecs;
pub mod modules;

pub mod maths {
    extern crate cgmath;
    pub use cgmath::*;
}
