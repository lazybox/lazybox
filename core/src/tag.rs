use fnv::FnvHashMap;
use std::any::{Any, TypeId};
use {Entities, Accessor, EntityRef};
use sync::SegQueue;
use std::ops::Index;
use std::fmt::Debug;

pub trait Tag: Debug + Any {}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TagType(TypeId);

impl TagType {
    #[inline]
    pub fn of<T: Tag>() -> TagType {
        TagType(TypeId::of::<T>())
    }
}

enum Query {
    Attach(EntityRef, TagType),
    Detach(TagType),
}

pub struct Tags {
    tag_to_entity: FnvHashMap<TagType, EntityRef>,
    queries: SegQueue<Query>,
}

impl Tags {
    pub fn new() -> Tags {
        Tags {
            tag_to_entity: FnvHashMap::default(),
            queries: SegQueue::new(),
        }
    }

    #[inline]
    pub fn tag_later<'a>(&self, entity_ref: EntityRef, tag: TagType) {
        self.queries.push(Query::Attach(entity_ref, tag));
    }

    #[inline]
    pub fn remove_later<'a>(&self, tag: TagType) {
        self.queries.push(Query::Detach(tag));
    }

    pub fn tagged<'a>(&self, entities: &'a Entities, tag: TagType) -> Option<Accessor<'a>> {
        let accessor = self.tag_to_entity
            .get(&tag)
            .and_then(|&entity_ref| entities.upgrade(entity_ref));

        match accessor {
            accessor @ Some(_) => accessor,
            None => {
                self.remove_later(tag);

                None
            }
        }
    }

    pub fn commit(&mut self) {
        while let Some(query) = self.queries.try_pop() {
            match query {
                Query::Attach(entity_ref, tag_type) => {
                    self.tag_to_entity.insert(tag_type, entity_ref);
                }
                Query::Detach(tag_type) => {
                    self.tag_to_entity.remove(&tag_type);
                }
            }
        }
    }
}
