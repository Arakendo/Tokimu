//! Small, original visualizer definitions for corpus-side authoring evidence.
//!
//! These definitions share the bounded audio observation and pass-graph
//! contracts. They intentionally do not define a general expression language,
//! MilkDrop compatibility object, renderer handle, or backend resource.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{RenderTargetGraphError, VisualizerPassGraph, VisualizerViewport};

/// Stable identities for the first original visualizer corpus definitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeVisualizerKind {
    SignalField,
    FeedbackBloom,
    SignalComposite,
    SpectrumBars,
}

impl NativeVisualizerKind {
    pub const ALL: [Self; 4] = [
        Self::SignalField,
        Self::FeedbackBloom,
        Self::SignalComposite,
        Self::SpectrumBars,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::SignalField => "signal-field",
            Self::FeedbackBloom => "feedback-bloom",
            Self::SignalComposite => "signal-composite",
            Self::SpectrumBars => "spectrum-bars",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::SignalField => "Signal Field",
            Self::FeedbackBloom => "Feedback Bloom",
            Self::SignalComposite => "Signal Composite",
            Self::SpectrumBars => "Spectrum Bars",
        }
    }
}

/// A bounded, UI-addressable parameter owned by one native visualizer
/// definition. The parameter is descriptive evidence until an execution model
/// has more than one independent consumer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NativeVisualizerParameter {
    pub id: String,
    pub default: f32,
    pub minimum: f32,
    pub maximum: f32,
}

/// One original, provider-neutral visualizer description.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NativeVisualizerDefinition {
    pub kind: NativeVisualizerKind,
    pub id: String,
    pub label: String,
    pub parameters: Vec<NativeVisualizerParameter>,
    pub pass_graph: VisualizerPassGraph,
}

impl NativeVisualizerDefinition {
    pub fn new(
        kind: NativeVisualizerKind,
        viewport: VisualizerViewport,
    ) -> Result<Self, NativeVisualizerDefinitionError> {
        let (parameters, pass_graph) = match kind {
            NativeVisualizerKind::SignalField => (
                vec![
                    parameter("phase-rate", 1.0, 0.0, 4.0),
                    parameter("band-gain", 1.0, 0.0, 2.0),
                ],
                VisualizerPassGraph::two_pass_signal(viewport.width, viewport.height),
            ),
            NativeVisualizerKind::FeedbackBloom => (
                vec![
                    parameter("decay", 0.94, 0.0, 1.0),
                    parameter("injection-gain", 0.75, 0.0, 2.0),
                ],
                VisualizerPassGraph::three_pass_feedback(viewport.width, viewport.height),
            ),
            NativeVisualizerKind::SignalComposite => (
                vec![
                    parameter("low-gain", 1.0, 0.0, 2.0),
                    parameter("high-gain", 1.0, 0.0, 2.0),
                ],
                VisualizerPassGraph::three_pass_signal(viewport.width, viewport.height),
            ),
            NativeVisualizerKind::SpectrumBars => (
                vec![
                    parameter("bar-count", 32.0, 1.0, 128.0),
                    parameter("bar-gain", 1.5, 0.0, 4.0),
                ],
                VisualizerPassGraph::two_pass_signal(viewport.width, viewport.height),
            ),
        };
        let definition = Self {
            kind,
            id: kind.id().to_owned(),
            label: kind.label().to_owned(),
            parameters,
            pass_graph,
        };
        definition.validate()?;
        Ok(definition)
    }

    pub fn all(viewport: VisualizerViewport) -> Result<Vec<Self>, NativeVisualizerDefinitionError> {
        NativeVisualizerKind::ALL
            .into_iter()
            .map(|kind| Self::new(kind, viewport))
            .collect()
    }

    pub fn validate(&self) -> Result<(), NativeVisualizerDefinitionError> {
        if self.id.trim().is_empty() {
            return Err(NativeVisualizerDefinitionError::EmptyId);
        }
        if self.label.trim().is_empty() {
            return Err(NativeVisualizerDefinitionError::EmptyLabel);
        }
        for (index, parameter) in self.parameters.iter().enumerate() {
            validate_parameter(parameter, index)?;
            if self.parameters[..index]
                .iter()
                .any(|prior| prior.id == parameter.id)
            {
                return Err(NativeVisualizerDefinitionError::DuplicateParameterId {
                    id: parameter.id.clone(),
                });
            }
        }
        self.pass_graph.validate()?;
        Ok(())
    }

