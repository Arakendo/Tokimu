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
    /// The native or browser presentation surface.
    Surface,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VisualizerPassRequirement {
    pub name: String,
    pub inputs: Vec<VisualizerResource>,
    pub output: VisualizerResource,
}

/// A bounded provider-neutral visualizer pass graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VisualizerPassGraph {
    pub targets: Vec<RenderTargetRequirement>,
    pub passes: Vec<VisualizerPassRequirement>,
}

impl VisualizerPassGraph {
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
            passes: vec![
                VisualizerPassRequirement {
                    name: "signal".to_owned(),
                    inputs: Vec::new(),
                    output: VisualizerResource::RenderTarget("signal".to_owned()),
                },
                VisualizerPassRequirement {
                    name: "present".to_owned(),
                    inputs: vec![VisualizerResource::RenderTarget("signal".to_owned())],
                    output: VisualizerResource::Surface,
                },
            ],
        }
    }

    /// Validates graph structure before a renderer allocates resources or
    /// records backend commands.
    pub fn validate(&self) -> Result<(), RenderTargetGraphError> {
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

        let mut pass_names = Vec::with_capacity(self.passes.len());
        let mut completed_targets = Vec::<String>::new();
        let mut produced_targets = Vec::<String>::new();
        let mut surface_outputs = 0_usize;
        for (pass_index, pass) in self.passes.iter().enumerate() {
            validate_name("render pass", &pass.name)?;
            if pass_names.contains(&pass.name) {
                return Err(RenderTargetGraphError::DuplicatePass(pass.name.clone()));
            }
            pass_names.push(pass.name.clone());

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
                        validate_name("source texture", name)?
                    }
                    VisualizerResource::RenderTarget(target) => {
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

        if surface_outputs != 1 {
            return Err(RenderTargetGraphError::ExpectedOneSurfaceOutput {
                actual: surface_outputs,
            });
        }
        Ok(())
    }

    /// Serializes validated pass intent for structural corpus review.
    pub fn to_structural_json(&self) -> Result<String, RenderTargetGraphError> {
        self.validate()?;
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
            passes: vec![VisualizerPassRequirement {
                name: "present".to_owned(),
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
                inputs: Vec::new(),
                output: VisualizerResource::RenderTarget("signal".to_owned()),
            },
        );
        assert!(matches!(
            graph.validate(),
            Err(RenderTargetGraphError::TargetWrittenMoreThanOnce { .. })
        ));
    }
}
