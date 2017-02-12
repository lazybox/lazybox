mod builder;
pub mod update_queue;

pub use self::builder::StateBuilder;
pub use self::update_queue::Monitors as UpdateMonitors;

use ecs::entity::{Entities, Entity, EntityRef, Accessor};
use ecs::module::{Component, StorageReadGuard, StorageWriteGuard};
use ecs::module::{Module, Modules, HasComponent};
use ecs::spawn::{SpawnRequest, Prototype};
use ecs::group::Groups;
use self::update_queue::{UpdateQueues, UpdateQueue, UpdateQueueReader};
use rayon;

pub struct State<Cx: Send> {
    entities: Entities,
    modules: Modules<Cx>,
    groups: Groups,
    update_queues: UpdateQueues,
}

impl<Cx: Send> State<Cx> {
    pub fn new(modules: Modules<Cx>,
               groups: Groups,
               update_queues: UpdateQueues)
               -> Self {
        State {
            entities: Entities::new(),
            modules: modules,
            groups: groups,
            update_queues: update_queues,
        }
    }

    pub fn entity_ref<'a>(&self, accessor: Accessor<'a>) -> EntityRef {
        self.entities.entity_ref(accessor)
    }

    pub fn accessor(&self, entity_ref: EntityRef) -> Option<Accessor> {
        self.entities.upgrade(entity_ref)
    }

    fn spawn_later(&self) -> Entity {
        let entity = self.entities.create();
        self.entities.spawn_later(entity);

        entity
    }

    fn attach_later<'a, C: Component>(&self, accessor: Accessor<'a>, component: C::Template) {
        self.update_queue::<C>().attach(accessor, component);
    }

    fn detach_later<'a, C: Component>(&self, accessor: Accessor<'a>) {
        self.update_queue::<C>().detach(accessor);
    }

    fn update_queue<C: Component>(&self) -> &UpdateQueue<C> {
        self.update_queues
            .get::<C>()
            .expect("the component has not been registered")
    }

    fn remove_later<'a>(&self, entity: Accessor<'a>) {
        self.entities.remove_later(entity);
    }

    pub fn read<C: Component>(&self) -> StorageReadGuard<<C::Module as HasComponent<C>>::Storage>
        where C::Module: Module<Cx>
    {
        self.module::<C::Module>().read()
    }

    pub fn write<C: Component>(&self) -> StorageWriteGuard<<C::Module as HasComponent<C>>::Storage>
        where C::Module: Module<Cx>
    {
        self.module::<C::Module>().write()
    }

    pub fn module<M: Module<Cx>>(&self) -> &M {
        self.modules
            .get::<M>()
            .expect("the requested module doesn't exists")
    }


    pub fn update(&mut self) -> Update<Cx> {
        Update { state: self }
    }

    fn commit(&mut self, cx: &mut Cx) {
        let world_removes = self.entities.push_removes();

        let &mut State { ref mut update_queues,
                         ref mut groups,
                         ref mut entities,
                         ref mut modules,
                         .. } = self;

        {
            entities.commit();
            
            let commit_args = CommitArgs {
                entities: &*entities,
                update_queues: update_queues,
                world_removes: &world_removes,
            };

            modules.commit(&commit_args, cx);
        }
        groups.commit(&update_queues.monitors());
        update_queues.clear_flags();
    }
}

pub struct Update<'a, Cx: Send + 'a> {
    state: &'a mut State<Cx>,
}

impl<'a, Cx: Send + 'a> Update<'a, Cx> {
    pub fn commit<F>(&mut self, context: &mut Cx, f: F)
        where F: FnOnce(&State<Cx>, Commit<Cx>, &mut Cx)
    {
        {
            let state = &*self.state;
            f(state, Commit { state: state }, context);
        }
        self.state.commit(context);
    }
}

pub struct Commit<'a, Cx: Send + 'a> {
    state: &'a State<Cx>,
}

impl<'a, Cx: Send + 'a> Commit<'a, Cx> {
    #[inline]
    pub fn spawn_later(self) -> SpawnRequest<'a, Cx> {
        let entity = self.state.spawn_later();
        SpawnRequest::new(entity, self)
    }

    #[inline]
    pub fn spawn_later_with<P: Prototype>(self, prototype: P) {
        let request = self.spawn_later();
        prototype.spawn_later_with(request);
    }

    #[inline]
    pub fn update_queue<C: Component>(self) -> &'a UpdateQueue<C> {
        self.state.update_queue::<C>()
    }


    #[inline]
    pub fn remove_later(self, entity: Accessor) {
        self.state.remove_later(entity)
    }

    #[inline]
    pub fn attach_later<C: Component>(self, entity: Accessor, component: C::Template) {
        self.state.attach_later::<C>(entity, component);
    }

    #[inline]
    pub fn detach_later<C: Component>(self, entity: Accessor) {
        self.state.detach_later::<C>(entity);
    }
}

impl<'a, Cx: Send + 'a> Clone for Commit<'a, Cx> {
    #[inline]
    fn clone(&self) -> Self {
        Commit { state: self.state }
    }
}

impl<'a, Cx: Send + 'a> Copy for Commit<'a, Cx> {}


pub struct CommitArgs<'a> {
    pub entities: &'a Entities,
    update_queues: &'a UpdateQueues,
    world_removes: &'a [Entity],
}

impl<'a> CommitArgs<'a> {
    pub fn update_reader_for<C: Component>(&self) -> UpdateQueueReader<C> {
        self.update_queues
            .get::<C>()
            .expect("the component has not been registered")
            .process(self.world_removes)
    }

    pub fn world_removes(&self) -> &[Entity] {
        &self.world_removes
    }
}
