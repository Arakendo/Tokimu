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
}
