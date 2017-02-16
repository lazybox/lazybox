use maths::{Point2, Vector2, Basis2, Rotation2, Rad, EuclideanSpace, Rotation, ApproxEq, Angle};
use std::ops::{Deref, DerefMut};
use ecs::entity::{Entities, Accessor, EntityRef};
use ecs::state::CommitArgs;
use ecs::module::{Component, Template};
use ecs::policy::Id;
use std::ops::Index;
use std::collections::VecDeque;
use vec_map::VecMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub position: Point2<f32>,
    pub rotation: Rad<f32>,
    pub scale: Vector2<f32>,
}

impl Transform {
    pub fn one() -> Self {
        Transform {
            position: Point2::new(0., 0.),
            rotation: Rad(0.),
            scale: Vector2::new(1., 1.),
        }
    }

    pub fn basis(&self) -> Basis2<f32> {
        Basis2::from_angle(self.rotation)
    }

    pub fn scale_vector(&self, v: Vector2<f32>) -> Vector2<f32> {
        Vector2::new(v.x * self.scale.x, v.y * self.scale.y)
    }
}

impl Template for Transform {}

impl ApproxEq for Transform {
    type Epsilon = f32;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        Self::Epsilon::default_epsilon()
    }

    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        Self::Epsilon::default_max_relative()
    }

    #[inline]
    fn default_max_ulps() -> u32 {
        Self::Epsilon::default_max_ulps()
    }

    #[inline]
    fn relative_eq(&self,
                   other: &Self,
                   epsilon: Self::Epsilon,
                   max_relative: Self::Epsilon)
                   -> bool {
        self.position.relative_eq(&other.position, epsilon, max_relative) &&
        self.rotation.relative_eq(&other.rotation, epsilon, max_relative) &&
        self.scale.relative_eq(&other.scale, epsilon, max_relative)
    }

    #[inline]
    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        self.position.ulps_eq(&other.position, epsilon, max_ulps) &&
        self.rotation.ulps_eq(&other.rotation, epsilon, max_ulps) &&
        self.scale.ulps_eq(&other.scale, epsilon, max_ulps)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TransformTemplate {
    pub parent: Option<EntityRef>,
    pub transform: Transform,
}

impl Template for TransformTemplate {}

#[derive(Debug)]
struct Instance {
    world: Transform,
    local: Transform,

    parent: Option<usize>,

    first_child: Option<usize>,
    next_sibling: Option<usize>,
    previous_sibling: Option<usize>,

    entity: Id,
}

type ParentIndex = usize;
type InstanceIndex = usize;

const INITIAL_STACK_CAPACITY: usize = 15;

pub struct TransformStorage {
    instances: Vec<Instance>,
    entity_to_instance: VecMap<InstanceIndex>,
    transform_stack: VecDeque<(ParentIndex, InstanceIndex)>,
}

impl TransformStorage {
    pub fn new() -> Self {
        TransformStorage {
            instances: Vec::new(),
            entity_to_instance: VecMap::new(),
            transform_stack: VecDeque::with_capacity(INITIAL_STACK_CAPACITY),
        }
    }

    fn insert(&mut self, entities: &Entities, entity: Id, template: TransformTemplate) {
        use vec_map::Entry;

        let parent_index = template.parent
            .and_then(|entity_ref| entities.upgrade(entity_ref))
            .and_then(|accessor| self.entity_to_instance.get(accessor.id() as usize).cloned());

        let instance_index = match self.entity_to_instance.entry(entity as usize) {
            Entry::Vacant(vacant) => {
                let index = self.instances.len();

                self.instances.push(Instance {
                    world: Transform::one(),
                    local: Transform::one(),
                    parent: None,
                    first_child: None,
                    next_sibling: None,
                    previous_sibling: None,
                    entity: entity,
                });

                *vacant.insert(index)
            }
            Entry::Occupied(mut occupied) => *occupied.get_mut(),
        };

        self.set_parent_for_instance(instance_index, parent_index);
        self.set_local_transform_impl(entity, template.transform);
    }

