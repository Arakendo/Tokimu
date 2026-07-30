use crate::PipelineHandle;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PipelineKind {
    #[default]
    SolidColor2d,
    Texture2d,
    LitColor3d,
    CustomWgsl2d,
}

/// Provider-neutral blending policy for a render pipeline.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlendMode {
    /// Write each fragment without blending it with the existing target color.
    Opaque,
    /// Apply conventional source-alpha blending.
    #[default]
    AlphaBlend,
}

/// Provider-neutral depth-test policy for a render pipeline.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DepthTest {
    /// Do not attach a depth target to this pipeline.
    Disabled,
    /// Render fragments whose depth is less than or equal to the stored value.
    #[default]
    LessEqual,
}

/// Provider-neutral face-culling policy for a render pipeline.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CullMode {
    #[default]
    None,
    Front,
    Back,
}

/// The color channels a pipeline may write to its target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColorWriteMask {
    pub red: bool,
    pub green: bool,
    pub blue: bool,
    pub alpha: bool,
}

impl ColorWriteMask {
    pub const ALL: Self = Self {
        red: true,
        green: true,
        blue: true,
        alpha: true,
    };

    pub const NONE: Self = Self {
        red: false,
        green: false,
        blue: false,
        alpha: false,
    };
}

impl Default for ColorWriteMask {
    fn default() -> Self {
        Self::ALL
    }
}

/// Render state selected by a draw pipeline rather than by material data.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PipelineRenderState {
    pub blend: BlendMode,
    pub depth_test: DepthTest,
    pub depth_write: bool,
    pub cull_mode: CullMode,
    pub color_write: ColorWriteMask,
}

impl PipelineRenderState {
    pub const fn painter_ordered_2d() -> Self {
        Self {
            blend: BlendMode::AlphaBlend,
            depth_test: DepthTest::LessEqual,
            depth_write: false,
            cull_mode: CullMode::None,
            color_write: ColorWriteMask::ALL,
        }
    }

    pub const fn depth_writing_3d() -> Self {
        Self {
            depth_write: true,
            ..Self::painter_ordered_2d()
        }
    }

    pub fn validate(self) -> Result<(), PipelineRenderStateError> {
        if self.depth_test == DepthTest::Disabled && self.depth_write {
            return Err(PipelineRenderStateError::DepthWriteWithoutDepthTest);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PipelineRenderStateError {
    #[error("a pipeline cannot write depth while its depth test is disabled")]
    DepthWriteWithoutDepthTest,
}

impl PipelineKind {
    pub fn default_entry_points(self) -> (&'static str, &'static str) {
        ("vs_main", "fs_main")
    }

    pub fn default_shader_source(self) -> Option<&'static str> {
        match self {
            PipelineKind::SolidColor2d => Some(default_2d_shader_source()),
            PipelineKind::Texture2d => Some(default_texture_2d_shader_source()),
            PipelineKind::LitColor3d => Some(default_lit_3d_shader_source()),
            PipelineKind::CustomWgsl2d => None,
        }
    }

    pub const fn default_render_state(self) -> PipelineRenderState {
        match self {
            PipelineKind::LitColor3d => PipelineRenderState::depth_writing_3d(),
            PipelineKind::SolidColor2d | PipelineKind::Texture2d | PipelineKind::CustomWgsl2d => {
                PipelineRenderState::painter_ordered_2d()
            }
        }
    }
}

pub fn default_texture_2d_shader_source() -> &'static str {
    r#"
@group(0) @binding(0) var<uniform> material_color: vec4<f32>;
@group(0) @binding(1) var material_texture: texture_2d<f32>;
@group(0) @binding(2) var material_sampler: sampler;
struct InstanceParams { translation: vec2<f32>, scale: vec2<f32>, rotation: vec2<f32>, padding: vec2<f32>, };
@group(1) @binding(0) var<uniform> instance_params: InstanceParams;
@group(2) @binding(0) var<uniform> camera_params: mat4x4<f32>;
struct VertexOutput { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, };
@vertex fn vs_main(@location(0) position: vec3<f32>) -> VertexOutput {
    let scaled = position.xy * instance_params.scale;
    let rotated = vec2<f32>((scaled.x * instance_params.rotation.y) - (scaled.y * instance_params.rotation.x), (scaled.x * instance_params.rotation.x) + (scaled.y * instance_params.rotation.y));
    var output: VertexOutput;
    output.position = camera_params * vec4<f32>(rotated.x + instance_params.translation.x, rotated.y + instance_params.translation.y, position.z, 1.0);
    output.uv = vec2<f32>(position.x + 0.5, 0.5 - position.y);
    return output;
}
@fragment fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(material_texture, material_sampler, uv) * material_color;
}
"#.trim()
}

pub fn default_2d_shader_source() -> &'static str {
    r#"
