use {Entity, EntityRef, Accessor, Component, Context};
use state::Commit;

/// An entity to be spawn
pub struct SpawnRequest<'a, Cx: Context + 'a> {
    entity: Entity,
    commit: Commit<'a, Cx>,
}

impl<'a, Cx: Context + 'a> SpawnRequest<'a, Cx> {
    /// Constructs a new SpawnRequest with the given `Entity`.
    #[doc(hidden)]
    pub fn new(entity: Entity, commit: Commit<'a, Cx>) -> Self {
        SpawnRequest {
            entity: entity,
            commit: commit,
        }
    }

    /// Sets a component to associate with the spawned entity.
    pub fn set<C: Component>(self, component: C::Template) -> Self {
        let accessor = unsafe { Accessor::new_unchecked(self.entity.id()) };
        self.commit.attach::<C>(accessor, component);

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
    fn spawn_later_with<'a, Cx: Context>(self, spawn: SpawnRequest<'a, Cx>) where Self: Sized;
}
