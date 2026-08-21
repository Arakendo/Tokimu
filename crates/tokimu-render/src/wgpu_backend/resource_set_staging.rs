//! WGPU realization of ADR-0018 atomic staged resource-set replacement.
//!
//! Provider allocation and reclamation remain WGPU-private. The stable
//! provider-neutral surface is [`crate::RenderResourceSetLifecycle`].

use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    Camera, CameraHandle, Color, Material, MaterialHandle, Mesh, MeshHandle, Pipeline,
    PipelineHandle, RenderCommand, RenderCommandSet, RenderResourceSetId,
    RenderResourceSetLifecycle, RenderStats, Renderer, Rgba8TextureDescriptor, TextureHandle,
};

use super::{PipelineRegistry, ResourceSetStageContext, WgpuBackend, WgpuBackendError};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WgpuResourceSetCommitObservation {
    pub retired_queued_draws: u32,
    pub retired_materials: u32,
    pub retired_textures: u32,
    pub retired_meshes: u32,
    pub retired_pipelines: u32,
    pub retired_cameras: u32,
    pub committed_queued_draws: u32,
    pub committed_materials: u32,
    pub committed_textures: u32,
    pub committed_meshes: u32,
    pub committed_pipelines: u32,
    pub committed_cameras: u32,
    pub retained_instance_bindings: u32,
}

pub struct WgpuResourceSetStage {
    backend: WgpuBackend,
    clear_color: Color,
}

/// Replacement-enabled WGPU session with authoritative set-scoped submission.
///
/// The wrapper intentionally does not implement [`Renderer`] or expose its
/// inner backend. Once a backend enters this lifecycle, ordinary unscoped
/// submission cannot bypass retired-set validation.
///
/// ```compile_fail
/// use tokimu_render::{RenderCommand, Renderer, WgpuResourceSetSession};
///
/// fn bypass(session: &mut WgpuResourceSetSession, commands: &[RenderCommand]) {
///     session.submit(commands);
/// }
/// ```
pub struct WgpuResourceSetSession {
    backend: WgpuBackend,
}

impl WgpuResourceSetSession {
    pub(super) fn backend(&self) -> &WgpuBackend {
        &self.backend
    }

    pub(super) fn backend_mut(&mut self) -> &mut WgpuBackend {
        &mut self.backend
    }

    pub fn begin_frame(&mut self) {
        self.backend.begin_frame();
    }

    pub fn present(&mut self) -> Result<RenderStats, WgpuBackendError> {
        self.backend.present()
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        self.backend.resize_surface(width, height);
    }

    pub fn adapter_name(&self) -> &str {
        self.backend.adapter_name()
    }

    pub fn backend_api(&self) -> &'static str {
        self.backend.backend_api()
    }

    pub fn device_kind(&self) -> &'static str {
        self.backend.device_kind()
    }

    pub fn drain_diagnostics(&self) -> Vec<tokimu_core::DiagnosticRecord> {
        self.backend.drain_diagnostics()
    }

    pub fn current_resource_set(&self) -> RenderResourceSetId {
        self.backend.current_resource_set
    }

    /// Replaces camera data inside the current authoritative set.
    ///
    /// This preserves live-view updates without reopening unscoped command
    /// submission. Scene replacement still goes through an isolated candidate
    /// and atomic commit.
    pub fn upload_camera(&mut self, handle: CameraHandle, camera: Camera) {
        self.backend.upload_camera(handle, camera);
    }
}

impl WgpuResourceSetStage {
    pub fn scope_render_commands(&self, commands: &[RenderCommand]) -> RenderCommandSet {
        RenderCommandSet::new(
            Arc::clone(&self.backend.resource_set_authority),
            self.backend.current_resource_set,
            commands,
        )
    }

    pub fn upload_mesh(&mut self, handle: MeshHandle, mesh: &Mesh) {
        self.backend.upload_mesh(handle, mesh);
    }

    pub fn create_texture_rgba8(
        &mut self,
        handle: TextureHandle,
        descriptor: Rgba8TextureDescriptor,
        rgba8: &[u8],
    ) -> Result<(), WgpuBackendError> {
        self.backend.create_texture_rgba8(handle, descriptor, rgba8)
    }

