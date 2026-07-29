use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use cgm_corpus::{
    inspect_binary_cgm_file, lower_picture_primitives, CgmInspection, CgmPrimitiveTopology,
    CgmVectorPrimitive, DecodeLimits, DelimiterElement,
};
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, Diagnostics, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow,
    PerformanceBudget, PerformanceMonitor, PerformanceUnit, Pipeline, PipelineHandle, PipelineKind,
    PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand, Renderer, WgpuBackend,
    WindowConfig,
};
use ui_tools::{layout_bitmap_text, tessellate_path_strokes, UiRect, UiTextRole, UiTextSpec};

const SOURCE: &str = "third-party/fixtures/webcgm-test-suite/upstream/static10/POLYLN01.cgm";
const QUAD_MESH: MeshHandle = MeshHandle(1);
const GLYPH_MESH: MeshHandle = MeshHandle(2);
const VECTOR_MESH: MeshHandle = MeshHandle(3);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const PANEL_MATERIAL: MaterialHandle = MaterialHandle(1);
const TEXT_MATERIAL: MaterialHandle = MaterialHandle(2);
const MUTED_TEXT_MATERIAL: MaterialHandle = MaterialHandle(3);
const ACCENT_MATERIAL: MaterialHandle = MaterialHandle(4);
const VECTOR_PANEL_MATERIAL: MaterialHandle = MaterialHandle(5);
const VECTOR_MATERIAL: MaterialHandle = MaterialHandle(6);
const CLASS_MATERIAL_BASE: u64 = 10;
const DIAGNOSTIC_STROKE_WIDTH: f32 = 0.012;

const CLASS_COLORS: [Color; 9] = [
    Color::rgb(0.45, 0.68, 0.92),
    Color::rgb(0.38, 0.77, 0.68),
    Color::rgb(0.92, 0.66, 0.36),
    Color::rgb(0.82, 0.49, 0.58),
    Color::rgb(0.62, 0.55, 0.88),
    Color::rgb(0.45, 0.72, 0.84),
    Color::rgb(0.78, 0.72, 0.42),
    Color::rgb(0.53, 0.61, 0.72),
    Color::rgb(0.72, 0.48, 0.78),
];

/// Fits a diagnostic mesh into a unit square without modifying source or
/// provider-neutral vector evidence. This is viewport framing only.
fn fit_diagnostic_mesh_to_unit_box(mut positions: Vec<[f32; 3]>) -> Vec<[f32; 3]> {
    let Some(first) = positions.first().copied() else {
        return positions;
    };
    let (mut minimum, mut maximum) = ([first[0], first[1]], [first[0], first[1]]);
    for position in &positions {
        minimum[0] = minimum[0].min(position[0]);
        minimum[1] = minimum[1].min(position[1]);
        maximum[0] = maximum[0].max(position[0]);
        maximum[1] = maximum[1].max(position[1]);
    }

    let span = (maximum[0] - minimum[0])
        .max(maximum[1] - minimum[1])
        .max(f32::EPSILON);
    let center = [
        (minimum[0] + maximum[0]) * 0.5,
        (minimum[1] + maximum[1]) * 0.5,
    ];
    for position in &mut positions {
        position[0] = (position[0] - center[0]) / span;
        position[1] = (position[1] - center[1]) / span;
    }
    positions
}

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello CGM".into(),
            width: 1120,
            height: 760,
        },
        HelloCgmApp::default(),
    )
}

struct HelloCgmApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    inspection: Option<CgmInspection>,
    lowered_primitives: Vec<CgmVectorPrimitive>,
    frame_index: u64,
    last_performance_report: Instant,
    diagnostics: Diagnostics,
    frame_time_monitor: PerformanceMonitor,
    presentation_time_monitor: PerformanceMonitor,
    present_time_monitor: PerformanceMonitor,
    presentation_rebuild_monitor: PerformanceMonitor,
    presentation_revision: u64,
    last_built_presentation_revision: Option<u64>,
    unchanged_presentation_rebuilds: u64,
}

