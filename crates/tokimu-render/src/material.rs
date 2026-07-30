use crate::Color;
use crate::TextureHandle;
use std::collections::BTreeMap;

/// Execution-ready material data accepted by the current renderer backends.
///
/// Pipeline selection remains explicit in draw submission. Higher-level material
/// definitions can lower into this compatibility shape without selecting one.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub label: String,
    pub base_color: Color,
    pub texture: Option<TextureHandle>,
}

impl Material {
    pub fn new(label: impl Into<String>, base_color: Color) -> Self {
        Self {
            label: label.into(),
            base_color,
            texture: None,
        }
    }

    pub fn with_texture(mut self, texture: TextureHandle) -> Self {
        self.texture = Some(texture);
        self
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new("default-material", Color::rgb(0.96, 0.72, 0.28))
    }
}

/// A transient color and opacity adjustment applied at one draw site.
///
/// This value never changes the uploaded source material and does not select a
/// pipeline. Renderer adapters may cache the derived binding below this
/// semantic boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialOverride {
    replacement_color: Option<Color>,
    opacity_multiplier: f32,
}

impl MaterialOverride {
    pub fn with_replacement_color(color: Color) -> Result<Self, MaterialModelError> {
        let override_value = Self {
            replacement_color: Some(color),
            ..Self::default()
        };
        override_value.validate()?;
        Ok(override_value)
    }

    pub fn with_opacity_multiplier(
        mut self,
        opacity_multiplier: f32,
    ) -> Result<Self, MaterialModelError> {
        self.opacity_multiplier = opacity_multiplier;
        self.validate()?;
        Ok(self)
    }

    pub fn apply_to_color(self, source: Color) -> Color {
        let mut resolved = self.replacement_color.unwrap_or(source);
        resolved.a *= self.opacity_multiplier;
        resolved
    }

    pub fn replacement_color(self) -> Option<Color> {
        self.replacement_color
    }

    pub fn opacity_multiplier(self) -> f32 {
        self.opacity_multiplier
    }

    fn validate(self) -> Result<(), MaterialModelError> {
        if !self.opacity_multiplier.is_finite() || !(0.0..=1.0).contains(&self.opacity_multiplier) {
            return Err(MaterialModelError::InvalidOverrideOpacity {
                value: self.opacity_multiplier,
            });
        }
        if let Some(color) = self.replacement_color {
            if ![color.r, color.g, color.b, color.a]
                .iter()
                .all(|value| value.is_finite())
            {
                return Err(MaterialModelError::NonFiniteOverrideColor);
            }
        }
        Ok(())
    }
}

impl Default for MaterialOverride {
    fn default() -> Self {
        Self {
            replacement_color: None,
            opacity_multiplier: 1.0,
        }
    }
}

/// Stable identity for a provider-neutral material definition.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaterialDefinitionId(String);

impl MaterialDefinitionId {
    pub fn new(value: impl Into<String>) -> Result<Self, MaterialModelError> {
        let value = value.into();
        validate_identifier("material definition", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The bounded value categories material definitions can expose to callers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialParameterKind {
    Float,
    Vector2,
    Vector3,
    Vector4,
    Color,
    Texture,
    Boolean,
}

/// A provider-neutral material parameter value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaterialParameterValue {
    Float(f32),
    Vector2([f32; 2]),
    Vector3([f32; 3]),
    Vector4([f32; 4]),
    Color(Color),
    Texture(Option<TextureHandle>),
    Boolean(bool),
}

impl MaterialParameterValue {
    pub fn kind(self) -> MaterialParameterKind {
        match self {
            Self::Float(_) => MaterialParameterKind::Float,
            Self::Vector2(_) => MaterialParameterKind::Vector2,
            Self::Vector3(_) => MaterialParameterKind::Vector3,
            Self::Vector4(_) => MaterialParameterKind::Vector4,
            Self::Color(_) => MaterialParameterKind::Color,
            Self::Texture(_) => MaterialParameterKind::Texture,
            Self::Boolean(_) => MaterialParameterKind::Boolean,
        }
    }

    fn is_finite(self) -> bool {
        match self {
            Self::Float(value) => value.is_finite(),
            Self::Vector2(values) => values.iter().all(|value| value.is_finite()),
            Self::Vector3(values) => values.iter().all(|value| value.is_finite()),
            Self::Vector4(values) => values.iter().all(|value| value.is_finite()),
            Self::Color(color) => [color.r, color.g, color.b, color.a]
                .iter()
                .all(|value| value.is_finite()),
            Self::Texture(_) | Self::Boolean(_) => true,
        }
    }
}

/// Inclusive bounds for a scalar material parameter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialFloatRange {
    pub min: f32,
    pub max: f32,
}

impl MaterialFloatRange {
    pub fn new(min: f32, max: f32) -> Result<Self, MaterialModelError> {
        if !min.is_finite() || !max.is_finite() || min > max {
            return Err(MaterialModelError::InvalidFloatRange { min, max });
        }

        Ok(Self { min, max })
    }