@group(0) @binding(0)
var<uniform> material_color: vec4<f32>;

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

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    let scaled_position = position.xy * instance_params.scale;
    let rotated_position = vec2<f32>(
        (scaled_position.x * instance_params.rotation.y) - (scaled_position.y * instance_params.rotation.x),
        (scaled_position.x * instance_params.rotation.x) + (scaled_position.y * instance_params.rotation.y)
    );
    let instance_position = rotated_position + instance_params.translation;
    let world_position = vec4<f32>(instance_position.x, instance_position.y, position.z, 1.0);
    return camera_params * world_position;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return material_color;
}
"#
    .trim()
}

pub fn default_lit_3d_shader_source() -> &'static str {
    r#"
struct MaterialColor {
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> material_color: MaterialColor;

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
    let scaled_position = position.xy * instance_params.scale;
    let rotated_position = vec2<f32>(
        (scaled_position.x * instance_params.rotation.y) - (scaled_position.y * instance_params.rotation.x),
        (scaled_position.x * instance_params.rotation.x) + (scaled_position.y * instance_params.rotation.y)
    );
    let instance_position = rotated_position + instance_params.translation;
    var output: VertexOutput;
    output.position = camera_params * vec4<f32>(instance_position.x, instance_position.y, position.z, 1.0);
    output.normal = normal;
    return output;
}

@fragment
fn fs_main(@location(0) normal: vec3<f32>) -> @location(0) vec4<f32> {
    let light_direction = normalize(vec3<f32>(0.35, 0.85, 0.45));
    let diffuse = max(dot(normalize(normal), light_direction), 0.0);
    let lighting = 0.20 + diffuse * 0.80;
    return vec4<f32>(material_color.color.rgb * lighting, material_color.color.a);
}
"#
    .trim()
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Pipeline {
    pub label: String,
    pub kind: PipelineKind,
    pub shader_source: Option<String>,
    pub vertex_entry_point: String,
    pub fragment_entry_point: String,
    pub render_state: PipelineRenderState,
}

