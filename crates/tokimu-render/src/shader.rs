use crate::{MaterialDefinition, MaterialParameterKind, Mesh, PipelineKind};
use std::collections::BTreeSet;
use thiserror::Error;

/// Upper bounds for the initial hand-written WGSL authoring path. They prevent
/// accidental or hostile declarations from turning semantic validation into an
/// unbounded renderer request; they are not backend capability limits.
pub const MAX_SHADER_SOURCE_BYTES: usize = 1_048_576;
pub const MAX_SHADER_BINDINGS: usize = 64;
pub const MAX_SHADER_VERTEX_INPUTS: usize = 16;

/// A provider-neutral WGSL shader module declaration.
///
/// This is presentation semantics, not a backend shader object. Renderer
/// adapters compile `source` and own any resulting native shader handles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShaderModuleDefinition {
    pub label: String,
    pub source: String,
    pub vertex_entry_point: String,
    pub fragment_entry_point: String,
    pub bindings: Vec<ShaderBindingDeclaration>,
    pub vertex_inputs: Vec<ShaderVertexInput>,
}

impl ShaderModuleDefinition {
    pub fn new(
        label: impl Into<String>,
        source: impl Into<String>,
        vertex_entry_point: impl Into<String>,
        fragment_entry_point: impl Into<String>,
        bindings: Vec<ShaderBindingDeclaration>,
        vertex_inputs: Vec<ShaderVertexInput>,
    ) -> Result<Self, ShaderModuleValidationError> {
        let definition = Self {
            label: label.into(),
            source: source.into(),
            vertex_entry_point: vertex_entry_point.into(),
            fragment_entry_point: fragment_entry_point.into(),
            bindings,
            vertex_inputs,
        };
        definition.validate()?;
        Ok(definition)
    }

    /// Describes the current built-in pipeline contracts without exposing a
    /// renderer-native shader module.
    pub fn built_in(kind: PipelineKind) -> Result<Self, ShaderModuleValidationError> {
        let (vertex_entry_point, fragment_entry_point) = kind.default_entry_points();
        let source = kind.default_shader_source().ok_or_else(|| {
            ShaderModuleValidationError::MissingSource {
                label: format!("{kind:?}"),
            }
        })?;

        let mut vertex_inputs = vec![ShaderVertexInput::new(0, ShaderVertexSemantic::Position3)];
        if kind == PipelineKind::LitColor3d {
            vertex_inputs.push(ShaderVertexInput::new(1, ShaderVertexSemantic::Normal3));
        }

        Self::new(
            format!("builtin-{kind:?}"),
            source,
            vertex_entry_point,
            fragment_entry_point,
            standard_bindings(),
            vertex_inputs,
        )
    }

    pub fn validate(&self) -> Result<(), ShaderModuleValidationError> {
        validate_identifier("shader module", &self.label)?;
        if self.source.trim().is_empty() {
            return Err(ShaderModuleValidationError::MissingSource {
                label: self.label.clone(),
            });
        }
        if self.source.len() > MAX_SHADER_SOURCE_BYTES {
            return Err(ShaderModuleValidationError::SourceTooLarge {
                label: self.label.clone(),
                bytes: self.source.len(),
                maximum: MAX_SHADER_SOURCE_BYTES,
            });
        }
        validate_entry_point(&self.label, "vertex", &self.vertex_entry_point)?;
        validate_entry_point(&self.label, "fragment", &self.fragment_entry_point)?;

        if self.bindings.len() > MAX_SHADER_BINDINGS {
            return Err(ShaderModuleValidationError::TooManyBindings {
                label: self.label.clone(),
                count: self.bindings.len(),
                maximum: MAX_SHADER_BINDINGS,
            });
        }
        if self.vertex_inputs.len() > MAX_SHADER_VERTEX_INPUTS {
            return Err(ShaderModuleValidationError::TooManyVertexInputs {
                label: self.label.clone(),
                count: self.vertex_inputs.len(),
                maximum: MAX_SHADER_VERTEX_INPUTS,
            });
        }

        let mut binding_slots = BTreeSet::new();
        for binding in &self.bindings {
            binding.validate()?;
            if !binding_slots.insert((binding.group, binding.binding)) {
                return Err(ShaderModuleValidationError::DuplicateBindingSlot {
                    label: self.label.clone(),
                    group: binding.group,
                    binding: binding.binding,
                });
            }
        }

        let mut vertex_locations = BTreeSet::new();
        for input in &self.vertex_inputs {
            if !vertex_locations.insert(input.location) {
                return Err(ShaderModuleValidationError::DuplicateVertexLocation {
                    label: self.label.clone(),
                    location: input.location,
                });
            }
        }

        Ok(())
    }

