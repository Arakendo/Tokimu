//! Browser/WASM host for the Doom-local synthetic presentation controls.
//!
//! Rust owns each source fixture, lowering result, render state, and bounded
//! observation. The small JavaScript host only selects a named fixture and
//! displays the returned evidence.

#[cfg(target_arch = "wasm32")]
use doom_geometry_provider::{
    lower_doom_paired_sky_boundary_triangles, lower_doom_seg_textured_wall_triangles,
    lower_doom_subsector_surfaces, lower_doom_two_sided_wall_bands,
    resolve_doom_subsector_bsp_paths, DoomSectorRuntimeHeightSnapshot, DoomSurfacePlane,
    DoomTextureExtent, DoomWallTextureRole,
};
#[cfg(target_arch = "wasm32")]
use hello_doom_visibility_conformance::{
    dynamic_door_snapshot_fixture, moving_platform_snapshot_fixture, one_sky_far_control_fixture,
    paired_sky_far_control_fixture, projection_close_forward_seg_fixture,
    projection_near_plane_crossing_fixture, projection_thin_forward_seg_fixture,
    shared_key_disjoint_plane_fixture, vertical_aperture_control_fixture,
};
#[cfg(target_arch = "wasm32")]
use tokimu::{
    math::{Mat4, Vec3},
    BlendMode, Camera, CameraHandle, CategoricalCutout, ClearCommand, Color, ColorWriteMask,
    CullMode, CutoutComparison, CutoutThreshold, DepthTest, DrawMeshCommand, Instance2d, Material,
    MaterialHandle, Mesh, MeshHandle, Pipeline, PipelineHandle, PipelineKind, PipelineRenderState,
    RenderCommand, Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle,
    WgpuBackend,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("hello-doom-visibility-conformance-web is a browser/WASM corpus consumer");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy)]
enum FixtureMode {
    PairedSky,
    OneSkyNegative,
    VerticalAperture,
    SharedKeyPlane,
    DynamicDoorSnapshot,
    PlatformSnapshot,
    ProjectionEpsilon,
    CutoutNonOccluder,
}

#[cfg(target_arch = "wasm32")]
impl FixtureMode {
    fn parse(value: &str) -> Result<Self, JsValue> {
        match value {
            "paired-sky" => Ok(Self::PairedSky),
            "one-sky-negative" => Ok(Self::OneSkyNegative),
            "vertical-aperture" => Ok(Self::VerticalAperture),
            "shared-key-plane" => Ok(Self::SharedKeyPlane),
            "dynamic-door-snapshot" => Ok(Self::DynamicDoorSnapshot),
            "platform-snapshot" => Ok(Self::PlatformSnapshot),
            "projection-epsilon" => Ok(Self::ProjectionEpsilon),
            "cutout-non-occluder" => Ok(Self::CutoutNonOccluder),
            other => Err(JsValue::from_str(&format!(
                "unknown visibility fixture `{other}`"
            ))),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::PairedSky => "paired-sky",
            Self::OneSkyNegative => "one-sky-negative",
            Self::VerticalAperture => "vertical-aperture",
            Self::SharedKeyPlane => "shared-key-plane",
            Self::DynamicDoorSnapshot => "dynamic-door-snapshot",
            Self::PlatformSnapshot => "platform-snapshot",
            Self::ProjectionEpsilon => "projection-epsilon",
            Self::CutoutNonOccluder => "cutout-non-occluder",
        }
    }
}

