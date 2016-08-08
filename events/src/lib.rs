#[macro_use]
extern crate mopa;
extern crate crossbeam;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use crossbeam::sync::SegQueue;
use std::sync::{RwLock, Arc, Weak};

trait AnyShared: mopa::Any + Sync + Send {}
impl<T: mopa::Any + Sync + Send> AnyShared for T {}

mopafy!(AnyShared);

pub trait Event: Send + Sync + Clone + Any {}

type Sender<E> = Weak<SegQueue<E>>;
type Senders<E> = Vec<Sender<E>>;

type Receiver<E> = Arc<SegQueue<E>>;

pub struct EventDispatcher {
    senders: RwLock<HashMap<TypeId, Box<AnyShared>>>
}

impl EventDispatcher {
    pub fn new() -> Self {
        EventDispatcher {
            senders: RwLock::new(HashMap::new())
        }
    }

    pub fn dispatch<E: Event>(&self, event: E) {
        let dropped_senders = self.send_to_all::<E>(event);
        self.clear_dropped::<E>(&dropped_senders);
    }

    fn send_to_all<E: Event>(&self, event: E) -> Vec<usize> {
        let senders = self.senders.read().unwrap();
        let mut dropped_senders = Vec::new();

        if let Some(event_senders) = senders.get(&TypeId::of::<Senders<E>>()) {
            let event_senders = event_senders.downcast_ref::<Senders<E>>().unwrap();

            for (index, event_sender) in event_senders.iter().enumerate() {
                match event_sender.upgrade() {
                    Some(sender) => sender.push(event.clone()),
                    None => dropped_senders.push(index),
                }
            }
        }

        dropped_senders
    }

    fn clear_dropped<E: Event>(&self, dropped_senders: &[usize]) {
        if dropped_senders.len() == 0 { return; }

        let mut senders = self.senders.write().unwrap();

        if let Some(event_senders) = senders.get_mut(&TypeId::of::<Senders<E>>()) {
            let event_senders = event_senders.downcast_mut::<Senders<E>>().unwrap();

            for &dropped in dropped_senders {
                event_senders.swap_remove(dropped);
            }
        }
    }

    fn register<E: Event>(&self, sender: Sender<E>) {
        let mut senders = self.senders.write().unwrap();

        let event_senders = senders.entry(TypeId::of::<Senders<E>>())
            .or_insert(Box::new(Vec::<Sender<E>>::new()));

        let event_senders = event_senders.downcast_mut::<Senders<E>>().unwrap();
        event_senders.push(sender);
    }

    pub fn listen_to<E: Event>(&self) -> EventReceiver<E> {
        let queue = Arc::new(SegQueue::new());

        let sender = Arc::downgrade(&queue);
        self.register::<E>(sender);
        
        EventReceiver {
            receiver: queue
        }
    }
}


pub struct EventReceiver<E: Event> {
    receiver: Receiver<E>
}

impl<E: Event> EventReceiver<E> {
    pub fn handle_with<F>(&mut self, mut handler: F)
        where F: FnMut(&E) 
    {
        while let Some(event) = self.receiver.try_pop() {
            handler(&event)
        }
    }
}