    pub fn to_structural_json(&self) -> Result<String, NativeVisualizerDefinitionError> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| NativeVisualizerDefinitionError::Serialization(error.to_string()))
    }
}

fn parameter(id: &str, default: f32, minimum: f32, maximum: f32) -> NativeVisualizerParameter {
    NativeVisualizerParameter {
        id: id.to_owned(),
        default,
        minimum,
        maximum,
    }
}

fn validate_parameter(
    parameter: &NativeVisualizerParameter,
    index: usize,
) -> Result<(), NativeVisualizerDefinitionError> {
    if parameter.id.trim().is_empty() {
        return Err(NativeVisualizerDefinitionError::EmptyParameterId { index });
    }
    if !parameter.default.is_finite()
        || !parameter.minimum.is_finite()
        || !parameter.maximum.is_finite()
        || parameter.minimum > parameter.maximum
        || !(parameter.minimum..=parameter.maximum).contains(&parameter.default)
    {
        return Err(NativeVisualizerDefinitionError::InvalidParameterRange {
            id: parameter.id.clone(),
        });
    }
    Ok(())
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum NativeVisualizerDefinitionError {
    #[error("native visualizer identity must not be empty")]
    EmptyId,
    #[error("native visualizer label must not be empty")]
    EmptyLabel,
    #[error("native visualizer parameter at index {index} must have an identity")]
    EmptyParameterId { index: usize },
    #[error("native visualizer parameter `{id}` is outside a finite inclusive range")]
    InvalidParameterRange { id: String },
    #[error("native visualizer parameter `{id}` is declared more than once")]
    DuplicateParameterId { id: String },
    #[error(transparent)]
    PassGraph(#[from] RenderTargetGraphError),
    #[error("could not serialize native visualizer definition: {0}")]
    Serialization(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport() -> VisualizerViewport {
        VisualizerViewport::new(640, 360).unwrap()
    }

    #[test]
    fn original_definitions_are_stable_and_use_distinct_pass_shapes() {
        let definitions = NativeVisualizerDefinition::all(viewport()).unwrap();
        assert_eq!(definitions.len(), NativeVisualizerKind::ALL.len());
        assert_eq!(definitions[0].id, "signal-field");
        assert_eq!(definitions[1].id, "feedback-bloom");
        assert_eq!(definitions[2].id, "signal-composite");
        assert_eq!(definitions[3].id, "spectrum-bars");
        assert_eq!(definitions[0].pass_graph.passes.len(), 2);
        assert_eq!(definitions[1].pass_graph.feedback_pairs.len(), 1);
        assert_eq!(definitions[2].pass_graph.passes.len(), 3);
        assert_ne!(
            definitions[0].to_structural_json().unwrap(),
            definitions[1].to_structural_json().unwrap()
        );
    }

    #[test]
    fn invalid_native_parameter_ranges_are_diagnosed() {
        let mut definition =
            NativeVisualizerDefinition::new(NativeVisualizerKind::SignalField, viewport()).unwrap();
        definition.parameters[0].default = 5.0;
        assert!(matches!(
            definition.validate(),
            Err(NativeVisualizerDefinitionError::InvalidParameterRange { .. })
        ));
    }

    #[test]
    fn invalid_parameter_identities_are_diagnosed_before_execution() {
        let mut definition =
            NativeVisualizerDefinition::new(NativeVisualizerKind::SignalField, viewport()).unwrap();
        definition.parameters[0].id.clear();
        assert!(matches!(
            definition.validate(),
            Err(NativeVisualizerDefinitionError::EmptyParameterId { index: 0 })
        ));

        definition.parameters[0].id = "phase-rate".to_owned();
        definition.parameters[1].id = "phase-rate".to_owned();
        assert!(matches!(
            definition.validate(),
            Err(NativeVisualizerDefinitionError::DuplicateParameterId { .. })
        ));
    }
}