impl Default for HelloCgmApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            inspection: None,
            lowered_primitives: Vec::new(),
            frame_index: 0,
            last_performance_report: Instant::now(),
            diagnostics: Diagnostics::default(),
            frame_time_monitor: PerformanceMonitor::new(
                PerformanceBudget::new(
                    "hello-cgm",
                    "platform-reported frame interval",
                    25.0,
                    PerformanceUnit::Milliseconds,
                )
                .with_required_consecutive_violations(3),
            ),
            presentation_time_monitor: PerformanceMonitor::new(
                PerformanceBudget::new(
                    "hello-cgm.presentation",
                    "presentation command construction CPU duration",
                    4.0,
                    PerformanceUnit::Milliseconds,
                )
                .with_required_consecutive_violations(3),
            ),
            present_time_monitor: PerformanceMonitor::new(
                PerformanceBudget::new(
                    "hello-cgm.renderer",
                    "renderer present call CPU wall duration",
                    16.0,
                    PerformanceUnit::Milliseconds,
                )
                .with_required_consecutive_violations(3),
            ),
            presentation_rebuild_monitor: PerformanceMonitor::new(
                PerformanceBudget::new(
                    "hello-cgm.presentation",
                    "presentation rebuilds without semantic revision",
                    0.0,
                    PerformanceUnit::Count,
                )
                .with_required_consecutive_violations(3),
            ),
            presentation_revision: 0,
            last_built_presentation_revision: None,
            unchanged_presentation_rebuilds: 0,
        }
    }
}

impl HelloCgmApp {
    fn record_presentation_build(&mut self) {
        if self.last_built_presentation_revision == Some(self.presentation_revision) {
            self.unchanged_presentation_rebuilds =
                self.unchanged_presentation_rebuilds.saturating_add(1);
        } else {
            self.unchanged_presentation_rebuilds = 0;
            self.last_built_presentation_revision = Some(self.presentation_revision);
        }
        self.presentation_rebuild_monitor.observe(
            self.unchanged_presentation_rebuilds as f64,
            &mut self.diagnostics,
        );
    }