    pub fn upload_material(
        &mut self,
        handle: MaterialHandle,
        material: &Material,
    ) -> Result<(), WgpuBackendError> {
        self.backend.upload_material(handle, material)
    }

    pub fn register_pipeline(
        &mut self,
        pipeline: &Pipeline,
    ) -> Result<PipelineHandle, WgpuBackendError> {
        self.backend.register_pipeline(pipeline)
    }

    pub fn upload_camera(&mut self, handle: CameraHandle, camera: Camera) {
        self.backend.upload_camera(handle, camera);
    }

    pub fn set_active_camera(&mut self, handle: CameraHandle) {
        self.backend.set_active_camera(handle);
    }

    pub fn begin_frame(&mut self) {
        self.backend.begin_frame();
    }

    pub fn submit(&mut self, commands: &[RenderCommand]) {
        if let Some(clear) = commands.iter().find_map(|command| match command {
            RenderCommand::Clear(clear) => Some(clear.color),
            _ => None,
        }) {
            self.clear_color = clear;
        }
        self.backend.submit(commands);
    }

    pub fn validate(&self) -> Result<(), WgpuBackendError> {
        for draw in &self.backend.queued_draws {
            match draw.geometry {
                super::QueuedGeometry::Persistent(handle) => {
                    if !self.backend.meshes.contains_key(&handle) {
                        return Err(WgpuBackendError::MissingMesh(handle.0));
                    }
                }
                #[cfg(feature = "experimental-submission-local-geometry")]
                super::QueuedGeometry::SubmissionLocal(slot) => {
                    if self.backend.submission_local_meshes.get(slot).is_none() {
                        return Err(WgpuBackendError::MissingSubmissionLocalGeometry(slot));
                    }
                }
            }
            if !self.backend.materials.contains_key(&draw.material) {
                return Err(WgpuBackendError::MissingMaterial(draw.material.0));
            }
            if !self.backend.pipelines.contains_key(&draw.pipeline) {
                return Err(WgpuBackendError::MissingPipeline(draw.pipeline.0));
            }
        }
        Ok(())
    }
}

impl WgpuBackend {
    /// Consumes an ordinary backend and enters the replacement-enabled session.
    pub fn into_resource_set_session(self) -> WgpuResourceSetSession {
        WgpuResourceSetSession { backend: self }
    }

    fn scope_resource_set_commands(&self, commands: &[RenderCommand]) -> RenderCommandSet {
        RenderCommandSet::new(
            Arc::clone(&self.resource_set_authority),
            self.current_resource_set,
            commands,
        )
    }

    fn submit_resource_set_commands(
        &mut self,
        command_set: &RenderCommandSet,
    ) -> Result<(), WgpuBackendError> {
        command_set.validate_for(&self.resource_set_authority, self.current_resource_set)?;
        self.submit(command_set.commands());
        Ok(())
    }

    fn begin_resource_set_stage(&self) -> Result<WgpuResourceSetStage, WgpuBackendError> {
        let surface = self
            .surface_state
            .as_ref()
            .ok_or(WgpuBackendError::ResourceSetStageRequiresSurface)?;
        let stage_context = ResourceSetStageContext {
            surface_format: surface.config.format,
            camera_bind_group_layout: surface.camera_bind_group_layout.clone(),
            material_bind_group_layout: surface.material_bind_group_layout.clone(),
            instance_bind_group_layout: surface.instance_bind_group_layout.clone(),
        };
        let candidate_resource_set = self.resource_set_authority.allocate_id()?;
        Ok(WgpuResourceSetStage {
            clear_color: surface.clear_color,
            backend: WgpuBackend {
                stats: crate::renderer::RenderStatsTracker::default(),
                queued_draws: Vec::new(),
                instance_bindings: Vec::new(),
                camera_bindings: HashMap::new(),
                meshes: HashMap::new(),
                #[cfg(feature = "experimental-submission-local-geometry")]
                submission_local_meshes: Vec::new(),
                materials: HashMap::new(),
                derived_materials: HashMap::new(),
                pipelines: HashMap::new(),
                pipeline_registry: PipelineRegistry::new(),
                renderables: HashMap::new(),
                textures: HashMap::new(),
                cameras: HashMap::new(),
                active_camera: CameraHandle::default(),
                _instance: self._instance.clone(),
                _device: self._device.clone(),
                _queue: self._queue.clone(),
                adapter_info: self.adapter_info.clone(),
                surface_state: None,
                backend_diagnostic_messages: Arc::clone(&self.backend_diagnostic_messages),
                resource_set_stage_context: Some(stage_context),
                resource_set_authority: Arc::clone(&self.resource_set_authority),
                current_resource_set: candidate_resource_set,
            },
        })
    }

