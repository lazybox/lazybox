mod builder;

pub use self::builder::StateBuilder;

use entity::{Entities, EntityRef, Accessor};
use module::component::storage::{StorageReadGuard, StorageWriteGuard};
use module::component::Component;
use module::{Module, Modules, HasComponent, CommitArgs};
use spawn::{SpawnQueue, SpawnRequest, PrototypeToken};
use rayon;
use schema::Schema;

pub struct State<Cx: Send> {
    schema: Schema,
    entities: Entities,
    modules: Modules<Cx>,
    spawn_queue: SpawnQueue
}

impl<Cx: Send> State<Cx> {
    pub fn new(schema: Schema) -> Self {
        State {
            schema: schema,
            entities: Entities::new(),
            modules: Modules::new(),
            spawn_queue: SpawnQueue::new()
        }
    }

    pub fn schema(&self) -> Schema {
        self.schema.clone()
    }

    pub fn entity_ref<'a>(&self, accessor: Accessor<'a>) -> EntityRef {
        self.entities.entity_ref(accessor)
    }

    pub fn accessor(&self, entity_ref: EntityRef) -> Option<Accessor> {
        self.entities.upgrade(entity_ref)
    }

    pub fn spawn_request(&self) -> SpawnRequest {
        let entity = self.entities.create();

        SpawnRequest::new(entity)
    }

    pub fn spawn_request_with<T: PrototypeToken>(&self) -> SpawnRequest {
        let entity = self.entities.create();

        SpawnRequest::with_prototype::<T>(entity)
    }

    fn spawn_later(&self, spawn: SpawnRequest) {
        self.spawn_queue.push(spawn);
    }

    fn remove_later<'a>(&self, entity: Accessor<'a>) {
        self.entities.remove_later(entity);
    }

    pub fn read<C: Component>(&self) -> StorageReadGuard<<C::Module as HasComponent<C>>::Storage>
        where C::Module: Module<Cx>
    {
        self.module::<C::Module>().read()
    }

    fn write<C: Component>(&self) -> StorageWriteGuard<<C::Module as HasComponent<C>>::Storage>
        where C::Module: Module<Cx>
    {
        self.module::<C::Module>().write()
    }

    pub fn module<M: Module<Cx>>(&self) -> &M {
        self.modules.get::<M>()
                    .expect("the requested module doesn't exists")
    }

    fn commit(&mut self, cx: &mut Cx) {
        let world_removes = self.entities.push_removes();

        
        let &mut State {    ref schema,
                            ref mut entities,
                            ref mut modules,
                            ref mut spawn_queue,
                            .. } = self;
            
        let requests = spawn_queue.take_requests();

        let commit_args = CommitArgs {
            prototypes: schema.prototypes(),
            requests: &requests,
            world_removes: &world_removes,
        };

        rayon::join(
            || entities.commit(&requests),
            || modules.commit(&commit_args, cx)
        );
    }
}

pub struct Update<'a, Cx: Send + 'a> {
    state: &'a mut State<Cx>
}

impl<'a, Cx: Send + 'a> Update<'a, Cx> {
    pub fn commit<F>(&mut self, context: &mut Cx, f: F) where F: FnOnce(&State<Cx>, Commit<Cx>, &mut Cx) {
        {
            let state = &*self.state;
            f(state, Commit { state: state }, context);
        }
        self.state.commit(context);
    }
}

pub struct Commit<'a, Cx: Send + 'a> {
    state: &'a State<Cx>
}

impl<'a, Cx: Send + 'a> Commit<'a, Cx> {
    #[inline]
    pub fn spawn_later(self, spawn: SpawnRequest) {
        self.state.spawn_later(spawn)
    }

    #[inline]
    pub fn remove_later(self, entity: Accessor) {
        self.state.remove_later(entity)
    }

    #[inline]
    pub fn write<C: Component>(&self) -> StorageWriteGuard<<C::Module as HasComponent<C>>::Storage>
        where C::Module: Module<Cx> {

        self.state.write::<C>()
    }
}

impl<'a, Cx: Send + 'a> Clone for Commit<'a, Cx> {
    #[inline]
    fn clone(&self) -> Self { Commit { state: self.state } }
}

impl<'a, Cx: Send + 'a> Copy for Commit<'a, Cx> {}
