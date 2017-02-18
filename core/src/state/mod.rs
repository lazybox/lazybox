mod builder;
pub mod update_queue;

pub use self::builder::StateBuilder;
pub use self::update_queue::Monitors as UpdateMonitors;

use entity::iter::SetIter;
use {Entities, Entity, EntityRef, Accessor};
use {Component, StorageReadGuard, StorageWriteGuard};
use {Module, Modules, HasComponent};
use spawn::{SpawnRequest, Prototype};
use interface::Interfaces;
use tag::{Tag, Tags, TagType};
use group::{Groups, GroupType, GroupToken};
use self::update_queue::{UpdateQueues, UpdateQueue, UpdateQueueReader};

pub trait Context: Sync {}

pub struct State<Cx: Context> {
    entities: Entities,
    modules: Modules<Cx>,
    interfaces: Interfaces,
    tags: Tags,
    groups: Groups,
    update_queues: UpdateQueues,
}

impl<Cx: Context> State<Cx> {
    pub fn new(modules: Modules<Cx>,
               interfaces: Interfaces,
               groups: Groups,
               update_queues: UpdateQueues)
               -> Self {
        State {
            entities: Entities::new(),
            modules: modules,
            interfaces: interfaces,
            tags: Tags::new(),
            groups: groups,
            update_queues: update_queues,
        }
    }

    #[inline]
    pub fn entity_ref<'a>(&self, accessor: Accessor<'a>) -> EntityRef {
        self.entities.entity_ref(accessor)
    }

    #[inline]
    pub fn accessor(&self, entity_ref: EntityRef) -> Option<Accessor> {
        self.entities.upgrade(entity_ref)
    }

    #[inline]
    pub fn tagged<T: Tag>(&self) -> Option<Accessor> {
        self.tags.tagged(&self.entities, TagType::of::<T>())
    }

    #[inline]
    pub fn has_group<'a, G: GroupToken>(&self, entity: Accessor<'a>) -> bool {
        self.groups.has_group(GroupType::of::<G>(), entity)
    }

    #[inline]
    pub fn group<'a, G: GroupToken>(&self) -> SetIter {
        self.groups.entities_in_group(GroupType::of::<G>())
    }

    #[inline]
    pub fn read<C: Component>(&self) -> StorageReadGuard<<C::Module as HasComponent<C>>::Storage>
        where C::Module: Module<Cx>
    {
        self.module::<C::Module>().read()
    }

    #[inline]
    pub fn write<C: Component>(&self) -> StorageWriteGuard<<C::Module as HasComponent<C>>::Storage>
        where C::Module: Module<Cx>
    {
        self.module::<C::Module>().write()
    }

    #[inline]
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
                         ref mut interfaces,
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
        interfaces.commit(&update_queues.monitors());
        groups.commit(&world_removes);
        update_queues.clear_flags();
    }
}

pub struct Update<'a, Cx: Context + 'a> {
    state: &'a mut State<Cx>,
}

impl<'a, Cx: Context + 'a> Update<'a, Cx> {
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

pub struct Commit<'a, Cx: Context + 'a> {
    state: &'a State<Cx>,
}

impl<'a, Cx: Context + 'a> Commit<'a, Cx> {
    #[inline]
    pub fn spawn(self) -> SpawnRequest<'a, Cx> {
        let entity = self.state.entities.create();
        self.state.entities.spawn_later(entity);

        SpawnRequest::new(entity, self)
    }

    #[inline]
    pub fn spawn_with<P: Prototype>(self, prototype: P) {
        let request = self.spawn();
        prototype.spawn_later_with(request);
    }


    #[inline]
    pub fn remove(self, entity: Accessor) {
        self.state.entities.remove_later(entity);
    }

    #[inline]
    pub fn attach<C: Component>(self, entity: Accessor, component: C::Template) {
        self.update_queue::<C>().attach(entity, component);
    }

    #[inline]
    pub fn detach<C: Component>(self, entity: Accessor) {
        self.update_queue::<C>().detach(entity);
    }

    #[inline]
    pub fn tag<T: Tag>(self, entity: Accessor) {
        let entity_ref = self.state.entities.entity_ref(entity);
        self.state.tags.tag_later(entity_ref, TagType::of::<T>());
    }

    #[inline]
    pub fn remove_tag<T: Tag>(self) {
        self.state.tags.remove_later(TagType::of::<T>());
    }

    #[inline]
    pub fn add_to_group<G: GroupToken>(self, entity: Accessor) {
        self.state.groups.add_later_to(GroupType::of::<G>(), entity);
    }

    #[inline]
    pub fn remove_from_group<G: GroupToken>(self, entity: Accessor) {
        self.state.groups.remove_later_from(GroupType::of::<G>(), entity);
    }

    #[inline]
    pub fn update_queue<C: Component>(self) -> &'a UpdateQueue<C> {
        self.state
            .update_queues
            .get::<C>()
            .expect("the component has not been registered")
    }
}

impl<'a, Cx: Context + 'a> Clone for Commit<'a, Cx> {
    #[inline]
    fn clone(&self) -> Self {
        Commit { state: self.state }
    }
}

impl<'a, Cx: Context + 'a> Copy for Commit<'a, Cx> {}


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