    fn remove(&mut self, entity: Id) {
        if let Some(instance_index) = self.entity_to_instance.remove(entity as usize) {
            self.detach_from_parent(instance_index);

            let old_index = self.instances.len() - 1;
            self.instances.swap_remove(instance_index);


            let &mut TransformStorage { ref mut entity_to_instance, ref mut instances, .. } = self;

            // We need to update the references of the swapped instance.
            let siblings = instances.get_mut(instance_index).map(|instance| {
                entity_to_instance[instance.entity as usize] = instance_index;

                (instance.parent, instance.previous_sibling, instance.next_sibling)
            });

            if let Some((parent, previous_sibling, next_sibling)) = siblings {
                // We have to check if we need to update parent first_child reference
                match parent {
                    Some(parent) => {
                        let parent = &mut instances[parent];

                        if parent.first_child == Some(old_index) {
                            parent.first_child = Some(instance_index);
                        }
                    }
                    _ => {}
                }

                previous_sibling.map(|index| instances[index].next_sibling = Some(instance_index));
                next_sibling.map(|index| instances[index].previous_sibling = Some(instance_index));
            }
        }
    }

    pub fn local(&self, entity: Accessor) -> Option<Transform> {
        self.entity_to_instance
            .get(entity.id() as usize)
            .map(|&index| self.instances[index].local)
    }

    pub fn world(&self, entity: Accessor) -> Option<Transform> {
        self.entity_to_instance
            .get(entity.id() as usize)
            .map(|&index| self.instances[index].world)
    }


    pub fn parent(&self, entity: Accessor) -> Option<Accessor> {
        if let Some(&instance_index) = self.entity_to_instance.get(entity.id() as usize) {
            let parent = self.instances[instance_index].parent;

            parent.map(|index| unsafe { Accessor::new_unchecked(self.instances[index].entity) })
        } else {
            None
        }
    }

    pub fn children(&self, entity: Accessor) -> Children {
        let instance_index = self.entity_to_instance[entity.id() as usize];

        Children { current: self.instances[instance_index].first_child }
    }

    pub fn set_parent(&mut self, entity: Accessor, parent: Accessor) {
        let entity_index = *self.entity_to_instance
            .get(entity.id() as usize)
            .expect("The entity does not have a transform attached to it.");

        let parent_index = *self.entity_to_instance
            .get(parent.id() as usize)
            .expect("The entity does not have a transform attached to it.");

        self.set_parent_for_instance(entity_index, Some(parent_index));

        let parent_transform = self.instances[parent_index].world;
        self.transform(&parent_transform, entity_index);
    }

    pub fn remove_parent(&mut self, entity: Accessor) {
        let entity_index = *self.entity_to_instance
            .get(entity.id() as usize)
            .expect("The entity does not have a transform attached to it.");

        self.set_parent_for_instance(entity_index, None);
        self.transform(&Transform::one(), entity_index);
    }

    pub fn set_local(&mut self, entity: Accessor, transform: Transform) {
        self.set_local_transform_impl(entity.id(), transform);
    }

    fn set_local_transform_impl(&mut self, entity: Id, transform: Transform) {
        let instance_index = *self.entity_to_instance
            .get(entity as usize)
            .expect("The entity does not have a transform attached to it.");

        let parent = {
            let mut entity_instance = &mut self.instances[instance_index];
            entity_instance.local = transform;

            entity_instance.parent
        };

        let parent_transform = match parent {
            Some(parent_instance_index) => self.instances[parent_instance_index].world,
            None => Transform::one(),
        };

        self.transform(&parent_transform, instance_index);


        if self.transform_stack.len() != 0 {
            let mut current_parent_index = instance_index;
            let mut current_parent_transform = self.instances[instance_index].world;

            while let Some((parent_index, instance_index)) = self.transform_stack.pop_front() {
                if parent_index != current_parent_index {
                    current_parent_index = parent_index;
                    current_parent_transform = self.instances[parent_index].world;
                }

                self.transform(&current_parent_transform, instance_index);
            }
        }
    }