/// Presents a named source-fixture control on browser WebGPU.
///
/// This intentionally reports semantic source counts and browser metadata, not
/// a pixel-identical native/browser claim.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn run_fixture(
    canvas: HtmlCanvasElement,
    fixture_name: String,
) -> Result<String, JsValue> {
    console_error_panic_hook::set_once();
    let mode = FixtureMode::parse(&fixture_name)?;
    let width = canvas.width().max(1);
    let height = canvas.height().max(1);
    let mut renderer = WgpuBackend::for_window(canvas, width, height)
        .await
        .map_err(js_debug)?;
    let backend = renderer.backend_api();
    let device = renderer.device_kind();
    let adapter = renderer.adapter_name().to_owned();

    let source_counts = upload_fixture(&mut renderer, mode)?;
    renderer.upload_camera(
        CameraHandle(1),
        Camera::orthographic_2d_with_height(width as f32, height as f32, 2.0),
    );
    let background_definition = Pipeline::new(
        "doom-visibility-browser-background",
        PipelineKind::SolidColor2d,
    )
    .with_render_state(PipelineRenderState {
        blend: BlendMode::Opaque,
        depth_test: DepthTest::LessEqual,
        depth_write: false,
        cull_mode: CullMode::None,
        color_write: ColorWriteMask::ALL,
    })
    .map_err(js_debug)?;
    let background = renderer
        .register_pipeline(&background_definition)
        .map_err(js_debug)?;
    let source_definition =
        Pipeline::new("doom-visibility-browser-source", PipelineKind::SolidColor2d)
            .with_render_state(PipelineRenderState {
                blend: BlendMode::Opaque,
                depth_test: DepthTest::LessEqual,
                depth_write: true,
                cull_mode: CullMode::None,
                color_write: if matches!(mode, FixtureMode::PairedSky) {
                    ColorWriteMask::NONE
                } else {
                    ColorWriteMask::ALL
                },
            })
            .map_err(js_debug)?;
    let source_pipeline = renderer
        .register_pipeline(&source_definition)
        .map_err(js_debug)?;
    let cutout_pipeline = if matches!(mode, FixtureMode::CutoutNonOccluder) {
        Some(
            renderer
                .register_pipeline(&Pipeline::textured_3d_cutout(
                    "doom-visibility-browser-cutout",
                    CategoricalCutout::new(
                        CutoutThreshold::new(0.5).map_err(js_debug)?,
                        CutoutComparison::DiscardBelow,
                    ),
                ))
                .map_err(js_debug)?,
        )
    } else {
        None
    };

    renderer.begin_frame();
    renderer.submit(&draws(mode, background, source_pipeline, cutout_pipeline));
    let first = renderer.present().map_err(js_debug)?;
    let diagnostics = renderer.drain_diagnostics();
    if let Some(record) = diagnostics.first() {
        return Err(JsValue::from_str(&format!(
            "provider diagnostic: category={:?}; source={}; message={}",
            record.kind, record.source, record.message
        )));
    }

    renderer.begin_frame();
    renderer.submit(&draws(mode, background, source_pipeline, cutout_pipeline));
    let warm = renderer.present().map_err(js_debug)?;
    let diagnostics = renderer.drain_diagnostics();
    if let Some(record) = diagnostics.first() {
        return Err(JsValue::from_str(&format!(
            "provider warm diagnostic: category={:?}; source={}; message={}",
            record.kind, record.source, record.message
        )));
    }
    if warm.frame.mesh_uploads != 0 || warm.frame.mesh_replacements != 0 {
        return Err(JsValue::from_str(&format!(
            "static warm frame mutated meshes: uploads={}; replacements={}",
            warm.frame.mesh_uploads, warm.frame.mesh_replacements
        )));
    }

    // This third presentation is deliberately small and deterministic. It
    // verifies that a camera update is not implemented by replacing static
    // source meshes on the browser provider.
    let mut jitter_camera = Camera::orthographic_2d_with_height(width as f32, height as f32, 2.0);
    jitter_camera.view = Mat4::from_translation(Vec3::new(-0.08, 0.0, 0.0));
    renderer.upload_camera(CameraHandle(1), jitter_camera);
    renderer.begin_frame();
    renderer.submit(&draws(mode, background, source_pipeline, cutout_pipeline));
    let jitter = renderer.present().map_err(js_debug)?;
    let diagnostics = renderer.drain_diagnostics();
    if let Some(record) = diagnostics.first() {
        return Err(JsValue::from_str(&format!(
            "provider camera-jitter diagnostic: category={:?}; source={}; message={}",
            record.kind, record.source, record.message
        )));
    }
    if jitter.frame.mesh_uploads != 0 || jitter.frame.mesh_replacements != 0 {
        return Err(JsValue::from_str(&format!(
            "camera jitter mutated static meshes: uploads={}; replacements={}",
            jitter.frame.mesh_uploads, jitter.frame.mesh_replacements
        )));
    }

    Ok(format!(
        "status=presented; fixture={}; source={source_counts}; first_draws={}; warm_draws={}; warm_mesh_uploads={}; warm_mesh_replacements={}; camera_jitter=offset_x:0.08; jitter_draws={}; jitter_mesh_uploads={}; jitter_mesh_replacements={}; backend={backend}; device={device}; adapter={adapter}; canvas={}x{}; host=DOM; comparison=semantic-not-pixel-identical",
        mode.label(),
        first.frame.draw_calls,
        warm.frame.draw_calls,
        warm.frame.mesh_uploads,
        warm.frame.mesh_replacements,
        jitter.frame.draw_calls,
        jitter.frame.mesh_uploads,
        jitter.frame.mesh_replacements,
        width,
        height,
    ))
}