    /// Ensures material-backed bindings name parameters with compatible kinds.
    ///
    /// Instance and camera bindings are renderer contracts, while material
    /// bindings must be checked against the selected material definition before
    /// a draw reaches a backend.
    pub fn validate_material_definition(
        &self,
        material: &MaterialDefinition,
    ) -> Result<(), ShaderMaterialCompatibilityError> {
        self.validate()
            .map_err(ShaderMaterialCompatibilityError::InvalidShaderModule)?;

        for binding in &self.bindings {
            let (parameter, expected_kind) = match &binding.source {
                ShaderBindingSource::MaterialParameter { parameter, kind } => (parameter, *kind),
                ShaderBindingSource::MaterialSampler { texture_parameter } => {
                    (texture_parameter, MaterialParameterKind::Texture)
                }
                ShaderBindingSource::InstanceTransform | ShaderBindingSource::Camera => continue,
            };

            let declaration = material.parameter(parameter).ok_or_else(|| {
                ShaderMaterialCompatibilityError::MissingMaterialParameter {
                    shader: self.label.clone(),
                    material: material.id.as_str().to_owned(),
                    parameter: parameter.clone(),
                }
            })?;
            if declaration.kind != expected_kind {
                return Err(
                    ShaderMaterialCompatibilityError::MaterialParameterKindMismatch {
                        shader: self.label.clone(),
                        material: material.id.as_str().to_owned(),
                        parameter: parameter.clone(),
                        expected: expected_kind,
                        actual: declaration.kind,
                    },
                );
            }
        }

        Ok(())
    }

    /// Ensures a mesh supplies every vertex semantic declared by this module.
    ///
    /// The current mesh contract supports position and normal streams only.
    /// More vertex semantics must be admitted deliberately rather than silently
    /// accepting a shader that the renderer cannot feed.
    pub fn validate_mesh(&self, mesh: &Mesh) -> Result<(), ShaderMeshCompatibilityError> {
        self.validate()
            .map_err(ShaderMeshCompatibilityError::InvalidShaderModule)?;

        for input in &self.vertex_inputs {
            let supplied = match input.semantic {
                ShaderVertexSemantic::Position3 => !mesh.positions.is_empty(),
                ShaderVertexSemantic::Normal3 => {
                    !mesh.normals.is_empty() && mesh.normals.len() == mesh.positions.len()
                }
            };
            if !supplied {
                return Err(ShaderMeshCompatibilityError::MissingVertexInput {
                    shader: self.label.clone(),
                    location: input.location,
                    semantic: input.semantic,
                });
            }
        }

        Ok(())
    }
}

/// One shader-visible resource binding. Slots are intentionally semantic data,
/// not `wgpu` binding-layout objects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShaderBindingDeclaration {
    pub group: u32,
    pub binding: u32,
    pub source: ShaderBindingSource,
}

impl ShaderBindingDeclaration {
    pub const fn new(group: u32, binding: u32, source: ShaderBindingSource) -> Self {
        Self {
            group,
            binding,
            source,
        }
    }

    fn validate(&self) -> Result<(), ShaderModuleValidationError> {
        match &self.source {
            ShaderBindingSource::MaterialParameter { parameter, .. } => {
                validate_identifier("material parameter", parameter)
            }
            ShaderBindingSource::MaterialSampler { texture_parameter } => {
                validate_identifier("texture parameter", texture_parameter)
            }
            ShaderBindingSource::InstanceTransform | ShaderBindingSource::Camera => Ok(()),
        }
    }
}

/// The semantic source of a shader binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShaderBindingSource {
    MaterialParameter {
        parameter: String,
        kind: MaterialParameterKind,
    },
    MaterialSampler {
        texture_parameter: String,
    },
    InstanceTransform,
    Camera,
}

/// A vertex attribute required by a shader module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderVertexInput {
    pub location: u32,
    pub semantic: ShaderVertexSemantic,
}

impl ShaderVertexInput {
    pub const fn new(location: u32, semantic: ShaderVertexSemantic) -> Self {
        Self { location, semantic }
    }
}

