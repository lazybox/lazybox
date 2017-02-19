use core;

pub struct Context;

pub mod modules {
    pub struct Context {}
}

pub mod processors {
    pub struct Context {}
}

impl core::Context for Context {
    type ForModules = self::modules::Context;
    type ForProcessors = self::processors::Context;
}