#[cfg(target_arch = "wasm32")]
fn upload_fixture(renderer: &mut WgpuBackend, mode: FixtureMode) -> Result<String, JsValue> {
    const BACKGROUND: MeshHandle = MeshHandle(1);
    const FAR: MeshHandle = MeshHandle(2);
    const FIRST_SOURCE: MeshHandle = MeshHandle(3);
    const SECOND_SOURCE: MeshHandle = MeshHandle(4);

    renderer.upload_mesh(BACKGROUND, &Mesh::quad());
    renderer.upload_mesh(FAR, &Mesh::quad());
    renderer
        .upload_material(
            MaterialHandle(1),
            &Material::new("sky", Color::rgb(0.08, 0.24, 0.48)),
        )
        .map_err(js_debug)?;
    renderer
        .upload_material(
            MaterialHandle(2),
            &Material::new("far-control", Color::rgb(0.92, 0.22, 0.12)),
        )
        .map_err(js_debug)?;

    match mode {
        FixtureMode::PairedSky | FixtureMode::OneSkyNegative => {
            let fixture = match mode {
                FixtureMode::PairedSky => paired_sky_far_control_fixture(),
                FixtureMode::OneSkyNegative => one_sky_far_control_fixture(),
                FixtureMode::VerticalAperture
                | FixtureMode::SharedKeyPlane
                | FixtureMode::DynamicDoorSnapshot
                | FixtureMode::PlatformSnapshot
                | FixtureMode::ProjectionEpsilon
                | FixtureMode::CutoutNonOccluder => unreachable!(),
            }
            .map_err(js_debug)?;
            let boundaries =
                lower_doom_paired_sky_boundary_triangles(&fixture.map).map_err(js_debug)?;
            let walls = lower_doom_seg_textured_wall_triangles(
                &fixture.map,
                &[DoomTextureExtent {
                    name: "WALL".to_owned(),
                    width: 64,
                    height: 128,
                }],
            )
            .map_err(js_debug)?;
            let near = fixture.map.segs[0].source;
            let first = if matches!(mode, FixtureMode::PairedSky) {
                mesh_from_triangles(boundaries.iter().map(|triangle| triangle.positions), -0.60)
            } else {
                mesh_from_triangles(
                    walls
                        .iter()
                        .filter(|triangle| triangle.source_seg == near)
                        .map(|triangle| triangle.positions),
                    -0.60,
                )
            };
            renderer.upload_mesh(FIRST_SOURCE, &first);
            renderer
                .upload_material(
                    MaterialHandle(3),
                    &Material::new(
                        if matches!(mode, FixtureMode::PairedSky) {
                            "paired-depth"
                        } else {
                            "ordinary-upper"
                        },
                        Color::rgb(0.12, 0.72, 0.28),
                    ),
                )
                .map_err(js_debug)?;
            Ok(format!(
                "boundary_triangles={}; source_control_vertices={}; wall_triangles={}",
                boundaries.len(),
                first.vertex_count(),
                walls.len()
            ))
        }
        FixtureMode::VerticalAperture => {
            let fixture = vertical_aperture_control_fixture().map_err(js_debug)?;
            let near = fixture.map.segs[0].source;
            let walls = lower_doom_seg_textured_wall_triangles(
                &fixture.map,
                &[DoomTextureExtent {
                    name: "WALL".to_owned(),
                    width: 64,
                    height: 128,
                }],
            )
            .map_err(js_debug)?;
            let upper = mesh_from_triangles(
                walls
                    .iter()
                    .filter(|triangle| {
                        triangle.source_seg == near && triangle.role == DoomWallTextureRole::Upper
                    })
                    .map(|triangle| triangle.positions),
                -0.60,
            );
            let lower = mesh_from_triangles(
                walls
                    .iter()
                    .filter(|triangle| {
                        triangle.source_seg == near && triangle.role == DoomWallTextureRole::Lower
                    })
                    .map(|triangle| triangle.positions),
                -0.60,
            );
            if upper.vertex_count() == 0 || lower.vertex_count() == 0 {
                return Err(JsValue::from_str(
                    "vertical aperture source tiers are absent",
                ));
            }
            renderer.upload_mesh(FIRST_SOURCE, &upper);
            renderer.upload_mesh(SECOND_SOURCE, &lower);
            renderer
                .upload_material(
                    MaterialHandle(3),
                    &Material::new("source-upper", Color::rgb(0.12, 0.72, 0.28)),
                )
                .map_err(js_debug)?;
            renderer
                .upload_material(
                    MaterialHandle(4),
                    &Material::new("source-lower", Color::rgb(0.94, 0.72, 0.08)),
                )
                .map_err(js_debug)?;
            Ok(format!(
                "upper_triangles={}; lower_triangles={}; source_wall_triangles={}",
                upper.vertex_count() / 3,
                lower.vertex_count() / 3,
                walls.len()
            ))
        }
        FixtureMode::SharedKeyPlane => {
            let fixture = shared_key_disjoint_plane_fixture().map_err(js_debug)?;
            let paths = resolve_doom_subsector_bsp_paths(&fixture.map).map_err(js_debug)?;
            let surfaces = lower_doom_subsector_surfaces(&fixture.map, &paths).map_err(js_debug)?;
            let near = mesh_from_floor_triangles(
                surfaces
                    .iter()
                    .filter(|surface| {
                        surface.plane == DoomSurfacePlane::Floor
                            && surface.source_sector.record_index == 0
                    })
                    .map(|surface| surface.positions),
                -0.40,
            );
            let far = mesh_from_floor_triangles(
                surfaces
                    .iter()
                    .filter(|surface| {
                        surface.plane == DoomSurfacePlane::Floor
                            && surface.source_sector.record_index == 1
                    })
                    .map(|surface| surface.positions),
                -0.20,
            );
            if near.vertex_count() == 0 || far.vertex_count() == 0 {
                return Err(JsValue::from_str(
                    "shared-key source plane regions are absent",
                ));
            }
            renderer.upload_mesh(FIRST_SOURCE, &near);
            renderer.upload_mesh(SECOND_SOURCE, &far);
            renderer
                .upload_material(
                    MaterialHandle(3),
                    &Material::new("source-sector-0", Color::rgb(0.12, 0.72, 0.28)),
                )
                .map_err(js_debug)?;
            renderer
                .upload_material(
                    MaterialHandle(4),
                    &Material::new("source-sector-1", Color::rgb(0.94, 0.44, 0.12)),
                )
                .map_err(js_debug)?;
            Ok(format!(
                "shared_floor_key=height-0; sector_0_triangles={}; sector_1_triangles={}",
                near.vertex_count() / 3,
                far.vertex_count() / 3,
            ))
        }
        FixtureMode::DynamicDoorSnapshot => {
            let fixture = dynamic_door_snapshot_fixture().map_err(js_debug)?;
            let dynamic_sector = fixture.map.sectors[1].source;
            let closed = fixture
                .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                    source_sector: dynamic_sector,
                    floor_height: None,
                    ceiling_height: Some(0),
                }])
                .map_err(js_debug)?;
            let open = fixture
                .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                    source_sector: dynamic_sector,
                    floor_height: None,
                    ceiling_height: Some(128),
                }])
                .map_err(js_debug)?;
            let closed_bands = lower_doom_two_sided_wall_bands(&closed.map).map_err(js_debug)?;
            let open_bands = lower_doom_two_sided_wall_bands(&open.map).map_err(js_debug)?;
            if closed_bands.is_empty() || !open_bands.is_empty() {
                return Err(JsValue::from_str(&format!(
                    "dynamic snapshot lowering unexpected: closed_triangles={}; open_triangles={}",
                    closed_bands.len(),
                    open_bands.len()
                )));
            }
            let closed_mesh = mesh_from_dynamic_door_triangles(
                closed_bands.iter().map(|triangle| triangle.positions),
                -0.60,
            );
            renderer.upload_mesh(FIRST_SOURCE, &closed_mesh);
            renderer.upload_mesh(SECOND_SOURCE, &Mesh::quad());
            renderer
                .upload_material(
                    MaterialHandle(3),
                    &Material::new("closed-source-height-band", Color::rgb(0.20, 0.78, 0.38)),
                )
                .map_err(js_debug)?;
            renderer
                .upload_material(
                    MaterialHandle(4),
                    &Material::new("open-aperture-far-control", Color::rgb(0.96, 0.40, 0.18)),
                )
                .map_err(js_debug)?;
            Ok(format!(
                "state=declared-snapshots-only; closed_triangles={}; open_triangles={}",
                closed_bands.len(),
                open_bands.len(),
            ))
        }
        FixtureMode::PlatformSnapshot => {
            let fixture = moving_platform_snapshot_fixture().map_err(js_debug)?;
            let source_sector = fixture.map.sectors[0].source;
            let low = fixture
                .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                    source_sector,
                    floor_height: Some(0),
                    ceiling_height: None,
                }])
                .map_err(js_debug)?;
            let raised = fixture
                .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                    source_sector,
                    floor_height: Some(48),
                    ceiling_height: None,
                }])
                .map_err(js_debug)?;
            let extents = [DoomTextureExtent {
                name: "WALL".to_owned(),
                width: 64,
                height: 64,
            }];
            let low_triangles =
                lower_doom_seg_textured_wall_triangles(&low.map, &extents).map_err(js_debug)?;
            let raised_triangles =
                lower_doom_seg_textured_wall_triangles(&raised.map, &extents).map_err(js_debug)?;
            let low_mesh = mesh_from_triangles(
                low_triangles.iter().map(|triangle| triangle.positions),
                -0.60,
            );
            let raised_mesh = mesh_from_triangles(
                raised_triangles.iter().map(|triangle| triangle.positions),
                -0.60,
            );
            if low_mesh.vertex_count() == 0 || raised_mesh.vertex_count() == 0 {
                return Err(JsValue::from_str(
                    "platform snapshot lowering produced an empty wall mesh",
                ));
            }
            renderer.upload_mesh(FIRST_SOURCE, &low_mesh);
            renderer.upload_mesh(SECOND_SOURCE, &raised_mesh);
            renderer
                .upload_material(
                    MaterialHandle(3),
                    &Material::new("floor-snapshot-0", Color::rgb(0.20, 0.78, 0.38)),
                )
                .map_err(js_debug)?;
            renderer
                .upload_material(
                    MaterialHandle(4),
                    &Material::new("floor-snapshot-48", Color::rgb(0.96, 0.62, 0.14)),
                )
                .map_err(js_debug)?;
            Ok(format!(
                "state=declared-snapshots-only; low_floor=0:triangles={}; raised_floor=48:triangles={}",
                low_triangles.len(),
                raised_triangles.len(),
            ))
        }
        FixtureMode::ProjectionEpsilon => {
            let near = projection_near_plane_crossing_fixture().map_err(js_debug)?;
            let thin = projection_thin_forward_seg_fixture().map_err(js_debug)?;
            let close = projection_close_forward_seg_fixture().map_err(js_debug)?;
            let near_observation = near.observe_classic_bsp().map_err(js_debug)?;
            let thin_observation = thin.observe_classic_bsp().map_err(js_debug)?;
            let close_observation = close.observe_classic_bsp().map_err(js_debug)?;
            if near_observation.near_plane_fail_open == 0
                || near_observation.solid_admitted != 0
                || thin_observation.solid_admitted != 1
                || close_observation.solid_admitted != 1
            {
                return Err(JsValue::from_str(&format!(
                    "projection-edge source observations unexpected: near={near_observation:?}; thin={thin_observation:?}; close={close_observation:?}"
                )));
            }
            let thin_mesh = projection_source_wall_mesh(&thin, 0.26).map_err(js_debug)?;
            let close_mesh = projection_source_wall_mesh(&close, 0.014).map_err(js_debug)?;
            if thin_mesh.vertex_count() == 0 || close_mesh.vertex_count() == 0 {
                return Err(JsValue::from_str(&format!(
                    "projection-edge lowerer produced empty valid source mesh: thin_vertices={}; close_vertices={}",
                    thin_mesh.vertex_count(), close_mesh.vertex_count()
                )));
            }
            renderer.upload_mesh(FIRST_SOURCE, &thin_mesh);
            renderer.upload_mesh(SECOND_SOURCE, &close_mesh);
            renderer
                .upload_material(
                    MaterialHandle(3),
                    &Material::new("thin-valid-source-seg", Color::rgb(0.25, 0.84, 0.45)),
                )
                .map_err(js_debug)?;
            renderer
                .upload_material(
                    MaterialHandle(4),
                    &Material::new("close-valid-source-seg", Color::rgb(0.98, 0.56, 0.26)),
                )
                .map_err(js_debug)?;
            Ok(format!(
                "near_plane=fail-open:solid_admitted={}; thin=valid:solid_admitted={}:covered_columns={}; close=valid:solid_admitted={}:covered_columns={}; presentation=source-x-magnified-per-control",
                near_observation.solid_admitted,
                thin_observation.solid_admitted,
                thin_observation.solid_range_covered_columns,
                close_observation.solid_admitted,
                close_observation.solid_range_covered_columns,
            ))
        }
        FixtureMode::CutoutNonOccluder => {
            // The shared backing pipeline is color-only. Keep the far wall at
            // the reference depth and place the categorical cutout nearer so
            // only its retained texels cover the far wall.
            renderer.upload_mesh(FAR, &quad_at_depth(0.0));
            renderer.upload_mesh(FIRST_SOURCE, &quad_at_depth(0.5));
            renderer
                .create_texture_rgba8(
                    TextureHandle(1),
                    Rgba8TextureDescriptor::new(4, 4, Rgba8TextureColorSpace::Srgb),
                    &cutout_checker_rgba8(),
                )
                .map_err(js_debug)?;
            renderer
                .upload_material(
                    MaterialHandle(3),
                    &Material::new("masked-middle-cutout", Color::rgb(1.0, 1.0, 1.0))
                        .with_texture(TextureHandle(1)),
                )
                .map_err(js_debug)?;
            Ok("cutout=declared-threshold-0.5; transparent-texels=far-wall-visible; source-authority=none".to_owned())
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn draws(
    mode: FixtureMode,
    background: tokimu::PipelineHandle,
    source: tokimu::PipelineHandle,
    cutout: Option<PipelineHandle>,
) -> Vec<RenderCommand> {
    let draw = |mesh, material, instance| {
        RenderCommand::DrawMesh(DrawMeshCommand {
            mesh,
            material,
            pipeline: source,
            instance,
            camera: Some(CameraHandle(1)),
            viewport: None,
        })
    };
    let mut commands = vec![
        RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.015, 0.02, 0.03),
        }),
        RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: MeshHandle(1),
            material: MaterialHandle(1),
            pipeline: background,
            instance: Instance2d::identity().with_scale([1.8, 1.6]),
            camera: Some(CameraHandle(1)),
            viewport: None,
        }),
    ];
    if matches!(mode, FixtureMode::CutoutNonOccluder) {
        commands.extend([
            draw(
                MeshHandle(2),
                MaterialHandle(2),
                Instance2d::identity()
                    .with_translation([0.0, -0.08])
                    .with_scale([0.82, 0.72]),
            ),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(3),
                material: MaterialHandle(3),
                pipeline: cutout.expect("cutout mode registers its pipeline"),
                instance: Instance2d::identity()
                    .with_translation([0.0, 0.12])
                    .with_scale([1.15, 0.78]),
                camera: Some(CameraHandle(1)),
                viewport: None,
            }),
        ]);
    } else if matches!(mode, FixtureMode::VerticalAperture) {
        // The far control is submitted before the production-lowered source
        // tiers. Their actual vertical opening remains the observation.
        commands.extend([
            draw(
                MeshHandle(2),
                MaterialHandle(2),
                Instance2d::identity().with_scale([0.76, 1.2]),
            ),
            draw(MeshHandle(3), MaterialHandle(3), Instance2d::identity()),
            draw(MeshHandle(4), MaterialHandle(4), Instance2d::identity()),
        ]);
    } else if matches!(mode, FixtureMode::SharedKeyPlane) {
        commands.extend([
            draw(MeshHandle(3), MaterialHandle(3), Instance2d::identity()),
            draw(MeshHandle(4), MaterialHandle(4), Instance2d::identity()),
        ]);
    } else if matches!(mode, FixtureMode::DynamicDoorSnapshot) {
        commands.extend([
            draw(
                MeshHandle(3),
                MaterialHandle(3),
                Instance2d::identity().with_translation([-0.50, -0.10]),
            ),
            draw(
                MeshHandle(4),
                MaterialHandle(4),
                Instance2d::identity()
                    .with_translation([0.50, -0.10])
                    .with_scale([0.35, 0.62]),
            ),
        ]);
    } else if matches!(mode, FixtureMode::PlatformSnapshot) {
        commands.extend([
            draw(
                MeshHandle(3),
                MaterialHandle(3),
                Instance2d::identity().with_translation([-0.45, -0.10]),
            ),
            draw(
                MeshHandle(4),
                MaterialHandle(4),
                Instance2d::identity().with_translation([0.45, -0.10]),
            ),
        ]);
    } else if matches!(mode, FixtureMode::ProjectionEpsilon) {
        commands.extend([
            draw(
                MeshHandle(3),
                MaterialHandle(3),
                Instance2d::identity().with_translation([-0.45, -0.10]),
            ),
            draw(
                MeshHandle(4),
                MaterialHandle(4),
                Instance2d::identity().with_translation([0.45, -0.10]),
            ),
        ]);
    } else {
        // The paired-sky boundary or one-sky ordinary wall precedes the far
        // control, preserving the corresponding native control order.
        commands.extend([
            draw(MeshHandle(3), MaterialHandle(3), Instance2d::identity()),
            draw(
                MeshHandle(2),
                MaterialHandle(2),
                Instance2d::identity().with_scale([0.76, 1.2]),
            ),
        ]);
    }
    commands
}

