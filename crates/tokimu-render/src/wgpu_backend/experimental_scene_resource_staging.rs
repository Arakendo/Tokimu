//! Corpus-only real-provider staging experiment.
//!
//! This module deliberately does not define stable generations, handles,
//! reclamation, or a general renderer lifecycle contract. It tests whether a
//! complete successor resource set can be allocated on the current WGPU
//! provider session before replacing the live set.

use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    Camera, CameraHandle, Color, ExperimentalRenderCommandSet, Material, MaterialHandle, Mesh,
    MeshHandle, Pipeline, PipelineHandle, RenderCommand, Renderer, Rgba8TextureDescriptor,
    TextureHandle,
};

use super::{ExperimentalSceneStageContext, PipelineRegistry, WgpuBackend, WgpuBackendError};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExperimentalSceneResourceStageObservation {
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

#[doc(hidden)]
pub struct ExperimentalSceneResourceStage {
    backend: WgpuBackend,
    clear_color: Color,
}

impl ExperimentalSceneResourceStage {
    pub fn scope_render_commands(
        &self,
        commands: &[RenderCommand],
    ) -> ExperimentalRenderCommandSet {
        ExperimentalRenderCommandSet::new(
            Arc::clone(&self.backend.experimental_resource_set_authority),
            self.backend.experimental_current_resource_set,
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
    #[doc(hidden)]
    pub fn experimental_scope_render_commands(
        &self,
        commands: &[RenderCommand],
    ) -> ExperimentalRenderCommandSet {
        ExperimentalRenderCommandSet::new(
            Arc::clone(&self.experimental_resource_set_authority),
            self.experimental_current_resource_set,
            commands,
        )
    }

    #[doc(hidden)]
    pub fn experimental_submit_render_command_set(
        &mut self,
        command_set: &ExperimentalRenderCommandSet,
    ) -> Result<(), WgpuBackendError> {
        command_set.validate_for(
            &self.experimental_resource_set_authority,
            self.experimental_current_resource_set,
        )?;
        self.submit(command_set.commands());
        Ok(())
    }

    #[doc(hidden)]
    pub const fn experimental_current_resource_set(
        &self,
    ) -> crate::ExperimentalRenderResourceSetId {
        self.experimental_current_resource_set
    }

    #[doc(hidden)]
    pub fn experimental_begin_scene_resource_stage(
        &self,
    ) -> Result<ExperimentalSceneResourceStage, WgpuBackendError> {
        let surface = self
            .surface_state
            .as_ref()
            .ok_or(WgpuBackendError::ExperimentalSceneStageRequiresSurface)?;
        let stage_context = ExperimentalSceneStageContext {
            surface_format: surface.config.format,
            camera_bind_group_layout: surface.camera_bind_group_layout.clone(),
            material_bind_group_layout: surface.material_bind_group_layout.clone(),
            instance_bind_group_layout: surface.instance_bind_group_layout.clone(),
        };
        let candidate_resource_set = self.experimental_resource_set_authority.allocate_id()?;
        Ok(ExperimentalSceneResourceStage {
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
                experimental_stage_context: Some(stage_context),
                experimental_resource_set_authority: Arc::clone(
                    &self.experimental_resource_set_authority,
                ),
                experimental_current_resource_set: candidate_resource_set,
            },
        })
    }

    #[doc(hidden)]
    pub fn experimental_commit_scene_resource_stage(
        &mut self,
        mut stage: ExperimentalSceneResourceStage,
    ) -> Result<ExperimentalSceneResourceStageObservation, WgpuBackendError> {
        if !Arc::ptr_eq(
            &self.experimental_resource_set_authority,
            &stage.backend.experimental_resource_set_authority,
        ) {
            return Err(WgpuBackendError::ExperimentalSceneStageWrongProviderSession);
        }
        stage.validate()?;

        let observation = ExperimentalSceneResourceStageObservation {
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
        self.experimental_current_resource_set = stage.backend.experimental_current_resource_set;
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