    fn draw_quad(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        material: MaterialHandle,
        rect: UiRect,
    ) {
        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: QUAD_MESH,
            material,
            pipeline,
            instance: Instance2d::new(rect.center, rect.size, 0.0),
            camera: Some(CAMERA_HANDLE),
            viewport: None,
        })]);
    }

    fn draw_text(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        material: MaterialHandle,
        text: impl Into<String>,
        rect: UiRect,
        role: UiTextRole,
        height: f32,
    ) {
        let spec = UiTextSpec::new(text, rect, role);
        let commands = layout_bitmap_text(&spec, height)
            .into_iter()
            .map(|quad| {
                RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: GLYPH_MESH,
                    material,
                    pipeline,
                    instance: Instance2d::new(quad.center, quad.size, 0.0),
                    camera: Some(CAMERA_HANDLE),
                    viewport: None,
                })
            })
            .collect::<Vec<_>>();
        renderer.submit(&commands);
    }

    fn draw_inspection(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        inspection: &CgmInspection,
        lowered_primitives: &[CgmVectorPrimitive],
    ) {
        Self::draw_quad(
            renderer,
            pipeline,
            PANEL_MATERIAL,
            UiRect::new([0.0, 0.0], [1.82, 1.22]),
        );
        Self::draw_quad(
            renderer,
            pipeline,
            ACCENT_MATERIAL,
            UiRect::new([0.0, 0.52], [1.82, 0.008]),
        );

        Self::draw_text(
            renderer,
            pipeline,
            TEXT_MATERIAL,
            "CGM SOURCE + VECTOR INSPECTION",
            UiRect::new([0.0, 0.57], [1.7, 0.10]),
            UiTextRole::Title,
            0.054,
        );
        Self::draw_text(
            renderer,
            pipeline,
            ACCENT_MATERIAL,
            inspection.metafile_name.to_uppercase(),
            UiRect::new([0.0, 0.44], [1.4, 0.08]),
            UiTextRole::Heading,
            0.040,
        );

        let picture_names = inspection
            .pictures
            .iter()
            .map(|picture| picture.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let summary = format!(
            "{} BYTES  |  {} ELEMENTS  |  {} PICTURES  |  {} ATTRIBUTES  |  {} PRIMITIVES  |  {} DIAGNOSTICS",
            inspection.source_bytes,
            inspection.elements.len(),
            inspection.pictures.len(),
            inspection
                .pictures
                .iter()
                .map(|picture| picture.attributes.len())
                .sum::<usize>(),
            inspection
                .pictures
                .iter()
                .map(|picture| picture.primitives.len())
                .sum::<usize>(),
            inspection.diagnostics.len()
        );
        Self::draw_text(
            renderer,
            pipeline,
            MUTED_TEXT_MATERIAL,
            summary,
            UiRect::new([0.0, 0.35], [1.68, 0.07]),
            UiTextRole::Caption,
            0.024,
        );
        Self::draw_text(
            renderer,
            pipeline,
            MUTED_TEXT_MATERIAL,
            format!("PICTURE: {picture_names}"),
            UiRect::new([0.0, 0.29], [1.68, 0.06]),
            UiTextRole::Caption,
            0.022,
        );
        let coordinate_summary = inspection
            .pictures
            .first()
            .and_then(|picture| picture.descriptor.vdc_extent)
            .map_or_else(
                || "VDC: unresolved".to_owned(),
                |extent| {
                    format!(
                        "VDC: ({}, {}) -> ({}, {})  |  {:?} / {:?}",
                        extent.first[0],
                        extent.first[1],
                        extent.second[0],
                        extent.second[1],
                        inspection.pictures[0].descriptor.scaling_mode,
                        inspection.pictures[0].descriptor.color_selection_mode,
                    )
                },
            );
        Self::draw_text(
            renderer,
            pipeline,
            MUTED_TEXT_MATERIAL,
            coordinate_summary,
            UiRect::new([0.0, 0.23], [1.68, 0.05]),
            UiTextRole::Caption,
            0.018,
        );

        let mut class_counts = [0usize; 16];
        for element in &inspection.elements {
            class_counts[element.class as usize] += 1;
        }
        let maximum = class_counts.iter().copied().max().unwrap_or(1) as f32;
        let populated = class_counts
            .iter()
            .enumerate()
            .filter(|(_, count)| **count > 0)
            .collect::<Vec<_>>();
        let bar_width = 0.82 / populated.len().max(1) as f32;
        for (column, (class, count)) in populated.iter().enumerate() {
            let normalized = **count as f32 / maximum;
            let height = 0.05 + normalized * 0.25;
            let x = -0.81 + bar_width * (column as f32 + 0.5);
            Self::draw_quad(
                renderer,
                pipeline,
                MaterialHandle(CLASS_MATERIAL_BASE + (*class % CLASS_COLORS.len()) as u64),
                UiRect::new([x, 0.04 + height * 0.5], [bar_width * 0.62, height]),
            );
            Self::draw_text(
                renderer,
                pipeline,
                TEXT_MATERIAL,
                format!("C{} {}", class, count),
                UiRect::new([x, -0.12], [bar_width * 0.95, 0.05]),
                UiTextRole::Caption,
                0.017,
            );
        }

        Self::draw_text(
            renderer,
            pipeline,
            MUTED_TEXT_MATERIAL,
            "SOURCE-ORDERED ELEMENT MAP",
            UiRect::new([-0.39, -0.20], [0.8, 0.05]),
            UiTextRole::Caption,
            0.020,
        );

        let columns = 26usize;
        let visible = inspection.elements.len().min(columns * 4);
        let cell_width = 0.82 / columns as f32;
        for (index, element) in inspection.elements.iter().take(visible).enumerate() {
            let column = index % columns;
            let row = index / columns;
            let x = -0.81 + cell_width * (column as f32 + 0.5);
            let y = -0.27 - row as f32 * 0.045;
            let material = if element.delimiter.is_some() {
                ACCENT_MATERIAL
            } else {
                MaterialHandle(
                    CLASS_MATERIAL_BASE + (element.class as usize % CLASS_COLORS.len()) as u64,
                )
            };
            Self::draw_quad(
                renderer,
                pipeline,
                material,
                UiRect::new([x, y], [cell_width * 0.76, 0.028]),
            );
        }

        let lifecycle = inspection
            .elements
            .iter()
            .filter_map(|element| element.delimiter)
            .map(|delimiter| match delimiter {
                DelimiterElement::BeginMetafile => "BEGIN MF",
                DelimiterElement::EndMetafile => "END MF",
                DelimiterElement::BeginPicture => "BEGIN PIC",
                DelimiterElement::BeginPictureBody => "BODY",
                DelimiterElement::EndPicture => "END PIC",
            })
            .collect::<Vec<_>>()
            .join("  >  ");
        Self::draw_text(
            renderer,
            pipeline,
            TEXT_MATERIAL,
            lifecycle,
            UiRect::new([-0.39, -0.47], [0.8, 0.06]),
            UiTextRole::Caption,
            0.020,
        );
        Self::draw_text(
            renderer,
            pipeline,
            MUTED_TEXT_MATERIAL,
            "SOURCE STATE + VECTOR LOWERING COMPLETE",
            UiRect::new([-0.39, -0.55], [0.8, 0.05]),
            UiTextRole::Caption,
            0.017,
        );

        // This is a provider-neutral geometry viewport, not CGM paint output.
        // Draw its mesh before labels so the diagnostic annotations remain readable.
        Self::draw_quad(
            renderer,
            pipeline,
            VECTOR_PANEL_MATERIAL,
            UiRect::new([0.48, -0.16], [0.70, 0.58]),
        );
        if !lowered_primitives.is_empty() {
            renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: VECTOR_MESH,
                material: VECTOR_MATERIAL,
                pipeline,
                instance: Instance2d::new([0.48, -0.17], [0.54, 0.30], 0.0),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            })]);
        }

        let closed = lowered_primitives
            .iter()
            .filter(|primitive| primitive.topology == CgmPrimitiveTopology::Closed)
            .count();
        let contours = lowered_primitives
            .iter()
            .map(|primitive| primitive.path.contours.len())
            .sum::<usize>();
        let points = lowered_primitives
            .iter()
            .flat_map(|primitive| primitive.path.contours.iter())
            .map(|contour| contour.points.len())
            .sum::<usize>();
        Self::draw_text(
            renderer,
            pipeline,
            ACCENT_MATERIAL,
            "NORMALIZED VECTOR DIAGNOSTIC",
            UiRect::new([0.48, 0.08], [0.64, 0.06]),
            UiTextRole::Heading,
            0.030,
        );
        Self::draw_text(
            renderer,
            pipeline,
            MUTED_TEXT_MATERIAL,
            format!(
                "{} PRIMITIVES  |  {} OPEN  |  {} CLOSED  |  {} CONTOURS  |  {} POINTS",
                lowered_primitives.len(),
                lowered_primitives.len().saturating_sub(closed),
                closed,
                contours,
                points,
            ),
            UiRect::new([0.48, 0.01], [0.64, 0.05]),
            UiTextRole::Caption,
            0.016,
        );
        Self::draw_text(
            renderer,
            pipeline,
            MUTED_TEXT_MATERIAL,
            "NEUTRAL PATH OUTLINE ONLY: CGM PAINT, EDGE, AND CLIPPING SEMANTICS REMAIN DEFERRED",
            UiRect::new([0.48, -0.40], [0.64, 0.06]),
            UiTextRole::Caption,
            0.014,
        );
    }
}

