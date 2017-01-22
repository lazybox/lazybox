use ecs::module::Component;
use ecs::entity::{Entity, EntityRef, Accessor};
use ecs::state::Commit;

/// An entity to be spawn
pub struct SpawnRequest<'a> {
    entity: Entity,
    commit: Commit<'a>,
}

impl<'a> SpawnRequest<'a> {
    /// Constructs a new SpawnRequest with the given `Entity`.
    pub(crate) fn new(entity: Entity,
                      commit: Commit<'a>)
                      -> Self {
        SpawnRequest {
            entity: entity,
            commit: commit,
        }
    }

    /// Sets a component to associate with the spawned entity.
    pub fn set<C: Component>(self, component: C::Template) -> Self {
        let accessor = unsafe { Accessor::new_unchecked(self.entity.id()) };
        self.commit.attach_later::<C>(accessor, component);

        self
    }

    /// Returns an `EntityRef` to the entity that will be spawned.
    ///
    /// The reference will be valid only at the next update.
    pub fn entity_ref(&self) -> EntityRef {
        EntityRef::from_entity(self.entity)
    }

    /// Returns the entity of this request.
    #[inline]
    pub fn entity(&self) -> Entity {
        self.entity
    }
}

pub trait Prototype: Sized {
    type Batch: Batch<Prototype = Self>;

    fn spawn_later_with<'a>(self, spawn: SpawnRequest<'a>) where Self: Sized;

    fn batch<'a>(commit: Commit<'a>) -> Self::Batch where Self: Sized;
}

pub trait Batch {
    type Prototype: Prototype;

    fn spawn_later<'a>(&self, prototype: Self::Prototype);
}
