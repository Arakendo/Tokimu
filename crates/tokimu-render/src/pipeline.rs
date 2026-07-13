#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PipelineKind {
    #[default]
    SolidColor2d,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Pipeline {
    pub label: String,
    pub kind: PipelineKind,
}

impl Pipeline {
    pub fn new(label: impl Into<String>, kind: PipelineKind) -> Self {
        Self {
            label: label.into(),
            kind,
        }
    }
}
