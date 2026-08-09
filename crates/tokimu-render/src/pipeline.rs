use crate::{
    MaterialDefinition, Mesh, PipelineHandle, ShaderDiagnosticStage,
    ShaderMaterialCompatibilityError, ShaderMeshCompatibilityError, ShaderModuleDefinition,
    ShaderModuleValidationError,
};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PipelineKind {
    #[default]
    SolidColor2d,
    Texture2d,
    LitColor3d,
    /// A generic textured 3D mesh pipeline. It requires caller-supplied UVs
    /// and preserves the existing `Texture2d` derived-coordinate behavior.
    Textured3d,
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
    /// Add source color and alpha to the existing target color and alpha.
    ///
    /// This is useful for emissive overlays such as visualizer traces. It is a
    /// pipeline policy rather than material data because the destination color
    /// participates in the result.
    Additive,
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

/// A pipeline declaration that cannot be submitted to a renderer backend.
///
/// This validates provider-neutral declaration facts only. Backend-specific WGSL
/// compilation and capability validation remain renderer adapter work.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PipelineValidationError {
    #[error("custom WGSL pipeline `{label}` is missing shader source")]
    MissingCustomShaderSource { label: String },
    #[error("pipeline `{label}` has an empty {stage} entry point")]
    EmptyEntryPoint { label: String, stage: &'static str },
    #[error("pipeline `{label}` has an invalid render state: {source}")]
    InvalidRenderState {
        label: String,
        #[source]
        source: PipelineRenderStateError,
    },
    #[error("pipeline `{label}` has an invalid shader module: {source}")]
    InvalidShaderModule {
        label: String,
        #[source]
        source: ShaderModuleValidationError,
    },
}

impl PipelineValidationError {
    /// Identifies pipeline declaration validation as the owning boundary.
    pub const fn stage(&self) -> ShaderDiagnosticStage {
        ShaderDiagnosticStage::PipelineValidation
    }
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
            PipelineKind::Textured3d => Some(default_textured_3d_shader_source()),
            PipelineKind::CustomWgsl2d => None,
        }
    }

    pub const fn default_render_state(self) -> PipelineRenderState {
        match self {
            PipelineKind::LitColor3d | PipelineKind::Textured3d => {
                PipelineRenderState::depth_writing_3d()
            }
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

pub fn default_textured_3d_shader_source() -> &'static str {
    r#"
@group(0) @binding(0) var<uniform> material_color: vec4<f32>;
@group(0) @binding(1) var material_texture: texture_2d<f32>;
@group(0) @binding(2) var material_sampler: sampler;
struct InstanceParams { translation: vec2<f32>, scale: vec2<f32>, rotation: vec2<f32>, padding: vec2<f32>, };
@group(1) @binding(0) var<uniform> instance_params: InstanceParams;
@group(2) @binding(0) var<uniform> camera_params: mat4x4<f32>;
struct VertexOutput { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, };
@vertex fn vs_main(@location(0) position: vec3<f32>, @location(1) _normal: vec3<f32>, @location(2) uv: vec2<f32>) -> VertexOutput {
    let scaled = position.xy * instance_params.scale;
    let rotated = vec2<f32>((scaled.x * instance_params.rotation.y) - (scaled.y * instance_params.rotation.x), (scaled.x * instance_params.rotation.x) + (scaled.y * instance_params.rotation.y));
    var output: VertexOutput;
    output.position = camera_params * vec4<f32>(rotated.x + instance_params.translation.x, rotated.y + instance_params.translation.y, position.z, 1.0);
    output.uv = uv;
    return output;
}
@fragment fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(material_texture, material_sampler, uv) * material_color;
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
    shader_module: Option<ShaderModuleDefinition>,
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
            shader_module: None,
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
            shader_module: None,
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
            shader_module: None,
        }
    }

    /// Creates a custom WGSL pipeline from a validated semantic shader module.
    ///
    /// The module remains provider-neutral; this compatibility declaration is
    /// what current renderer adapters consume to compile and register a native
    /// pipeline.
    pub fn custom_wgsl_module(
        label: impl Into<String>,
        shader_module: ShaderModuleDefinition,
    ) -> Result<Self, ShaderModuleValidationError> {
        shader_module.validate()?;

        Ok(Self {
            label: label.into(),
            kind: PipelineKind::CustomWgsl2d,
            shader_source: Some(shader_module.source.clone()),
            vertex_entry_point: shader_module.vertex_entry_point.clone(),
            fragment_entry_point: shader_module.fragment_entry_point.clone(),
            render_state: PipelineKind::CustomWgsl2d.default_render_state(),
            shader_module: Some(shader_module),
        })
    }

    pub fn with_render_state(
        mut self,
        render_state: PipelineRenderState,
    ) -> Result<Self, PipelineRenderStateError> {
        render_state.validate()?;
        self.render_state = render_state;
        Ok(self)
    }

    /// Produces a provider-neutral shader declaration for this pipeline.
    ///
    /// The legacy public WGSL fields remain the execution compatibility path
    /// during this transition. New callers can validate the resulting semantic
    /// module before any renderer adapter compiles it.
    pub fn shader_module_definition(
        &self,
    ) -> Result<ShaderModuleDefinition, ShaderModuleValidationError> {
        if let Some(shader_module) = &self.shader_module {
            return Ok(shader_module.clone());
        }
        if self.kind != PipelineKind::CustomWgsl2d {
            return ShaderModuleDefinition::built_in(self.kind);
        }

        ShaderModuleDefinition::new(
            self.label.clone(),
            self.shader_source.clone().unwrap_or_default(),
            self.vertex_entry_point.clone(),
            self.fragment_entry_point.clone(),
            Vec::new(),
            Vec::new(),
        )
    }

    /// Returns the explicitly supplied semantic shader-module identity, when
    /// this pipeline was constructed from one. Built-in and legacy custom
    /// pipelines retain their pipeline label as the adapter diagnostic label.
    pub fn shader_module_label(&self) -> Option<&str> {
        self.shader_module
            .as_ref()
            .map(|shader_module| shader_module.label.as_str())
    }

    /// A diagnostic label that keeps semantic module and entry-point identity
    /// visible to a renderer adapter without exposing backend-native objects.
    pub fn backend_shader_label(&self) -> String {
        let module = self.shader_module_label().unwrap_or(&self.label);
        format!(
            "tokimu shader module `{module}` [vertex `{}`, fragment `{}`]",
            self.vertex_entry_point, self.fragment_entry_point
        )
    }

    /// Validates that material-backed shader bindings are compatible before a
    /// draw reaches a renderer backend.
    pub fn validate_material_definition(
        &self,
        material: &MaterialDefinition,
    ) -> Result<(), ShaderMaterialCompatibilityError> {
        self.shader_module_definition()?
            .validate_material_definition(material)
    }

    /// Validates the material and mesh facts required by this pipeline's shader
    /// before a caller submits an execution-ready draw to a renderer backend.
    pub fn validate_draw_contract(
        &self,
        material: &MaterialDefinition,
        mesh: &Mesh,
    ) -> Result<(), PipelineDrawContractError> {
        self.validate()?;
        let shader_module = self.shader_module_definition()?;
        shader_module.validate_material_definition(material)?;
        shader_module.validate_mesh(mesh)?;
        Ok(())
    }

    /// Validates the provider-neutral declaration before backend submission.
    ///
    /// All current built-in pipelines share the material binding schema of a
    /// color, texture, and sampler. A material without a texture is compatible:
    /// renderer adapters bind a deterministic white fallback texture.
    pub fn validate(&self) -> Result<(), PipelineValidationError> {
        self.render_state.validate().map_err(|source| {
            PipelineValidationError::InvalidRenderState {
                label: self.label.clone(),
                source,
            }
        })?;

        if self.vertex_entry_point.trim().is_empty() {
            return Err(PipelineValidationError::EmptyEntryPoint {
                label: self.label.clone(),
                stage: "vertex",
            });
        }
        if self.fragment_entry_point.trim().is_empty() {
            return Err(PipelineValidationError::EmptyEntryPoint {
                label: self.label.clone(),
                stage: "fragment",
            });
        }
        if self.kind == PipelineKind::CustomWgsl2d
            && self
                .shader_source
                .as_deref()
                .is_none_or(|source| source.trim().is_empty())
        {
            return Err(PipelineValidationError::MissingCustomShaderSource {
                label: self.label.clone(),
            });
        }

        if let Some(shader_module) = &self.shader_module {
            shader_module.validate().map_err(|source| {
                PipelineValidationError::InvalidShaderModule {
                    label: self.label.clone(),
                    source,
                }
            })?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PipelineDrawContractError {
    #[error("invalid pipeline declaration: {0}")]
    InvalidPipeline(#[from] PipelineValidationError),
    #[error("incompatible shader material contract: {0}")]
    Material(#[from] ShaderMaterialCompatibilityError),
    #[error("incompatible shader mesh contract: {0}")]
    Mesh(#[from] ShaderMeshCompatibilityError),
    #[error("invalid provider-neutral shader module: {0}")]
    ShaderModule(#[from] ShaderModuleValidationError),
}

impl PipelineDrawContractError {
    /// Identifies material/mesh compatibility as the owning boundary.
    pub const fn stage(&self) -> ShaderDiagnosticStage {
        match self {
            Self::InvalidPipeline(error) => error.stage(),
            Self::Material(_) | Self::Mesh(_) => ShaderDiagnosticStage::DrawContractValidation,
            Self::ShaderModule(error) => error.stage(),
        }
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
            PipelineKind::Textured3d.default_entry_points(),
            ("vs_main", "fs_main")
        );
        assert_eq!(
            PipelineKind::CustomWgsl2d.default_entry_points(),
            ("vs_main", "fs_main")
        );
        assert!(PipelineKind::SolidColor2d.default_shader_source().is_some());
        assert!(PipelineKind::LitColor3d.default_shader_source().is_some());
        assert!(PipelineKind::Textured3d.default_shader_source().is_some());
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
    fn creates_default_textured_3d_pipeline() {
        let pipeline = Pipeline::new("textured", PipelineKind::Textured3d);

        assert_eq!(pipeline.kind, PipelineKind::Textured3d);
        assert_eq!(
            pipeline.shader_source.as_deref(),
            Some(default_textured_3d_shader_source())
        );
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
    fn creates_a_custom_pipeline_from_a_semantic_shader_module() {
        let module = ShaderModuleDefinition::new(
            "inspection-shader",
            "@vertex fn main_vs() -> @builtin(position) vec4<f32> { return vec4<f32>(); }\n@fragment fn main_fs() -> @location(0) vec4<f32> { return vec4<f32>(); }",
            "main_vs",
            "main_fs",
            vec![],
            vec![],
        )
        .expect("shader module must be valid");
        let pipeline = Pipeline::custom_wgsl_module("inspection-pipeline", module)
            .expect("pipeline must retain a valid shader module");

        assert_eq!(pipeline.shader_source.as_deref(), Some("@vertex fn main_vs() -> @builtin(position) vec4<f32> { return vec4<f32>(); }\n@fragment fn main_fs() -> @location(0) vec4<f32> { return vec4<f32>(); }"));
        assert_eq!(pipeline.vertex_entry_point, "main_vs");
        assert_eq!(
            pipeline
                .shader_module_definition()
                .expect("module must remain available")
                .label,
            "inspection-shader"
        );
        assert_eq!(pipeline.shader_module_label(), Some("inspection-shader"));
        assert_eq!(
            pipeline.backend_shader_label(),
            "tokimu shader module `inspection-shader` [vertex `main_vs`, fragment `main_fs`]"
        );
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
    fn retains_additive_blend_as_a_provider_neutral_pipeline_policy() {
        let state = PipelineRenderState {
            blend: BlendMode::Additive,
            ..PipelineRenderState::painter_ordered_2d()
        };
        let pipeline = Pipeline::new("additive-overlay", PipelineKind::SolidColor2d)
            .with_render_state(state)
            .expect("additive two-dimensional state is valid");

        assert_eq!(pipeline.render_state.blend, BlendMode::Additive);
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
    fn rejects_custom_pipelines_without_a_shader_before_backend_submission() {
        let pipeline = Pipeline::new("missing-custom-source", PipelineKind::CustomWgsl2d);

        let error = pipeline
            .validate()
            .expect_err("missing source must reject the pipeline");
        assert_eq!(
            error,
            PipelineValidationError::MissingCustomShaderSource {
                label: "missing-custom-source".to_owned(),
            }
        );
        assert_eq!(error.stage(), ShaderDiagnosticStage::PipelineValidation);
    }

    #[test]
    fn rejects_empty_stage_entry_points_before_backend_submission() {
        let mut pipeline = Pipeline::new("empty-entry", PipelineKind::SolidColor2d);
        pipeline.fragment_entry_point.clear();

        assert_eq!(
            pipeline.validate(),
            Err(PipelineValidationError::EmptyEntryPoint {
                label: "empty-entry".to_owned(),
                stage: "fragment",
            })
        );
    }

    #[test]
    fn exposes_a_provider_neutral_builtin_shader_module() {
        let pipeline = Pipeline::new("lit", PipelineKind::LitColor3d);
        let shader = pipeline
            .shader_module_definition()
            .expect("built-in pipeline declaration must be valid");

        assert_eq!(shader.vertex_inputs.len(), 2);
        assert_eq!(shader.bindings.len(), 5);
    }

    #[test]
    fn rejects_incompatible_meshes_before_draw_submission() {
        let pipeline = Pipeline::new("lit", PipelineKind::LitColor3d);
        let material = MaterialDefinition::solid_color(
            crate::MaterialDefinitionId::new("surface").expect("valid material id"),
            crate::Color::rgb(1.0, 1.0, 1.0),
        );
        let mesh = crate::Mesh::new(vec![[0.0, 0.0, 0.0]], vec![]);

        let error = pipeline
            .validate_draw_contract(&material, &mesh)
            .expect_err("missing normals must reject the draw contract");
        assert!(matches!(
            error,
            PipelineDrawContractError::Mesh(
                ShaderMeshCompatibilityError::MissingVertexInput { .. }
            )
        ));
        assert_eq!(error.stage(), ShaderDiagnosticStage::DrawContractValidation);
    }

    #[test]
    fn textured_3d_draw_contract_rejects_a_mesh_without_uvs() {
        let pipeline = Pipeline::new("textured", PipelineKind::Textured3d);
        let material = MaterialDefinition::solid_color(
            crate::MaterialDefinitionId::new("surface").expect("valid material id"),
            crate::Color::rgb(1.0, 1.0, 1.0),
        );

        let error = pipeline
            .validate_draw_contract(&material, &crate::Mesh::triangle())
            .expect_err("textured meshes without UVs must reject before backend submission");
        assert!(matches!(
            error,
            PipelineDrawContractError::Mesh(ShaderMeshCompatibilityError::MissingVertexInput {
                semantic: ShaderVertexSemantic::TextureCoordinate2,
                ..
            })
        ));
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