    fn transform(&mut self, parent_transform: &Transform, instance_index: InstanceIndex) {
        let mut current_child = {
            let instance = &mut self.instances[instance_index];

            let direction = parent_transform.basis()
                .rotate_vector(instance.local.position.to_vec());
            let parent_to_child = parent_transform.scale_vector(direction);

            instance.world = Transform {
                position: parent_transform.position + parent_to_child,
                rotation: (parent_transform.rotation + instance.local.rotation).normalize(),
                scale: parent_transform.scale_vector(instance.local.scale),
            };

            instance.first_child
        };


        while let Some(child) = current_child {
            self.transform_stack.push_back((instance_index, child));
            current_child = self.instances[child].next_sibling;
        }
    }


    fn set_parent_for_instance(&mut self,
                               instance_index: InstanceIndex,
                               parent: Option<ParentIndex>) {

        debug_assert!(!self.cycle_reference_check(instance_index, parent));

        self.detach_from_parent(instance_index);
        if let Some(parent_index) = parent {

            self.instances[instance_index].parent = parent;
            self.append_child_to(parent_index, instance_index);
        }
    }

    fn cycle_reference_check(&self,
                             instance_index: InstanceIndex,
                             parent: Option<ParentIndex>)
                             -> bool {
        if let Some(parent_index) = parent {
            if instance_index == parent_index {
                return true;
            }

            let mut current_child = self.instances[instance_index].first_child;

            while let Some(child) = current_child {
                if child == parent_index {
                    return true;
                }

                current_child = self.instances[child].next_sibling;
            }
        }

        false
    }

    fn append_child_to(&mut self, parent_index: ParentIndex, instance_index: InstanceIndex) {
        let mut current_child = self.instances[parent_index].first_child;
        let mut last_child = None;

        while let Some(child) = current_child {
            last_child = current_child;
            current_child = self.instances[child].next_sibling;
        }

        match last_child {
            Some(last_child_index) => {
                self.instances[last_child_index].next_sibling = Some(instance_index);
                self.instances[instance_index].previous_sibling = Some(last_child_index);
            }
            None => {
                self.instances[parent_index].first_child = Some(instance_index);
            }
        }
    }

    fn detach_from_parent(&mut self, instance_index: InstanceIndex) {
        let linked_list_instances = {
            let instance = &mut self.instances[instance_index];

            let parent = instance.parent.take();
            (parent, instance.previous_sibling, instance.next_sibling)
        };

        match linked_list_instances {
            (None, _, _) => {}
            (Some(parent), None, Some(next_sibling)) => {
                // We were the first child of the parent so we need to update the parent first_child link
                // to point on our next_sibling
                self.instances[parent].first_child = Some(next_sibling);
                self.instances[next_sibling].previous_sibling = None;
            }
            (_, Some(previous_sibling), Some(next_sibling)) => {
                // Here we have to update both sibling to point to each others;

                self.instances[previous_sibling].next_sibling = Some(next_sibling);
                self.instances[next_sibling].previous_sibling = Some(previous_sibling);
            }
            (_, Some(previous_sibling), None) => {
                // We were the tail, so we update the previous sibling to make it the new tail.
                self.instances[previous_sibling].next_sibling = None;
            }
            (Some(parent), None, None) => {
                // We were the only child
                self.instances[parent].first_child = None;
            }
        }
    }

    #[doc(hidden)]
    pub fn commit(&mut self, args: &CommitArgs) {
        let mut reader = args.update_reader_for::<Transform>();

        while let Some((entity, template)) = reader.next_attach_query() {
            self.insert(args.entities, entity, template);
        }

        while let Some(entity) = reader.next_detach_query() {
            self.remove(entity);
        }

        for entity in args.world_removes() {
            self.remove(entity.id());
        }
    }
}

