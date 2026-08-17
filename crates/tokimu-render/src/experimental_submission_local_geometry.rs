//! Unstable corpus-only submission-local geometry intake.
//!
//! This module is deliberately feature-gated and hidden from generated public
//! documentation. It exists to test AR-0030's G2 lifetime hypothesis without
//! admitting a stable renderer contract.

use std::collections::BTreeSet;

use thiserror::Error;

use crate::{
    CameraHandle, Instance2d, MaterialHandle, MaterialOverride, Mesh, PipelineHandle, ViewportRect,
};

pub const MAX_EXPERIMENTAL_LOCAL_PAYLOADS: usize = 4_096;
pub const MAX_EXPERIMENTAL_LOCAL_DRAWS: usize = 16_384;
pub const MAX_EXPERIMENTAL_LOCAL_VERTICES: usize = 4_000_000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExperimentalSubmissionIdentity(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExperimentalLocalGeometryId {
    submission: ExperimentalSubmissionIdentity,
    slot: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExperimentalLocalGeometryDraw {
    pub geometry: ExperimentalLocalGeometryId,
    pub material: MaterialHandle,
    pub pipeline: PipelineHandle,
    pub instance: Instance2d,
    pub camera: Option<CameraHandle>,
    pub viewport: Option<ViewportRect>,
    pub material_override: Option<MaterialOverride>,
}

impl ExperimentalLocalGeometryDraw {
    pub(crate) fn geometry_slot_for_backend(&self) -> usize {
        self.geometry.slot as usize
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExperimentalSubmissionLocalGeometry {
    identity: ExperimentalSubmissionIdentity,
    payloads: Vec<Mesh>,
    draws: Vec<ExperimentalLocalGeometryDraw>,
    total_vertices: usize,
}

impl ExperimentalSubmissionLocalGeometry {
    pub fn identity(&self) -> ExperimentalSubmissionIdentity {
        self.identity
    }

    pub fn payloads(&self) -> &[Mesh] {
        &self.payloads
    }

    pub fn draws(&self) -> &[ExperimentalLocalGeometryDraw] {
        &self.draws
    }

    pub fn total_vertices(&self) -> usize {
        self.total_vertices
    }
}

#[derive(Debug)]
pub struct ExperimentalSubmissionLocalGeometryBuilder {
    identity: ExperimentalSubmissionIdentity,
    payloads: Vec<Mesh>,
    draws: Vec<ExperimentalLocalGeometryDraw>,
    total_vertices: usize,
}

impl ExperimentalSubmissionLocalGeometryBuilder {
    pub fn new(identity: ExperimentalSubmissionIdentity) -> Self {
        Self {
            identity,
            payloads: Vec::new(),
            draws: Vec::new(),
            total_vertices: 0,
        }
    }

    pub fn add_geometry(
        &mut self,
        mesh: Mesh,
    ) -> Result<ExperimentalLocalGeometryId, ExperimentalSubmissionLocalGeometryError> {
        validate_mesh(&mesh)?;
        if self.payloads.len() == MAX_EXPERIMENTAL_LOCAL_PAYLOADS {
            return Err(
                ExperimentalSubmissionLocalGeometryError::PayloadCapacityExceeded {
                    limit: MAX_EXPERIMENTAL_LOCAL_PAYLOADS,
                },
            );
        }
        let total_vertices = self
            .total_vertices
            .checked_add(mesh.positions.len())
            .ok_or(
                ExperimentalSubmissionLocalGeometryError::VertexCapacityExceeded {
                    limit: MAX_EXPERIMENTAL_LOCAL_VERTICES,
                },
            )?;
        if total_vertices > MAX_EXPERIMENTAL_LOCAL_VERTICES {
            return Err(
                ExperimentalSubmissionLocalGeometryError::VertexCapacityExceeded {
                    limit: MAX_EXPERIMENTAL_LOCAL_VERTICES,
                },
            );
        }
        let id = ExperimentalLocalGeometryId {
            submission: self.identity,
            slot: self.payloads.len() as u32,
        };
        self.payloads.push(mesh);
        self.total_vertices = total_vertices;
        Ok(id)
    }

    pub fn add_draw(
        &mut self,
        draw: ExperimentalLocalGeometryDraw,
    ) -> Result<(), ExperimentalSubmissionLocalGeometryError> {
        self.resolve_slot(draw.geometry)?;
        if self.draws.len() == MAX_EXPERIMENTAL_LOCAL_DRAWS {
            return Err(
                ExperimentalSubmissionLocalGeometryError::DrawCapacityExceeded {
                    limit: MAX_EXPERIMENTAL_LOCAL_DRAWS,
                },
            );
        }
        self.draws.push(draw);
        Ok(())
    }

    pub fn finish(
        self,
    ) -> Result<ExperimentalSubmissionLocalGeometry, ExperimentalSubmissionLocalGeometryError> {
        let referenced = self
            .draws
            .iter()
            .map(|draw| draw.geometry.slot)
            .collect::<BTreeSet<_>>();
        if let Some(slot) = (0..self.payloads.len() as u32).find(|slot| !referenced.contains(slot))
        {
            return Err(ExperimentalSubmissionLocalGeometryError::UnreferencedGeometry { slot });
        }
        Ok(ExperimentalSubmissionLocalGeometry {
            identity: self.identity,
            payloads: self.payloads,
            draws: self.draws,
            total_vertices: self.total_vertices,
        })
    }

    fn resolve_slot(
        &self,
        id: ExperimentalLocalGeometryId,
    ) -> Result<usize, ExperimentalSubmissionLocalGeometryError> {
        if id.submission != self.identity {
            return Err(
                ExperimentalSubmissionLocalGeometryError::ForeignSubmission {
                    expected: self.identity,
                    observed: id.submission,
                },
            );
        }
        let slot = id.slot as usize;
        if slot >= self.payloads.len() {
            return Err(ExperimentalSubmissionLocalGeometryError::MissingGeometry {
                slot: id.slot,
            });
        }
        Ok(slot)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExperimentalSubmissionLocalGeometryObservation {
    pub payloads: u32,
    pub draws: u32,
    pub vertices: u32,
    pub persistent_mesh_identities_created: u32,
    pub persistent_mesh_replacements: u32,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ExperimentalSubmissionLocalGeometryError {
    #[error("submission-local geometry has {vertices} vertices; triangle lists require a non-zero multiple of three")]
    InvalidTriangleList { vertices: usize },
    #[error("submission-local geometry has {positions} positions but {normals} normals")]
    NormalCountMismatch { positions: usize, normals: usize },
    #[error("submission-local geometry has {positions} positions but {texture_coordinates} texture coordinates")]
    TextureCoordinateCountMismatch {
        positions: usize,
        texture_coordinates: usize,
    },
    #[error("submission-local geometry vertex {vertex} contains a non-finite value")]
    NonFiniteVertex { vertex: usize },
    #[error("submission-local geometry payload capacity {limit} was exceeded")]
    PayloadCapacityExceeded { limit: usize },
    #[error("submission-local geometry draw capacity {limit} was exceeded")]
    DrawCapacityExceeded { limit: usize },
    #[error("submission-local geometry vertex capacity {limit} was exceeded")]
    VertexCapacityExceeded { limit: usize },
    #[error("submission {expected:?} cannot resolve geometry from {observed:?}")]
    ForeignSubmission {
        expected: ExperimentalSubmissionIdentity,
        observed: ExperimentalSubmissionIdentity,
    },
    #[error("submission-local geometry slot {slot} does not exist")]
    MissingGeometry { slot: u32 },
    #[error("submission-local geometry slot {slot} is never drawn")]
    UnreferencedGeometry { slot: u32 },
    #[error("material handle {0} has not been uploaded")]
    MissingMaterial(u64),
    #[error("pipeline handle {0} has not been uploaded")]
    MissingPipeline(u64),
}

fn validate_mesh(mesh: &Mesh) -> Result<(), ExperimentalSubmissionLocalGeometryError> {
    if mesh.positions.is_empty() || !mesh.positions.len().is_multiple_of(3) {
        return Err(
            ExperimentalSubmissionLocalGeometryError::InvalidTriangleList {
                vertices: mesh.positions.len(),
            },
        );
    }
    if mesh.positions.len() != mesh.normals.len() {
        return Err(
            ExperimentalSubmissionLocalGeometryError::NormalCountMismatch {
                positions: mesh.positions.len(),
                normals: mesh.normals.len(),
            },
        );
    }
    for (vertex, (position, normal)) in mesh.positions.iter().zip(&mesh.normals).enumerate() {
        let uv = mesh
            .texture_coordinates
            .get(vertex)
            .copied()
            .unwrap_or([0.0, 0.0]);
        if position
            .iter()
            .chain(normal)
            .chain(&uv)
            .any(|value| !value.is_finite())
        {
            return Err(ExperimentalSubmissionLocalGeometryError::NonFiniteVertex { vertex });
        }
    }
    if !mesh.texture_coordinates.is_empty()
        && mesh.texture_coordinates.len() != mesh.positions.len()
    {
        return Err(
            ExperimentalSubmissionLocalGeometryError::TextureCoordinateCountMismatch {
                positions: mesh.positions.len(),
                texture_coordinates: mesh.texture_coordinates.len(),
            },
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draw(geometry: ExperimentalLocalGeometryId, material: u64) -> ExperimentalLocalGeometryDraw {
        ExperimentalLocalGeometryDraw {
            geometry,
            material: MaterialHandle(material),
            pipeline: PipelineHandle(1),
            instance: Instance2d::default(),
            camera: None,
            viewport: None,
            material_override: None,
        }
    }

    #[test]
    fn local_slots_are_scoped_to_one_submission() {
        let mut first =
            ExperimentalSubmissionLocalGeometryBuilder::new(ExperimentalSubmissionIdentity(41));
        let first_id = first.add_geometry(Mesh::triangle()).unwrap();
        first.add_draw(draw(first_id, 1)).unwrap();

        let mut second =
            ExperimentalSubmissionLocalGeometryBuilder::new(ExperimentalSubmissionIdentity(42));
        let second_id = second.add_geometry(Mesh::triangle()).unwrap();
        assert_eq!(first_id.slot, second_id.slot);
        assert_eq!(
            second.add_draw(draw(first_id, 1)),
            Err(
                ExperimentalSubmissionLocalGeometryError::ForeignSubmission {
                    expected: ExperimentalSubmissionIdentity(42),
                    observed: ExperimentalSubmissionIdentity(41),
                }
            )
        );
    }

    #[test]
    fn rejection_does_not_produce_a_partial_submission() {
        let mut builder =
            ExperimentalSubmissionLocalGeometryBuilder::new(ExperimentalSubmissionIdentity(7));
        assert_eq!(
            builder.add_geometry(Mesh::new(vec![[0.0; 3]; 2], vec![[0.0; 3]; 2])),
            Err(ExperimentalSubmissionLocalGeometryError::InvalidTriangleList { vertices: 2 })
        );
        let id = builder.add_geometry(Mesh::triangle()).unwrap();
        builder.add_draw(draw(id, 1)).unwrap();
        assert_eq!(builder.finish().unwrap().payloads().len(), 1);
    }

    #[test]
    fn malformed_optional_uv_stream_is_rejected_explicitly() {
        let mut mesh = Mesh::triangle();
        mesh.texture_coordinates = vec![[0.0, 0.0]];
        let mut builder =
            ExperimentalSubmissionLocalGeometryBuilder::new(ExperimentalSubmissionIdentity(8));
        assert_eq!(
            builder.add_geometry(mesh),
            Err(
                ExperimentalSubmissionLocalGeometryError::TextureCoordinateCountMismatch {
                    positions: 3,
                    texture_coordinates: 1,
                }
            )
        );
    }
}