#[cfg(target_arch = "wasm32")]
fn mesh_from_floor_triangles(
    triangles: impl IntoIterator<Item = [[f64; 3]; 3]>,
    clip_depth: f32,
) -> Mesh {
    let positions = triangles
        .into_iter()
        .flatten()
        .map(|position| {
            [
                position[0] as f32 / 112.0,
                position[2] as f32 / 144.0 - 0.70,
                clip_depth,
            ]
        })
        .collect();
    Mesh::uniform_normal(positions, [0.0, 0.0, -1.0])
}

#[cfg(target_arch = "wasm32")]
fn mesh_from_triangles(
    triangles: impl IntoIterator<Item = [[f64; 3]; 3]>,
    clip_depth: f32,
) -> Mesh {
    Mesh::uniform_normal(
        triangles
            .into_iter()
            .flatten()
            .map(|position| {
                [
                    position[0] as f32 / 64.0,
                    position[1] as f32 / 100.0 - 0.80,
                    clip_depth,
                ]
            })
            .collect(),
        [0.0, 0.0, -1.0],
    )
}

#[cfg(target_arch = "wasm32")]
fn mesh_from_dynamic_door_triangles(
    triangles: impl IntoIterator<Item = [[f64; 3]; 3]>,
    clip_depth: f32,
) -> Mesh {
    Mesh::uniform_normal(
        triangles
            .into_iter()
            .flatten()
            // The dynamic doorway's source segment runs along Z, unlike the
            // existing aperture fixture. Use its varying source axis for the
            // 2D presentation to avoid an edge-on control mesh.
            .map(|position| {
                [
                    position[2] as f32 / 96.0,
                    position[1] as f32 / 96.0 - 0.67,
                    clip_depth,
                ]
            })
            .collect(),
        [0.0, 0.0, -1.0],
    )
}

