use std::any::{Any, TypeId};
use std::fmt::Debug;
use bit_set::BitSet;
use fnv::FnvHashMap;
use sync::SegQueue;
use entity::iter::{SetIter, accessors_from_set};
use policy::Id;
use {Accessor, Entity};

pub trait GroupToken: Debug + Any {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GroupType(TypeId);

impl GroupType {
    pub fn of<G: GroupToken>() -> GroupType {
        GroupType(TypeId::of::<G>())
    }
}

type Group = BitSet;

enum Query {
    Attach(GroupType, usize),
    Detach(GroupType, usize),
}

pub struct Groups {
    groups: FnvHashMap<GroupType, Group>,
    queries: SegQueue<Query>,
}

impl Groups {
    pub fn new() -> Self {
        Groups {
            groups: FnvHashMap::default(),
            queries: SegQueue::new(),
        }
    }

    pub fn insert_empty(&mut self, group_type: GroupType) {
        self.groups.insert(group_type, Group::new());
    }

    pub fn has_group<'a>(&self, group_type: GroupType, entity: Accessor) -> bool {
        let group = self.groups.get(&group_type).expect("group not registered");

        group.contains(entity.index())
    }

    pub fn entities_in_group(&self, group_type: GroupType) -> SetIter {
        let group = self.groups.get(&group_type).expect("group not registered");
        unsafe { accessors_from_set(&group) }
    }

    #[inline]
    pub fn add_later_to<'a>(&self, group_type: GroupType, entity: Accessor<'a>) {
        self.queries.push(Query::Attach(group_type, entity.index()));
    }

    #[inline]
    pub fn remove_later_from<'a>(&self, group_type: GroupType, entity: Accessor<'a>) {
        self.queries.push(Query::Detach(group_type, entity.index()));
    }

    pub fn commit(&mut self, world_removes: &[Entity]) {
        while let Some(query) = self.queries.try_pop() {
            match query {
                Query::Attach(ref group_type, entity) => {
                    let group = self.groups.get_mut(group_type).expect("group not registered");
                    group.insert(entity);
                }
                Query::Detach(ref group_type, entity) => {
                    if let Some(group) = self.groups.get_mut(group_type) {
                        group.remove(entity);
                    }
                }
            }
        }

        for removed in world_removes {
            for (_, group) in &mut self.groups {
                group.remove(removed.index());
            }
        }
    }
}