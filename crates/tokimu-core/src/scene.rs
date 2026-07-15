use crate::World;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SceneDoc {
    pub entities: Vec<SceneEntityDoc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneEntityDoc {
    pub position: Option<ScenePosition>,
    pub parent: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenePosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneHistoryRecord {
    pub what: SceneChange,
    pub system: String,
    pub why: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SceneChange {
    SpawnedEntity,
    SetPosition {
        entity_index: usize,
        position: ScenePosition,
    },
    LinkedParent {
        child_index: usize,
        parent_index: usize,
    },
}

#[derive(Clone, Debug)]
pub struct SceneParent;

pub fn compile_scene(scene: &SceneDoc) -> World {
    let mut world = World::default();
    let mut entity_ids = Vec::with_capacity(scene.entities.len());

    for entity_doc in &scene.entities {
        let entity_id = world.spawn();
        if let Some(position) = entity_doc.position {
            world.insert_component(entity_id, position);
        }
        entity_ids.push(entity_id);
    }

    for (index, entity_doc) in scene.entities.iter().enumerate() {
        if let Some(parent_index) = entity_doc.parent {
            if let Some(&parent_id) = entity_ids.get(parent_index) {
                let child_id = entity_ids[index];
                world.add_relationship::<SceneParent>(child_id, parent_id);
            }
        }
    }

    world
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntityId;

    #[test]
    fn compile_scene_spawns_entities_components_and_relationships() {
        let scene = SceneDoc {
            entities: vec![
                SceneEntityDoc {
                    position: Some(ScenePosition { x: 1.0, y: 2.0 }),
                    parent: None,
                },
                SceneEntityDoc {
                    position: Some(ScenePosition { x: 3.0, y: 4.0 }),
                    parent: Some(0),
                },
            ],
        };

        let mut world = compile_scene(&scene);

        assert_eq!(world.spawn(), EntityId(2));
        assert_eq!(
            world.component::<ScenePosition>(EntityId(0)),
            Some(&ScenePosition { x: 1.0, y: 2.0 })
        );
        assert_eq!(
            world.component::<ScenePosition>(EntityId(1)),
            Some(&ScenePosition { x: 3.0, y: 4.0 })
        );
        assert!(world.has_relationship::<SceneParent>(EntityId(1), EntityId(0)));
        assert_eq!(
            world.query_relationships::<SceneParent>(EntityId(1)),
            vec![EntityId(0)]
        );
    }

    #[test]
    fn history_record_captures_what_system_and_why() {
        let record = SceneHistoryRecord {
            what: SceneChange::SetPosition {
                entity_index: 1,
                position: ScenePosition { x: 5.0, y: 6.0 },
            },
            system: "layout-pass".into(),
            why: Some("align the spawn point".into()),
        };

        assert_eq!(record.system, "layout-pass");
        assert_eq!(record.why.as_deref(), Some("align the spawn point"));
    }
}