impl PlatformEventHandler for HelloCgmApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];

        let inspection = inspect_binary_cgm_file(source_path(), DecodeLimits::default())?;
        let lowered_primitives = inspection
            .pictures
            .first()
            .map(lower_picture_primitives)
            .transpose()?
            .unwrap_or_default();
        window.set_title(&format!(
            "Tokimu Hello CGM | {} | elements={} | pictures={} | vectors={}",
            inspection.metafile_name,
            inspection.elements.len(),
            inspection.pictures.len(),
            lowered_primitives.len(),
        ));
        println!(
            "hello-cgm loaded {}: bytes={}, elements={}, pictures={}, vectors={}, diagnostics={}, padding={}",
            inspection.metafile_name,
            inspection.source_bytes,
            inspection.elements.len(),
            inspection.pictures.len(),
            lowered_primitives.len(),
            inspection.diagnostics.len(),
            inspection.trailing_padding_bytes
        );

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD_MESH, &Mesh::quad());
        renderer.upload_mesh(GLYPH_MESH, &Mesh::quad());
        let vector_paths = lowered_primitives
            .iter()
            .map(|primitive| primitive.path.clone())
            .collect::<Vec<_>>();
        renderer.upload_mesh(
            VECTOR_MESH,
            &Mesh::uniform_normal(
                fit_diagnostic_mesh_to_unit_box(tessellate_path_strokes(
                    &vector_paths,
                    DIAGNOSTIC_STROKE_WIDTH,
                )),
                [0.0, 0.0, 1.0],
            ),
        );
        renderer.upload_material(
            PANEL_MATERIAL,
            &Material::new("cgm-panel", Color::rgb(0.075, 0.09, 0.12)),
        )?;
        renderer.upload_material(
            TEXT_MATERIAL,
            &Material::new("cgm-text", Color::rgb(0.88, 0.92, 0.97)),
        )?;
        renderer.upload_material(
            MUTED_TEXT_MATERIAL,
            &Material::new("cgm-muted-text", Color::rgb(0.55, 0.62, 0.70)),
        )?;
        renderer.upload_material(
            ACCENT_MATERIAL,
            &Material::new("cgm-accent", Color::rgb(0.38, 0.68, 0.92)),
        )?;
        renderer.upload_material(
            VECTOR_MATERIAL,
            &Material::new("cgm-vector-diagnostic", Color::rgb(0.56, 0.86, 0.76)),
        )?;
        renderer.upload_material(
            VECTOR_PANEL_MATERIAL,
            &Material::new("cgm-vector-panel", Color::rgb(0.045, 0.06, 0.08)),
        )?;
        for (index, color) in CLASS_COLORS.into_iter().enumerate() {
            renderer.upload_material(
                MaterialHandle(CLASS_MATERIAL_BASE + index as u64),
                &Material::new(format!("cgm-class-{index}"), color),
            )?;
        }
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-cgm-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.inspection = Some(inspection);
        self.lowered_primitives = lowered_primitives;
        self.presentation_revision = self.presentation_revision.saturating_add(1);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::Resized { width, height } = event {
            self.window_size = [width.max(1) as f32, height.max(1) as f32];
            self.presentation_revision = self.presentation_revision.saturating_add(1);
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }
        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        if self.renderer.is_none() || self.inspection.is_none() {
            return Ok(FrameOutcome::Continue);
        }
        let diagnostic_count = self.diagnostics.records().len();
        self.record_presentation_build();

        let (Some(renderer), Some(inspection)) = (self.renderer.as_mut(), self.inspection.as_ref())
        else {
            unreachable!("renderer and inspection presence checked above");
        };
        renderer.upload_camera(
            CAMERA_HANDLE,
            Camera::orthographic_2d(self.window_size[0], self.window_size[1]),
        );
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.035, 0.045, 0.065),
        })]);
        let presentation_start = Instant::now();
        Self::draw_inspection(
            renderer,
            self.pipeline,
            inspection,
            &self.lowered_primitives,
        );
        let presentation_time = presentation_start.elapsed();
        let present_start = Instant::now();
        let stats = renderer.present()?;
        let present_time = present_start.elapsed();
        self.frame_time_monitor
            .observe(delta_seconds * 1000.0, &mut self.diagnostics);
        self.presentation_time_monitor.observe(
            presentation_time.as_secs_f64() * 1000.0,
            &mut self.diagnostics,
        );
        self.present_time_monitor
            .observe(present_time.as_secs_f64() * 1000.0, &mut self.diagnostics);
        for diagnostic in &self.diagnostics.records()[diagnostic_count..] {
            eprintln!("{diagnostic}");
        }

        if self.frame_index < 3 || self.last_performance_report.elapsed() >= Duration::from_secs(2)
        {
            let cpu_timings = stats.frame.cpu_timings;
            println!(
                "hello-cgm frame {}: presentation_revision={}, unchanged_presentation_rebuilds={}, platform_frame_interval_ms={:.3}, presentation_build_cpu_ms={:.3}, renderer_present_call_cpu_ms={:.3}, surface_acquire_call_cpu_ms={:.3}, resource_preparation_cpu_ms={:.3}, command_encoding_cpu_ms={:.3}, queue_submit_call_cpu_ms={:.3}, surface_present_call_cpu_ms={:.3}, frame_draws={}, frame_submits={}, frame_binding_allocations={}, frame_uniform_writes={}, frame_mesh_uploads={}, frame_mesh_replacements={}, lifetime_binding_allocations={}, lifetime_uniform_writes={}, lifetime_mesh_uploads={}, lifetime_mesh_replacements={}",
                self.frame_index,
                self.presentation_revision,
                self.unchanged_presentation_rebuilds,
                delta_seconds * 1000.0,
                presentation_time.as_secs_f64() * 1000.0,
                present_time.as_secs_f64() * 1000.0,
                cpu_timings
                    .surface_acquire_call
                    .unwrap_or_default()
                    .as_secs_f64()
                    * 1000.0,
                cpu_timings
                    .resource_preparation
                    .unwrap_or_default()
                    .as_secs_f64()
                    * 1000.0,
                cpu_timings
                    .command_encoding
                    .unwrap_or_default()
                    .as_secs_f64()
                    * 1000.0,
                cpu_timings
                    .queue_submit_call
                    .unwrap_or_default()
                    .as_secs_f64()
                    * 1000.0,
                cpu_timings
                    .surface_present_call
                    .unwrap_or_default()
                    .as_secs_f64()
                    * 1000.0,
                stats.frame.draw_calls,
                stats.frame.submit_calls,
                stats.frame.binding_allocations,
                stats.frame.uniform_buffer_writes,
                stats.frame.mesh_uploads,
                stats.frame.mesh_replacements,
                stats.lifetime.binding_allocations,
                stats.lifetime.uniform_buffer_writes,
                stats.lifetime.mesh_uploads,
                stats.lifetime.mesh_replacements,
            );
            self.last_performance_report = Instant::now();
        }
        self.frame_index += 1;
        Ok(FrameOutcome::Continue)
    }
}

