use crate::{Color, RenderCommand};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderFrameCpuTimings {
    /// CPU wall time spent acquiring the current surface texture.
    pub surface_acquire_call: Option<Duration>,
    /// CPU wall time spent preparing renderer-owned resources for queued draws.
    pub resource_preparation: Option<Duration>,
    /// CPU wall time spent encoding and finishing the frame command buffer.
    pub command_encoding: Option<Duration>,
    /// CPU wall time spent inside the queue submission call.
    pub queue_submit_call: Option<Duration>,
    /// CPU wall time spent inside the surface presentation call.
    pub surface_present_call: Option<Duration>,
    /// CPU wall time spent encoding renderer-owned offscreen target passes.
    ///
    /// This is provider-observable CPU work only. It does not measure GPU
    /// execution or completion of the target pass.
    pub render_target_command_encoding: Option<Duration>,
    /// CPU wall time spent submitting renderer-owned offscreen target passes.
    ///
    /// This is provider-observable CPU work only. It does not measure GPU
    /// execution or completion of the target pass.
    pub render_target_queue_submit_call: Option<Duration>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderFrameStats {
    /// Draw commands accepted between the latest `begin_frame` and `end_frame`.
    pub draw_calls: u32,
    /// Command slices submitted between the latest `begin_frame` and `end_frame`.
    pub submit_calls: u32,
    /// GPU uniform bindings allocated between the latest frame boundaries.
    pub binding_allocations: u32,
    /// Existing GPU uniform buffers updated between the latest frame boundaries.
    pub uniform_buffer_writes: u32,
    /// Material bindings resolved for accepted draws between the latest frame
    /// boundaries. This is independent of GPU allocation.
    pub material_resolutions: u32,
    /// Pipeline state selections required by the latest frame. The first
    /// pipeline bound in a render pass counts as a selection.
    pub pipeline_switches: u32,
    /// Backend pipeline objects created between the latest frame boundaries.
    ///
    /// This is deliberately distinct from `pipeline_switches`: selecting an
    /// existing pipeline for a draw must not imply recompiling one.
    pub pipeline_creations: u32,
    /// Pipeline creations that replaced an existing registered handle during
    /// the latest frame.
    pub pipeline_replacements: u32,
    /// Draws whose resolved material color has non-opaque alpha.
    ///
    /// This is measured after per-draw presentation overrides are applied. It
    /// reports semantic transparency pressure, not whether a backend selected
    /// a particular blend implementation.
    pub transparent_draws: u32,
    /// Reused derived material bindings for transient per-draw overrides.
    pub derived_material_cache_hits: u32,
    /// New derived material bindings created for transient per-draw overrides.
    pub derived_material_cache_misses: u32,
    /// Mesh uploads performed between the latest frame boundaries.
    pub mesh_uploads: u32,
    /// Mesh replacements performed between the latest frame boundaries.
    pub mesh_replacements: u32,
    /// Backend texture allocations performed between the latest frame boundaries.
    pub texture_allocations: u32,
    /// Texture allocations that replaced an existing handle during the latest frame.
    pub texture_replacements: u32,
    /// Complete texture payload writes performed during the latest frame.
    pub texture_writes: u32,
    /// Provider-observable CPU phases for the latest frame.
    ///
    /// `None` means the provider cannot observe that phase. These durations do
    /// not measure GPU execution or completion.
    pub cpu_timings: RenderFrameCpuTimings,
}

impl RenderFrameStats {
    pub const EMPTY: Self = Self {
        draw_calls: 0,
        submit_calls: 0,
        binding_allocations: 0,
        uniform_buffer_writes: 0,
        material_resolutions: 0,
        pipeline_switches: 0,
        pipeline_creations: 0,
        pipeline_replacements: 0,
        transparent_draws: 0,
        derived_material_cache_hits: 0,
        derived_material_cache_misses: 0,
        mesh_uploads: 0,
        mesh_replacements: 0,
        texture_allocations: 0,
        texture_replacements: 0,
        texture_writes: 0,
        cpu_timings: RenderFrameCpuTimings {
            surface_acquire_call: None,
            resource_preparation: None,
            command_encoding: None,
            queue_submit_call: None,
            surface_present_call: None,
            render_target_command_encoding: None,
            render_target_queue_submit_call: None,
        },
    };
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderLifetimeStats {
    /// GPU uniform bindings allocated since the renderer was created.
    pub binding_allocations: u64,
    /// Existing GPU uniform buffers updated since the renderer was created.
    pub uniform_buffer_writes: u64,
    /// Material bindings resolved since renderer creation.
    pub material_resolutions: u64,
    /// Pipeline state selections since renderer creation.
    pub pipeline_switches: u64,
    /// Backend pipeline objects created since renderer creation.
    pub pipeline_creations: u64,
    /// Pipeline creations that replaced an existing registered handle since
    /// renderer creation.
    pub pipeline_replacements: u64,
    /// Draws with a resolved non-opaque material color since renderer creation.
    pub transparent_draws: u64,
    /// Reused derived material bindings since renderer creation.
    pub derived_material_cache_hits: u64,
    /// New derived material bindings created since renderer creation.
    pub derived_material_cache_misses: u64,
    /// Mesh uploads performed since the renderer was created.
    pub mesh_uploads: u64,
    /// Uploads that replaced an existing mesh handle since renderer creation.
    pub mesh_replacements: u64,
    /// Backend texture allocations since renderer creation.
    pub texture_allocations: u64,
    /// Texture allocations that replaced an existing handle since renderer creation.
    pub texture_replacements: u64,
    /// Complete texture payload writes since renderer creation.
    pub texture_writes: u64,
}

impl RenderLifetimeStats {
    pub const EMPTY: Self = Self {
        binding_allocations: 0,
        uniform_buffer_writes: 0,
        material_resolutions: 0,
        pipeline_switches: 0,
        pipeline_creations: 0,
        pipeline_replacements: 0,
        transparent_draws: 0,
        derived_material_cache_hits: 0,
        derived_material_cache_misses: 0,
        mesh_uploads: 0,
        mesh_replacements: 0,
        texture_allocations: 0,
        texture_replacements: 0,
        texture_writes: 0,
    };
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderStats {
    /// Work observed between the latest `begin_frame` and `end_frame`.
    pub frame: RenderFrameStats,
    /// Resource churn observed since this renderer instance was created.
    pub lifetime: RenderLifetimeStats,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RenderStatsTracker {
    frame: RenderFrameStats,
    lifetime: RenderLifetimeStats,
}

impl RenderStatsTracker {
    pub(crate) fn begin_frame(&mut self) {
        self.frame = RenderFrameStats::default();
    }

    pub(crate) fn record_draw_calls(&mut self, count: u32) {
        self.frame.draw_calls = self.frame.draw_calls.saturating_add(count);
    }

    pub(crate) fn record_submit_call(&mut self) {
        self.frame.submit_calls = self.frame.submit_calls.saturating_add(1);
    }

    pub(crate) fn record_binding_allocation(&mut self) {
        self.frame.binding_allocations = self.frame.binding_allocations.saturating_add(1);
        self.lifetime.binding_allocations = self.lifetime.binding_allocations.saturating_add(1);
    }

    pub(crate) fn record_uniform_buffer_write(&mut self) {
        self.frame.uniform_buffer_writes = self.frame.uniform_buffer_writes.saturating_add(1);
        self.lifetime.uniform_buffer_writes = self.lifetime.uniform_buffer_writes.saturating_add(1);
    }

    pub(crate) fn record_material_resolution(&mut self) {
        self.frame.material_resolutions = self.frame.material_resolutions.saturating_add(1);
        self.lifetime.material_resolutions = self.lifetime.material_resolutions.saturating_add(1);
    }

    pub(crate) fn record_pipeline_switch(&mut self) {
        self.frame.pipeline_switches = self.frame.pipeline_switches.saturating_add(1);
        self.lifetime.pipeline_switches = self.lifetime.pipeline_switches.saturating_add(1);
    }

    pub(crate) fn record_pipeline_creation(&mut self, replaced_existing: bool) {
        self.frame.pipeline_creations = self.frame.pipeline_creations.saturating_add(1);
        self.lifetime.pipeline_creations = self.lifetime.pipeline_creations.saturating_add(1);
        if replaced_existing {
            self.frame.pipeline_replacements = self.frame.pipeline_replacements.saturating_add(1);
            self.lifetime.pipeline_replacements =
                self.lifetime.pipeline_replacements.saturating_add(1);
        }
    }

    pub(crate) fn record_transparent_draw(&mut self) {
        self.frame.transparent_draws = self.frame.transparent_draws.saturating_add(1);
        self.lifetime.transparent_draws = self.lifetime.transparent_draws.saturating_add(1);
    }

    pub(crate) fn record_derived_material_cache_hit(&mut self) {
        self.frame.derived_material_cache_hits =
            self.frame.derived_material_cache_hits.saturating_add(1);
        self.lifetime.derived_material_cache_hits =
            self.lifetime.derived_material_cache_hits.saturating_add(1);
    }

    pub(crate) fn record_derived_material_cache_miss(&mut self) {
        self.frame.derived_material_cache_misses =
            self.frame.derived_material_cache_misses.saturating_add(1);
        self.lifetime.derived_material_cache_misses = self
            .lifetime
            .derived_material_cache_misses
            .saturating_add(1);
    }

    pub(crate) fn record_mesh_upload(&mut self, replaced_existing: bool) {
        self.frame.mesh_uploads = self.frame.mesh_uploads.saturating_add(1);
        self.lifetime.mesh_uploads = self.lifetime.mesh_uploads.saturating_add(1);
        if replaced_existing {
            self.frame.mesh_replacements = self.frame.mesh_replacements.saturating_add(1);
            self.lifetime.mesh_replacements = self.lifetime.mesh_replacements.saturating_add(1);
        }
    }

    pub(crate) fn record_texture_allocation(&mut self, replaced_existing: bool) {
        self.frame.texture_allocations = self.frame.texture_allocations.saturating_add(1);
        self.lifetime.texture_allocations = self.lifetime.texture_allocations.saturating_add(1);
        if replaced_existing {
            self.frame.texture_replacements = self.frame.texture_replacements.saturating_add(1);
            self.lifetime.texture_replacements =
                self.lifetime.texture_replacements.saturating_add(1);
        }
    }

    pub(crate) fn record_texture_write(&mut self) {
        self.frame.texture_writes = self.frame.texture_writes.saturating_add(1);
        self.lifetime.texture_writes = self.lifetime.texture_writes.saturating_add(1);
    }

    pub(crate) fn record_frame_cpu_timings(&mut self, timings: RenderFrameCpuTimings) {
        self.frame.cpu_timings.surface_acquire_call = timings.surface_acquire_call;
        self.frame.cpu_timings.resource_preparation = timings.resource_preparation;
        self.frame.cpu_timings.command_encoding = timings.command_encoding;
        self.frame.cpu_timings.queue_submit_call = timings.queue_submit_call;
        self.frame.cpu_timings.surface_present_call = timings.surface_present_call;
    }

    pub(crate) fn record_render_target_cpu_timings(
        &mut self,
        command_encoding: Duration,
        queue_submit_call: Duration,
    ) {
        self.frame.cpu_timings.render_target_command_encoding = Some(
            self.frame
                .cpu_timings
                .render_target_command_encoding
                .unwrap_or_default()
                .saturating_add(command_encoding),
        );
        self.frame.cpu_timings.render_target_queue_submit_call = Some(
            self.frame
                .cpu_timings
                .render_target_queue_submit_call
                .unwrap_or_default()
                .saturating_add(queue_submit_call),
        );
    }

    pub(crate) fn snapshot(&self) -> RenderStats {
        RenderStats {
            frame: self.frame,
            lifetime: self.lifetime,
        }
    }

    pub(crate) fn has_frame_draws(&self) -> bool {
        self.frame.draw_calls > 0
    }
}

pub trait Renderer {
    fn name(&self) -> &'static str;
    fn clear_color(&self) -> Color;
    fn begin_frame(&mut self);
    fn submit(&mut self, commands: &[RenderCommand]);
    fn end_frame(&mut self) -> RenderStats;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_boundary_resets_frame_counters_but_preserves_lifetime_counters() {
        let mut tracker = RenderStatsTracker::default();
        tracker.record_mesh_upload(false);
        tracker.record_binding_allocation();

        tracker.begin_frame();
        let frame = tracker.snapshot();

        assert_eq!(frame.frame, RenderFrameStats::default());
        assert_eq!(frame.lifetime.mesh_uploads, 1);
        assert_eq!(frame.lifetime.binding_allocations, 1);
    }

    #[test]
    fn frame_snapshot_reports_only_work_since_the_latest_boundary() {
        let mut tracker = RenderStatsTracker::default();
        tracker.begin_frame();
        tracker.record_draw_calls(4);
        tracker.record_submit_call();
        tracker.record_mesh_upload(true);
        tracker.record_texture_allocation(false);
        tracker.record_texture_write();
        tracker.record_texture_write();
        tracker.record_uniform_buffer_write();
        tracker.record_material_resolution();
        tracker.record_pipeline_switch();
        tracker.record_pipeline_creation(false);
        tracker.record_transparent_draw();
        tracker.record_derived_material_cache_hit();
        tracker.record_derived_material_cache_miss();
        tracker
            .record_render_target_cpu_timings(Duration::from_millis(3), Duration::from_millis(1));
        tracker.record_frame_cpu_timings(RenderFrameCpuTimings {
            command_encoding: Some(Duration::from_millis(2)),
            ..RenderFrameCpuTimings::default()
        });

        let first = tracker.snapshot();
        assert_eq!(first.frame.draw_calls, 4);
        assert_eq!(first.frame.submit_calls, 1);
        assert_eq!(first.frame.mesh_uploads, 1);
        assert_eq!(first.frame.mesh_replacements, 1);
        assert_eq!(first.frame.texture_allocations, 1);
        assert_eq!(first.frame.texture_replacements, 0);
        assert_eq!(first.frame.texture_writes, 2);
        assert_eq!(first.frame.material_resolutions, 1);
        assert_eq!(first.frame.pipeline_switches, 1);
        assert_eq!(first.frame.pipeline_creations, 1);
        assert_eq!(first.frame.pipeline_replacements, 0);
        assert_eq!(first.frame.transparent_draws, 1);
        assert_eq!(first.frame.derived_material_cache_hits, 1);
        assert_eq!(first.frame.derived_material_cache_misses, 1);
        assert_eq!(first.lifetime.mesh_uploads, 1);
        assert_eq!(first.lifetime.mesh_replacements, 1);
        assert_eq!(first.lifetime.texture_allocations, 1);
        assert_eq!(first.lifetime.texture_replacements, 0);
        assert_eq!(first.lifetime.texture_writes, 2);
        assert_eq!(first.lifetime.material_resolutions, 1);
        assert_eq!(first.lifetime.pipeline_creations, 1);
        assert_eq!(first.lifetime.pipeline_switches, 1);
        assert_eq!(first.lifetime.transparent_draws, 1);
        assert_eq!(
            first.frame.cpu_timings.command_encoding,
            Some(Duration::from_millis(2))
        );
        assert_eq!(
            first.frame.cpu_timings.render_target_command_encoding,
            Some(Duration::from_millis(3))
        );
        assert_eq!(
            first.frame.cpu_timings.render_target_queue_submit_call,
            Some(Duration::from_millis(1))
        );

        tracker.begin_frame();
        tracker.record_draw_calls(2);
        let second = tracker.snapshot();

        assert_eq!(second.frame.draw_calls, 2);
        assert_eq!(second.frame.submit_calls, 0);
        assert_eq!(second.frame.mesh_uploads, 0);
        assert_eq!(second.frame.mesh_replacements, 0);
        assert_eq!(second.frame.texture_allocations, 0);
        assert_eq!(second.frame.texture_replacements, 0);
        assert_eq!(second.frame.texture_writes, 0);
        assert_eq!(second.frame.material_resolutions, 0);
        assert_eq!(second.frame.pipeline_switches, 0);
        assert_eq!(second.frame.pipeline_creations, 0);
        assert_eq!(second.frame.pipeline_replacements, 0);
        assert_eq!(second.frame.transparent_draws, 0);
        assert_eq!(second.frame.derived_material_cache_hits, 0);
        assert_eq!(second.frame.derived_material_cache_misses, 0);
        assert_eq!(second.frame.cpu_timings, RenderFrameCpuTimings::default());
        assert_eq!(second.lifetime.mesh_uploads, 1);
        assert_eq!(second.lifetime.mesh_replacements, 1);
        assert_eq!(second.lifetime.texture_allocations, 1);
        assert_eq!(second.lifetime.texture_replacements, 0);
        assert_eq!(second.lifetime.texture_writes, 2);
        assert_eq!(second.lifetime.transparent_draws, 1);
        assert_eq!(second.lifetime.pipeline_creations, 1);
    }

    #[test]
    fn texture_replacement_is_distinct_from_steady_state_writes() {
        let mut tracker = RenderStatsTracker::default();
        tracker.begin_frame();
        tracker.record_texture_allocation(true);
        tracker.record_texture_write();

        let replacement = tracker.snapshot();
        assert_eq!(replacement.frame.texture_allocations, 1);
        assert_eq!(replacement.frame.texture_replacements, 1);
        assert_eq!(replacement.frame.texture_writes, 1);

        tracker.begin_frame();
        tracker.record_texture_write();
        let update = tracker.snapshot();
        assert_eq!(update.frame.texture_allocations, 0);
        assert_eq!(update.frame.texture_replacements, 0);
        assert_eq!(update.frame.texture_writes, 1);
        assert_eq!(update.lifetime.texture_allocations, 1);
        assert_eq!(update.lifetime.texture_replacements, 1);
        assert_eq!(update.lifetime.texture_writes, 2);
    }
}
