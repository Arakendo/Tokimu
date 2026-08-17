use crate::experimental_submission_local_geometry::{
    ExperimentalSubmissionLocalGeometry, ExperimentalSubmissionLocalGeometryError,
    ExperimentalSubmissionLocalGeometryObservation,
};

use super::mesh_resources::create_gpu_mesh;
use super::{QueuedDraw, QueuedGeometry, WgpuBackend};

impl WgpuBackend {
    /// Accepts one validated, immutable submission-local geometry batch.
    ///
    /// This corpus-only method is deliberately absent from [`crate::Renderer`].
    /// Geometry is uploaded into a backend-private arena whose contents are
    /// discarded by the next `begin_frame`; no [`crate::MeshHandle`] is
    /// allocated or replaced.
    pub fn submit_experimental_submission_local_geometry(
        &mut self,
        submission: &ExperimentalSubmissionLocalGeometry,
    ) -> Result<
        ExperimentalSubmissionLocalGeometryObservation,
        ExperimentalSubmissionLocalGeometryError,
    > {
        // Validate every durable dependency before changing either the local
        // arena or draw queue. Rejection therefore cannot leave a partial batch.
        for draw in submission.draws() {
            if !self.materials.contains_key(&draw.material) {
                return Err(ExperimentalSubmissionLocalGeometryError::MissingMaterial(
                    draw.material.0,
                ));
            }
            if !self.pipelines.contains_key(&draw.pipeline) {
                return Err(ExperimentalSubmissionLocalGeometryError::MissingPipeline(
                    draw.pipeline.0,
                ));
            }
        }

        let base_slot = self.submission_local_meshes.len();
        let meshes = submission
            .payloads()
            .iter()
            .map(|mesh| {
                create_gpu_mesh(
                    &self._device,
                    mesh,
                    "tokimu-experimental-submission-local-vertex-buffer",
                )
            })
            .collect::<Vec<_>>();
        let queued_draws = submission
            .draws()
            .iter()
            .map(|draw| QueuedDraw {
                geometry: QueuedGeometry::SubmissionLocal(
                    base_slot + draw.geometry_slot_for_backend(),
                ),
                material: draw.material,
                pipeline: draw.pipeline,
                instance: draw.instance,
                camera: draw.camera,
                viewport: draw.viewport,
                material_override: draw.material_override,
            })
            .collect::<Vec<_>>();

        self.submission_local_meshes.extend(meshes);
        self.queued_draws.extend(queued_draws);
        self.stats.record_submit_call();
        self.stats
            .record_draw_calls(submission.draws().len() as u32);

        Ok(ExperimentalSubmissionLocalGeometryObservation {
            payloads: submission.payloads().len() as u32,
            draws: submission.draws().len() as u32,
            vertices: submission.total_vertices() as u32,
            persistent_mesh_identities_created: 0,
            persistent_mesh_replacements: 0,
        })
    }
}
