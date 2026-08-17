//! Corpus-private G2 experiment for AR-0030.
//!
//! This is an executable lifetime and identity model, not a renderer API. It
//! deliberately stops before GPU realization so the experiment cannot make a
//! provider staging mechanism or Doom source term stable by accident.

use std::collections::BTreeSet;

use thiserror::Error;

use crate::AuthoritativeSkyDepthManifest;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SubmissionIdentity(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SubmissionLocalGeometryId {
    pub submission: SubmissionIdentity,
    pub slot: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubmissionLocalGeometryLimits {
    pub max_payloads: usize,
    pub max_draws: usize,
    pub max_vertices: usize,
    pub max_vertices_per_payload: usize,
}

impl Default for SubmissionLocalGeometryLimits {
    fn default() -> Self {
        Self {
            max_payloads: 1_024,
            max_draws: 4_096,
            max_vertices: 1_000_000,
            max_vertices_per_payload: 250_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubmissionLocalGeometryPayload {
    pub id: SubmissionLocalGeometryId,
    pub positions: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubmissionLocalDraw {
    pub geometry: SubmissionLocalGeometryId,
    pub persistent_material_key: String,
    pub source_correlation: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubmissionLocalGeometrySnapshot {
    pub submission: SubmissionIdentity,
    pub payloads: Vec<SubmissionLocalGeometryPayload>,
    pub draws: Vec<SubmissionLocalDraw>,
    pub persistent_material_keys: BTreeSet<String>,
    pub total_vertices: usize,
    pub total_triangles: usize,
    pub persistent_mesh_identities: usize,
    pub structural_fingerprint: String,
}

impl SubmissionLocalGeometrySnapshot {
    pub fn resolve(
        &self,
        id: SubmissionLocalGeometryId,
    ) -> Result<&SubmissionLocalGeometryPayload, SubmissionLocalGeometryError> {
        if id.submission != self.submission {
            return Err(SubmissionLocalGeometryError::ForeignSubmission {
                expected: self.submission,
                observed: id.submission,
            });
        }
        self.payloads.get(id.slot as usize).ok_or(
            SubmissionLocalGeometryError::MissingLocalGeometry {
                submission: id.submission,
                slot: id.slot,
            },
        )
    }
}

#[derive(Clone, Debug)]
pub struct SubmissionLocalGeometryBuilder {
    submission: SubmissionIdentity,
    limits: SubmissionLocalGeometryLimits,
    payloads: Vec<SubmissionLocalGeometryPayload>,
    draws: Vec<SubmissionLocalDraw>,
    total_vertices: usize,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SubmissionLocalGeometryError {
    #[error("submission-local geometry contains no vertices")]
    EmptyGeometry,
    #[error("submission-local geometry vertex count {vertices} is not a triangle list")]
    InvalidTriangleList { vertices: usize },
    #[error("submission-local geometry contains a non-finite position at vertex {vertex}")]
    NonFinitePosition { vertex: usize },
    #[error("submission-local payload capacity {limit} exceeded")]
    PayloadCapacityExceeded { limit: usize },
    #[error("submission-local geometry cannot represent another local identity")]
    LocalIdentityCapacityExceeded,
    #[error("submission-local draw capacity {limit} exceeded")]
    DrawCapacityExceeded { limit: usize },
    #[error("submission-local vertex capacity {limit} exceeded by requested total {requested}")]
    VertexCapacityExceeded { limit: usize, requested: usize },
    #[error("submission-local payload vertex capacity {limit} exceeded by {requested}")]
    PayloadVertexCapacityExceeded { limit: usize, requested: usize },
    #[error("geometry belongs to submission {observed:?}, expected {expected:?}")]
    ForeignSubmission {
        expected: SubmissionIdentity,
        observed: SubmissionIdentity,
    },
    #[error("submission {submission:?} has no local geometry slot {slot}")]
    MissingLocalGeometry {
        submission: SubmissionIdentity,
        slot: u32,
    },
    #[error("persistent material key must not be empty")]
    EmptyMaterialKey,
    #[error("source correlation must not be empty")]
    EmptySourceCorrelation,
    #[error("submission-local geometry slot {slot} has no ordered draw")]
    UnreferencedGeometry { slot: u32 },
}

impl SubmissionLocalGeometryBuilder {
    pub fn new(submission: SubmissionIdentity, limits: SubmissionLocalGeometryLimits) -> Self {
        Self {
            submission,
            limits,
            payloads: Vec::new(),
            draws: Vec::new(),
            total_vertices: 0,
        }
    }

    pub fn add_geometry(
        &mut self,
        positions: Vec<[f32; 3]>,
    ) -> Result<SubmissionLocalGeometryId, SubmissionLocalGeometryError> {
        if positions.is_empty() {
            return Err(SubmissionLocalGeometryError::EmptyGeometry);
        }
        if !positions.len().is_multiple_of(3) {
            return Err(SubmissionLocalGeometryError::InvalidTriangleList {
                vertices: positions.len(),
            });
        }
        if let Some(vertex) = positions
            .iter()
            .position(|position| position.iter().any(|component| !component.is_finite()))
        {
            return Err(SubmissionLocalGeometryError::NonFinitePosition { vertex });
        }
        if self.payloads.len() >= self.limits.max_payloads {
            return Err(SubmissionLocalGeometryError::PayloadCapacityExceeded {
                limit: self.limits.max_payloads,
            });
        }
        if positions.len() > self.limits.max_vertices_per_payload {
            return Err(
                SubmissionLocalGeometryError::PayloadVertexCapacityExceeded {
                    limit: self.limits.max_vertices_per_payload,
                    requested: positions.len(),
                },
            );
        }
        let requested = self.total_vertices.saturating_add(positions.len());
        if requested > self.limits.max_vertices {
            return Err(SubmissionLocalGeometryError::VertexCapacityExceeded {
                limit: self.limits.max_vertices,
                requested,
            });
        }
        let slot = u32::try_from(self.payloads.len())
            .map_err(|_| SubmissionLocalGeometryError::LocalIdentityCapacityExceeded)?;
        let id = SubmissionLocalGeometryId {
            submission: self.submission,
            slot,
        };
        self.total_vertices = requested;
        self.payloads
            .push(SubmissionLocalGeometryPayload { id, positions });
        Ok(id)
    }

    pub fn add_draw(
        &mut self,
        geometry: SubmissionLocalGeometryId,
        persistent_material_key: impl Into<String>,
        source_correlation: impl Into<String>,
    ) -> Result<(), SubmissionLocalGeometryError> {
        if geometry.submission != self.submission {
            return Err(SubmissionLocalGeometryError::ForeignSubmission {
                expected: self.submission,
                observed: geometry.submission,
            });
        }
        if self.payloads.get(geometry.slot as usize).is_none() {
            return Err(SubmissionLocalGeometryError::MissingLocalGeometry {
                submission: geometry.submission,
                slot: geometry.slot,
            });
        }
        if self.draws.len() >= self.limits.max_draws {
            return Err(SubmissionLocalGeometryError::DrawCapacityExceeded {
                limit: self.limits.max_draws,
            });
        }
        let persistent_material_key = persistent_material_key.into();
        if persistent_material_key.is_empty() {
            return Err(SubmissionLocalGeometryError::EmptyMaterialKey);
        }
        let source_correlation = source_correlation.into();
        if source_correlation.is_empty() {
            return Err(SubmissionLocalGeometryError::EmptySourceCorrelation);
        }
        self.draws.push(SubmissionLocalDraw {
            geometry,
            persistent_material_key,
            source_correlation,
        });
        Ok(())
    }

    pub fn finish(self) -> Result<SubmissionLocalGeometrySnapshot, SubmissionLocalGeometryError> {
        let referenced = self
            .draws
            .iter()
            .map(|draw| draw.geometry.slot)
            .collect::<BTreeSet<_>>();
        if let Some(payload) = self
            .payloads
            .iter()
            .find(|payload| !referenced.contains(&payload.id.slot))
        {
            return Err(SubmissionLocalGeometryError::UnreferencedGeometry {
                slot: payload.id.slot,
            });
        }
        let persistent_material_keys = self
            .draws
            .iter()
            .map(|draw| draw.persistent_material_key.clone())
            .collect::<BTreeSet<_>>();
        let total_triangles = self.total_vertices / 3;
        let structural_fingerprint = fingerprint(
            self.submission,
            &self.payloads,
            &self.draws,
            &persistent_material_keys,
        );
        Ok(SubmissionLocalGeometrySnapshot {
            submission: self.submission,
            payloads: self.payloads,
            draws: self.draws,
            persistent_material_keys,
            total_vertices: self.total_vertices,
            total_triangles,
            persistent_mesh_identities: 0,
            structural_fingerprint,
        })
    }
}

pub fn prepare_authoritative_sky_submission_local_geometry(
    manifest: &AuthoritativeSkyDepthManifest,
    submission: SubmissionIdentity,
    limits: SubmissionLocalGeometryLimits,
) -> Result<SubmissionLocalGeometrySnapshot, SubmissionLocalGeometryError> {
    let mut builder = SubmissionLocalGeometryBuilder::new(submission, limits);
    for declaration in &manifest.declarations {
        let geometry = builder.add_geometry(declaration.positions.clone())?;
        builder.add_draw(
            geometry,
            declaration.persistent_material_key.clone(),
            format!(
                "doom-plane={:?};instance={};fixture={};source-position={:?};heading={:.12};eye-height={};snapshot={}",
                declaration.source_plane,
                declaration.source_plane_instance,
                declaration.prepared_view.fixture,
                declaration.prepared_view.source_position,
                declaration.prepared_view.heading_radians,
                declaration.prepared_view.source_eye_height,
                declaration.runtime_snapshot,
            ),
        )?;
    }
    builder.finish()
}

fn fingerprint(
    submission: SubmissionIdentity,
    payloads: &[SubmissionLocalGeometryPayload],
    draws: &[SubmissionLocalDraw],
    materials: &BTreeSet<String>,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"tokimu-ar-0030-g2-submission-local-geometry-v1\0");
    hasher.update(&submission.0.to_le_bytes());
    for payload in payloads {
        hasher.update(&payload.id.slot.to_le_bytes());
        for position in &payload.positions {
            for component in position {
                hasher.update(&component.to_bits().to_le_bytes());
            }
        }
    }
    for draw in draws {
        hasher.update(&draw.geometry.slot.to_le_bytes());
        hasher.update(draw.persistent_material_key.as_bytes());
        hasher.update(&[0]);
        hasher.update(draw.source_correlation.as_bytes());
        hasher.update(&[0]);
    }
    for material in materials {
        hasher.update(material.as_bytes());
        hasher.update(&[0]);
    }
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle(offset: f32) -> Vec<[f32; 3]> {
        vec![
            [offset, 0.0, 0.25],
            [offset + 1.0, 0.0, 0.25],
            [offset, 1.0, 0.25],
        ]
    }

    #[test]
    fn local_ids_do_not_cross_submission_boundaries() {
        let mut first = SubmissionLocalGeometryBuilder::new(
            SubmissionIdentity(1),
            SubmissionLocalGeometryLimits::default(),
        );
        let first_id = first.add_geometry(triangle(0.0)).unwrap();
        first.add_draw(first_id, "sky", "source-a").unwrap();
        let first = first.finish().unwrap();

        let mut second = SubmissionLocalGeometryBuilder::new(
            SubmissionIdentity(2),
            SubmissionLocalGeometryLimits::default(),
        );
        let second_id = second.add_geometry(triangle(0.0)).unwrap();
        second.add_draw(second_id, "sky", "source-a").unwrap();
        let second = second.finish().unwrap();

        assert_ne!(first_id, second_id);
        assert_eq!(
            second.resolve(first_id),
            Err(SubmissionLocalGeometryError::ForeignSubmission {
                expected: SubmissionIdentity(2),
                observed: SubmissionIdentity(1),
            })
        );
        assert_eq!(first.resolve(first_id).unwrap().positions.len(), 3);
    }

    #[test]
    fn invalid_and_unbounded_payloads_fail_before_handoff() {
        let limits = SubmissionLocalGeometryLimits {
            max_payloads: 1,
            max_draws: 1,
            max_vertices: 3,
            max_vertices_per_payload: 3,
        };
        let mut builder = SubmissionLocalGeometryBuilder::new(SubmissionIdentity(7), limits);
        assert_eq!(
            builder.add_geometry(vec![[0.0; 3]; 2]),
            Err(SubmissionLocalGeometryError::InvalidTriangleList { vertices: 2 })
        );
        let mut non_finite = triangle(0.0);
        non_finite[1][2] = f32::NAN;
        assert_eq!(
            builder.add_geometry(non_finite),
            Err(SubmissionLocalGeometryError::NonFinitePosition { vertex: 1 })
        );
        let id = builder.add_geometry(triangle(0.0)).unwrap();
        assert_eq!(
            builder.add_geometry(triangle(2.0)),
            Err(SubmissionLocalGeometryError::PayloadCapacityExceeded { limit: 1 })
        );
        builder.add_draw(id, "sky", "source").unwrap();
        assert_eq!(builder.finish().unwrap().total_triangles, 1);
    }

    #[test]
    fn finalization_rejects_unreferenced_local_geometry() {
        let mut builder = SubmissionLocalGeometryBuilder::new(
            SubmissionIdentity(8),
            SubmissionLocalGeometryLimits::default(),
        );
        builder.add_geometry(triangle(0.0)).unwrap();
        assert_eq!(
            builder.finish(),
            Err(SubmissionLocalGeometryError::UnreferencedGeometry { slot: 0 })
        );
    }

    #[test]
    fn persistent_material_and_local_geometry_identities_remain_separate() {
        let mut builder = SubmissionLocalGeometryBuilder::new(
            SubmissionIdentity(9),
            SubmissionLocalGeometryLimits::default(),
        );
        for offset in [0.0, 2.0] {
            let id = builder.add_geometry(triangle(offset)).unwrap();
            builder
                .add_draw(id, "doom-sky:SKY1", format!("source-{offset}"))
                .unwrap();
        }
        let snapshot = builder.finish().unwrap();
        assert_eq!(snapshot.payloads.len(), 2);
        assert_eq!(snapshot.draws.len(), 2);
        assert_eq!(snapshot.persistent_material_keys.len(), 1);
        assert_eq!(snapshot.persistent_mesh_identities, 0);
    }

    #[test]
    fn structural_fingerprint_is_deterministic_and_submission_scoped() {
        fn build(submission: u64) -> SubmissionLocalGeometrySnapshot {
            let mut builder = SubmissionLocalGeometryBuilder::new(
                SubmissionIdentity(submission),
                SubmissionLocalGeometryLimits::default(),
            );
            let id = builder.add_geometry(triangle(0.0)).unwrap();
            builder.add_draw(id, "sky", "source").unwrap();
            builder.finish().unwrap()
        }
        assert_eq!(
            build(10).structural_fingerprint,
            build(10).structural_fingerprint
        );
        assert_ne!(
            build(10).structural_fingerprint,
            build(11).structural_fingerprint
        );
    }

    #[test]
    fn authoritative_sky_uses_one_durable_material_without_durable_mesh_identity() {
        let fixture = crate::terminal_sky_ordered_fixture().unwrap();
        let regions =
            crate::observe_authoritative_sky_regions(&fixture, 41, "static-source-fixture")
                .unwrap();
        let depth =
            crate::prepare_authoritative_sky_depth_declarations(&regions, 0.25, "doom-sky:SKY1");
        let snapshot = prepare_authoritative_sky_submission_local_geometry(
            &depth,
            SubmissionIdentity(12),
            SubmissionLocalGeometryLimits::default(),
        )
        .unwrap();

        assert_eq!(snapshot.payloads.len(), 2);
        assert_eq!(snapshot.draws.len(), 2);
        assert_eq!(snapshot.total_vertices, 12);
        assert_eq!(snapshot.total_triangles, 4);
        assert_eq!(
            snapshot.persistent_material_keys,
            BTreeSet::from(["doom-sky:SKY1".to_owned()])
        );
        assert_eq!(snapshot.persistent_mesh_identities, 0);
        assert!(snapshot
            .draws
            .iter()
            .all(|draw| draw.source_correlation.contains("terminal-sky-ordered")));
    }

    #[test]
    fn authoritative_sky_capacity_failure_produces_no_partial_snapshot() {
        let fixture = crate::terminal_sky_ordered_fixture().unwrap();
        let regions =
            crate::observe_authoritative_sky_regions(&fixture, 41, "static-source-fixture")
                .unwrap();
        let depth =
            crate::prepare_authoritative_sky_depth_declarations(&regions, 0.25, "doom-sky:SKY1");
        let error = prepare_authoritative_sky_submission_local_geometry(
            &depth,
            SubmissionIdentity(13),
            SubmissionLocalGeometryLimits {
                max_payloads: 1,
                ..SubmissionLocalGeometryLimits::default()
            },
        )
        .unwrap_err();

        assert_eq!(
            error,
            SubmissionLocalGeometryError::PayloadCapacityExceeded { limit: 1 }
        );
    }
}
