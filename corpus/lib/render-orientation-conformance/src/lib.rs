//! Shared AR-0021 renderer-orientation conformance evidence.
//!
//! This corpus fixture deliberately keeps geometric winding, authored shading
//! normals, fragment facing, culling, and reflection compensation observable as
//! separate facts.

use tokimu_core::math::{Mat4, Vec3, Vec4};
use tokimu_render::{
    BlendMode, ColorWriteMask, CullMode, DepthTest, Instance2d, MaterialParameterKind, Mesh,
    Pipeline, PipelineRenderState, ShaderBindingDeclaration, ShaderBindingSource,
    ShaderModuleDefinition, ShaderVertexInput, ShaderVertexSemantic,
};

pub const FRONT_COLOR: [f32; 4] = [0.12, 0.90, 0.34, 1.0];
pub const BACK_COLOR: [f32; 4] = [0.94, 0.16, 0.68, 1.0];
pub const DIRECTIONAL_ATLAS_WIDTH: u32 = 320;
pub const DIRECTIONAL_ATLAS_HEIGHT: u32 = 192;
pub const CAMERA_CONFORMANCE_PITCH_LIMIT: f32 = 0.7;

/// Corpus-local pose used to compare deterministic camera commands with live
/// native/browser input. This is study vocabulary, not an admitted camera API.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraConformancePose {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for CameraConformancePose {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, -6.0),
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraConformanceBasis {
    pub forward: Vec3,
    pub up: Vec3,
    pub right: Vec3,
}

/// Corpus-local world-to-NDC observation used to compare CPU projection,
/// picking, and presented landmark placement. NDC depth remains Tokimu's
/// GL-style `[-1, 1]` value; WGPU conversion is intentionally not applied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectedPointObservation {
    pub world: Vec3,
    pub ndc: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PickingRayObservation {
    pub origin: Vec3,
    pub direction: Vec3,
}

/// Builds the deterministic non-Doom camera matrix shared by projection and
/// picking evidence. This is corpus vocabulary, not a public camera contract.
pub fn camera_conformance_view_projection(pose: CameraConformancePose, aspect_ratio: f32) -> Mat4 {
    let (view, projection) = camera_conformance_matrices(pose, aspect_ratio);
    projection * view
}

pub fn camera_conformance_matrices(pose: CameraConformancePose, aspect_ratio: f32) -> (Mat4, Mat4) {
    let basis = pose.basis();
    let view = tokimu_core::math::try_view_look_at_rh(
        pose.position,
        pose.position + basis.forward,
        basis.up,
    )
    .expect("camera basis must be finite and non-degenerate");
    let projection = tokimu_core::math::try_projection_perspective_rh_gl(
        60.0_f32.to_radians(),
        aspect_ratio,
        0.1,
        100.0,
    )
    .expect("perspective parameters must be finite and ordered");
    (view, projection)
}

