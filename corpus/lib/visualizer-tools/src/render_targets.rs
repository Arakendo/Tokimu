//! Corpus-local multipass render-target requirements.
//!
//! This is deliberately evidence, not an admitted general texture-requirement
//! API. It lets the visualizer corpus validate resource intent and pass
//! ordering without exposing WGPU textures, views, command encoders, or
//! sampler objects above the renderer boundary.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_RENDER_TARGET_DIMENSION: u32 = 8_192;
pub const MAX_RENDER_TARGETS: usize = 8;
pub const MAX_RENDER_PASSES: usize = 16;

/// The semantic interpretation expected for a corpus render target.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderTargetColorInterpretation {
    SrgbColor,
    LinearData,
}

/// Whether a completed target may be read by a later pass.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderTargetSampling {
    Sampled,
    NotSampled,
}

/// The initial content policy for a render pass that writes a target.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderTargetLoadBehavior {
    Clear([u8; 4]),
    Preserve,
}

/// Provider-neutral requirement for one renderer-owned offscreen target.
///
/// This type is scoped to the visualizer corpus while `AR-0006` remains under
/// review. It contains presentation intent, never backend allocation objects.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RenderTargetRequirement {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub color_interpretation: RenderTargetColorInterpretation,
    pub sampling: RenderTargetSampling,
    pub initial_load: RenderTargetLoadBehavior,
}

/// A resource a visualizer pass can consume or produce.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "name", rename_all = "kebab-case")]
pub enum VisualizerResource {
    /// An already-resolved external image. The graph intentionally does not
    /// describe its format, asset path, or backend texture identity.
    SourceTexture(String),
    /// A target produced by an earlier visualizer pass.
    RenderTarget(String),
    /// The previous-frame member of a declared feedback target pair.
    ///
    /// This is intentionally distinct from `RenderTarget`: the source is
    /// completed by the prior frame, not an earlier pass in the current frame.
    PreviousFrameTarget(String),
    /// The native or browser presentation surface.
    Surface,
}

/// Initialization policy for a renderer-owned previous-frame target pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeedbackInitialization {
    Clear([u8; 4]),
}

/// A named ping-pong pair whose previous member is sampled this frame and whose
/// current member is written this frame.
///
/// The graph records temporal intent only. Target allocation, reset work, role
/// swapping, and resource lifetime remain renderer-owned execution concerns.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FeedbackTargetPairRequirement {
    pub name: String,
    pub previous_target: String,
    pub current_target: String,
    pub initialization: FeedbackInitialization,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VisualizerPassRequirement {
    pub name: String,
    /// Provider-neutral pipeline selection. This identifies semantic work to
    /// execute without exposing a renderer pipeline handle or shader object.
    pub pipeline: String,
    pub inputs: Vec<VisualizerResource>,
    pub output: VisualizerResource,
}

/// A bounded provider-neutral visualizer pass graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VisualizerPassGraph {
    pub targets: Vec<RenderTargetRequirement>,
    #[serde(default)]
    pub feedback_pairs: Vec<FeedbackTargetPairRequirement>,
    pub passes: Vec<VisualizerPassRequirement>,
}

/// Bounded structural facts collected while validating a pass graph.
///
/// The summary is corpus evidence only. It contains no renderer resources,
/// timings, or backend command objects.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VisualizerPassGraphSummary {
    pub pass_count: usize,
    pub render_target_count: usize,
    pub source_texture_reads: usize,
    pub render_target_reads: usize,
    pub previous_frame_target_reads: usize,
    pub render_target_writes: usize,
    pub feedback_pair_count: usize,
    pub surface_outputs: usize,
    pub distinct_pipeline_count: usize,
    pub maximum_target_width: u32,
    pub maximum_target_height: u32,
}

impl VisualizerPassGraph {
    /// Returns the minimal surface-only graph for a visualizer that does not
    /// need an intermediate target. This keeps a simple fullscreen shader a
    /// first-class corpus case rather than making it imitate multipass work.
    pub fn single_pass_surface(pipeline: impl Into<String>) -> Self {
        Self {
            targets: Vec::new(),
            feedback_pairs: Vec::new(),
            passes: vec![VisualizerPassRequirement {
                name: "present".to_owned(),
                pipeline: pipeline.into(),
                inputs: Vec::new(),
                output: VisualizerResource::Surface,
            }],
        }
    }

