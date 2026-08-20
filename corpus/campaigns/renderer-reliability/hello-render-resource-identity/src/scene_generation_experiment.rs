//! Corpus-private Alternative-C scene-generation experiment.
//!
//! These deliberately local types test staging, atomic commit, failed-stage
//! containment, and stale identity. They are not proposed renderer handles,
//! a public arena API, or a physical GPU-reclamation model.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CorpusSceneResourceKind {
    Mesh,
    Texture,
    Material,
    Pipeline,
    Camera,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CorpusSceneResourceKey {
    pub kind: CorpusSceneResourceKind,
    pub index: u32,
}

impl CorpusSceneResourceKey {
    const fn new(kind: CorpusSceneResourceKind, index: u32) -> Self {
        Self { kind, index }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CorpusSceneResource {
    pub key: CorpusSceneResourceKey,
    pub source_label: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CorpusSceneGeneration(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CorpusSceneResourceHandle {
    generation: CorpusSceneGeneration,
    pub resource: CorpusSceneResourceKey,
}

impl CorpusSceneResourceHandle {
    pub const fn generation(self) -> u32 {
        self.generation.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CorpusSceneDraw {
    mesh: CorpusSceneResourceKey,
    material: CorpusSceneResourceKey,
    pipeline: CorpusSceneResourceKey,
    camera: CorpusSceneResourceKey,
}

#[derive(Clone, Copy, Debug)]
struct CorpusSceneBlueprint {
    map: &'static str,
    resources: &'static [CorpusSceneResource],
    draws: &'static [CorpusSceneDraw],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CorpusSceneGenerationError {
    GenerationExhausted {
        current_generation: u32,
    },
    DuplicateResource {
        map: &'static str,
        resource: CorpusSceneResourceKey,
    },
    InjectedStagingFailure {
        map: &'static str,
        resource: CorpusSceneResourceKey,
        staged_resources: usize,
    },
    MissingDrawResource {
        map: &'static str,
        resource: CorpusSceneResourceKey,
    },
    CandidatePredecessorChanged {
        candidate_map: &'static str,
        expected_generation: Option<u32>,
        current_generation: Option<u32>,
    },
    NoCurrentScene,
    StaleGeneration {
        requested_generation: u32,
        current_generation: u32,
    },
    MissingResource {
        generation: u32,
        resource: CorpusSceneResourceKey,
    },
}

#[derive(Clone, Debug)]
struct StagedCorpusScene {
    map: &'static str,
    generation: CorpusSceneGeneration,
    expected_predecessor: Option<CorpusSceneGeneration>,
    resources: BTreeMap<CorpusSceneResourceKey, CorpusSceneResource>,
    draw_count: usize,
}

impl StagedCorpusScene {
    fn handle(&self, resource: CorpusSceneResourceKey) -> CorpusSceneResourceHandle {
        CorpusSceneResourceHandle {
            generation: self.generation,
            resource,
        }
    }
}

#[derive(Clone, Debug)]
struct CommittedCorpusScene {
    map: &'static str,
    generation: CorpusSceneGeneration,
    resources: BTreeMap<CorpusSceneResourceKey, CorpusSceneResource>,
    draw_count: usize,
}

#[derive(Clone, Debug, Default)]
struct CorpusSceneGenerationExperiment {
    current: Option<CommittedCorpusScene>,
}

impl CorpusSceneGenerationExperiment {
    fn stage(
        &self,
        blueprint: CorpusSceneBlueprint,
        injected_failure: Option<CorpusSceneResourceKey>,
    ) -> Result<StagedCorpusScene, CorpusSceneGenerationError> {
        let generation = match self.current.as_ref() {
            Some(current) => CorpusSceneGeneration(current.generation.0.checked_add(1).ok_or(
                CorpusSceneGenerationError::GenerationExhausted {
                    current_generation: current.generation.0,
                },
            )?),
            None => CorpusSceneGeneration(0),
        };
        let expected_predecessor = self.current.as_ref().map(|current| current.generation);
        let mut resources = BTreeMap::new();
        for resource in blueprint.resources {
            if resources.insert(resource.key, *resource).is_some() {
                return Err(CorpusSceneGenerationError::DuplicateResource {
                    map: blueprint.map,
                    resource: resource.key,
                });
            }
            if injected_failure == Some(resource.key) {
                return Err(CorpusSceneGenerationError::InjectedStagingFailure {
                    map: blueprint.map,
                    resource: resource.key,
                    staged_resources: resources.len(),
                });
            }
        }
        for draw in blueprint.draws {
            for resource in [draw.mesh, draw.material, draw.pipeline, draw.camera] {
                if !resources.contains_key(&resource) {
                    return Err(CorpusSceneGenerationError::MissingDrawResource {
                        map: blueprint.map,
                        resource,
                    });
                }
            }
        }
        Ok(StagedCorpusScene {
            map: blueprint.map,
            generation,
            expected_predecessor,
            resources,
            draw_count: blueprint.draws.len(),
        })
    }

    fn commit(
        &mut self,
        candidate: StagedCorpusScene,
    ) -> Result<Option<&'static str>, CorpusSceneGenerationError> {
        let current_generation = self.current.as_ref().map(|current| current.generation);
        if candidate.expected_predecessor != current_generation {
            return Err(CorpusSceneGenerationError::CandidatePredecessorChanged {
                candidate_map: candidate.map,
                expected_generation: candidate.expected_predecessor.map(|value| value.0),
                current_generation: current_generation.map(|value| value.0),
            });
        }
        let retired_map = self.current.as_ref().map(|current| current.map);
        self.current = Some(CommittedCorpusScene {
            map: candidate.map,
            generation: candidate.generation,
            resources: candidate.resources,
            draw_count: candidate.draw_count,
        });
        Ok(retired_map)
    }

    fn resolve(
        &self,
        handle: CorpusSceneResourceHandle,
    ) -> Result<CorpusSceneResource, CorpusSceneGenerationError> {
        let current = self
            .current
            .as_ref()
            .ok_or(CorpusSceneGenerationError::NoCurrentScene)?;
        if handle.generation != current.generation {
            return Err(CorpusSceneGenerationError::StaleGeneration {
                requested_generation: handle.generation.0,
                current_generation: current.generation.0,
            });
        }
        current.resources.get(&handle.resource).copied().ok_or(
            CorpusSceneGenerationError::MissingResource {
                generation: current.generation.0,
                resource: handle.resource,
            },
        )
    }

    fn current_map(&self) -> Option<&'static str> {
        self.current.as_ref().map(|current| current.map)
    }

    fn current_draw_count(&self) -> usize {
        self.current
            .as_ref()
            .map_or(0, |current| current.draw_count)
    }
}

const MESH_1: CorpusSceneResourceKey =
    CorpusSceneResourceKey::new(CorpusSceneResourceKind::Mesh, 1);
const TEXTURE_1: CorpusSceneResourceKey =
    CorpusSceneResourceKey::new(CorpusSceneResourceKind::Texture, 1);
const MATERIAL_1: CorpusSceneResourceKey =
    CorpusSceneResourceKey::new(CorpusSceneResourceKind::Material, 1);
const PIPELINE_1: CorpusSceneResourceKey =
    CorpusSceneResourceKey::new(CorpusSceneResourceKind::Pipeline, 1);
const CAMERA_1: CorpusSceneResourceKey =
    CorpusSceneResourceKey::new(CorpusSceneResourceKind::Camera, 1);

const E1M1_RESOURCES: [CorpusSceneResource; 5] = [
    CorpusSceneResource {
        key: MESH_1,
        source_label: "E1M1 mesh 1",
    },
    CorpusSceneResource {
        key: TEXTURE_1,
        source_label: "E1M1 texture 1",
    },
    CorpusSceneResource {
        key: MATERIAL_1,
        source_label: "E1M1 material 1",
    },
    CorpusSceneResource {
        key: PIPELINE_1,
        source_label: "E1M1 pipeline 1",
    },
    CorpusSceneResource {
        key: CAMERA_1,
        source_label: "E1M1 camera 1",
    },
];
const E1M2_RESOURCES: [CorpusSceneResource; 5] = [
    CorpusSceneResource {
        key: MESH_1,
        source_label: "E1M2 mesh 1",
    },
    CorpusSceneResource {
        key: TEXTURE_1,
        source_label: "E1M2 texture 1",
    },
    CorpusSceneResource {
        key: MATERIAL_1,
        source_label: "E1M2 material 1",
    },
    CorpusSceneResource {
        key: PIPELINE_1,
        source_label: "E1M2 pipeline 1",
    },
    CorpusSceneResource {
        key: CAMERA_1,
        source_label: "E1M2 camera 1",
    },
];
const ONE_DRAW: [CorpusSceneDraw; 1] = [CorpusSceneDraw {
    mesh: MESH_1,
    material: MATERIAL_1,
    pipeline: PIPELINE_1,
    camera: CAMERA_1,
}];

fn e1m1_blueprint() -> CorpusSceneBlueprint {
    CorpusSceneBlueprint {
        map: "E1M1",
        resources: &E1M1_RESOURCES,
        draws: &ONE_DRAW,
    }
}

fn e1m2_blueprint() -> CorpusSceneBlueprint {
    CorpusSceneBlueprint {
        map: "E1M2",
        resources: &E1M2_RESOURCES,
        draws: &ONE_DRAW,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusSceneGenerationEvidence {
    pub generation_a: u32,
    pub failed_generation_b: CorpusSceneGenerationError,
    pub map_after_failed_stage: Option<&'static str>,
    pub generation_a_after_failed_stage: CorpusSceneResource,
    pub generation_b: u32,
    pub retired_map: Option<&'static str>,
    pub map_after_commit: Option<&'static str>,
    pub committed_draw_count: usize,
    pub generation_a_after_commit: CorpusSceneGenerationError,
    pub generation_b_after_commit: CorpusSceneResource,
}

/// Counts observed from an actual corpus composition before renderer upload.
/// This is intentionally a correlation input rather than a renderer contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusSceneResourceInventory {
    pub source_label: String,
    pub meshes: u64,
    pub textures: u64,
    pub materials: u64,
    pub pipelines: u64,
    pub cameras: u64,
    pub commands: u64,
}

impl CorpusSceneResourceInventory {
    fn count(&self, kind: CorpusSceneResourceKind) -> u64 {
        match kind {
            CorpusSceneResourceKind::Mesh => self.meshes,
            CorpusSceneResourceKind::Texture => self.textures,
            CorpusSceneResourceKind::Material => self.materials,
            CorpusSceneResourceKind::Pipeline => self.pipelines,
            CorpusSceneResourceKind::Camera => self.cameras,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CorpusSceneInventoryCorrelationError {
    EmptyRequiredFamily {
        source_label: String,
        family: CorpusSceneResourceKind,
        commands: u64,
    },
    InjectedStagingFailure {
        source_label: String,
        family: CorpusSceneResourceKind,
        staged_resources: u64,
    },
    StaleGeneration {
        requested_generation: u32,
        current_generation: u32,
    },
    CandidatePredecessorChanged {
        expected_generation: Option<u32>,
        current_generation: Option<u32>,
    },
    MissingResource {
        generation: u32,
        family: CorpusSceneResourceKind,
        index: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusSceneInventoryCorrelationEvidence {
    pub source_a: String,
    pub source_b: String,
    pub generation_a: u32,
    pub generation_b: u32,
    pub generation_a_resources: u64,
    pub generation_b_resources: u64,
    pub failed_stage: CorpusSceneInventoryCorrelationError,
    pub source_after_failed_stage: String,
    pub retired_source: String,
    pub source_after_commit: String,
    pub generation_a_after_commit: CorpusSceneInventoryCorrelationError,
    pub generation_b_reused_mesh_key_resolves: bool,
}

fn validate_inventory(
    inventory: &CorpusSceneResourceInventory,
) -> Result<(), CorpusSceneInventoryCorrelationError> {
    for family in [
        CorpusSceneResourceKind::Mesh,
        CorpusSceneResourceKind::Material,
        CorpusSceneResourceKind::Pipeline,
        CorpusSceneResourceKind::Camera,
    ] {
        if inventory.count(family) == 0 {
            return Err(CorpusSceneInventoryCorrelationError::EmptyRequiredFamily {
                source_label: inventory.source_label.clone(),
                family,
                commands: inventory.commands,
            });
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct InventoryHandle {
    generation: u32,
    family: CorpusSceneResourceKind,
    index: u64,
}

#[derive(Debug)]
struct StagedInventory {
    generation: u32,
    expected_predecessor: Option<u32>,
    inventory: CorpusSceneResourceInventory,
}

#[derive(Default)]
struct InventoryGenerationExperiment {
    current: Option<(u32, CorpusSceneResourceInventory)>,
}

impl InventoryGenerationExperiment {
    fn stage(
        &self,
        inventory: CorpusSceneResourceInventory,
        injected_failure: Option<CorpusSceneResourceKind>,
    ) -> Result<StagedInventory, CorpusSceneInventoryCorrelationError> {
        validate_inventory(&inventory)?;
        let expected_predecessor = self.current.as_ref().map(|(generation, _)| *generation);
        let generation = expected_predecessor.map_or(0, |value| value + 1);
        let mut staged_resources = 0_u64;
        for family in [
            CorpusSceneResourceKind::Mesh,
            CorpusSceneResourceKind::Texture,
            CorpusSceneResourceKind::Material,
            CorpusSceneResourceKind::Pipeline,
            CorpusSceneResourceKind::Camera,
        ] {
            let count = inventory.count(family);
            if injected_failure == Some(family) {
                return Err(
                    CorpusSceneInventoryCorrelationError::InjectedStagingFailure {
                        source_label: inventory.source_label,
                        family,
                        staged_resources: staged_resources.saturating_add(count.min(1)),
                    },
                );
            }
            staged_resources = staged_resources.saturating_add(count);
        }
        Ok(StagedInventory {
            generation,
            expected_predecessor,
            inventory,
        })
    }

    fn commit(
        &mut self,
        candidate: StagedInventory,
    ) -> Result<Option<CorpusSceneResourceInventory>, CorpusSceneInventoryCorrelationError> {
        let current_generation = self.current.as_ref().map(|(generation, _)| *generation);
        if candidate.expected_predecessor != current_generation {
            return Err(
                CorpusSceneInventoryCorrelationError::CandidatePredecessorChanged {
                    expected_generation: candidate.expected_predecessor,
                    current_generation,
                },
            );
        }
        Ok(self
            .current
            .replace((candidate.generation, candidate.inventory))
            .map(|(_, inventory)| inventory))
    }

    fn handle(
        &self,
        family: CorpusSceneResourceKind,
        index: u64,
    ) -> Result<InventoryHandle, CorpusSceneInventoryCorrelationError> {
        let (generation, inventory) = self.current.as_ref().expect("committed inventory required");
        if index == 0 || index > inventory.count(family) {
            return Err(CorpusSceneInventoryCorrelationError::MissingResource {
                generation: *generation,
                family,
                index,
            });
        }
        Ok(InventoryHandle {
            generation: *generation,
            family,
            index,
        })
    }

    fn resolve(&self, handle: InventoryHandle) -> Result<(), CorpusSceneInventoryCorrelationError> {
        let (current_generation, inventory) =
            self.current.as_ref().expect("committed inventory required");
        if handle.generation != *current_generation {
            return Err(CorpusSceneInventoryCorrelationError::StaleGeneration {
                requested_generation: handle.generation,
                current_generation: *current_generation,
            });
        }
        if handle.index == 0 || handle.index > inventory.count(handle.family) {
            return Err(CorpusSceneInventoryCorrelationError::MissingResource {
                generation: *current_generation,
                family: handle.family,
                index: handle.index,
            });
        }
        Ok(())
    }
}

fn inventory_resource_count(inventory: &CorpusSceneResourceInventory) -> u64 {
    inventory
        .meshes
        .saturating_add(inventory.textures)
        .saturating_add(inventory.materials)
        .saturating_add(inventory.pipelines)
        .saturating_add(inventory.cameras)
}

/// Replays the proven A/fail-B/commit-B generation sequence over two resource
/// inventories measured by real corpus compositions. It remains a semantic
/// shadow: no provider resource is staged, committed, or reclaimed here.
pub fn correlate_scene_resource_inventories(
    generation_a: CorpusSceneResourceInventory,
    generation_b: CorpusSceneResourceInventory,
) -> Result<CorpusSceneInventoryCorrelationEvidence, CorpusSceneInventoryCorrelationError> {
    let mut experiment = InventoryGenerationExperiment::default();
    let staged_a = experiment.stage(generation_a.clone(), None)?;
    experiment.commit(staged_a)?;
    let handle_a = experiment.handle(CorpusSceneResourceKind::Mesh, 1)?;

    let failed_stage = experiment
        .stage(
            generation_b.clone(),
            Some(CorpusSceneResourceKind::Material),
        )
        .expect_err("the correlation failure point must reject staging");
    experiment.resolve(handle_a)?;
    let source_after_failed_stage = experiment
        .current
        .as_ref()
        .expect("generation A remains committed")
        .1
        .source_label
        .clone();

    let staged_b = experiment.stage(generation_b.clone(), None)?;
    let retired = experiment
        .commit(staged_b)?
        .expect("generation B must retire generation A");
    let generation_a_after_commit = experiment
        .resolve(handle_a)
        .expect_err("generation A must be stale after generation B commits");
    let handle_b = experiment.handle(CorpusSceneResourceKind::Mesh, 1)?;
    let generation_b_reused_mesh_key_resolves = experiment.resolve(handle_b).is_ok();

    Ok(CorpusSceneInventoryCorrelationEvidence {
        source_a: generation_a.source_label.clone(),
        source_b: generation_b.source_label.clone(),
        generation_a: 0,
        generation_b: 1,
        generation_a_resources: inventory_resource_count(&generation_a),
        generation_b_resources: inventory_resource_count(&generation_b),
        failed_stage,
        source_after_failed_stage,
        retired_source: retired.source_label,
        source_after_commit: generation_b.source_label,
        generation_a_after_commit,
        generation_b_reused_mesh_key_resolves,
    })
}

/// Executes the exact Alternative-C proof sequence without a renderer or GPU:
/// commit E1M1 A, fail E1M2 B staging, prove A survives, commit a complete E1M2
/// B, then prove A is stale while the same local resource identity resolves B.
pub fn observe_e1m1_e1m2_generation_replacement() -> CorpusSceneGenerationEvidence {
    let mut experiment = CorpusSceneGenerationExperiment::default();
    let generation_a_candidate = experiment
        .stage(e1m1_blueprint(), None)
        .expect("the fixed E1M1 candidate must stage");
    let generation_a_handle = generation_a_candidate.handle(MESH_1);
    experiment
        .commit(generation_a_candidate)
        .expect("the first candidate has no competing predecessor");

    let failed_generation_b = experiment
        .stage(e1m2_blueprint(), Some(MATERIAL_1))
        .expect_err("the fixed E1M2 failure point must reject staging");
    let map_after_failed_stage = experiment.current_map();
    let generation_a_after_failed_stage = experiment
        .resolve(generation_a_handle)
        .expect("failed staging must leave generation A usable");

    let generation_b_candidate = experiment
        .stage(e1m2_blueprint(), None)
        .expect("the complete E1M2 candidate must stage");
    let generation_b_handle = generation_b_candidate.handle(MESH_1);
    let retired_map = experiment
        .commit(generation_b_candidate)
        .expect("the E1M2 candidate must still target generation A");
    let map_after_commit = experiment.current_map();
    let committed_draw_count = experiment.current_draw_count();
    let generation_a_after_commit = experiment
        .resolve(generation_a_handle)
        .expect_err("generation A must be stale after B commits");
    let generation_b_after_commit = experiment
        .resolve(generation_b_handle)
        .expect("generation B must resolve after commit");

    CorpusSceneGenerationEvidence {
        generation_a: generation_a_handle.generation(),
        failed_generation_b,
        map_after_failed_stage,
        generation_a_after_failed_stage,
        generation_b: generation_b_handle.generation(),
        retired_map,
        map_after_commit,
        committed_draw_count,
        generation_a_after_commit,
        generation_b_after_commit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_stage_preserves_a_and_commit_b_makes_a_stale() {
        let evidence = observe_e1m1_e1m2_generation_replacement();
        assert_eq!(evidence.generation_a, 0);
        assert!(matches!(
            evidence.failed_generation_b,
            CorpusSceneGenerationError::InjectedStagingFailure {
                map: "E1M2",
                resource: MATERIAL_1,
                staged_resources: 3,
            }
        ));
        assert_eq!(evidence.map_after_failed_stage, Some("E1M1"));
        assert_eq!(
            evidence.generation_a_after_failed_stage.source_label,
            "E1M1 mesh 1"
        );
        assert_eq!(evidence.generation_b, 1);
        assert_eq!(evidence.retired_map, Some("E1M1"));
        assert_eq!(evidence.map_after_commit, Some("E1M2"));
        assert_eq!(evidence.committed_draw_count, 1);
        assert_eq!(
            evidence.generation_a_after_commit,
            CorpusSceneGenerationError::StaleGeneration {
                requested_generation: 0,
                current_generation: 1,
            }
        );
        assert_eq!(
            evidence.generation_b_after_commit.source_label,
            "E1M2 mesh 1"
        );
        assert_eq!(
            evidence.generation_a_after_failed_stage.key,
            evidence.generation_b_after_commit.key
        );
    }

    #[test]
    fn competing_candidate_cannot_replace_a_newer_commit() {
        let mut experiment = CorpusSceneGenerationExperiment::default();
        let first = experiment.stage(e1m1_blueprint(), None).unwrap();
        let competing = experiment.stage(e1m2_blueprint(), None).unwrap();
        experiment.commit(first).unwrap();
        assert_eq!(
            experiment.commit(competing),
            Err(CorpusSceneGenerationError::CandidatePredecessorChanged {
                candidate_map: "E1M2",
                expected_generation: None,
                current_generation: Some(0),
            })
        );
        assert_eq!(experiment.current_map(), Some("E1M1"));
    }

    #[test]
    fn incomplete_candidate_rejects_before_commit() {
        const INCOMPLETE: [CorpusSceneResource; 1] = [CorpusSceneResource {
            key: MESH_1,
            source_label: "incomplete mesh",
        }];
        let experiment = CorpusSceneGenerationExperiment::default();
        assert_eq!(
            experiment
                .stage(
                    CorpusSceneBlueprint {
                        map: "BROKEN",
                        resources: &INCOMPLETE,
                        draws: &ONE_DRAW,
                    },
                    None,
                )
                .unwrap_err(),
            CorpusSceneGenerationError::MissingDrawResource {
                map: "BROKEN",
                resource: MATERIAL_1,
            }
        );
        assert_eq!(experiment.current_map(), None);
    }

    #[test]
    fn generation_exhaustion_rejects_without_mutating_current() {
        let experiment = CorpusSceneGenerationExperiment {
            current: Some(CommittedCorpusScene {
                map: "MAX",
                generation: CorpusSceneGeneration(u32::MAX),
                resources: BTreeMap::new(),
                draw_count: 0,
            }),
        };
        assert_eq!(
            experiment.stage(e1m2_blueprint(), None).unwrap_err(),
            CorpusSceneGenerationError::GenerationExhausted {
                current_generation: u32::MAX,
            }
        );
        assert_eq!(experiment.current_map(), Some("MAX"));
    }

    #[test]
    fn heterogeneous_inventory_shapes_preserve_generation_semantics() {
        let evidence = correlate_scene_resource_inventories(
            CorpusSceneResourceInventory {
                source_label: "representative resource-rich A".into(),
                meshes: 2_325,
                textures: 71,
                materials: 73,
                pipelines: 7,
                cameras: 1,
                commands: 4_652,
            },
            CorpusSceneResourceInventory {
                source_label: "representative resource-rich B".into(),
                meshes: 1_921,
                textures: 83,
                materials: 85,
                pipelines: 7,
                cameras: 1,
                commands: 3_844,
            },
        )
        .unwrap();
        assert_eq!(evidence.source_after_failed_stage, evidence.source_a);
        assert_eq!(evidence.retired_source, evidence.source_a);
        assert_eq!(evidence.source_after_commit, evidence.source_b);
        assert_eq!(
            evidence.generation_a_after_commit,
            CorpusSceneInventoryCorrelationError::StaleGeneration {
                requested_generation: 0,
                current_generation: 1,
            }
        );
        assert!(evidence.generation_b_reused_mesh_key_resolves);
    }

    #[test]
    fn inventory_with_commands_but_no_camera_rejects_before_staging() {
        let error = correlate_scene_resource_inventories(
            CorpusSceneResourceInventory {
                source_label: "broken A".into(),
                meshes: 1,
                textures: 1,
                materials: 1,
                pipelines: 1,
                cameras: 0,
                commands: 1,
            },
            CorpusSceneResourceInventory {
                source_label: "unused B".into(),
                meshes: 1,
                textures: 1,
                materials: 1,
                pipelines: 1,
                cameras: 1,
                commands: 1,
            },
        )
        .unwrap_err();
        assert_eq!(
            error,
            CorpusSceneInventoryCorrelationError::EmptyRequiredFamily {
                source_label: "broken A".into(),
                family: CorpusSceneResourceKind::Camera,
                commands: 1,
            }
        );
    }
}