pub fn project_world_point(
    view_projection: Mat4,
    world: Vec3,
) -> Option<ProjectedPointObservation> {
    let clip = view_projection * world.extend(1.0);
    if !clip.is_finite() || clip.w.abs() <= f32::EPSILON {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    ndc.is_finite()
        .then_some(ProjectedPointObservation { world, ndc })
}

/// Constructs a world-space ray through one GL-style NDC point. Near and far
/// use `-1` and `+1`; applying WGPU's provider conversion here would duplicate
/// the backend boundary and deliberately fail the retained depth checks.
pub fn picking_ray_from_ndc(
    view_projection: Mat4,
    ndc_xy: [f32; 2],
) -> Option<PickingRayObservation> {
    let inverse = view_projection.inverse();
    if !inverse.is_finite() {
        return None;
    }
    let unproject = |depth| {
        let homogeneous = inverse * Vec4::new(ndc_xy[0], ndc_xy[1], depth, 1.0);
        (homogeneous.is_finite() && homogeneous.w.abs() > f32::EPSILON)
            .then(|| homogeneous.truncate() / homogeneous.w)
    };
    let near = unproject(-1.0)?;
    let far = unproject(1.0)?;
    let direction = (far - near).normalize_or_zero();
    (direction != Vec3::ZERO).then_some(PickingRayObservation {
        origin: near,
        direction,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CameraConformanceCommand {
    Yaw(f32),
    Pitch(f32),
    MoveForward(f32),
    StrafeRight(f32),
    MoveUp(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerMotionObservation {
    pub delta_x: f32,
    pub delta_y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FirstPersonLookPolicy {
    pub yaw_radians_per_pixel: f32,
    pub pitch_radians_per_pixel: f32,
    pub pitch_limit: f32,
}

impl Default for FirstPersonLookPolicy {
    fn default() -> Self {
        Self {
            yaw_radians_per_pixel: 0.0032,
            pitch_radians_per_pixel: 0.0024,
            pitch_limit: CAMERA_CONFORMANCE_PITCH_LIMIT,
        }
    }
}

impl FirstPersonLookPolicy {
    /// Maps a physical pointer observation into semantic camera commands. A
    /// positive raw X delta requests negative mathematical yaw so motion to the
    /// right turns toward the current screen-right basis.
    pub fn map_pointer_motion(
        self,
        observation: PointerMotionObservation,
    ) -> [CameraConformanceCommand; 2] {
        [
            CameraConformanceCommand::Yaw(-observation.delta_x * self.yaw_radians_per_pixel),
            CameraConformanceCommand::Pitch(-observation.delta_y * self.pitch_radians_per_pixel),
        ]
    }
}

impl CameraConformancePose {
    pub fn basis(self) -> CameraConformanceBasis {
        let forward = Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        )
        .normalize_or_zero();
        let right = Vec3::new(forward.x, 0.0, forward.z)
            .normalize_or_zero()
            .cross(Vec3::Y)
            .normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();
        CameraConformanceBasis { forward, up, right }
    }

    pub fn apply(&mut self, command: CameraConformanceCommand) {
        match command {
            CameraConformanceCommand::Yaw(radians) => self.yaw += radians,
            CameraConformanceCommand::Pitch(radians) => {
                self.pitch = (self.pitch + radians).clamp(
                    -CAMERA_CONFORMANCE_PITCH_LIMIT,
                    CAMERA_CONFORMANCE_PITCH_LIMIT,
                );
            }
            CameraConformanceCommand::MoveForward(distance) => {
                let forward = self.basis().forward;
                self.position +=
                    Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero() * distance;
            }
            CameraConformanceCommand::StrafeRight(distance) => {
                self.position += self.basis().right * distance;
            }
            CameraConformanceCommand::MoveUp(distance) => self.position += Vec3::Y * distance,
        }
    }

    pub fn apply_pointer_motion(
        &mut self,
        policy: FirstPersonLookPolicy,
        observation: PointerMotionObservation,
    ) -> [CameraConformanceCommand; 2] {
        let commands = policy.map_pointer_motion(observation);
        for command in commands {
            self.apply(command);
        }
        self.pitch = self.pitch.clamp(-policy.pitch_limit, policy.pitch_limit);
        commands
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisLandmark {
    pub label: &'static str,
    pub center: Vec3,
    pub color: [f32; 4],
    pub positive: bool,
}

pub fn axis_landmarks() -> [AxisLandmark; 6] {
    [
        AxisLandmark {
            label: "+X",
            center: Vec3::X * 2.0,
            color: [1.0, 0.15, 0.12, 1.0],
            positive: true,
        },
        AxisLandmark {
            label: "-X",
            center: Vec3::NEG_X * 2.0,
            color: [0.45, 0.05, 0.04, 1.0],
            positive: false,
        },
        AxisLandmark {
            label: "+Y",
            center: Vec3::Y * 2.0,
            color: [0.15, 1.0, 0.25, 1.0],
            positive: true,
        },
        AxisLandmark {
            label: "-Y",
            center: Vec3::NEG_Y * 2.0,
            color: [0.04, 0.42, 0.10, 1.0],
            positive: false,
        },
        AxisLandmark {
            label: "+Z",
            center: Vec3::Z * 2.0,
            color: [0.15, 0.35, 1.0, 1.0],
            positive: true,
        },
        AxisLandmark {
            label: "-Z",
            center: Vec3::NEG_Z * 2.0,
            color: [0.04, 0.10, 0.45, 1.0],
            positive: false,
        },
    ]
}

pub fn landmark_mesh(landmark: AxisLandmark) -> Mesh {
    let mut mesh = Mesh::cube();
    let scale = if landmark.positive { 0.34 } else { 0.22 };
    for position in &mut mesh.positions {
        position[0] = position[0] * scale + landmark.center.x;
        position[1] = position[1] * scale + landmark.center.y;
        position[2] = position[2] * scale + landmark.center.z;
    }
    mesh
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpectedFacing {
    LeftFrontRightBack,
    LeftBackRightFront,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrientationFixtureCase {
    pub label: &'static str,
    pub mesh: Mesh,
    pub instance: Instance2d,
    pub expected_facing: ExpectedFacing,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixtureLayoutCell {
    pub case_index: usize,
    pub cull_index: usize,
    pub instance: Instance2d,
    pub viewport: [f32; 4],
}

/// Returns identity, ordinary-transform, uncompensated-reflection, and
/// compensated-reflection specimens over the same paired triangles.
pub fn fixture_cases() -> [OrientationFixtureCase; 4] {
    [
        OrientationFixtureCase {
            label: "identity",
            mesh: paired_triangles(false),
            instance: Instance2d::identity(),
            expected_facing: ExpectedFacing::LeftFrontRightBack,
        },
        OrientationFixtureCase {
            label: "rotate-translate",
            mesh: paired_triangles(false),
            instance: Instance2d::new([0.08, -0.04], [0.92, 0.92], 0.18),
            expected_facing: ExpectedFacing::LeftFrontRightBack,
        },
        OrientationFixtureCase {
            label: "reflect-x-uncompensated",
            mesh: paired_triangles(false),
            instance: Instance2d::new([0.0, 0.0], [-1.0, 1.0], 0.0),
            expected_facing: ExpectedFacing::LeftBackRightFront,
        },
        OrientationFixtureCase {
            label: "reflect-x-compensated",
            mesh: paired_triangles(true),
            instance: Instance2d::new([0.0, 0.0], [-1.0, 1.0], 0.0),
            expected_facing: ExpectedFacing::LeftFrontRightBack,
        },
    ]
}

pub const fn cull_modes() -> [CullMode; 3] {
    [CullMode::None, CullMode::Back, CullMode::Front]
}

/// Lays out the retained four transform rows and three cull columns for either
/// a native surface or a browser canvas.
pub fn fixture_layout(width: u32, height: u32) -> Vec<FixtureLayoutCell> {
    let width = width.max(1) as f32;
    let height = height.max(1) as f32;
    let cell_width = width / 3.0;
    let cell_height = height / 4.0;
    let mut cells = Vec::with_capacity(12);
    for (row, case) in fixture_cases().into_iter().enumerate() {
        for column in 0..3 {
            let center = [
                -2.0 / 3.0 + column as f32 * 2.0 / 3.0,
                0.75 - row as f32 * 0.5,
            ];
            cells.push(FixtureLayoutCell {
                case_index: row,
                cull_index: column,
                instance: fit_case_in_cell(case.instance, center),
                viewport: [
                    column as f32 * cell_width,
                    row as f32 * cell_height,
                    cell_width,
                    cell_height,
                ],
            });
        }
    }
    cells
}

fn fit_case_in_cell(case: Instance2d, center: [f32; 2]) -> Instance2d {
    Instance2d::new(
        [
            center[0] + case.translation[0] * 0.28,
            center[1] + case.translation[1] * 0.22,
        ],
        [case.scale[0] * 0.28, case.scale[1] * 0.22],
        case.rotation,
    )
}

/// Builds the exact semantic shader and render state used by every fixture
/// consumer. The shader colors front fragments green and back fragments
/// magenta; a separate normal-derived factor controls brightness.
pub fn conformance_pipeline(cull_mode: CullMode) -> Pipeline {
    let module = ShaderModuleDefinition::new(
        "orientation-conformance-shader",
        CONFORMANCE_SHADER,
        "vs_main",
        "fs_main",
        vec![
            ShaderBindingDeclaration::new(
                0,
                0,
                ShaderBindingSource::MaterialParameter {
                    parameter: "base_color".to_owned(),
                    kind: MaterialParameterKind::Color,
                },
            ),
            ShaderBindingDeclaration::new(
                0,
                1,
                ShaderBindingSource::MaterialParameter {
                    parameter: "base_texture".to_owned(),
                    kind: MaterialParameterKind::Texture,
                },
            ),
            ShaderBindingDeclaration::new(
                0,
                2,
                ShaderBindingSource::MaterialSampler {
                    texture_parameter: "base_texture".to_owned(),
                },
            ),
            ShaderBindingDeclaration::new(1, 0, ShaderBindingSource::InstanceTransform),
            ShaderBindingDeclaration::new(2, 0, ShaderBindingSource::Camera),
        ],
        vec![
            ShaderVertexInput::new(0, ShaderVertexSemantic::Position3),
            ShaderVertexInput::new(1, ShaderVertexSemantic::Normal3),
            ShaderVertexInput::new(2, ShaderVertexSemantic::TextureCoordinate2),
        ],
    )
    .expect("the retained conformance shader declaration must remain valid");

    Pipeline::custom_wgsl_module(format!("orientation-conformance-{cull_mode:?}"), module)
        .expect("the retained conformance pipeline module must remain valid")
        .with_render_state(PipelineRenderState {
            blend: BlendMode::Opaque,
            depth_test: DepthTest::LessEqual,
            depth_write: true,
            cull_mode,
            color_write: ColorWriteMask::ALL,
        })
        .expect("the retained conformance render state must remain valid")
}

fn paired_triangles(reverse_each_triangle: bool) -> Mesh {
    let left = [
        ([-0.95, -0.45, 0.0], [0.0, 1.0]),
        ([-0.08, -0.45, 0.0], [1.0, 1.0]),
        ([-0.08, 0.22, 0.0], [1.0, 0.26]),
        ([-0.28, 0.45, 0.0], [0.77, 0.0]),
        ([-0.95, 0.45, 0.0], [0.0, 0.0]),
    ];
    let right = [
        ([0.08, -0.45, 0.0], [0.0, 1.0]),
        ([0.95, -0.45, 0.0], [1.0, 1.0]),
        ([0.95, 0.22, 0.0], [1.0, 0.26]),
        ([0.75, 0.45, 0.0], [0.77, 0.0]),
        ([0.08, 0.45, 0.0], [0.0, 0.0]),
    ];
    let triangles = [[0, 1, 2], [0, 2, 3], [0, 3, 4]];
    let mut positions = Vec::with_capacity(18);
    let mut texture_coordinates = Vec::with_capacity(18);
    for triangle in triangles {
        for index in triangle {
            positions.push(left[index].0);
            texture_coordinates.push(left[index].1);
        }
    }
    for triangle in triangles {
        for index in triangle.into_iter().rev() {
            positions.push(right[index].0);
            texture_coordinates.push(right[index].1);
        }
    }
    if reverse_each_triangle {
        for triangle in positions.chunks_exact_mut(3) {
            triangle.swap(1, 2);
        }
        for triangle in texture_coordinates.chunks_exact_mut(3) {
            triangle.swap(1, 2);
        }
    }

    // Intentionally identical on the CW and CCW panels: supplied-normal
    // direction remains observable without standing in for geometric facing.
    Mesh::uniform_normal(positions, [0.0, 0.0, 1.0])
        .with_texture_coordinates(texture_coordinates)
        .expect("the retained atlas coordinates must remain position-aligned")
}

/// Generates the retained two-face directional atlas without a font or image
/// provider dependency. The top half is sampled for front-facing fragments and
/// the bottom half for back-facing fragments by the corpus-local shader.
pub fn directional_atlas_rgba8() -> Vec<u8> {
    let mut rgba8 = vec![0; (DIRECTIONAL_ATLAS_WIDTH * DIRECTIONAL_ATLAS_HEIGHT * 4) as usize];
    paint_face(&mut rgba8, 0, "FRONT", [15, 74, 49, 255]);
    paint_face(
        &mut rgba8,
        DIRECTIONAL_ATLAS_HEIGHT / 2,
        "BACK",
        [91, 20, 67, 255],
    );
    rgba8
}

fn paint_face(rgba8: &mut [u8], y_offset: u32, face: &str, background: [u8; 4]) {
    let half_height = DIRECTIONAL_ATLAS_HEIGHT / 2;
    fill_rect(
        rgba8,
        0,
        y_offset,
        DIRECTIONAL_ATLAS_WIDTH,
        half_height,
        background,
    );
    fill_rect(rgba8, 0, y_offset, 14, 14, [255, 55, 45, 255]);
    fill_rect(
        rgba8,
        DIRECTIONAL_ATLAS_WIDTH - 14,
        y_offset,
        14,
        14,
        [255, 220, 35, 255],
    );
    fill_rect(
        rgba8,
        0,
        y_offset + half_height - 14,
        14,
        14,
        [50, 100, 255, 255],
    );
    fill_rect(
        rgba8,
        DIRECTIONAL_ATLAS_WIDTH - 14,
        y_offset + half_height - 14,
        14,
        14,
        [245, 245, 245, 255],
    );
    draw_text(rgba8, 4, y_offset + 3, "1", 1, [255, 255, 255, 255]);
    draw_text(
        rgba8,
        DIRECTIONAL_ATLAS_WIDTH - 10,
        y_offset + 3,
        "2",
        1,
        [15, 15, 15, 255],
    );
    draw_text(
        rgba8,
        4,
        y_offset + half_height - 11,
        "3",
        1,
        [255, 255, 255, 255],
    );
    draw_text(
        rgba8,
        DIRECTIONAL_ATLAS_WIDTH - 10,
        y_offset + half_height - 11,
        "4",
        1,
        [15, 15, 15, 255],
    );
    draw_text(rgba8, 106, y_offset + 5, face, 3, [255, 255, 255, 255]);
    draw_text(rgba8, 18, y_offset + 34, "U- LEFT", 2, [255, 235, 120, 255]);
    draw_text(
        rgba8,
        198,
        y_offset + 34,
        "RIGHT U+",
        2,
        [255, 235, 120, 255],
    );
    draw_text(
        rgba8,
        18,
        y_offset + 58,
        "V- TOP UP",
        2,
        [130, 220, 255, 255],
    );
    draw_text(rgba8, 184, y_offset + 58, "N +Z", 2, [130, 255, 160, 255]);
    draw_text(
        rgba8,
        18,
        y_offset + 78,
        "V+ BOTTOM",
        2,
        [130, 220, 255, 255],
    );
}

fn fill_rect(rgba8: &mut [u8], x: u32, y: u32, width: u32, height: u32, color: [u8; 4]) {
    for pixel_y in y..(y + height).min(DIRECTIONAL_ATLAS_HEIGHT) {
        for pixel_x in x..(x + width).min(DIRECTIONAL_ATLAS_WIDTH) {
            let offset = ((pixel_y * DIRECTIONAL_ATLAS_WIDTH + pixel_x) * 4) as usize;
            rgba8[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

fn draw_text(rgba8: &mut [u8], x: u32, y: u32, text: &str, scale: u32, color: [u8; 4]) {
    let mut cursor = x;
    for character in text.chars() {
        let glyph = glyph(character);
        for (row, bits) in glyph.into_iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    fill_rect(
                        rgba8,
                        cursor + column * scale,
                        y + row as u32 * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
        cursor += 6 * scale;
    }
}

fn glyph(character: char) -> [u8; 7] {
    match character {
        'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'B' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        'C' => [0x0f, 0x10, 0x10, 0x10, 0x10, 0x10, 0x0f],
        'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        'F' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x10],
        'G' => [0x0f, 0x10, 0x10, 0x13, 0x11, 0x11, 0x0f],
        'H' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'I' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x1f],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        'M' => [0x11, 0x1b, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        'R' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        'Z' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1f],
        '1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        '2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        '3' => [0x1e, 0x01, 0x01, 0x0e, 0x01, 0x01, 0x1e],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        '+' => [0x00, 0x04, 0x04, 0x1f, 0x04, 0x04, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00],
        ' ' => [0; 7],
        _ => [0x1f, 0x11, 0x02, 0x04, 0x04, 0x00, 0x04],
    }
}

pub const CONFORMANCE_SHADER: &str = r#"
@group(0) @binding(0)
var<uniform> material_color: vec4<f32>;
@group(0) @binding(1)
var material_texture: texture_2d<f32>;
@group(0) @binding(2)
var material_sampler: sampler;
struct InstanceParams {
    translation: vec2<f32>,
    scale: vec2<f32>,
    rotation: vec2<f32>,
    padding: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> instance_params: InstanceParams;

@group(2) @binding(0)
var<uniform> camera_params: mat4x4<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> VertexOutput {
    let scaled = position.xy * instance_params.scale;
    let rotated = vec2<f32>(
        scaled.x * instance_params.rotation.y - scaled.y * instance_params.rotation.x,
        scaled.x * instance_params.rotation.x + scaled.y * instance_params.rotation.y,
    );
    var output: VertexOutput;
    output.position = camera_params * vec4<f32>(rotated + instance_params.translation, position.z, 1.0);
    output.normal = normal;
    output.uv = uv;
    return output;
}

@fragment
fn fs_main(
    input: VertexOutput,
    @builtin(front_facing) front_facing: bool,
) -> @location(0) vec4<f32> {
    let atlas_uv = vec2<f32>(input.uv.x, select(0.5 + input.uv.y * 0.5, input.uv.y * 0.5, front_facing));
    let label_color = textureSample(material_texture, material_sampler, atlas_uv) * material_color;
    let normal_light = 0.35 + 0.65 * max(dot(normalize(input.normal), vec3<f32>(0.0, 0.0, 1.0)), 0.0);
    return vec4<f32>(label_color.rgb * normal_light, label_color.a);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tokimu_render::{Color, MaterialDefinition, MaterialDefinitionId};

    #[test]
    fn fixture_retains_the_full_case_and_cull_matrix() {
        let cases = fixture_cases();
        assert_eq!(cases.len() * cull_modes().len(), 12);
        assert_eq!(
            cases.map(|case| case.expected_facing),
            [
                ExpectedFacing::LeftFrontRightBack,
                ExpectedFacing::LeftFrontRightBack,
                ExpectedFacing::LeftBackRightFront,
                ExpectedFacing::LeftFrontRightBack,
            ]
        );
    }

    #[test]
    fn fixture_winding_and_reflection_expectations_are_geometric() {
        for case in fixture_cases() {
            let determinant = case.instance.scale[0] * case.instance.scale[1];
            let source_signs = triangle_signs(&case.mesh);
            let transformed_signs = source_signs
                .into_iter()
                .map(|sign| (sign * determinant.signum()).signum())
                .collect::<Vec<_>>();
            let expected = match case.expected_facing {
                ExpectedFacing::LeftFrontRightBack => {
                    vec![1.0, 1.0, 1.0, -1.0, -1.0, -1.0]
                }
                ExpectedFacing::LeftBackRightFront => {
                    vec![-1.0, -1.0, -1.0, 1.0, 1.0, 1.0]
                }
            };
            assert_eq!(transformed_signs, expected, "case `{}`", case.label);
        }
    }

    #[test]
    fn asymmetric_panels_retain_complete_caller_supplied_uvs() {
        for case in fixture_cases() {
            assert_eq!(case.mesh.positions.len(), 18, "case `{}`", case.label);
            assert!(case.mesh.has_texture_coordinates(), "case `{}`", case.label);
            assert!(case
                .mesh
                .texture_coordinates
                .iter()
                .any(|uv| uv[0] == 0.77 && uv[1] == 0.0));
        }
    }

    #[test]
    fn directional_atlas_has_exact_dimensions_and_distinct_uv_corners() {
        let atlas = directional_atlas_rgba8();
        assert_eq!(
            atlas.len(),
            (DIRECTIONAL_ATLAS_WIDTH * DIRECTIONAL_ATLAS_HEIGHT * 4) as usize
        );
        assert_eq!(pixel(&atlas, 0, 0), [255, 55, 45, 255]);
        assert_eq!(
            pixel(&atlas, DIRECTIONAL_ATLAS_WIDTH - 1, 0),
            [255, 220, 35, 255]
        );
        assert_eq!(
            pixel(&atlas, 0, DIRECTIONAL_ATLAS_HEIGHT / 2 - 1),
            [50, 100, 255, 255]
        );
        assert_eq!(
            pixel(
                &atlas,
                DIRECTIONAL_ATLAS_WIDTH - 1,
                DIRECTIONAL_ATLAS_HEIGHT / 2 - 1,
            ),
            [245, 245, 245, 255]
        );
    }

    #[test]
    fn camera_basis_is_orthonormal_and_declares_positive_yaw_separately_from_right() {
        let pose = CameraConformancePose::default();
        let initial = pose.basis();
        assert_vec3_close(initial.forward, Vec3::Z);
        assert_vec3_close(initial.up, Vec3::Y);
        assert_vec3_close(initial.right, Vec3::NEG_X);

        let mut positive_yaw = pose;
        positive_yaw.apply(CameraConformanceCommand::Yaw(std::f32::consts::FRAC_PI_2));
        assert_vec3_close(positive_yaw.basis().forward, Vec3::X);

        for basis in [initial, positive_yaw.basis()] {
            assert!((basis.forward.length() - 1.0).abs() < 0.000_1);
            assert!((basis.up.length() - 1.0).abs() < 0.000_1);
            assert!((basis.right.length() - 1.0).abs() < 0.000_1);
            assert!(basis.forward.dot(basis.up).abs() < 0.000_1);
            assert!(basis.forward.dot(basis.right).abs() < 0.000_1);
            assert!(basis.up.dot(basis.right).abs() < 0.000_1);
        }
    }

    #[test]
    fn deterministic_camera_commands_move_along_the_declared_local_basis() {
        let mut pose = CameraConformancePose::default();
        pose.apply(CameraConformanceCommand::MoveForward(2.0));
        assert_vec3_close(pose.position, Vec3::new(0.0, 0.0, -4.0));
        pose.apply(CameraConformanceCommand::StrafeRight(1.5));
        assert_vec3_close(pose.position, Vec3::new(-1.5, 0.0, -4.0));
        pose.apply(CameraConformanceCommand::MoveUp(0.5));
        assert_vec3_close(pose.position, Vec3::new(-1.5, 0.5, -4.0));
        pose.apply(CameraConformanceCommand::MoveForward(-2.0));
        pose.apply(CameraConformanceCommand::StrafeRight(-1.5));
        pose.apply(CameraConformanceCommand::MoveUp(-0.5));
        assert_vec3_close(pose.position, CameraConformancePose::default().position);
    }

    #[test]
    fn pointer_observation_and_first_person_policy_are_distinct_evidence() {
        let observation = PointerMotionObservation {
            delta_x: 100.0,
            delta_y: -50.0,
        };
        let policy = FirstPersonLookPolicy::default();
        let commands = policy.map_pointer_motion(observation);
        let CameraConformanceCommand::Yaw(yaw) = commands[0] else {
            panic!("first pointer command must be yaw")
        };
        let CameraConformanceCommand::Pitch(pitch) = commands[1] else {
            panic!("second pointer command must be pitch")
        };
        assert!((yaw + 0.32).abs() < 0.000_1);
        assert!((pitch - 0.12).abs() < 0.000_1);

        let mut pose = CameraConformancePose::default();
        pose.apply_pointer_motion(policy, observation);
        assert!(pose.basis().forward.dot(Vec3::NEG_X) > 0.0);
        assert!(pose.basis().forward.y > 0.0);
    }

    #[test]
    fn all_six_axis_landmarks_have_unique_declared_identity() {
        let landmarks = axis_landmarks();
        let labels = landmarks.map(|landmark| landmark.label);
        assert_eq!(labels, ["+X", "-X", "+Y", "-Y", "+Z", "-Z"]);
        for landmark in landmarks {
            let mesh = landmark_mesh(landmark);
            assert_eq!(mesh.positions.len(), 36);
            assert!(mesh
                .positions
                .iter()
                .all(|position| position.iter().all(|value| value.is_finite())));
        }
    }

    #[test]
    fn cpu_projection_places_signed_landmarks_in_declared_screen_directions() {
        let view_projection =
            camera_conformance_view_projection(CameraConformancePose::default(), 16.0 / 9.0);
        let projected = axis_landmarks().map(|landmark| {
            (
                landmark.label,
                project_world_point(view_projection, landmark.center)
                    .unwrap()
                    .ndc,
            )
        });
        let ndc = |label| {
            projected
                .iter()
                .find_map(|(candidate, ndc)| (*candidate == label).then_some(*ndc))
                .unwrap()
        };

        // Initial camera right is -X, so world +X appears on screen-left.
        assert!(ndc("+X").x < 0.0);
        assert!(ndc("-X").x > 0.0);
        assert!(ndc("+Y").y > 0.0);
        assert!(ndc("-Y").y < 0.0);
        assert!(ndc("-Z").z < ndc("+Z").z);
        for (_, point) in projected {
            assert!((-1.0..=1.0).contains(&point.z));
        }
    }

    #[test]
    fn picking_rays_through_projected_landmarks_return_to_the_same_world_points() {
        let view_projection =
            camera_conformance_view_projection(CameraConformancePose::default(), 16.0 / 9.0);
        for landmark in axis_landmarks() {
            let projected = project_world_point(view_projection, landmark.center).unwrap();
            let ray =
                picking_ray_from_ndc(view_projection, [projected.ndc.x, projected.ndc.y]).unwrap();
            let along_ray = (landmark.center - ray.origin).dot(ray.direction);
            assert!(
                along_ray > 0.0,
                "{} must be in front of its ray",
                landmark.label
            );
            let closest = ray.origin + ray.direction * along_ray;
            assert!(
                (closest - landmark.center).length() < 0.000_2,
                "{} ray missed: closest={closest:?}",
                landmark.label
            );
        }
    }

    #[test]
    fn picking_rejects_a_noninvertible_projection() {
        assert_eq!(picking_ray_from_ndc(Mat4::ZERO, [0.0, 0.0]), None);
    }

    #[test]
    fn every_pipeline_accepts_every_fixture_mesh() {
        let material = MaterialDefinition::solid_color(
            MaterialDefinitionId::new("orientation-fixture-material").unwrap(),
            Color::rgb(1.0, 1.0, 1.0),
        );
        for cull_mode in cull_modes() {
            let pipeline = conformance_pipeline(cull_mode);
            assert_eq!(pipeline.render_state.cull_mode, cull_mode);
            for case in fixture_cases() {
                let supplied_uvs = case.mesh.texture_coordinates.clone();
                pipeline
                    .validate_draw_contract(&material, &case.mesh)
                    .unwrap_or_else(|error| {
                        panic!("case `{}` must satisfy {cull_mode:?}: {error}", case.label)
                    });
                assert_eq!(case.mesh.texture_coordinates, supplied_uvs);
            }
        }
    }

    #[test]
    fn shared_layout_retains_every_case_cull_pair_and_determinant() {
        let cells = fixture_layout(1200, 800);
        assert_eq!(cells.len(), 12);
        for (index, cell) in cells.into_iter().enumerate() {
            assert_eq!(cell.case_index, index / 3);
            assert_eq!(cell.cull_index, index % 3);
            let source = fixture_cases()[cell.case_index].instance;
            assert_eq!(
                (cell.instance.scale[0] * cell.instance.scale[1]).signum(),
                (source.scale[0] * source.scale[1]).signum()
            );
        }
    }

    fn triangle_signs(mesh: &Mesh) -> Vec<f32> {
        mesh.positions
            .chunks_exact(3)
            .map(|triangle| {
                let a = triangle[0];
                let b = triangle[1];
                let c = triangle[2];
                (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
            })
            .collect()
    }

    fn pixel(rgba8: &[u8], x: u32, y: u32) -> [u8; 4] {
        let offset = ((y * DIRECTIONAL_ATLAS_WIDTH + x) * 4) as usize;
        rgba8[offset..offset + 4].try_into().unwrap()
    }

    fn assert_vec3_close(actual: Vec3, expected: Vec3) {
        assert!(
            (actual - expected).length() < 0.000_1,
            "expected {expected:?}, received {actual:?}"
        );
    }
}