    /// Returns the initial two-pass corpus graph: render the signal to an
    /// offscreen target, then sample that completed target on the surface.
    pub fn two_pass_signal(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            targets: vec![RenderTargetRequirement {
                name: "signal".to_owned(),
                width: viewport_width,
                height: viewport_height,
                color_interpretation: RenderTargetColorInterpretation::SrgbColor,
                sampling: RenderTargetSampling::Sampled,
                initial_load: RenderTargetLoadBehavior::Clear([4, 12, 22, 255]),
            }],
            feedback_pairs: Vec::new(),
            passes: vec![
                VisualizerPassRequirement {
                    name: "signal".to_owned(),
                    pipeline: "visualizer-signal".to_owned(),
                    inputs: Vec::new(),
                    output: VisualizerResource::RenderTarget("signal".to_owned()),
                },
                VisualizerPassRequirement {
                    name: "present".to_owned(),
                    pipeline: "visualizer-present".to_owned(),
                    inputs: vec![VisualizerResource::RenderTarget("signal".to_owned())],
                    output: VisualizerResource::Surface,
                },
            ],
        }
    }

    /// Returns a three-pass graph with an explicit intermediate composite.
    /// Feedback is intentionally not implied; it requires the separate
    /// ping-pong contract described by the visualizer plan.
    pub fn three_pass_signal(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            targets: vec![
                RenderTargetRequirement {
                    name: "signal".to_owned(),
                    width: viewport_width,
                    height: viewport_height,
                    color_interpretation: RenderTargetColorInterpretation::SrgbColor,
                    sampling: RenderTargetSampling::Sampled,
                    initial_load: RenderTargetLoadBehavior::Clear([4, 12, 22, 255]),
                },
                RenderTargetRequirement {
                    name: "composite".to_owned(),
                    width: viewport_width,
                    height: viewport_height,
                    color_interpretation: RenderTargetColorInterpretation::SrgbColor,
                    sampling: RenderTargetSampling::Sampled,
                    initial_load: RenderTargetLoadBehavior::Clear([0, 0, 0, 255]),
                },
            ],
            feedback_pairs: Vec::new(),
            passes: vec![
                VisualizerPassRequirement {
                    name: "signal".to_owned(),
                    pipeline: "visualizer-signal".to_owned(),
                    inputs: Vec::new(),
                    output: VisualizerResource::RenderTarget("signal".to_owned()),
                },
                VisualizerPassRequirement {
                    name: "composite".to_owned(),
                    pipeline: "visualizer-composite".to_owned(),
                    inputs: vec![VisualizerResource::RenderTarget("signal".to_owned())],
                    output: VisualizerResource::RenderTarget("composite".to_owned()),
                },
                VisualizerPassRequirement {
                    name: "present".to_owned(),
                    pipeline: "visualizer-present".to_owned(),
                    inputs: vec![VisualizerResource::RenderTarget("composite".to_owned())],
                    output: VisualizerResource::Surface,
                },
            ],
        }
    }

    /// Returns a three-pass graph that records a single bounded feedback role.
    ///
    /// `history` is sampled from its previous-frame member and written to its
    /// current-frame member. The renderer is responsible for initializing and
    /// swapping those members between frames.
    pub fn three_pass_feedback(viewport_width: u32, viewport_height: u32) -> Self {
        let color_target = |name: &str, clear| RenderTargetRequirement {
            name: name.to_owned(),
            width: viewport_width,
            height: viewport_height,
            color_interpretation: RenderTargetColorInterpretation::SrgbColor,
            sampling: RenderTargetSampling::Sampled,
            initial_load: RenderTargetLoadBehavior::Clear(clear),
        };

        Self {
            targets: vec![
                color_target("signal", [4, 12, 22, 255]),
                color_target("history-previous", [0, 0, 0, 255]),
                color_target("history-current", [0, 0, 0, 255]),
            ],
            feedback_pairs: vec![FeedbackTargetPairRequirement {
                name: "history".to_owned(),
                previous_target: "history-previous".to_owned(),
                current_target: "history-current".to_owned(),
                initialization: FeedbackInitialization::Clear([0, 0, 0, 255]),
            }],
            passes: vec![
                VisualizerPassRequirement {
                    name: "signal".to_owned(),
                    pipeline: "visualizer-signal".to_owned(),
                    inputs: Vec::new(),
                    output: VisualizerResource::RenderTarget("signal".to_owned()),
                },
                VisualizerPassRequirement {
                    name: "feedback".to_owned(),
                    pipeline: "visualizer-feedback".to_owned(),
                    inputs: vec![
                        VisualizerResource::RenderTarget("signal".to_owned()),
                        VisualizerResource::PreviousFrameTarget("history".to_owned()),
                    ],
                    output: VisualizerResource::RenderTarget("history-current".to_owned()),
                },
                VisualizerPassRequirement {
                    name: "present".to_owned(),
                    pipeline: "visualizer-present".to_owned(),
                    inputs: vec![VisualizerResource::RenderTarget(
                        "history-current".to_owned(),
                    )],
                    output: VisualizerResource::Surface,
                },
            ],
        }
    }

    /// Validates graph structure before a renderer allocates resources or
    /// records backend commands.
    pub fn validate(&self) -> Result<(), RenderTargetGraphError> {
        self.validate_with_summary().map(|_| ())
    }

    /// Validates the graph and returns structural evidence for diagnostics.
    pub fn validate_with_summary(
        &self,
    ) -> Result<VisualizerPassGraphSummary, RenderTargetGraphError> {
        if self.targets.len() > MAX_RENDER_TARGETS {
            return Err(RenderTargetGraphError::TooManyTargets {
                actual: self.targets.len(),
                maximum: MAX_RENDER_TARGETS,
            });
        }
        if self.passes.is_empty() {
            return Err(RenderTargetGraphError::NoPasses);
        }
        if self.passes.len() > MAX_RENDER_PASSES {
            return Err(RenderTargetGraphError::TooManyPasses {
                actual: self.passes.len(),
                maximum: MAX_RENDER_PASSES,
            });
        }

        let mut target_names = Vec::with_capacity(self.targets.len());
        for target in &self.targets {
            validate_name("render target", &target.name)?;
            if target.width == 0
                || target.height == 0
                || target.width > MAX_RENDER_TARGET_DIMENSION
                || target.height > MAX_RENDER_TARGET_DIMENSION
            {
                return Err(RenderTargetGraphError::InvalidTargetDimensions {
                    target: target.name.clone(),
                    width: target.width,
                    height: target.height,
                    maximum: MAX_RENDER_TARGET_DIMENSION,
                });
            }
            if target_names.contains(&target.name) {
                return Err(RenderTargetGraphError::DuplicateTarget(target.name.clone()));
            }
            target_names.push(target.name.clone());
        }

        let mut feedback_names = Vec::with_capacity(self.feedback_pairs.len());
        for pair in &self.feedback_pairs {
            validate_name("feedback target pair", &pair.name)?;
            if feedback_names.contains(&pair.name) {
                return Err(RenderTargetGraphError::DuplicateFeedbackPair(
                    pair.name.clone(),
                ));
            }
            feedback_names.push(pair.name.clone());
            if pair.previous_target == pair.current_target {
                return Err(RenderTargetGraphError::FeedbackTargetsMustDiffer {
                    pair: pair.name.clone(),
                    target: pair.previous_target.clone(),
                });
            }
            for target in [&pair.previous_target, &pair.current_target] {
                if !target_names.contains(target) {
                    return Err(RenderTargetGraphError::UnknownFeedbackTarget {
                        pair: pair.name.clone(),
                        target: target.clone(),
                    });
                }
                let requirement = self
                    .targets
                    .iter()
                    .find(|candidate| candidate.name == *target)
                    .expect("feedback target name was validated against the target list");
                if requirement.sampling != RenderTargetSampling::Sampled {
                    return Err(RenderTargetGraphError::FeedbackTargetIsNotSampleable {
                        pair: pair.name.clone(),
                        target: target.clone(),
                    });
                }
            }

            let previous = self
                .targets
                .iter()
                .find(|candidate| candidate.name == pair.previous_target)
                .expect("feedback previous target name was validated against the target list");
            let current = self
                .targets
                .iter()
                .find(|candidate| candidate.name == pair.current_target)
                .expect("feedback current target name was validated against the target list");
            if previous.width != current.width
                || previous.height != current.height
                || previous.color_interpretation != current.color_interpretation
            {
                return Err(RenderTargetGraphError::FeedbackTargetsIncompatible {
                    pair: pair.name.clone(),
                    previous: pair.previous_target.clone(),
                    current: pair.current_target.clone(),
                });
            }
        }

        let mut pass_names = Vec::with_capacity(self.passes.len());
        let mut completed_targets = Vec::<String>::new();
        let mut produced_targets = Vec::<String>::new();
        let mut surface_outputs = 0_usize;
        let mut source_texture_reads = 0_usize;
        let mut render_target_reads = 0_usize;
        let mut previous_frame_target_reads = 0_usize;
        let mut pipeline_names = Vec::<String>::new();
        for (pass_index, pass) in self.passes.iter().enumerate() {
            validate_name("render pass", &pass.name)?;
            validate_name("render pipeline", &pass.pipeline)?;
            if pass_names.contains(&pass.name) {
                return Err(RenderTargetGraphError::DuplicatePass(pass.name.clone()));
            }
            pass_names.push(pass.name.clone());
            if !pipeline_names.contains(&pass.pipeline) {
                pipeline_names.push(pass.pipeline.clone());
            }

            if let VisualizerResource::RenderTarget(target) = &pass.output {
                if !target_names.contains(target) {
                    return Err(RenderTargetGraphError::UnknownTarget {
                        pass: pass.name.clone(),
                        target: target.clone(),
                    });
                }
            }
            if matches!(pass.output, VisualizerResource::Surface) {
                surface_outputs += 1;
                if pass_index + 1 != self.passes.len() {
                    return Err(RenderTargetGraphError::SurfaceOutputMustBeFinal {
                        pass: pass.name.clone(),
                    });
                }
            }

            for input in &pass.inputs {
                match input {
                    VisualizerResource::SourceTexture(name) => {
                        validate_name("source texture", name)?;
                        source_texture_reads += 1;
                    }
                    VisualizerResource::RenderTarget(target) => {
                        render_target_reads += 1;
                        if !target_names.contains(target) {
                            return Err(RenderTargetGraphError::UnknownTarget {
                                pass: pass.name.clone(),
                                target: target.clone(),
                            });
                        }
                        let target_requirement = self
                            .targets
                            .iter()
                            .find(|candidate| candidate.name == *target)
                            .expect("target name was validated against the target list");
                        if target_requirement.sampling != RenderTargetSampling::Sampled {
                            return Err(RenderTargetGraphError::TargetIsNotSampleable {
                                pass: pass.name.clone(),
                                target: target.clone(),
                            });
                        }
                        if matches!(&pass.output, VisualizerResource::RenderTarget(output) if output == target)
                        {
                            return Err(RenderTargetGraphError::ReadWriteHazard {
                                pass: pass.name.clone(),
                                target: target.clone(),
                            });
                        }
                        if !completed_targets.contains(target) {
                            return Err(RenderTargetGraphError::TargetReadBeforeCompletion {
                                pass: pass.name.clone(),
                                target: target.clone(),
                            });
                        }
                    }
                    VisualizerResource::PreviousFrameTarget(pair_name) => {
                        let pair = self
                            .feedback_pairs
                            .iter()
                            .find(|candidate| candidate.name == *pair_name)
                            .ok_or_else(|| RenderTargetGraphError::UnknownFeedbackPair {
                                pass: pass.name.clone(),
                                pair: pair_name.clone(),
                            })?;
                        previous_frame_target_reads += 1;
                        if matches!(&pass.output, VisualizerResource::RenderTarget(output) if output == &pair.previous_target)
                        {
                            return Err(RenderTargetGraphError::FeedbackReadWriteHazard {
                                pass: pass.name.clone(),
                                pair: pair.name.clone(),
                            });
                        }
                    }
                    VisualizerResource::Surface => {
                        return Err(RenderTargetGraphError::SurfaceRead {
                            pass: pass.name.clone(),
                        });
                    }
                }
            }

            if let VisualizerResource::RenderTarget(target) = &pass.output {
                if produced_targets.contains(target) {
                    return Err(RenderTargetGraphError::TargetWrittenMoreThanOnce {
                        pass: pass.name.clone(),
                        target: target.clone(),
                    });
                }
                produced_targets.push(target.clone());
                if !completed_targets.contains(target) {
                    completed_targets.push(target.clone());
                }
            }
        }

        for pair in &self.feedback_pairs {
            if produced_targets.contains(&pair.previous_target) {
                return Err(RenderTargetGraphError::FeedbackPreviousTargetWritten {
                    pair: pair.name.clone(),
                    target: pair.previous_target.clone(),
                });
            }
            if !produced_targets.contains(&pair.current_target) {
                return Err(RenderTargetGraphError::FeedbackCurrentTargetNotWritten {
                    pair: pair.name.clone(),
                    target: pair.current_target.clone(),
                });
            }
        }

        if surface_outputs != 1 {
            return Err(RenderTargetGraphError::ExpectedOneSurfaceOutput {
                actual: surface_outputs,
            });
        }
        Ok(VisualizerPassGraphSummary {
            pass_count: self.passes.len(),
            render_target_count: self.targets.len(),
            source_texture_reads,
            render_target_reads,
            previous_frame_target_reads,
            render_target_writes: produced_targets.len(),
            feedback_pair_count: self.feedback_pairs.len(),
            surface_outputs,
            distinct_pipeline_count: pipeline_names.len(),
            maximum_target_width: self
                .targets
                .iter()
                .map(|target| target.width)
                .max()
                .unwrap_or(0),
            maximum_target_height: self
                .targets
                .iter()
                .map(|target| target.height)
                .max()
                .unwrap_or(0),
        })
    }

    /// Serializes validated pass intent for structural corpus review.
    pub fn to_structural_json(&self) -> Result<String, RenderTargetGraphError> {
        self.validate_with_summary()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| RenderTargetGraphError::Serialization(error.to_string()))
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum RenderTargetGraphError {
    #[error("visualizer pass graph must contain at least one pass")]
    NoPasses,
    #[error("visualizer pass graph contains {actual} targets; maximum is {maximum}")]
    TooManyTargets { actual: usize, maximum: usize },
    #[error("visualizer pass graph contains {actual} passes; maximum is {maximum}")]
    TooManyPasses { actual: usize, maximum: usize },
    #[error("{kind} name must be a non-empty ASCII identifier, received `{name}`")]
    InvalidName { kind: &'static str, name: String },
    #[error(
        "render target `{target}` has invalid dimensions {width}x{height}; maximum is {maximum}"
    )]
    InvalidTargetDimensions {
        target: String,
        width: u32,
        height: u32,
        maximum: u32,
    },
    #[error("render target `{0}` is declared more than once")]
    DuplicateTarget(String),
    #[error("render pass `{0}` is declared more than once")]
    DuplicatePass(String),
    #[error("feedback target pair `{0}` is declared more than once")]
    DuplicateFeedbackPair(String),
    #[error("feedback target pair `{pair}` refers to unknown render target `{target}`")]
    UnknownFeedbackTarget { pair: String, target: String },
    #[error("feedback target pair `{pair}` requires distinct previous/current targets, received `{target}`")]
    FeedbackTargetsMustDiffer { pair: String, target: String },
    #[error("feedback target pair `{pair}` requires sampleable target `{target}`")]
    FeedbackTargetIsNotSampleable { pair: String, target: String },
    #[error(
        "feedback target pair `{pair}` requires compatible dimensions and color interpretation for `{previous}` and `{current}`"
    )]
    FeedbackTargetsIncompatible {
        pair: String,
        previous: String,
        current: String,
    },
    #[error("pass `{pass}` refers to unknown feedback target pair `{pair}`")]
    UnknownFeedbackPair { pass: String, pair: String },
    #[error(
        "pass `{pass}` samples and writes the previous-frame member of feedback pair `{pair}`"
    )]
    FeedbackReadWriteHazard { pass: String, pair: String },
    #[error("feedback target pair `{pair}` writes its previous-frame target `{target}` in the current graph")]
    FeedbackPreviousTargetWritten { pair: String, target: String },
    #[error("feedback target pair `{pair}` does not write its current-frame target `{target}`")]
    FeedbackCurrentTargetNotWritten { pair: String, target: String },
    #[error("pass `{pass}` refers to unknown render target `{target}`")]
    UnknownTarget { pass: String, target: String },
    #[error("pass `{pass}` samples render target `{target}` while writing it")]
    ReadWriteHazard { pass: String, target: String },
    #[error("pass `{pass}` samples render target `{target}` before an earlier pass completes it")]
    TargetReadBeforeCompletion { pass: String, target: String },
    #[error("pass `{pass}` samples render target `{target}` that was not declared sampleable")]
    TargetIsNotSampleable { pass: String, target: String },
    #[error("pass `{pass}` writes render target `{target}` more than once in the same graph")]
    TargetWrittenMoreThanOnce { pass: String, target: String },
    #[error("pass `{pass}` attempts to read the presentation surface")]
    SurfaceRead { pass: String },
    #[error("pass `{pass}` writes the presentation surface before the final pass")]
    SurfaceOutputMustBeFinal { pass: String },
    #[error("visualizer pass graph must have exactly one final surface output, received {actual}")]
    ExpectedOneSurfaceOutput { actual: usize },
    #[error("could not serialize visualizer pass graph: {0}")]
    Serialization(String),
}

