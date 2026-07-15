use crate::{Component, EntityId, Relation, Resource};
use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldSnapshot {
    pub entities: Vec<EntitySnapshot>,
    pub component_types: Vec<TypeSnapshot>,
    pub resource_types: Vec<TypeSnapshot>,
    pub relationship_types: Vec<RelationshipSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntitySnapshot {
    pub id: EntityId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeSnapshot {
    pub type_name: &'static str,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationshipSnapshot {
    pub type_name: &'static str,
    pub edges: Vec<(EntityId, Vec<EntityId>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentInspection {
    pub entity: EntityId,
    pub component_name: &'static str,
    pub value: String,
}

impl fmt::Display for ComponentInspection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "entity {}", self.entity.0)?;
        writeln!(f, "  {}: {}", self.component_name, self.value)
    }
}

impl fmt::Display for WorldSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "world snapshot")?;
        writeln!(f, "entities:")?;
        for entity in &self.entities {
            writeln!(f, "  - {}", entity.id.0)?;
        }

        writeln!(f, "components:")?;
        for component_type in &self.component_types {
            writeln!(
                f,
                "  - {} ({})",
                component_type.type_name, component_type.count
            )?;
        }

        writeln!(f, "resources:")?;
        for resource_type in &self.resource_types {
            writeln!(
                f,
                "  - {} ({})",
                resource_type.type_name, resource_type.count
            )?;
        }

        writeln!(f, "relationships:")?;
        for relationship_type in &self.relationship_types {
            writeln!(f, "  - {}", relationship_type.type_name)?;
            for (source, targets) in &relationship_type.edges {
                write!(f, "    {} ->", source.0)?;
                for target in targets {
                    write!(f, " {}", target.0)?;
                }
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct ComponentStore {
    type_name: &'static str,
    count: usize,
    values: Box<dyn Any + Send + Sync>,
}

#[derive(Debug)]
struct ResourceStore {
    type_name: &'static str,
    count: usize,
    value: Box<dyn Any + Send + Sync>,
}

#[derive(Debug)]
struct RelationshipStore {
    type_name: &'static str,
    count: usize,
    values: Box<dyn Any + Send + Sync>,
}

#[derive(Debug, Default)]
pub struct World {
    next_entity_id: u64,
    components: HashMap<TypeId, ComponentStore>,
    resources: HashMap<TypeId, ResourceStore>,
    relationships: HashMap<TypeId, RelationshipStore>,
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
            .insert(
                TypeId::of::<T>(),
                ResourceStore {
                    type_name: type_name::<T>(),
                    count: 1,
                    value: Box::new(resource),
                },
            )
            .and_then(|store| store.value.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    pub fn resource<T>(&self) -> Option<&T>
    where
        T: Resource,
    {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|resource| resource.value.downcast_ref::<T>())
    }

    pub fn resource_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Resource,
    {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|resource| resource.value.downcast_mut::<T>())
    }

    pub fn insert_component<T>(&mut self, entity: EntityId, component: T) -> Option<T>
    where
        T: Component,
    {
        let store = self.component_store_mut::<T>();
        let values = store
            .values
            .downcast_mut::<HashMap<EntityId, T>>()
            .expect("component storage type should match requested type");
        let previous = values.insert(entity, component);
        let inserted_new = previous.is_none();
        if inserted_new {
            store.count += 1;
        }
        previous
    }

    pub fn component<T>(&self, entity: EntityId) -> Option<&T>
    where
        T: Component,
    {
        self.component_storage::<T>()
            .and_then(|components| components.get(&entity))
    }

    pub fn component_mut<T>(&mut self, entity: EntityId) -> Option<&mut T>
    where
        T: Component,
    {
        self.component_store_mut::<T>()
            .values
            .downcast_mut::<HashMap<EntityId, T>>()
            .expect("component storage type should match requested type")
            .get_mut(&entity)
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
            .map(|components| {
                components
                    .iter()
                    .map(|(&entity, component)| (entity, component))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn inspect_component<T>(&self, entity: EntityId) -> Option<ComponentInspection>
    where
        T: Component + fmt::Debug,
    {
        self.component::<T>(entity)
            .map(|component| ComponentInspection {
                entity,
                component_name: type_name::<T>(),
                value: format!("{:?}", component),
            })
    }

    pub fn add_relationship<R>(&mut self, source: EntityId, target: EntityId) -> bool
    where
        R: Relation,
    {
        let store = self
            .relationships
            .entry(TypeId::of::<R>())
            .or_insert_with(|| RelationshipStore {
                type_name: type_name::<R>(),
                count: 0,
                values: Box::new(HashMap::<EntityId, Vec<EntityId>>::new()),
            });
        let relationships = store
            .values
            .downcast_mut::<HashMap<EntityId, Vec<EntityId>>>()
            .expect("relationship storage type should match requested type");
        let targets = relationships.entry(source).or_default();
        if targets.is_empty() {
            store.count += 1;
        }

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

    pub fn snapshot(&self) -> WorldSnapshot {
        let entities = (0..self.next_entity_id)
            .map(|entity_index| EntitySnapshot {
                id: EntityId(entity_index),
            })
            .collect();

        let mut component_types: Vec<TypeSnapshot> = self
            .components
            .values()
            .map(|store| TypeSnapshot {
                type_name: store.type_name,
                count: store.count,
            })
            .collect();
        component_types.sort_by_key(|snapshot| snapshot.type_name);

        let mut resource_types: Vec<TypeSnapshot> = self
            .resources
            .values()
            .map(|store| TypeSnapshot {
                type_name: store.type_name,
                count: store.count,
            })
            .collect();
        resource_types.sort_by_key(|snapshot| snapshot.type_name);

        let mut relationship_types: Vec<RelationshipSnapshot> = self
            .relationships
            .values()
            .map(|store| RelationshipSnapshot {
                type_name: store.type_name,
                edges: store
                    .values
                    .downcast_ref::<HashMap<EntityId, Vec<EntityId>>>()
                    .map(|relationships| {
                        let mut edges: Vec<_> = relationships
                            .iter()
                            .map(|(&source, targets)| (source, targets.clone()))
                            .collect();
                        edges.sort_by_key(|(source, _)| source.0);
                        edges
                    })
                    .unwrap_or_default(),
            })
            .collect();
        relationship_types.sort_by_key(|snapshot| snapshot.type_name);

        WorldSnapshot {
            entities,
            component_types,
            resource_types,
            relationship_types,
        }
    }

    fn component_storage<T>(&self) -> Option<&HashMap<EntityId, T>>
    where
        T: Component,
    {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|storage| storage.values.downcast_ref::<HashMap<EntityId, T>>())
    }

    fn component_store_mut<T>(&mut self) -> &mut ComponentStore
    where
        T: Component,
    {
        self.components
            .entry(TypeId::of::<T>())
            .or_insert_with(|| ComponentStore {
                type_name: type_name::<T>(),
                count: 0,
                values: Box::new(HashMap::<EntityId, T>::new()),
            })
    }

    fn relationship_storage<R>(&self) -> Option<&HashMap<EntityId, Vec<EntityId>>>
    where
        R: Relation,
    {
        self.relationships
            .get(&TypeId::of::<R>())
            .and_then(|storage| {
                storage
                    .values
                    .downcast_ref::<HashMap<EntityId, Vec<EntityId>>>()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::type_name;

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

        assert_eq!(
            world.insert_component(entity, Position { x: 1.0, y: 2.0 }),
            None
        );
        assert_eq!(
            world.component::<Position>(entity),
            Some(&Position { x: 1.0, y: 2.0 })
        );

        if let Some(position) = world.component_mut::<Position>(entity) {
            position.x = 3.0;
            position.y = 4.0;
        }

        assert_eq!(
            world.component::<Position>(entity),
            Some(&Position { x: 3.0, y: 4.0 })
        );
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
        assert_eq!(
            world.relationships_from::<ParentOf>(parent),
            Some(&[child][..])
        );
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

    #[test]
    fn inspects_a_named_component_value_for_one_entity() {
        let mut world = World::default();
        let entity = world.spawn();

        world.insert_component(entity, Position { x: 9.0, y: 3.5 });

        let inspection = world
            .inspect_component::<Position>(entity)
            .expect("component should exist");

        assert_eq!(inspection.entity, entity);
        assert_eq!(inspection.component_name, type_name::<Position>());
        assert_eq!(inspection.value, "Position { x: 9.0, y: 3.5 }");
        assert_eq!(
            format!("{inspection}"),
            format!(
                "entity {}\n  {}: Position {{ x: 9.0, y: 3.5 }}\n",
                entity.0,
                type_name::<Position>()
            )
        );
    }

    #[test]
    fn snapshots_world_state_without_mutating_it() {
        let mut world = World::default();
        let first = world.spawn();
        let second = world.spawn();

        world.insert_component(first, Position { x: 1.0, y: 2.0 });
        world.insert_resource(Score(7));
        world.add_relationship::<ParentOf>(first, second);

        let snapshot = world.snapshot();

        assert_eq!(
            snapshot.entities,
            vec![EntitySnapshot { id: first }, EntitySnapshot { id: second }]
        );
        assert_eq!(
            snapshot.component_types,
            vec![TypeSnapshot {
                type_name: type_name::<Position>(),
                count: 1,
            }]
        );
        assert_eq!(
            snapshot.resource_types,
            vec![TypeSnapshot {
                type_name: type_name::<Score>(),
                count: 1,
            }]
        );
        assert_eq!(
            snapshot.relationship_types,
            vec![RelationshipSnapshot {
                type_name: type_name::<ParentOf>(),
                edges: vec![(first, vec![second])],
            }]
        );
        assert_eq!(
            format!("{snapshot}"),
            format!(
                "world snapshot\nentities:\n  - {}\n  - {}\ncomponents:\n  - {} (1)\nresources:\n  - {} (1)\nrelationships:\n  - {}\n    {} -> {}\n",
                first.0,
                second.0,
                type_name::<Position>(),
                type_name::<Score>(),
                type_name::<ParentOf>(),
                first.0,
                second.0,
            )
        );
    }
}