    fn contains(self, value: f32) -> bool {
        value >= self.min && value <= self.max
    }
}

/// One named value accepted by a material definition.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialParameterDeclaration {
    pub name: String,
    pub kind: MaterialParameterKind,
    pub default: MaterialParameterValue,
    pub float_range: Option<MaterialFloatRange>,
}

impl MaterialParameterDeclaration {
    pub fn new(
        name: impl Into<String>,
        kind: MaterialParameterKind,
        default: MaterialParameterValue,
    ) -> Result<Self, MaterialModelError> {
        let declaration = Self {
            name: name.into(),
            kind,
            default,
            float_range: None,
        };
        declaration.validate()?;
        Ok(declaration)
    }

    pub fn with_float_range(
        mut self,
        float_range: MaterialFloatRange,
    ) -> Result<Self, MaterialModelError> {
        if self.kind != MaterialParameterKind::Float {
            return Err(MaterialModelError::FloatRangeRequiresFloat {
                name: self.name.clone(),
                kind: self.kind,
            });
        }

        self.float_range = Some(float_range);
        self.validate()?;
        Ok(self)
    }

    fn validate(&self) -> Result<(), MaterialModelError> {
        validate_identifier("material parameter", &self.name)?;
        validate_parameter_value(self, self.default)
    }
}

/// Provider-neutral material parameter schema.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialDefinition {
    pub id: MaterialDefinitionId,
    parameters: BTreeMap<String, MaterialParameterDeclaration>,
}

impl MaterialDefinition {
    pub fn new(
        id: MaterialDefinitionId,
        declarations: impl IntoIterator<Item = MaterialParameterDeclaration>,
    ) -> Result<Self, MaterialModelError> {
        let mut parameters = BTreeMap::new();
        for declaration in declarations {
            declaration.validate()?;
            let name = declaration.name.clone();
            if parameters.insert(name.clone(), declaration).is_some() {
                return Err(MaterialModelError::DuplicateParameter { name });
            }
        }

        Ok(Self { id, parameters })
    }

    /// A bounded compatibility schema for the current solid-color backend path.
    pub fn solid_color(id: MaterialDefinitionId, base_color: Color) -> Self {
        Self::new(
            id,
            [
                MaterialParameterDeclaration::new(
                    "base_color",
                    MaterialParameterKind::Color,
                    MaterialParameterValue::Color(base_color),
                )
                .expect("base_color is a valid built-in material parameter"),
                MaterialParameterDeclaration::new(
                    "base_texture",
                    MaterialParameterKind::Texture,
                    MaterialParameterValue::Texture(None),
                )
                .expect("base_texture is a valid built-in material parameter"),
            ],
        )
        .expect("built-in solid-color material definition is valid")
    }

    pub fn parameter(&self, name: &str) -> Option<&MaterialParameterDeclaration> {
        self.parameters.get(name)
    }

    pub fn parameters(&self) -> impl ExactSizeIterator<Item = &MaterialParameterDeclaration> {
        self.parameters.values()
    }

    /// Lowers the bounded solid-color schema into the execution material used by
    /// current renderer backends. It deliberately does not choose a pipeline.
    pub fn lower_to_legacy_material(
        &self,
        instance: &MaterialInstance,
        label: impl Into<String>,
    ) -> Result<Material, MaterialModelError> {
        instance.ensure_definition(self)?;

        let color = match instance.value("base_color") {
            Some(MaterialParameterValue::Color(color)) => *color,
            Some(value) => {
                return Err(MaterialModelError::ParameterKindMismatch {
                    name: "base_color".to_owned(),
                    expected: MaterialParameterKind::Color,
                    actual: value.kind(),
                });
            }
            None => {
                return Err(MaterialModelError::UnknownParameter {
                    name: "base_color".to_owned(),
                })
            }
        };

        let mut material = Material::new(label, color);
        match instance.value("base_texture") {
            Some(MaterialParameterValue::Texture(Some(texture))) => {
                material = material.with_texture(*texture);
            }
            Some(MaterialParameterValue::Texture(None)) | None => {}
            Some(value) => {
                return Err(MaterialModelError::ParameterKindMismatch {
                    name: "base_texture".to_owned(),
                    expected: MaterialParameterKind::Texture,
                    actual: value.kind(),
                });
            }
        }

        Ok(material)
    }
}