fn validate_name(kind: &'static str, name: &str) -> Result<(), RenderTargetGraphError> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(RenderTargetGraphError::InvalidName {
            kind,
            name: name.to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_pass_signal_graph_is_a_valid_headless_contract() {
        let graph = VisualizerPassGraph::two_pass_signal(1280, 720);
        assert_eq!(graph.validate(), Ok(()));
        assert_eq!(
            graph.to_structural_json().unwrap(),
            VisualizerPassGraph::two_pass_signal(1280, 720)
                .to_structural_json()
                .unwrap()
        );
    }

    #[test]
    fn single_pass_surface_graph_requires_no_intermediate_target() {
        let graph = VisualizerPassGraph::single_pass_surface("visualizer-signal");
        let summary = graph.validate_with_summary().unwrap();

        assert_eq!(graph.targets, Vec::new());
        assert_eq!(summary.pass_count, 1);
        assert_eq!(summary.render_target_count, 0);
        assert_eq!(summary.surface_outputs, 1);
        assert_eq!(summary.distinct_pipeline_count, 1);
        assert_eq!(graph.passes[0].output, VisualizerResource::Surface);
    }

    #[test]
    fn three_pass_signal_graph_reports_structural_evidence() {
        let graph = VisualizerPassGraph::three_pass_signal(640, 360);
        let summary = graph.validate_with_summary().unwrap();
        assert_eq!(
            summary,
            VisualizerPassGraphSummary {
                pass_count: 3,
                render_target_count: 2,
                source_texture_reads: 0,
                render_target_reads: 2,
                previous_frame_target_reads: 0,
                render_target_writes: 2,
                feedback_pair_count: 0,
                surface_outputs: 1,
                distinct_pipeline_count: 3,
                maximum_target_width: 640,
                maximum_target_height: 360,
            }
        );
        assert!(graph.to_structural_json().unwrap().contains("composite"));
    }

    #[test]
    fn feedback_graph_keeps_previous_frame_reads_outside_current_frame_ordering() {
        let graph = VisualizerPassGraph::three_pass_feedback(640, 360);
        let summary = graph.validate_with_summary().unwrap();

        assert_eq!(summary.pass_count, 3);
        assert_eq!(summary.feedback_pair_count, 1);
        assert_eq!(summary.previous_frame_target_reads, 1);
        assert_eq!(summary.render_target_reads, 2);
        assert_eq!(summary.render_target_writes, 2);
        assert!(graph
            .to_structural_json()
            .unwrap()
            .contains("previous-frame-target"));
    }

    #[test]
    fn feedback_graph_rejects_writing_the_previous_frame_member() {
        let mut graph = VisualizerPassGraph::three_pass_feedback(64, 64);
        graph.passes[1]
            .inputs
            .retain(|input| !matches!(input, VisualizerResource::PreviousFrameTarget(_)));
        graph.passes[1].output = VisualizerResource::RenderTarget("history-previous".to_owned());
        graph.passes[2].inputs = vec![VisualizerResource::RenderTarget(
            "history-previous".to_owned(),
        )];

        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::FeedbackPreviousTargetWritten { .. })
        ));
    }

    #[test]
    fn feedback_graph_rejects_an_unsampleable_history_member() {
        let mut graph = VisualizerPassGraph::three_pass_feedback(64, 64);
        let previous = graph
            .targets
            .iter_mut()
            .find(|target| target.name == "history-previous")
            .expect("fixture declares the previous history target");
        previous.sampling = RenderTargetSampling::NotSampled;

        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::FeedbackTargetIsNotSampleable { .. })
        ));
    }

    #[test]
    fn feedback_graph_rejects_history_members_with_incompatible_interpretation() {
        let mut graph = VisualizerPassGraph::three_pass_feedback(64, 64);
        let current = graph
            .targets
            .iter_mut()
            .find(|target| target.name == "history-current")
            .expect("fixture declares the current history target");
        current.color_interpretation = RenderTargetColorInterpretation::LinearData;

        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::FeedbackTargetsIncompatible { .. })
        ));
    }

    #[test]
    fn target_cannot_be_sampled_while_its_pass_writes_it() {
        let mut graph = VisualizerPassGraph::two_pass_signal(64, 64);
        graph.passes[0]
            .inputs
            .push(VisualizerResource::RenderTarget("signal".to_owned()));
        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::ReadWriteHazard { .. })
        ));
    }

    #[test]
    fn target_must_be_completed_before_a_later_pass_samples_it() {
        let graph = VisualizerPassGraph {
            targets: vec![RenderTargetRequirement {
                name: "signal".to_owned(),
                width: 64,
                height: 64,
                color_interpretation: RenderTargetColorInterpretation::SrgbColor,
                sampling: RenderTargetSampling::Sampled,
                initial_load: RenderTargetLoadBehavior::Clear([0, 0, 0, 255]),
            }],
            feedback_pairs: Vec::new(),
            passes: vec![VisualizerPassRequirement {
                name: "present".to_owned(),
                pipeline: "visualizer-present".to_owned(),
                inputs: vec![VisualizerResource::RenderTarget("signal".to_owned())],
                output: VisualizerResource::Surface,
            }],
        };
        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::TargetReadBeforeCompletion { .. })
        ));
    }

    #[test]
    fn invalid_target_dimensions_and_intermediate_surface_writes_fail_early() {
        let mut graph = VisualizerPassGraph::two_pass_signal(64, 64);
        graph.targets[0].width = 0;
        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::InvalidTargetDimensions { .. })
        ));

        let mut graph = VisualizerPassGraph::two_pass_signal(64, 64);
        graph.passes[0].output = VisualizerResource::Surface;
        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::SurfaceOutputMustBeFinal { .. })
        ));
    }

    #[test]
    fn every_pass_requires_a_provider_neutral_pipeline_name() {
        let mut graph = VisualizerPassGraph::two_pass_signal(64, 64);
        graph.passes[0].pipeline.clear();

        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::InvalidName {
                kind: "render pipeline",
                ..
            })
        ));
    }

    #[test]
    fn graph_rejects_unsampleable_and_repeated_target_writes() {
        let mut graph = VisualizerPassGraph::two_pass_signal(64, 64);
        graph.targets[0].sampling = RenderTargetSampling::NotSampled;
        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::TargetIsNotSampleable { .. })
        ));

        let mut graph = VisualizerPassGraph::two_pass_signal(64, 64);
        graph.passes.insert(
            1,
            VisualizerPassRequirement {
                name: "signal-again".to_owned(),
                pipeline: "visualizer-signal".to_owned(),
                inputs: Vec::new(),
                output: VisualizerResource::RenderTarget("signal".to_owned()),
            },
        );
        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::TargetWrittenMoreThanOnce { .. })
        ));
    }

    #[test]
    fn graph_rejects_missing_surface_output_and_excessive_graph_sizes() {
        let mut graph = VisualizerPassGraph::two_pass_signal(64, 64);
        graph.passes.pop();
        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::ExpectedOneSurfaceOutput { actual: 0 })
        ));

        let mut graph = VisualizerPassGraph::two_pass_signal(64, 64);
        graph.targets = (0..=MAX_RENDER_TARGETS)
            .map(|index| RenderTargetRequirement {
                name: format!("target-{index}"),
                width: 64,
                height: 64,
                color_interpretation: RenderTargetColorInterpretation::SrgbColor,
                sampling: RenderTargetSampling::Sampled,
                initial_load: RenderTargetLoadBehavior::Clear([0, 0, 0, 255]),
            })
            .collect();
        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::TooManyTargets { .. })
        ));

        let mut graph = VisualizerPassGraph::two_pass_signal(64, 64);
        graph.passes = (0..=MAX_RENDER_PASSES)
            .map(|index| VisualizerPassRequirement {
                name: format!("pass-{index}"),
                pipeline: "visualizer-signal".to_owned(),
                inputs: Vec::new(),
                output: VisualizerResource::Surface,
            })
            .collect();
        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::TooManyPasses { .. })
        ));
    }
}
