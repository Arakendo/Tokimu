use crate::{Component, EntityId, Relation, Resource};
use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct World {
    next_entity_id: u64,
    components: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    relationships: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl World {
    pub fn spawn(&mut self) -> EntityId {
        let id = EntityId(self.next_entity_id);
        self.next_entity_id += 1;
        id
    }

    pub fn insert_resource<T>(&mut self, resource: T) -> Option<T>
    where
        T: Resource,
    {
        self.resources
            .insert(TypeId::of::<T>(), Box::new(resource))
            .and_then(|value| value.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    pub fn resource<T>(&self) -> Option<&T>
    where
        T: Resource,
    {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|resource| resource.downcast_ref::<T>())
    }

    pub fn resource_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Resource,
    {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|resource| resource.downcast_mut::<T>())
    }

    pub fn insert_component<T>(&mut self, entity: EntityId, component: T) -> Option<T>
    where
        T: Component,
    {
        self.component_storage_mut::<T>()
            .insert(entity, component)
    }

    pub fn component<T>(&self, entity: EntityId) -> Option<&T>
    where
        T: Component,
    {
        self.component_storage::<T>().and_then(|components| components.get(&entity))
    }

    pub fn component_mut<T>(&mut self, entity: EntityId) -> Option<&mut T>
    where
        T: Component,
    {
        self.component_storage_mut::<T>().get_mut(&entity)
    }

    pub fn for_each_component<T, F>(&self, mut visit: F)
    where
        T: Component,
        F: FnMut(EntityId, &T),
    {
        if let Some(components) = self.component_storage::<T>() {
            for (&entity, component) in components {
                visit(entity, component);
            }
        }
    }

    pub fn query_component<T>(&self) -> Vec<(EntityId, &T)>
    where
        T: Component,
    {
        self.component_storage::<T>()
            .map(|components| components.iter().map(|(&entity, component)| (entity, component)).collect())
            .unwrap_or_default()
    }

    pub fn add_relationship<R>(&mut self, source: EntityId, target: EntityId) -> bool
    where
        R: Relation,
    {
        let targets = self.relationship_storage_mut::<R>().entry(source).or_default();

        if targets.contains(&target) {
            false
        } else {
            targets.push(target);
            true
        }
    }

    pub fn relationships_from<R>(&self, source: EntityId) -> Option<&[EntityId]>
    where
        R: Relation,
    {
        self.relationship_storage::<R>()
            .and_then(|relationships| relationships.get(&source))
            .map(Vec::as_slice)
    }

    pub fn has_relationship<R>(&self, source: EntityId, target: EntityId) -> bool
    where
        R: Relation,
    {
        self.relationships_from::<R>(source)
            .is_some_and(|targets| targets.contains(&target))
    }

    pub fn query_relationships<R>(&self, source: EntityId) -> Vec<EntityId>
    where
        R: Relation,
    {
        self.relationships_from::<R>(source)
            .map(|targets| targets.to_vec())
            .unwrap_or_default()
    }

    fn component_storage<T>(&self) -> Option<&HashMap<EntityId, T>>
    where
        T: Component,
    {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|storage| storage.downcast_ref::<HashMap<EntityId, T>>())
    }

    fn component_storage_mut<T>(&mut self) -> &mut HashMap<EntityId, T>
    where
        T: Component,
    {
        self.components
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(HashMap::<EntityId, T>::new()))
            .downcast_mut::<HashMap<EntityId, T>>()
            .expect("component storage type should match requested type")
    }

    fn relationship_storage<R>(&self) -> Option<&HashMap<EntityId, Vec<EntityId>>>
    where
        R: Relation,
    {
        self.relationships
            .get(&TypeId::of::<R>())
            .and_then(|storage| storage.downcast_ref::<HashMap<EntityId, Vec<EntityId>>>())
    }

    fn relationship_storage_mut<R>(&mut self) -> &mut HashMap<EntityId, Vec<EntityId>>
    where
        R: Relation,
    {
        self.relationships
            .entry(TypeId::of::<R>())
            .or_insert_with(|| Box::new(HashMap::<EntityId, Vec<EntityId>>::new()))
            .downcast_mut::<HashMap<EntityId, Vec<EntityId>>>()
            .expect("relationship storage type should match requested type")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Score(u32);

    #[derive(Clone, Debug)]
    struct ParentOf;

    #[test]
    fn spawn_returns_deterministic_entity_ids() {
        let mut world = World::default();

        assert_eq!(world.spawn(), EntityId(0));
        assert_eq!(world.spawn(), EntityId(1));
    }

    #[test]
    fn stores_and_reads_resources() {
        let mut world = World::default();

        assert_eq!(world.insert_resource(Score(3)), None);
        assert_eq!(world.resource::<Score>(), Some(&Score(3)));

        if let Some(score) = world.resource_mut::<Score>() {
            score.0 += 2;
        }

        assert_eq!(world.resource::<Score>(), Some(&Score(5)));
    }

    #[test]
    fn stores_and_reads_components() {
        let mut world = World::default();
        let entity = world.spawn();

        assert_eq!(world.insert_component(entity, Position { x: 1.0, y: 2.0 }), None);
        assert_eq!(world.component::<Position>(entity), Some(&Position { x: 1.0, y: 2.0 }));

        if let Some(position) = world.component_mut::<Position>(entity) {
            position.x = 3.0;
            position.y = 4.0;
        }

        assert_eq!(world.component::<Position>(entity), Some(&Position { x: 3.0, y: 4.0 }));
    }

    #[test]
    fn iterates_components() {
        let mut world = World::default();
        let first = world.spawn();
        let second = world.spawn();

        world.insert_component(first, Position { x: 1.0, y: 2.0 });
        world.insert_component(second, Position { x: 3.0, y: 4.0 });

        let mut seen = Vec::new();
        world.for_each_component::<Position, _>(|entity, position| {
            seen.push((entity, position.clone()));
        });

        seen.sort_by_key(|(entity, _)| entity.0);
        assert_eq!(
            seen,
            vec![
                (first, Position { x: 1.0, y: 2.0 }),
                (second, Position { x: 3.0, y: 4.0 }),
            ]
        );
    }

    #[test]
    fn queries_components() {
        let mut world = World::default();
        let first = world.spawn();
        let second = world.spawn();

        world.insert_component(first, Position { x: 1.0, y: 2.0 });
        world.insert_component(second, Position { x: 3.0, y: 4.0 });

        let mut matches = world.query_component::<Position>();
        matches.sort_by_key(|(entity, _)| entity.0);

        assert_eq!(
            matches,
            vec![
                (first, &Position { x: 1.0, y: 2.0 }),
                (second, &Position { x: 3.0, y: 4.0 }),
            ]
        );
    }

    #[test]
    fn stores_directional_relationships_from_source_to_targets() {
        let mut world = World::default();
        let parent = world.spawn();
        let child = world.spawn();

        assert!(world.add_relationship::<ParentOf>(parent, child));
        assert!(!world.add_relationship::<ParentOf>(parent, child));
        assert_eq!(world.relationships_from::<ParentOf>(parent), Some(&[child][..]));
        assert!(world.has_relationship::<ParentOf>(parent, child));
        assert_eq!(world.relationships_from::<ParentOf>(child), None);
    }

    #[test]
    fn queries_relationship_targets() {
        let mut world = World::default();
        let parent = world.spawn();
        let first_child = world.spawn();
        let second_child = world.spawn();

        world.add_relationship::<ParentOf>(parent, first_child);
        world.add_relationship::<ParentOf>(parent, second_child);

        let mut targets = world.query_relationships::<ParentOf>(parent);
        targets.sort_by_key(|entity| entity.0);

        assert_eq!(targets, vec![first_child, second_child]);
    }
}
