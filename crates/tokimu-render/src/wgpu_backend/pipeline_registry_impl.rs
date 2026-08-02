use crate::{Pipeline, PipelineHandle, PipelineKind};

use super::{
    pipeline_support::{create_custom_pipeline, create_solid_color_pipeline},
    WgpuBackend, WgpuBackendError, DEPTH_FORMAT,
};

impl WgpuBackend {
    pub fn upload_pipeline(
        &mut self,
        handle: PipelineHandle,
        pipeline: &Pipeline,
    ) -> Result<(), WgpuBackendError> {
        if let Err(error) = pipeline.validate() {
            self.record_backend_diagnostic(format!(
                "pipeline `{}` declaration was rejected before backend compilation: {error}",
                pipeline.label
            ));
            return Err(error.into());
        }
        let Some(surface_state) = self.surface_state.as_ref() else {
            return Ok(());
        };
        let shader_label = pipeline.backend_shader_label();

        let compiled = match pipeline.kind {
            PipelineKind::SolidColor2d => create_solid_color_pipeline(
                &self._device,
                surface_state.config.format,
                DEPTH_FORMAT,
                &surface_state.material_bind_group_layout,
                &surface_state.instance_bind_group_layout,
                &surface_state.camera_bind_group_layout,
                pipeline.render_state,
            ),
            PipelineKind::Texture2d => create_custom_pipeline(
                &self._device,
                surface_state.config.format,
                DEPTH_FORMAT,
                &surface_state.material_bind_group_layout,
                &surface_state.instance_bind_group_layout,
                &surface_state.camera_bind_group_layout,
                &pipeline.label,
                &shader_label,
                pipeline
                    .shader_source
                    .as_deref()
                    .or_else(|| pipeline.kind.default_shader_source())
                    .unwrap(),
                &pipeline.vertex_entry_point,
                &pipeline.fragment_entry_point,
                pipeline.render_state,
            ),
            PipelineKind::LitColor3d => create_custom_pipeline(
                &self._device,
                surface_state.config.format,
                DEPTH_FORMAT,
                &surface_state.material_bind_group_layout,
                &surface_state.instance_bind_group_layout,
                &surface_state.camera_bind_group_layout,
                &pipeline.label,
                &shader_label,
                pipeline
                    .shader_source
                    .as_deref()
                    .or_else(|| pipeline.kind.default_shader_source())
                    .unwrap_or(Pipeline::default_2d_shader_source()),
                &pipeline.vertex_entry_point,
                &pipeline.fragment_entry_point,
                pipeline.render_state,
            ),
            PipelineKind::CustomWgsl2d => create_custom_pipeline(
                &self._device,
                surface_state.config.format,
                DEPTH_FORMAT,
                &surface_state.material_bind_group_layout,
                &surface_state.instance_bind_group_layout,
                &surface_state.camera_bind_group_layout,
                &pipeline.label,
                &shader_label,
                pipeline
                    .shader_source
                    .as_deref()
                    .expect("validated custom WGSL pipelines always contain shader source"),
                &pipeline.vertex_entry_point,
                &pipeline.fragment_entry_point,
                pipeline.render_state,
            ),
        };

        let replaced_existing = self.pipelines.contains_key(&handle);
        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipelines.insert(handle, compiled);
        self.stats.record_pipeline_creation(replaced_existing);
        Ok(())
    }

    pub fn register_pipeline(
        &mut self,
        pipeline: &Pipeline,
    ) -> Result<PipelineHandle, WgpuBackendError> {
        if let Err(error) = pipeline.validate() {
            self.record_backend_diagnostic(format!(
                "pipeline `{}` declaration was rejected before backend compilation: {error}",
                pipeline.label
            ));
            return Err(error.into());
        }
        let handle = self.pipeline_registry.register(pipeline);
        self.upload_pipeline(handle, pipeline)?;
        Ok(handle)
    }

    pub fn pipeline_handle(&self, label: &str) -> Option<PipelineHandle> {
        self.pipeline_registry.handle_for_label(label)
    }
}