pub struct Children {
    current: Option<usize>,
}

impl Children {
    pub fn next(&mut self, transforms: &TransformStorage) -> Option<Accessor> {
        match self.current {
            Some(current) => {
                let (entity_id, next_child) = {
                    let instance = &transforms.instances[current];

                    (instance.entity, instance.next_sibling)
                };

                self.current = next_child;
                let accessor = unsafe { Accessor::new_unchecked(entity_id) };

                Some(accessor)
            }
            None => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use maths::{Vector2, Rad, Deg, Point2, ApproxEq};
    use ecs::entity::{Entities, Entity, Accessor};

    macro_rules! transform_approx_eq {
        ($left:expr, $right:expr) => (
            match ($left, $right) {
                (Some(left), Some(right)) => assert_ulps_eq!(left, right),
                _ => assert_eq!($left, $right)
            }
        )
    }

    #[test]
    fn test_insert_transform_root() {
        let mut storage = TransformStorage::new();
        let mut entities = Entities::new();

        let (entity, accessor) = spawn_entity(&mut entities);
        let transform = Transform {
            position: Point2::new(5., 5.),
            rotation: Rad(0.4),
            scale: Vector2::new(1., 1.),
        };

        storage.insert(&entities,
                       entity.id(),
                       TransformTemplate {
                           transform: transform,
                           parent: None,
                       });

        transform_approx_eq!(storage.local(accessor), Some(transform));
        transform_approx_eq!(storage.world(accessor), Some(transform));
        assert_eq!(storage.parent(accessor), None);
    }

    #[test]
    fn test_remove_transform_root() {
        let mut storage = TransformStorage::new();
        let mut entities = Entities::new();

        let (entity, accessor) = spawn_entity(&mut entities);

        let transform = Transform {
            position: Point2::new(5., 5.),
            rotation: Rad(0.4),
            scale: Vector2::new(1., 1.),
        };

        storage.insert(&entities,
                       entity.id(),
                       TransformTemplate {
                           transform: transform,
                           parent: None,
                       });

        storage.remove(entity.id());

        transform_approx_eq!(storage.local(accessor), None);
        transform_approx_eq!(storage.world(accessor), None);
        assert_eq!(storage.parent(accessor), None);
    }

    #[test]
    fn test_remove_twice() {
        let mut storage = TransformStorage::new();
        let mut entities = Entities::new();

        let (entity, _) = spawn_entity(&mut entities);

        storage.remove(entity.id());
        storage.remove(entity.id());
    }

    #[test]
    fn test_insert_twice() {
        let mut storage = TransformStorage::new();
        let mut entities = Entities::new();

        let (parent, parent_accessor) = spawn_entity(&mut entities);
        let (child, child_accessor) = spawn_entity(&mut entities);

        let parent_transform = Transform {
            position: Point2::new(0., 5.),
            rotation: Rad::from(Deg(180.)),
            scale: Vector2::new(2., 1.),
        };

        storage.insert(&entities,
                       parent.id(),
                       TransformTemplate {
                           transform: parent_transform,
                           parent: None,
                       });

        storage.insert(&entities,
                       child.id(),
                       TransformTemplate {
                           transform: Transform::one(),
                           parent: None,
                       });

        let child_transform = Transform {
            position: Point2::new(5., 0.),
            rotation: Rad(0.),
            scale: Vector2::new(1., 1.),
        };

        storage.insert(&entities,
                       child.id(),
                       TransformTemplate {
                           transform: child_transform,
                           parent: Some(entities.entity_ref(parent_accessor)),
                       });

        let expected = Transform {
            position: Point2::new(-10., 5.),
            rotation: Rad::from(Deg(180.)),
            scale: Vector2::new(2., 1.),
        };

        transform_approx_eq!(storage.local(child_accessor), Some(child_transform));
        transform_approx_eq!(storage.world(child_accessor), Some(expected));
        assert_eq!(storage.parent(child_accessor), Some(parent_accessor));
    }

    #[test]
    fn test_parent_transform() {
        let mut storage = TransformStorage::new();
        let mut entities = Entities::new();

        let (parent, parent_accessor) = spawn_entity(&mut entities);
        let (child, child_accessor) = spawn_entity(&mut entities);

        let parent_transform = Transform {
            position: Point2::new(0., 5.),
            rotation: Rad::from(Deg(180.)),
            scale: Vector2::new(2., 1.),
        };

        let child_transform = Transform {
            position: Point2::new(5., 0.),
            rotation: Rad(0.),
            scale: Vector2::new(1., 1.),
        };

        storage.insert(&entities,
                       parent.id(),
                       TransformTemplate {
                           transform: parent_transform,
                           parent: None,
                       });

        storage.insert(&entities,
                       child.id(),
                       TransformTemplate {
                           transform: child_transform,
                           parent: Some(entities.entity_ref(parent_accessor)),
                       });

        transform_approx_eq!(storage.local(parent_accessor), Some(parent_transform));
        transform_approx_eq!(storage.world(parent_accessor), Some(parent_transform));
        assert_eq!(storage.parent(parent_accessor), None);

        let expected = Transform {
            position: Point2::new(-10., 5.),
            rotation: Rad::from(Deg(180.)),
            scale: Vector2::new(2., 1.),
        };

        transform_approx_eq!(storage.local(child_accessor), Some(child_transform));
        transform_approx_eq!(storage.world(child_accessor), Some(expected));
        assert_eq!(storage.parent(child_accessor), Some(parent_accessor));
    }

    #[test]
    fn test_remove_parent() {
        let mut storage = TransformStorage::new();
        let mut entities = Entities::new();

        let (parent, parent_accessor) = spawn_entity(&mut entities);
        let (child, child_accessor) = spawn_entity(&mut entities);

        let parent_transform = Transform {
            position: Point2::new(0., 5.),
            rotation: Rad::from(Deg(180.)),
            scale: Vector2::new(2., 1.),
        };

        let child_transform = Transform {
            position: Point2::new(5., 0.),
            rotation: Rad(0.),
            scale: Vector2::new(1., 1.),
        };

        storage.insert(&entities,
                       parent.id(),
                       TransformTemplate {
                           transform: parent_transform,
                           parent: None,
                       });

        storage.insert(&entities,
                       child.id(),
                       TransformTemplate {
                           transform: child_transform,
                           parent: Some(entities.entity_ref(parent_accessor)),
                       });

        storage.remove_parent(child_accessor);

        transform_approx_eq!(storage.local(child_accessor), Some(child_transform));
        transform_approx_eq!(storage.world(child_accessor), Some(child_transform));
        assert_eq!(storage.parent(child_accessor), None);
    }

    #[test]
    fn test_set_parent() {
        let mut storage = TransformStorage::new();
        let mut entities = Entities::new();

        let (parent, parent_accessor) = spawn_entity(&mut entities);
        let (child, child_accessor) = spawn_entity(&mut entities);

        let parent_transform = Transform {
            position: Point2::new(0., 5.),
            rotation: Rad::from(Deg(180.)),
            scale: Vector2::new(2., 1.),
        };

        let child_transform = Transform {
            position: Point2::new(5., 0.),
            rotation: Rad(0.),
            scale: Vector2::new(1., 1.),
        };

        storage.insert(&entities,
                       parent.id(),
                       TransformTemplate {
                           transform: parent_transform,
                           parent: None,
                       });

        storage.insert(&entities,
                       child.id(),
                       TransformTemplate {
                           transform: child_transform,
                           parent: None,
                       });

        storage.set_parent(child_accessor, parent_accessor);

        let expected = Transform {
            position: Point2::new(-10., 5.),
            rotation: Rad::from(Deg(180.)),
            scale: Vector2::new(2., 1.),
        };

        transform_approx_eq!(storage.local(child_accessor), Some(child_transform));
        transform_approx_eq!(storage.world(child_accessor), Some(expected));
        assert_eq!(storage.parent(child_accessor), Some(parent_accessor));
    }

    #[test]
    fn test_children_forward() {
        let mut storage = TransformStorage::new();
        let mut entities = Entities::new();

        let (parent, spawned_entities) = spawn_entity_with_children(&mut entities, &mut storage, 3);

        let mut cursor = storage.children(parent);

        for child in spawned_entities {
            assert_eq!(cursor.next(&storage), Some(child));
        }

        assert_eq!(cursor.next(&storage), None);
    }

    #[test]
    fn test_remove_the_only_child() {
        let mut storage = TransformStorage::new();
        let mut entities = Entities::new();

        let (parent, spawned_entities) = spawn_entity_with_children(&mut entities, &mut storage, 1);
        let child = spawned_entities[0];

        storage.remove(child.id());

        let mut cursor = storage.children(parent);
        assert_eq!(cursor.next(&storage), None);
    }

    #[test]
    fn test_remove_child_head() {
        let mut storage = TransformStorage::new();
        let mut entities = Entities::new();

        let (parent, spawned_entities) = spawn_entity_with_children(&mut entities, &mut storage, 2);
        let child_head = spawned_entities[0];

        storage.remove(child_head.id());
        let mut cursor = storage.children(parent);

        for child in spawned_entities.into_iter().skip(1) {
            assert_eq!(cursor.next(&storage), Some(child));
        }
        assert_eq!(cursor.next(&storage), None);
    }

    #[test]
    fn test_remove_child_tail() {
        let mut storage = TransformStorage::new();
        let mut entities = Entities::new();

        let (parent, mut spawned_entities) =
            spawn_entity_with_children(&mut entities, &mut storage, 2);
        let child_tail = spawned_entities.pop().unwrap();

        storage.remove(child_tail.id());

        let mut cursor = storage.children(parent);

        for child in spawned_entities.into_iter() {
            assert_eq!(cursor.next(&storage), Some(child));
        }
        assert_eq!(cursor.next(&storage), None);
    }

    #[test]
    fn test_remove_child_in_the_middle() {
        let mut storage = TransformStorage::new();
        let mut entities = Entities::new();

        let (parent, mut spawned_entities) =
            spawn_entity_with_children(&mut entities, &mut storage, 3);

        let removed_child = spawned_entities.swap_remove(1);
        storage.remove(removed_child.id());

        let mut cursor = storage.children(parent);

        for child in spawned_entities.into_iter() {
            assert_eq!(cursor.next(&storage), Some(child));
        }
        assert_eq!(cursor.next(&storage), None);
    }

    fn spawn_entity_with_children<'a>(entities: &'a mut Entities,
                                      storage: &mut TransformStorage,
                                      children_count: usize)
                                      -> (Accessor<'a>, Vec<Accessor<'a>>) {

        let (parent, parent_accessor) = spawn_entity(entities);
        let parent_transform_template = TransformTemplate {
            transform: Transform::one(),
            parent: None,
        };


        storage.insert(&entities, parent.id(), parent_transform_template);

        let mut children = Vec::new();

        let child_transform_template = TransformTemplate {
            transform: Transform::one(),
            parent: Some(entities.entity_ref(parent_accessor)),
        };

        for _ in 0..children_count {
            let (entity, accessor) = spawn_entity(entities);
            storage.insert(&entities, entity.id(), child_transform_template);
            children.push(accessor);
        }

        (parent_accessor, children)
    }

    fn spawn_entity<'a>(entities: &mut Entities) -> (Entity, Accessor<'a>) {
        let entity = entities.create();
        entities.spawn(entity);

        (entity, unsafe { Accessor::new_unchecked(entity.id()) })
    }
}
