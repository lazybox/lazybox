use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::mem;
use mopa;
use parking_lot::Mutex;

use module::component::{Component, Template, ComponentType};
use entity::{Entity, EntityRef};

/// An entity to be spawn
#[derive(Debug)]
pub struct SpawnRequest {
    entity: Entity,
    overrides: Prototype,
    prototype: Option<PrototypeType>,
}

impl SpawnRequest {
    /// Constructs a new SpawnRequest with the given `Entity`.
    pub(crate) fn new(entity: Entity) -> Self {
        SpawnRequest {
            entity: entity,
            overrides: Prototype::new(),
            prototype: None,
        }
    }

    /// Associate this request to a given base prototype.
    pub(crate) fn with_prototype<T: PrototypeToken>(entity: Entity) -> Self {
        SpawnRequest {
            entity: entity,
            overrides: Prototype::new(),
            prototype: Some(PrototypeType::of::<T>()),
        }
    }

    /// Sets the overriding prototype of this prototype.
    pub fn with_override(mut self, prototype: Prototype) -> Self {
        self.overrides = prototype;

        self
    }

    /// Sets a component to associate with the spawned entity.
    ///
    /// If the component already exists in the prototype, it will override it.
    pub fn set<C: Component>(mut self, component: C::Template) -> Self {
        self.overrides = self.overrides.set::<C>(component);
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
    pub(crate) fn entity(&self) -> Entity {
        self.entity
    }

    pub(crate) fn get<C: Component>(&self, prototypes: &Prototypes) -> Option<C::Template> {
        self.overrides.get::<C>().or_else(|| {
            self.prototype.and_then(|p| prototypes.get(p).get::<C>())
        })
    }

    /// Returns the type of the prototype associated with this request.
    #[inline]
    pub fn prototype(&self) -> Option<PrototypeType> {
        self.prototype
    }
}

trait AnyTemplate: mopa::Any + Send + Debug + Sync {}
mopafy!(AnyTemplate);

impl<T: Template> AnyTemplate for T {}

/// A prototype that defines the skeleton of an Entity.
///
/// It is represented by a set of components with default values.
#[derive(Debug)]
pub struct Prototype {
    components: HashMap<ComponentType, Box<AnyTemplate>>,
}

impl Prototype {
    /// Constructs a new empty `Prototype`.
    pub fn new() -> Self {
        Prototype { components: HashMap::new() }
    }

    /// Adds a default `component` to the prototype.
    pub fn set<C: Component>(mut self, component: C::Template) -> Self {
        self.components.insert(ComponentType::of::<C>(), Box::new(component));
        self
    }

    /// Returns the component `C` value of this prototype.
    pub fn get<C: Component>(&self) -> Option<C::Template> {
        self.components
            .get(&ComponentType::of::<C>())
            .and_then(|data| data.downcast_ref())
            .cloned()
    }
}

/// A token identity to a `Prototype`.
pub trait PrototypeToken: Any {
    fn prototype() -> Prototype;
}

/// A unique id for a given `PrototypeToken` type.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct PrototypeType(TypeId);

impl PrototypeType {
    /// Returns the type of `PrototypeType` of token `T`.
    pub fn of<T: PrototypeToken>() -> Self {
        PrototypeType(TypeId::of::<T>())
    }
}

/// The manager responsible to hold defined prototype
#[derive(Debug)]
pub(crate) struct Prototypes {
    prototypes: HashMap<PrototypeType, Prototype>,
}

impl Prototypes {
    /// Constructs a new empty `Prototypes`
    pub fn new() -> Self {
        Prototypes { prototypes: HashMap::new() }
    }

    /// Registers a new prototype with the given token.
    pub fn register<T: PrototypeToken>(&mut self) {
        let key = PrototypeType::of::<T>();
        self.prototypes.entry(key).or_insert_with(|| T::prototype());
    }

    /// Returns a prototype
    pub fn get(&self, prototype_type: PrototypeType) -> &Prototype {
        self.prototypes
            .get(&prototype_type)
            .expect("the prototype has not been registered")
    }
}

#[derive(Debug)]
pub(crate) struct SpawnQueue {
    queue: Mutex<Vec<SpawnRequest>>,
}

impl SpawnQueue {
    pub fn new() -> SpawnQueue {
        SpawnQueue { queue: Mutex::new(Vec::new()) }
    }

    pub fn push(&self, request: SpawnRequest) {
        let mut queue = self.queue.lock();
        queue.push(request);
    }

    pub fn take_requests(&mut self) -> Vec<SpawnRequest> {
        let mut queue = self.queue.lock();
        mem::replace(&mut *queue, Vec::new())
    }
}