/// Values bound to one material definition.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialInstance {
    pub definition: MaterialDefinitionId,
    values: BTreeMap<String, MaterialParameterValue>,
}

impl MaterialInstance {
    pub fn from_definition(definition: &MaterialDefinition) -> Self {
        let values = definition
            .parameters
            .iter()
            .map(|(name, declaration)| (name.clone(), declaration.default))
            .collect();
        Self {
            definition: definition.id.clone(),
            values,
        }
    }

    pub fn value(&self, name: &str) -> Option<&MaterialParameterValue> {
        self.values.get(name)
    }

    pub fn set(
        &mut self,
        definition: &MaterialDefinition,
        name: &str,
        value: MaterialParameterValue,
    ) -> Result<(), MaterialModelError> {
        self.ensure_definition(definition)?;
        let declaration =
            definition
                .parameter(name)
                .ok_or_else(|| MaterialModelError::UnknownParameter {
                    name: name.to_owned(),
                })?;
        validate_parameter_value(declaration, value)?;
        self.values.insert(name.to_owned(), value);
        Ok(())
    }

    fn ensure_definition(&self, definition: &MaterialDefinition) -> Result<(), MaterialModelError> {
        if self.definition != definition.id {
            return Err(MaterialModelError::DefinitionMismatch {
                expected: definition.id.clone(),
                actual: self.definition.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MaterialModelError {
    #[error("material override opacity {value} must be finite and within [0, 1]")]
    InvalidOverrideOpacity { value: f32 },
    #[error("material override color must contain only finite values")]
    NonFiniteOverrideColor,
    #[error("{kind} identifier must be non-empty, trimmed, at most 128 bytes, and contain no control characters: {value:?}")]
    InvalidIdentifier { kind: &'static str, value: String },
    #[error("material parameter {name:?} is declared more than once")]
    DuplicateParameter { name: String },
    #[error("material parameter {name:?} is not declared")]
    UnknownParameter { name: String },
    #[error("material parameter {name:?} expects {expected:?}, received {actual:?}")]
    ParameterKindMismatch {
        name: String,
        expected: MaterialParameterKind,
        actual: MaterialParameterKind,
    },
    #[error("material parameter {name:?} contains a non-finite value")]
    NonFiniteParameter { name: String },
    #[error("material parameter {name:?} value {value} is outside [{min}, {max}]")]
    ParameterOutOfRange {
        name: String,
        value: f32,
        min: f32,
        max: f32,
    },
    #[error("material float range [{min}, {max}] must be finite and ordered")]
    InvalidFloatRange { min: f32, max: f32 },
    #[error("material parameter {name:?} with kind {kind:?} cannot declare a float range")]
    FloatRangeRequiresFloat {
        name: String,
        kind: MaterialParameterKind,
    },
    #[error("material instance belongs to {actual:?}, not {expected:?}")]
    DefinitionMismatch {
        expected: MaterialDefinitionId,
        actual: MaterialDefinitionId,
    },
}

fn validate_identifier(kind: &'static str, value: &str) -> Result<(), MaterialModelError> {
    if value.is_empty()
        || value.trim() != value
        || value.len() > 128
        || value.chars().any(char::is_control)
    {
        return Err(MaterialModelError::InvalidIdentifier {
            kind,
            value: value.to_owned(),
        });
    }
    Ok(())
}

fn validate_parameter_value(
    declaration: &MaterialParameterDeclaration,
    value: MaterialParameterValue,
) -> Result<(), MaterialModelError> {
    if declaration.kind != value.kind() {
        return Err(MaterialModelError::ParameterKindMismatch {
            name: declaration.name.clone(),
            expected: declaration.kind,
            actual: value.kind(),
        });
    }
    if !value.is_finite() {
        return Err(MaterialModelError::NonFiniteParameter {
            name: declaration.name.clone(),
        });
    }
    if let (Some(range), MaterialParameterValue::Float(value)) = (declaration.float_range, value) {
        if !range.contains(value) {
            return Err(MaterialModelError::ParameterOutOfRange {
                name: declaration.name.clone(),
                value,
                min: range.min,
                max: range.max,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inspection_definition() -> MaterialDefinition {
        let opacity = MaterialParameterDeclaration::new(
            "opacity",
            MaterialParameterKind::Float,
            MaterialParameterValue::Float(1.0),
        )
        .unwrap()
        .with_float_range(MaterialFloatRange::new(0.0, 1.0).unwrap())
        .unwrap();
        MaterialDefinition::new(
            MaterialDefinitionId::new("inspection-surface").unwrap(),
            [
                MaterialParameterDeclaration::new(
                    "base_color",
                    MaterialParameterKind::Color,
                    MaterialParameterValue::Color(Color::rgb(0.7, 0.8, 0.9)),
                )
                .unwrap(),
                opacity,
                MaterialParameterDeclaration::new(
                    "enabled",
                    MaterialParameterKind::Boolean,
                    MaterialParameterValue::Boolean(true),
                )
                .unwrap(),
            ],
        )
        .unwrap()
    }

    #[test]
    fn instance_uses_schema_defaults_and_accepts_valid_updates() {
        let definition = inspection_definition();
        let mut instance = MaterialInstance::from_definition(&definition);

        instance
            .set(&definition, "opacity", MaterialParameterValue::Float(0.35))
            .unwrap();

        assert_eq!(
            instance.value("opacity"),
            Some(&MaterialParameterValue::Float(0.35))
        );
        assert_eq!(
            instance.value("enabled"),
            Some(&MaterialParameterValue::Boolean(true))
        );
    }

    #[test]
    fn invalid_values_have_deterministic_model_errors() {
        let definition = inspection_definition();
        let mut instance = MaterialInstance::from_definition(&definition);

        assert!(matches!(
            instance.set(&definition, "opacity", MaterialParameterValue::Float(1.1),),
            Err(MaterialModelError::ParameterOutOfRange { .. })
        ));
        assert!(matches!(
            instance.set(
                &definition,
                "opacity",
                MaterialParameterValue::Float(f32::NAN),
            ),
            Err(MaterialModelError::NonFiniteParameter { .. })
        ));
        assert!(matches!(
            instance.set(
                &definition,
                "missing",
                MaterialParameterValue::Boolean(true),
            ),
            Err(MaterialModelError::UnknownParameter { .. })
        ));
        assert!(matches!(
            instance.set(&definition, "enabled", MaterialParameterValue::Float(1.0),),
            Err(MaterialModelError::ParameterKindMismatch { .. })
        ));
    }

    #[test]
    fn compatibility_lowering_preserves_existing_solid_color_material_shape() {
        let definition = MaterialDefinition::solid_color(
            MaterialDefinitionId::new("solid-color").unwrap(),
            Color::rgb(0.2, 0.4, 0.6),
        );
        let mut instance = MaterialInstance::from_definition(&definition);
        instance
            .set(
                &definition,
                "base_texture",
                MaterialParameterValue::Texture(Some(TextureHandle(7))),
            )
            .unwrap();

        let material = definition
            .lower_to_legacy_material(&instance, "compatibility-material")
            .unwrap();

        assert_eq!(material.label, "compatibility-material");
        assert_eq!(material.base_color, Color::rgb(0.2, 0.4, 0.6));
        assert_eq!(material.texture, Some(TextureHandle(7)));
    }

    #[test]
    fn per_draw_override_changes_a_color_without_mutating_its_source() {
        let source = Color::rgba(0.2, 0.4, 0.6, 0.8);
        let override_value =
            MaterialOverride::with_replacement_color(Color::rgba(1.0, 0.3, 0.1, 0.9))
                .unwrap()
                .with_opacity_multiplier(0.5)
                .unwrap();

        assert_eq!(
            override_value.apply_to_color(source),
            Color::rgba(1.0, 0.3, 0.1, 0.45)
        );
        assert_eq!(source, Color::rgba(0.2, 0.4, 0.6, 0.8));
        assert!(matches!(
            MaterialOverride::default().with_opacity_multiplier(1.1),
            Err(MaterialModelError::InvalidOverrideOpacity { .. })
        ));
    }

    #[test]
    fn duplicate_names_and_cross_definition_updates_are_rejected() {
        let declaration = MaterialParameterDeclaration::new(
            "opacity",
            MaterialParameterKind::Float,
            MaterialParameterValue::Float(1.0),
        )
        .unwrap();
        assert!(matches!(
            MaterialDefinition::new(
                MaterialDefinitionId::new("duplicate").unwrap(),
                [declaration.clone(), declaration],
            ),
            Err(MaterialModelError::DuplicateParameter { .. })
        ));

        let first = inspection_definition();
        let second = MaterialDefinition::solid_color(
            MaterialDefinitionId::new("second-definition").unwrap(),
            Color::BLACK,
        );
        let mut instance = MaterialInstance::from_definition(&first);
        assert!(matches!(
            instance.set(
                &second,
                "base_color",
                MaterialParameterValue::Color(Color::BLACK),
            ),
            Err(MaterialModelError::DefinitionMismatch { .. })
        ));
    }
}