fn source_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(SOURCE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokimu::DiagnosticKind;

    #[test]
    fn selected_source_reaches_the_expected_lifecycle() {
        let inspection = inspect_binary_cgm_file(source_path(), DecodeLimits::default())
            .expect("selected CGM fixture should inspect");

        assert_eq!(inspection.metafile_name, "POLYLN01");
        assert_eq!(inspection.pictures.len(), 1);
        assert_eq!(inspection.pictures[0].name, "picture 1");
        assert!(inspection.elements.len() > 10);
    }

    #[test]
    fn unchanged_presentation_rebuilds_emit_at_the_application_boundary() {
        let mut app = HelloCgmApp {
            presentation_revision: 1,
            ..HelloCgmApp::default()
        };

        app.record_presentation_build();
        app.record_presentation_build();
        app.record_presentation_build();
        app.record_presentation_build();

        assert_eq!(app.unchanged_presentation_rebuilds, 3);
        assert_eq!(app.diagnostics.records().len(), 1);
        assert_eq!(
            app.diagnostics.records()[0].kind,
            DiagnosticKind::PerformanceBudgetExceeded
        );
        assert_eq!(
            app.diagnostics.records()[0].source,
            "hello-cgm.presentation"
        );

        app.presentation_revision = 2;
        app.record_presentation_build();

        assert_eq!(app.unchanged_presentation_rebuilds, 0);
        assert_eq!(
            app.diagnostics.records()[1].kind,
            DiagnosticKind::PerformanceRecovered
        );
    }

    #[test]
    fn selected_source_lowers_to_one_or_more_provider_neutral_paths() {
        let inspection = inspect_binary_cgm_file(source_path(), DecodeLimits::default())
            .expect("selected CGM fixture should inspect");
        let lowered = lower_picture_primitives(&inspection.pictures[0])
            .expect("selected CGM picture should lower");

        assert!(!lowered.is_empty());
        assert!(lowered
            .iter()
            .flat_map(|primitive| primitive.path.contours.iter())
            .all(|contour| contour
                .points
                .iter()
                .flatten()
                .all(|coordinate| coordinate.is_finite())));
    }

    #[test]
    fn diagnostic_mesh_fit_centers_and_preserves_aspect_ratio() {
        let fitted = fit_diagnostic_mesh_to_unit_box(vec![
            [10.0, -2.0, 0.0],
            [14.0, -2.0, 0.0],
            [14.0, 0.0, 0.0],
        ]);

        let minimum_x = fitted
            .iter()
            .map(|point| point[0])
            .fold(f32::INFINITY, f32::min);
        let maximum_x = fitted
            .iter()
            .map(|point| point[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let minimum_y = fitted
            .iter()
            .map(|point| point[1])
            .fold(f32::INFINITY, f32::min);
        let maximum_y = fitted
            .iter()
            .map(|point| point[1])
            .fold(f32::NEG_INFINITY, f32::max);

        assert!((minimum_x + 0.5).abs() < f32::EPSILON);
        assert!((maximum_x - 0.5).abs() < f32::EPSILON);
        assert!((minimum_y + 0.25).abs() < f32::EPSILON);
        assert!((maximum_y - 0.25).abs() < f32::EPSILON);
    }
}
