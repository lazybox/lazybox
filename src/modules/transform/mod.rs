mod dynamic;

pub use self::dynamic::{Transform, TransformTemplate};

use modules::storages::packed::{self, Packed};
use ecs::entity::{Accessor, EntityRef};
use ecs::state::CommitArgs;
use ecs::module::{Module, StorageLock, StorageReadGuard, StorageWriteGuard, Template};
use ecs::Context;
use ecs::policy::Id;
use std::ops::Index;
use self::dynamic::TransformStorage;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StaticTransform(Transform);

impl Template for StaticTransform {}

pub struct StaticTransformStorage {
    transforms: Packed<Transform>,
}

impl StaticTransformStorage {
    pub fn new() -> Self {
        StaticTransformStorage { transforms: Packed::new() }
    }

    fn insert(&mut self, entity: Id, transform: StaticTransform) {
        let accessor = unsafe { Accessor::new_unchecked(entity) };

        self.transforms.insert(accessor, transform.0);
    }

    fn remove(&mut self, entity: Id) {
        let accessor = unsafe { Accessor::new_unchecked(entity) };

        self.transforms.remove(accessor);
    }

    #[inline]
    pub fn world(&self, entity: Accessor) -> Option<&Transform> {
        self.transforms.get(entity)
    }

    pub fn iter(&self) -> packed::Iter<Transform> {
        self.transforms.iter()
    }

    
    fn commit(&mut self, args: &CommitArgs) {
        let mut reader = args.update_reader_for::<StaticTransform>();

        while let Some((entity, template)) = reader.next_attach_query() {
            self.insert(entity, template);
        }

        while let Some(entity) = reader.next_detach_query() {
            self.remove(entity);
        }

        for entity in args.world_removes() {
            self.remove(entity.id());
        }
    }
}

impl<'a> Index<Accessor<'a>> for StaticTransformStorage {
    type Output = Transform;

    fn index(&self, index: Accessor<'a>) -> &Transform {
        &self.transforms[index]
    }
}

impl<'a> IntoIterator for &'a StaticTransformStorage {
    type Item = (Accessor<'a>, &'a Transform);
    type IntoIter = packed::Iter<'a, Transform>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct TransformModule {
    statics: StorageLock<StaticTransformStorage>,
    dynamics: StorageLock<TransformStorage>,
}

impl TransformModule {
    pub fn new() -> Self {
        TransformModule {
            statics: StorageLock::new(StaticTransformStorage::new()),
            dynamics: StorageLock::new(TransformStorage::new()),
        }
    }
}

impl<Cx: Context> Module<Cx> for TransformModule {
    fn commit(&mut self, args: &CommitArgs, _cx: &mut Cx) {
        let mut statics = self.statics.write();
        let mut dynamics = self.dynamics.write();
        
        statics.commit(args);
        dynamics.commit(args);
    }
}

derive_component!(Transform, TransformTemplate, TransformModule);
impl_has_component!(Transform, TransformStorage, TransformModule => dynamics);

derive_component!(StaticTransform, StaticTransform, TransformModule);
impl_has_component!(StaticTransform, StaticTransformStorage, TransformModule => statics);