    fn commit_resource_set_stage(
        &mut self,
        mut stage: WgpuResourceSetStage,
    ) -> Result<WgpuResourceSetCommitObservation, WgpuBackendError> {
        if !Arc::ptr_eq(
            &self.resource_set_authority,
            &stage.backend.resource_set_authority,
        ) {
            return Err(WgpuBackendError::ResourceSetStageWrongProviderSession);
        }
        stage.validate()?;

        let observation = WgpuResourceSetCommitObservation {
            retired_queued_draws: self.queued_draws.len() as u32,
            retired_materials: self.materials.len() as u32,
            retired_textures: self.textures.len() as u32,
            retired_meshes: self.meshes.len() as u32,
            retired_pipelines: self.pipelines.len() as u32,
            retired_cameras: self.cameras.len() as u32,
            committed_queued_draws: stage.backend.queued_draws.len() as u32,
            committed_materials: stage.backend.materials.len() as u32,
            committed_textures: stage.backend.textures.len() as u32,
            committed_meshes: stage.backend.meshes.len() as u32,
            committed_pipelines: stage.backend.pipelines.len() as u32,
            committed_cameras: stage.backend.cameras.len() as u32,
            retained_instance_bindings: self.instance_bindings.len() as u32,
        };

        self.queued_draws = std::mem::take(&mut stage.backend.queued_draws);
        self.renderables = std::mem::take(&mut stage.backend.renderables);
        self.derived_materials.clear();
        self.materials = std::mem::take(&mut stage.backend.materials);
        self.textures = std::mem::take(&mut stage.backend.textures);
        self.meshes = std::mem::take(&mut stage.backend.meshes);
        self.pipelines = std::mem::take(&mut stage.backend.pipelines);
        self.pipeline_registry = std::mem::replace(
            &mut stage.backend.pipeline_registry,
            PipelineRegistry::new(),
        );
        self.cameras = std::mem::take(&mut stage.backend.cameras);
        self.active_camera = stage.backend.active_camera;
        self.current_resource_set = stage.backend.current_resource_set;
        self.camera_bindings.clear();
        #[cfg(feature = "experimental-submission-local-geometry")]
        {
            self.submission_local_meshes =
                std::mem::take(&mut stage.backend.submission_local_meshes);
        }
        if let Some(surface) = self.surface_state.as_mut() {
            surface.clear_color = stage.clear_color;
        }

        Ok(observation)
    }
}

impl RenderResourceSetLifecycle for WgpuResourceSetSession {
    type Candidate = WgpuResourceSetStage;
    type Error = WgpuBackendError;
    type CommitObservation = WgpuResourceSetCommitObservation;

    fn begin_resource_set_stage(&self) -> Result<Self::Candidate, Self::Error> {
        self.backend.begin_resource_set_stage()
    }

    fn commit_resource_set_stage(
        &mut self,
        candidate: Self::Candidate,
    ) -> Result<Self::CommitObservation, Self::Error> {
        self.backend.commit_resource_set_stage(candidate)
    }

    fn scope_render_commands(&self, commands: &[RenderCommand]) -> RenderCommandSet {
        self.backend.scope_resource_set_commands(commands)
    }

    fn submit_render_command_set(
        &mut self,
        command_set: &RenderCommandSet,
    ) -> Result<(), Self::Error> {
        self.backend.submit_resource_set_commands(command_set)
    }
}
