#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PipelineKind {
    #[default]
    SolidColor2d,
    CustomWgsl2d,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Pipeline {
    pub label: String,
    pub kind: PipelineKind,
    pub shader_source: Option<String>,
}

impl Pipeline {
    pub fn new(label: impl Into<String>, kind: PipelineKind) -> Self {
        Self {
            label: label.into(),
            kind,
            shader_source: None,
        }
    }

    pub fn custom_wgsl(label: impl Into<String>, shader_source: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: PipelineKind::CustomWgsl2d,
            shader_source: Some(shader_source.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_default_solid_color_pipeline() {
        let pipeline = Pipeline::new("solid", PipelineKind::SolidColor2d);

        assert_eq!(pipeline.label, "solid");
        assert_eq!(pipeline.kind, PipelineKind::SolidColor2d);
        assert_eq!(pipeline.shader_source, None);
    }

    #[test]
    fn creates_custom_wgsl_pipeline() {
        let pipeline = Pipeline::custom_wgsl("custom", "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(); }");

        assert_eq!(pipeline.label, "custom");
        assert_eq!(pipeline.kind, PipelineKind::CustomWgsl2d);
        assert_eq!(pipeline.shader_source.as_deref(), Some("@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(); }"));
    }
}
