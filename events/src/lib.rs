extern crate crossbeam;

use std::any::Any;
use std::sync::Arc;
use crossbeam::sync::SegQueue;

pub trait Message: Any + Sync + Send {}

struct MessageBuffer<M: Message> {
    buffer: SegQueue<M>,
}

impl<M: Message> MessageBuffer<M> {
    pub fn new() -> Self {
        MessageBuffer { buffer: SegQueue::new() }
    }

    #[inline]
    pub fn append(&self, message: M) {
        self.buffer.push(message)
    }

    #[inline]
    pub fn next(&self) -> Option<M> {
        self.buffer.try_pop()
    }
}

pub struct MessageBox<M: Message>(Arc<MessageBuffer<M>>);


impl<M: Message> MessageBox<M> {
    pub fn new() -> Self {
        MessageBox(Arc::new(MessageBuffer::new()))
    }

    pub fn sender(&self) -> MessageSender<M> {
        MessageSender(self.0.clone())
    }

    pub fn drain(&self) -> DrainIter<M> {
        DrainIter { inner: &*self.0 }
    }
}

pub struct MessageSender<M: Message>(Arc<MessageBuffer<M>>);

impl<M: Message> MessageBox<M> {
    #[inline]
    pub fn send(&self, message: M) {
        self.0.append(message);
    }
}

pub struct DrainIter<'a, M: Message> {
    inner: &'a MessageBuffer<M>,
}

impl<'a, M: Message> Iterator for DrainIter<'a, M> {
    type Item = M;

    fn next(&mut self) -> Option<M> {
        self.inner.next()
    }
}