impl Pipeline {
    pub fn default_2d_shader_source() -> &'static str {
        default_2d_shader_source()
    }

    pub fn new(label: impl Into<String>, kind: PipelineKind) -> Self {
        let (vertex_entry_point, fragment_entry_point) = kind.default_entry_points();

        Self {
            label: label.into(),
            kind,
            shader_source: kind.default_shader_source().map(str::to_string),
            vertex_entry_point: vertex_entry_point.into(),
            fragment_entry_point: fragment_entry_point.into(),
            render_state: kind.default_render_state(),
        }
    }

    pub fn custom_wgsl(label: impl Into<String>, shader_source: impl Into<String>) -> Self {
        let (vertex_entry_point, fragment_entry_point) =
            PipelineKind::CustomWgsl2d.default_entry_points();

        Self {
            label: label.into(),
            kind: PipelineKind::CustomWgsl2d,
            shader_source: Some(shader_source.into()),
            vertex_entry_point: vertex_entry_point.into(),
            fragment_entry_point: fragment_entry_point.into(),
            render_state: PipelineKind::CustomWgsl2d.default_render_state(),
        }
    }

    pub fn custom_wgsl_with_entry_points(
        label: impl Into<String>,
        shader_source: impl Into<String>,
        vertex_entry_point: impl Into<String>,
        fragment_entry_point: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            kind: PipelineKind::CustomWgsl2d,
            shader_source: Some(shader_source.into()),
            vertex_entry_point: vertex_entry_point.into(),
            fragment_entry_point: fragment_entry_point.into(),
            render_state: PipelineKind::CustomWgsl2d.default_render_state(),
        }
    }

    pub fn with_render_state(
        mut self,
        render_state: PipelineRenderState,
    ) -> Result<Self, PipelineRenderStateError> {
        render_state.validate()?;
        self.render_state = render_state;
        Ok(self)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PipelineRegistry {
    next_handle: u64,
    handles_by_label: HashMap<String, PipelineHandle>,
}

impl PipelineRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, pipeline: &Pipeline) -> PipelineHandle {
        let handle = PipelineHandle(self.next_handle);
        self.next_handle += 1;
        self.handles_by_label.insert(pipeline.label.clone(), handle);
        handle
    }

    pub fn register_with_handle(&mut self, handle: PipelineHandle, pipeline: &Pipeline) {
        self.handles_by_label.insert(pipeline.label.clone(), handle);
    }

    pub fn handle_for_label(&self, label: &str) -> Option<PipelineHandle> {
        self.handles_by_label.get(label).copied()
    }

    pub fn label_count(&self) -> usize {
        self.handles_by_label.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_kind_defaults() {
        assert_eq!(
            PipelineKind::SolidColor2d.default_entry_points(),
            ("vs_main", "fs_main")
        );
        assert_eq!(
            PipelineKind::LitColor3d.default_entry_points(),
            ("vs_main", "fs_main")
        );
        assert_eq!(
            PipelineKind::CustomWgsl2d.default_entry_points(),
            ("vs_main", "fs_main")
        );
        assert!(PipelineKind::SolidColor2d.default_shader_source().is_some());
        assert!(PipelineKind::LitColor3d.default_shader_source().is_some());
        assert!(PipelineKind::CustomWgsl2d.default_shader_source().is_none());
    }

    #[test]
    fn exposes_the_default_2d_shader_source() {
        let shader_source = default_2d_shader_source();

        assert!(shader_source.contains("@vertex"));
        assert!(shader_source.contains("@fragment"));
        assert!(shader_source.contains("material_color"));
        assert!(shader_source.contains("vec3<f32>"));
    }

    #[test]
    fn creates_default_solid_color_pipeline() {
        let pipeline = Pipeline::new("solid", PipelineKind::SolidColor2d);

        assert_eq!(pipeline.label, "solid");
        assert_eq!(pipeline.kind, PipelineKind::SolidColor2d);
        assert_eq!(
            pipeline.shader_source.as_deref(),
            Some(default_2d_shader_source())
        );
        assert_eq!(pipeline.vertex_entry_point, "vs_main");
        assert_eq!(pipeline.fragment_entry_point, "fs_main");
        assert_eq!(
            pipeline.render_state,
            PipelineRenderState::painter_ordered_2d()
        );
    }

    #[test]
    fn creates_default_lit_3d_pipeline() {
        let pipeline = Pipeline::new("lit", PipelineKind::LitColor3d);

        assert_eq!(pipeline.label, "lit");
        assert_eq!(pipeline.kind, PipelineKind::LitColor3d);
        assert_eq!(
            pipeline.shader_source.as_deref(),
            Some(default_lit_3d_shader_source())
        );
        assert_eq!(pipeline.vertex_entry_point, "vs_main");
        assert_eq!(pipeline.fragment_entry_point, "fs_main");
        assert_eq!(
            pipeline.render_state,
            PipelineRenderState::depth_writing_3d()
        );
    }

    #[test]
    fn creates_custom_wgsl_pipeline() {
        let pipeline = Pipeline::custom_wgsl(
            "custom",
            "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(); }",
        );

        assert_eq!(pipeline.label, "custom");
        assert_eq!(pipeline.kind, PipelineKind::CustomWgsl2d);
        assert_eq!(
            pipeline.shader_source.as_deref(),
            Some("@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(); }")
        );
        assert_eq!(pipeline.vertex_entry_point, "vs_main");
        assert_eq!(pipeline.fragment_entry_point, "fs_main");
    }

    #[test]
    fn creates_custom_wgsl_pipeline_with_explicit_entry_points() {
        let pipeline = Pipeline::custom_wgsl_with_entry_points(
            "custom",
            "@vertex fn main_vs() -> @builtin(position) vec4<f32> { return vec4<f32>(); }",
            "main_vs",
            "main_fs",
        );

        assert_eq!(pipeline.vertex_entry_point, "main_vs");
        assert_eq!(pipeline.fragment_entry_point, "main_fs");
    }

    #[test]
    fn validates_explicit_render_state_without_involving_material_data() {
        let state = PipelineRenderState {
            blend: BlendMode::Opaque,
            depth_test: DepthTest::Disabled,
            depth_write: false,
            cull_mode: CullMode::Back,
            color_write: ColorWriteMask {
                alpha: false,
                ..ColorWriteMask::ALL
            },
        };
        let pipeline = Pipeline::new("opaque-backface", PipelineKind::SolidColor2d)
            .with_render_state(state)
            .expect("valid render state");

        assert_eq!(pipeline.render_state, state);
    }

    #[test]
    fn rejects_depth_writes_without_a_depth_test() {
        let error = Pipeline::new("invalid", PipelineKind::SolidColor2d)
            .with_render_state(PipelineRenderState {
                depth_test: DepthTest::Disabled,
                depth_write: true,
                ..PipelineRenderState::default()
            })
            .expect_err("invalid render state must be rejected");

        assert_eq!(error, PipelineRenderStateError::DepthWriteWithoutDepthTest);
    }

    #[test]
    fn registers_named_pipelines_and_resolves_handles() {
        let mut registry = PipelineRegistry::new();
        let solid = Pipeline::new("solid", PipelineKind::SolidColor2d);
        let lit = Pipeline::new("lit", PipelineKind::LitColor3d);

        let solid_handle = registry.register(&solid);
        let lit_handle = registry.register(&lit);

        assert_eq!(solid_handle, PipelineHandle(0));
        assert_eq!(lit_handle, PipelineHandle(1));
        assert_eq!(registry.handle_for_label("solid"), Some(solid_handle));
        assert_eq!(registry.handle_for_label("lit"), Some(lit_handle));
        assert_eq!(registry.label_count(), 2);
    }
}