#[cfg(target_arch = "wasm32")]
fn projection_source_wall_mesh(
    fixture: &hello_doom_visibility_conformance::DoomVisibilityFixture,
    horizontal_scale: f32,
) -> Result<Mesh, doom_geometry_provider::DoomGeometryError> {
    let walls = lower_doom_seg_textured_wall_triangles(
        &fixture.map,
        &[DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 128,
        }],
    )?;
    let positions = walls
        .into_iter()
        .flat_map(|triangle| triangle.positions)
        .map(|position| {
            [
                position[0] as f32 * horizontal_scale,
                position[1] as f32 / 96.0 - 0.67,
                -0.60,
            ]
        })
        .collect();
    Ok(Mesh::uniform_normal(positions, [0.0, 0.0, -1.0]))
}

#[cfg(target_arch = "wasm32")]
fn quad_at_depth(depth: f32) -> Mesh {
    let mut mesh = Mesh::quad()
        .with_texture_coordinates(vec![
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [0.0, 0.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ])
        .expect("fixed UV count matches quad");
    for position in &mut mesh.positions {
        position[2] = depth;
    }
    mesh
}

#[cfg(target_arch = "wasm32")]
fn cutout_checker_rgba8() -> Vec<u8> {
    let mut rgba8 = Vec::with_capacity(4 * 4 * 4);
    for row in 0..4 {
        for column in 0..4 {
            let opaque = (row + column) % 2 == 0;
            rgba8.extend_from_slice(&[0x35, 0xd9, 0x78, if opaque { 0xff } else { 0x00 }]);
        }
    }
    rgba8
}

#[cfg(target_arch = "wasm32")]
fn js_debug(error: impl std::fmt::Debug) -> JsValue {
    JsValue::from_str(&format!("{error:?}"))
}