/// Bounded semantic vertex inputs admitted by the current renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShaderVertexSemantic {
    Position3,
    Normal3,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ShaderModuleValidationError {
    #[error("{kind} identifier `{value}` is empty or contains unsupported characters")]
    InvalidIdentifier { kind: &'static str, value: String },
    #[error("shader module `{label}` is missing WGSL source")]
    MissingSource { label: String },
    #[error("shader module `{label}` has {bytes} bytes of WGSL; the maximum is {maximum}")]
    SourceTooLarge {
        label: String,
        bytes: usize,
        maximum: usize,
    },
    #[error("shader module `{label}` has an empty {stage} entry point")]
    EmptyEntryPoint { label: String, stage: &'static str },
    #[error(
        "shader module `{label}` declares binding group {group}, binding {binding} more than once"
    )]
    DuplicateBindingSlot {
        label: String,
        group: u32,
        binding: u32,
    },
    #[error("shader module `{label}` declares vertex location {location} more than once")]
    DuplicateVertexLocation { label: String, location: u32 },
    #[error("shader module `{label}` declares {count} bindings; the maximum is {maximum}")]
    TooManyBindings {
        label: String,
        count: usize,
        maximum: usize,
    },
    #[error("shader module `{label}` declares {count} vertex inputs; the maximum is {maximum}")]
    TooManyVertexInputs {
        label: String,
        count: usize,
        maximum: usize,
    },
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ShaderMaterialCompatibilityError {
    #[error("shader module declaration is invalid: {0}")]
    InvalidShaderModule(#[from] ShaderModuleValidationError),
    #[error("shader module `{shader}` requires missing parameter `{parameter}` from material `{material}`")]
    MissingMaterialParameter {
        shader: String,
        material: String,
        parameter: String,
    },
    #[error("shader module `{shader}` requires {expected:?} parameter `{parameter}` but material `{material}` declares {actual:?}")]
    MaterialParameterKindMismatch {
        shader: String,
        material: String,
        parameter: String,
        expected: MaterialParameterKind,
        actual: MaterialParameterKind,
    },
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ShaderMeshCompatibilityError {
    #[error("shader module declaration is invalid: {0}")]
    InvalidShaderModule(#[from] ShaderModuleValidationError),
    #[error("shader module `{shader}` requires {semantic:?} at vertex location {location}, but the mesh cannot supply it")]
    MissingVertexInput {
        shader: String,
        location: u32,
        semantic: ShaderVertexSemantic,
    },
}

fn standard_bindings() -> Vec<ShaderBindingDeclaration> {
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
    ]
}

fn validate_identifier(kind: &'static str, value: &str) -> Result<(), ShaderModuleValidationError> {
    if value.is_empty()
        || !value.bytes().all(|character| {
            character.is_ascii_alphanumeric() || character == b'_' || character == b'-'
        })
    {
        return Err(ShaderModuleValidationError::InvalidIdentifier {
            kind,
            value: value.to_owned(),
        });
    }
    Ok(())
}

fn validate_entry_point(
    label: &str,
    stage: &'static str,
    entry_point: &str,
) -> Result<(), ShaderModuleValidationError> {
    let mut characters = entry_point.bytes();
    let Some(first) = characters.next() else {
        return Err(ShaderModuleValidationError::EmptyEntryPoint {
            label: label.to_owned(),
            stage,
        });
    };

    // Entry points are passed directly to WGSL. Keep this semantic declaration
    // within the portable WGSL identifier subset instead of asking backends to
    // diagnose punctuation or whitespace that no backend can interpret.
    if !(first.is_ascii_alphabetic() || first == b'_')
        || !characters.all(|character| character.is_ascii_alphanumeric() || character == b'_')
    {
        return Err(ShaderModuleValidationError::InvalidIdentifier {
            kind: "shader entry point",
            value: entry_point.to_owned(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, MaterialDefinitionId};

    #[test]
    fn describes_the_builtin_lit_pipeline_without_backend_types() {
        let shader = ShaderModuleDefinition::built_in(PipelineKind::LitColor3d)
            .expect("built-in shader declaration must be valid");

        assert_eq!(shader.vertex_entry_point, "vs_main");
        assert_eq!(shader.fragment_entry_point, "fs_main");
        assert_eq!(shader.vertex_inputs.len(), 2);
        assert_eq!(shader.bindings.len(), 5);
    }

    #[test]
    fn rejects_duplicate_binding_slots() {
        let error = ShaderModuleDefinition::new(
            "duplicate-bindings",
            "@vertex fn vs_main() {}",
            "vs_main",
            "fs_main",
            vec![
                ShaderBindingDeclaration::new(0, 0, ShaderBindingSource::Camera),
                ShaderBindingDeclaration::new(0, 0, ShaderBindingSource::InstanceTransform),
            ],
            vec![],
        )
        .expect_err("duplicate binding slots must be rejected");

        assert_eq!(
            error,
            ShaderModuleValidationError::DuplicateBindingSlot {
                label: "duplicate-bindings".to_owned(),
                group: 0,
                binding: 0,
            }
        );
    }

    #[test]
    fn bounds_hand_written_wgsl_source_before_backend_compilation() {
        let error = ShaderModuleDefinition::new(
            "large-module",
            "x".repeat(MAX_SHADER_SOURCE_BYTES + 1),
            "vs_main",
            "fs_main",
            vec![],
            vec![],
        )
        .expect_err("oversized WGSL must be rejected");

        assert_eq!(
            error,
            ShaderModuleValidationError::SourceTooLarge {
                label: "large-module".to_owned(),
                bytes: MAX_SHADER_SOURCE_BYTES + 1,
                maximum: MAX_SHADER_SOURCE_BYTES,
            }
        );
    }

    #[test]
    fn bounds_shader_bindings_and_vertex_inputs_before_backend_compilation() {
        let bindings = (0..=MAX_SHADER_BINDINGS)
            .map(|binding| {
                ShaderBindingDeclaration::new(0, binding as u32, ShaderBindingSource::Camera)
            })
            .collect();
        let binding_error = ShaderModuleDefinition::new(
            "many-bindings",
            "@vertex fn vs_main() {}",
            "vs_main",
            "fs_main",
            bindings,
            vec![],
        )
        .expect_err("excessive shader bindings must be rejected");
        assert_eq!(
            binding_error,
            ShaderModuleValidationError::TooManyBindings {
                label: "many-bindings".to_owned(),
                count: MAX_SHADER_BINDINGS + 1,
                maximum: MAX_SHADER_BINDINGS,
            }
        );

        let inputs = (0..=MAX_SHADER_VERTEX_INPUTS)
            .map(|location| {
                ShaderVertexInput::new(location as u32, ShaderVertexSemantic::Position3)
            })
            .collect();
        let input_error = ShaderModuleDefinition::new(
            "many-inputs",
            "@vertex fn vs_main() {}",
            "vs_main",
            "fs_main",
            vec![],
            inputs,
        )
        .expect_err("excessive vertex inputs must be rejected");
        assert_eq!(
            input_error,
            ShaderModuleValidationError::TooManyVertexInputs {
                label: "many-inputs".to_owned(),
                count: MAX_SHADER_VERTEX_INPUTS + 1,
                maximum: MAX_SHADER_VERTEX_INPUTS,
            }
        );
    }

    #[test]
    fn rejects_invalid_wgsl_entry_point_identifiers_before_backend_compilation() {
        let error = ShaderModuleDefinition::new(
            "invalid-entry-point",
            "@vertex fn vs_main() {}",
            "vertex-main",
            "fs_main",
            vec![],
            vec![],
        )
        .expect_err("WGSL entry points cannot contain punctuation");

        assert_eq!(
            error,
            ShaderModuleValidationError::InvalidIdentifier {
                kind: "shader entry point",
                value: "vertex-main".to_owned(),
            }
        );
    }

    #[test]
    fn validates_material_binding_kinds_before_backend_submission() {
        let shader = ShaderModuleDefinition::built_in(PipelineKind::SolidColor2d)
            .expect("built-in shader declaration must be valid");
        let material = MaterialDefinition::solid_color(
            MaterialDefinitionId::new("surface").expect("valid material id"),
            Color::rgb(1.0, 1.0, 1.0),
        );

        assert_eq!(shader.validate_material_definition(&material), Ok(()));
    }

    #[test]
    fn diagnoses_missing_material_parameters() {
        let shader = ShaderModuleDefinition::built_in(PipelineKind::SolidColor2d)
            .expect("built-in shader declaration must be valid");
        let material = MaterialDefinition::new(
            MaterialDefinitionId::new("incomplete").expect("valid material id"),
            [],
        )
        .expect("empty material schema is valid");

        assert_eq!(
            shader.validate_material_definition(&material),
            Err(ShaderMaterialCompatibilityError::MissingMaterialParameter {
                shader: "builtin-SolidColor2d".to_owned(),
                material: "incomplete".to_owned(),
                parameter: "base_color".to_owned(),
            })
        );
    }

    #[test]
    fn diagnoses_missing_mesh_normal_streams_before_backend_submission() {
        let shader = ShaderModuleDefinition::built_in(PipelineKind::LitColor3d)
            .expect("built-in shader declaration must be valid");
        let mesh = Mesh::new(vec![[0.0, 0.0, 0.0]], vec![]);

        assert_eq!(
            shader.validate_mesh(&mesh),
            Err(ShaderMeshCompatibilityError::MissingVertexInput {
                shader: "builtin-LitColor3d".to_owned(),
                location: 1,
                semantic: ShaderVertexSemantic::Normal3,
            })
        );
    }
}
