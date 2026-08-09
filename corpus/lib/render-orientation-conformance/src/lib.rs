//! Shared AR-0021 renderer-orientation conformance evidence.
//!
//! This corpus fixture deliberately keeps geometric winding, authored shading
//! normals, fragment facing, culling, and reflection compensation observable as
//! separate facts.

use tokimu_render::{
    BlendMode, ColorWriteMask, CullMode, DepthTest, Instance2d, Mesh, Pipeline,
    PipelineRenderState, ShaderBindingDeclaration, ShaderBindingSource, ShaderModuleDefinition,
    ShaderVertexInput, ShaderVertexSemantic,
};

pub const FRONT_COLOR: [f32; 4] = [0.12, 0.90, 0.34, 1.0];
pub const BACK_COLOR: [f32; 4] = [0.94, 0.16, 0.68, 1.0];

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
            ShaderBindingDeclaration::new(1, 0, ShaderBindingSource::InstanceTransform),
            ShaderBindingDeclaration::new(2, 0, ShaderBindingSource::Camera),
        ],
        vec![
            ShaderVertexInput::new(0, ShaderVertexSemantic::Position3),
            ShaderVertexInput::new(1, ShaderVertexSemantic::Normal3),
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
    let mut positions = vec![
        // Left: right-handed cross product points +Z.
        [-0.88, -0.38, 0.0],
        [-0.58, -0.38, 0.0],
        [-0.73, 0.38, 0.0],
        // Right: right-handed cross product points -Z.
        [0.58, -0.38, 0.0],
        [0.73, 0.38, 0.0],
        [0.88, -0.38, 0.0],
    ];
    if reverse_each_triangle {
        for triangle in positions.chunks_exact_mut(3) {
            triangle.swap(1, 2);
        }
    }

    // Intentionally identical on the CW and CCW triangles: lighting remains
    // observable without being allowed to stand in for geometric facing.
    Mesh::uniform_normal(positions, [0.0, 0.0, 1.0])
}

pub const CONFORMANCE_SHADER: &str = r#"
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
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOutput {
    let scaled = position.xy * instance_params.scale;
    let rotated = vec2<f32>(
        scaled.x * instance_params.rotation.y - scaled.y * instance_params.rotation.x,
        scaled.x * instance_params.rotation.x + scaled.y * instance_params.rotation.y,
    );
    var output: VertexOutput;
    output.position = camera_params * vec4<f32>(rotated + instance_params.translation, position.z, 1.0);
    output.normal = normal;
    return output;
}

@fragment
fn fs_main(
    input: VertexOutput,
    @builtin(front_facing) front_facing: bool,
) -> @location(0) vec4<f32> {
    let front_color = vec3<f32>(0.12, 0.90, 0.34);
    let back_color = vec3<f32>(0.94, 0.16, 0.68);
    let facing_color = select(back_color, front_color, front_facing);
    let normal_light = 0.35 + 0.65 * max(dot(normalize(input.normal), vec3<f32>(0.0, 0.0, 1.0)), 0.0);
    return vec4<f32>(facing_color * normal_light, 1.0);
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
            let transformed_signs = source_signs.map(|sign| (sign * determinant.signum()).signum());
            let expected = match case.expected_facing {
                ExpectedFacing::LeftFrontRightBack => [1.0, -1.0],
                ExpectedFacing::LeftBackRightFront => [-1.0, 1.0],
            };
            assert_eq!(transformed_signs, expected, "case `{}`", case.label);
        }
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
                pipeline
                    .validate_draw_contract(&material, &case.mesh)
                    .unwrap_or_else(|error| {
                        panic!("case `{}` must satisfy {cull_mode:?}: {error}", case.label)
                    });
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

    fn triangle_signs(mesh: &Mesh) -> [f32; 2] {
        let mut signs = [0.0; 2];
        for (triangle_index, triangle) in mesh.positions.chunks_exact(3).enumerate() {
            let a = triangle[0];
            let b = triangle[1];
            let c = triangle[2];
            signs[triangle_index] = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        }
        signs
    }
}
