use std::sync::MutexGuard;
use utils::{Pool, Guard};

#[derive(Clone, Copy, Debug)]
pub struct LayerId(pub u8);

impl LayerId {
    fn index(&self) -> usize { self.0 as usize } 
}

#[derive(Clone, Copy, Debug)]
pub struct LayerOrder(pub u8);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayerOcclusion {
    Ignore,
    Stack,
}

pub(crate) struct Layers<T: 'static + Default, D = ()> {
    pub(crate) vec: Vec<Layer<T, D>>,
}

pub(crate) struct Layer<T: 'static + Default, D> {
    pool: Pool<T>,
    data: D,
}

impl<T: 'static + Default, D> Layers<T, D> {
    pub fn new() -> Self {
        Layers { vec: Vec::new() }
    }

    pub fn push(&mut self, data: D) -> LayerId {
        let id = LayerId(self.count());
        self.vec.push(Layer {
            pool: Pool::new(),
            data: data
        });
        id
    }

    pub fn get(&self, id: LayerId) -> Guard<T> {
        self.vec[id.index()].pool.get()
    }

    pub fn count(&self) -> u8 {
        self.vec.len() as u8
    }
}

impl<T: 'static + Default, D> Layer<T, D> {
    pub fn access(&mut self) -> (MutexGuard<Vec<T>>, &mut D) {
        (self.pool.availables(), &mut self.data)
    }
}