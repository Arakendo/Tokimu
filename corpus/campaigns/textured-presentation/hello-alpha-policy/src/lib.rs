//! Headless comparative evidence for AR-0023.
//!
//! All types in this crate are corpus-local study vocabulary. They do not
//! admit or propose a public `tokimu-render` alpha-policy contract.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use thiserror::Error;

pub const VIEWPORT: [u32; 2] = [960, 600];
pub const INTERIOR_THRESHOLD: f32 = 128.0 / 255.0;
pub const VISUAL_PROFILE_TRANSLATIONS: [[f32; 2]; 3] = [[-1.0, 0.35], [0.0, 0.35], [1.0, 0.35]];
pub const VISUAL_PROFILE_SCALE: [f32; 2] = [0.52, 0.52];
pub const VISUAL_DEPTH_TRANSLATION: [f32; 2] = [0.0, -0.55];
pub const VISUAL_DEPTH_SCALE: [f32; 2] = [0.95, 0.36];
pub const VISUAL_FOREGROUND_DEPTH: f32 = 0.0;
pub const VISUAL_BACKGROUND_DEPTH: f32 = 0.5;
pub const BLEND_NEAR_DEPTH: f32 = 0.5;
pub const BLEND_FAR_DEPTH: f32 = 0.25;
pub const BLEND_BACKGROUND_DEPTH: f32 = 0.0;
pub const BLEND_REFERENCE_DEPTH: f32 = 0.75;
pub const BLEND_PANEL_SCALE: [f32; 2] = [0.64, 0.44];
pub const BLEND_PANELS: [[f32; 2]; 4] =
    [[-0.55, 0.36], [0.55, 0.36], [-0.55, -0.44], [0.55, -0.44]];
pub const BLEND_FAR_OFFSET: [f32; 2] = [-0.09, 0.0];
pub const BLEND_NEAR_OFFSET: [f32; 2] = [0.09, 0.0];
pub const BLEND_REFERENCE_TRANSLATION: [f32; 2] = [-0.84, 0.84];
pub const INTERACTION_PANELS: [[f32; 2]; 3] = [[-0.58, 0.34], [0.58, 0.34], [0.0, -0.46]];
pub const INTERACTION_PANEL_SCALE: [f32; 2] = [0.68, 0.42];
// Under the established 2D orthographic fixture camera, larger positive
// Tokimu Z is nearer after the WGPU boundary conversion. Keep the backing at
// zero and make the sloped Blend cross the fixed cutout plane at 0.5.
pub const INTERACTION_BACKGROUND_DEPTH: f32 = 0.0;
pub const INTERACTION_FOREGROUND_DEPTH: f32 = 0.5;
pub const INTERACTION_BLEND_LEFT_DEPTH: f32 = 0.6;
pub const INTERACTION_BLEND_RIGHT_DEPTH: f32 = 0.3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FixtureId {
    OpaqueControl,
    BinaryMask,
    ThresholdBoundary,
    ContinuousGradient,
    MixedAlpha,
    ColoredTransparent,
}

impl FixtureId {
    pub const fn label(self) -> &'static str {
        match self {
            Self::OpaqueControl => "opaque-control",
            Self::BinaryMask => "binary-mask",
            Self::ThresholdBoundary => "threshold-boundary",
            Self::ContinuousGradient => "continuous-gradient",
            Self::MixedAlpha => "mixed-alpha",
            Self::ColoredTransparent => "colored-transparent",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgba8Fixture {
    id: FixtureId,
    width: u32,
    height: u32,
    rgba8: Vec<u8>,
}

impl Rgba8Fixture {
    fn new(id: FixtureId, width: u32, height: u32, pixels: &[[u8; 4]]) -> Self {
        let rgba8 = pixels.iter().flatten().copied().collect();
        Self {
            id,
            width,
            height,
            rgba8,
        }
    }

    pub const fn id(&self) -> FixtureId {
        self.id
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub fn rgba8(&self) -> &[u8] {
        &self.rgba8
    }

    pub fn alpha_distribution(&self) -> BTreeMap<u8, usize> {
        let mut distribution = BTreeMap::new();
        for alpha in self.rgba8.chunks_exact(4).map(|pixel| pixel[3]) {
            *distribution.entry(alpha).or_insert(0) += 1;
        }
        distribution
    }

    pub fn fingerprint_blake3(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.width.to_le_bytes());
        hasher.update(&self.height.to_le_bytes());
        hasher.update(&self.rgba8);
        hasher.finalize().to_hex().to_string()
    }

    pub fn validate(&self) -> Result<(), StudyError> {
        validate_rgba8(self.width, self.height, &self.rgba8)
    }
}

pub fn fixtures() -> Vec<Rgba8Fixture> {
    let opaque = [
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [255, 255, 255, 255],
    ];
    let binary = [
        [255, 32, 32, 0],
        [32, 255, 32, 255],
        [32, 32, 255, 0],
        [255, 255, 32, 255],
    ];
    let threshold = [
        [255, 64, 64, 0],
        [255, 96, 64, 127],
        [255, 128, 64, 128],
        [255, 160, 64, 129],
        [255, 224, 64, 255],
    ];
    let gradient = (0_u16..=255)
        .map(|alpha| {
            let alpha = alpha as u8;
            [255_u8.wrapping_sub(alpha), alpha, 192, alpha]
        })
        .collect::<Vec<_>>();
    let mixed = [
        [255, 32, 32, 0],
        [255, 128, 32, 64],
        [32, 255, 128, 128],
        [32, 128, 255, 192],
        [255, 255, 255, 255],
    ];
    let colored_transparent = [
        [255, 0, 0, 0],
        [0, 255, 0, 0],
        [0, 0, 255, 0],
        [255, 255, 255, 0],
    ];

    vec![
        Rgba8Fixture::new(FixtureId::OpaqueControl, 4, 1, &opaque),
        Rgba8Fixture::new(FixtureId::BinaryMask, 4, 1, &binary),
        Rgba8Fixture::new(FixtureId::ThresholdBoundary, 5, 1, &threshold),
        Rgba8Fixture::new(FixtureId::ContinuousGradient, 256, 1, &gradient),
        Rgba8Fixture::new(FixtureId::MixedAlpha, 5, 1, &mixed),
        Rgba8Fixture::new(FixtureId::ColoredTransparent, 4, 1, &colored_transparent),
    ]
}

pub fn validate_rgba8(width: u32, height: u32, rgba8: &[u8]) -> Result<(), StudyError> {
    let expected = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(StudyError::DimensionsOverflow { width, height })? as usize;
    if rgba8.len() != expected {
        return Err(StudyError::MalformedRgba8 {
            width,
            height,
            expected,
            actual: rgba8.len(),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub struct CutoutThreshold(f32);

impl CutoutThreshold {
    pub fn new(value: f32) -> Result<Self, StudyError> {
        if !value.is_finite() {
            return Err(StudyError::NonFiniteThreshold);
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(StudyError::ThresholdOutOfRange { value });
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThresholdComparison {
    DiscardBelow,
    DiscardAtOrBelow,
}

/// Builds the corpus-local cutout realization shared by native and browser
/// targets. This is executable study evidence, not admitted renderer
/// vocabulary or a proposed stable shader-authoring contract.
pub fn cutout_shader_source(comparison: ThresholdComparison, threshold: CutoutThreshold) -> String {
    let operator = match comparison {
        ThresholdComparison::DiscardBelow => "<",
        ThresholdComparison::DiscardAtOrBelow => "<=",
    };
    let threshold = threshold.get();
    format!(
        r#"
@group(0) @binding(0) var<uniform> material_color: vec4<f32>;
@group(0) @binding(1) var material_texture: texture_2d<f32>;
@group(0) @binding(2) var material_sampler: sampler;
struct InstanceParams {{ translation: vec2<f32>, scale: vec2<f32>, rotation: vec2<f32>, padding: vec2<f32>, }};
@group(1) @binding(0) var<uniform> instance_params: InstanceParams;
@group(2) @binding(0) var<uniform> camera_params: mat4x4<f32>;
struct VertexOutput {{ @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, }};
@vertex fn vs_main(@location(0) position: vec3<f32>, @location(1) _normal: vec3<f32>, @location(2) uv: vec2<f32>) -> VertexOutput {{
    let scaled = position.xy * instance_params.scale;
    let rotated = vec2<f32>((scaled.x * instance_params.rotation.y) - (scaled.y * instance_params.rotation.x), (scaled.x * instance_params.rotation.x) + (scaled.y * instance_params.rotation.y));
    var output: VertexOutput;
    output.position = camera_params * vec4<f32>(rotated.x + instance_params.translation.x, rotated.y + instance_params.translation.y, position.z, 1.0);
    output.uv = uv;
    return output;
}}
@fragment fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {{
    let sampled = textureSample(material_texture, material_sampler, uv) * material_color;
    if (sampled.a {operator} {threshold:.7}) {{ discard; }}
    return sampled;
}}
"#
    )
}

/// Builds the corpus-local straight-alpha blend realization used by the
/// ordering/depth comparison. The blend equation remains the experimental
/// pipeline state; this shader merely preserves the supplied RGBA sample.
/// It is not a public shader or material contract.
pub fn blend_shader_source() -> &'static str {
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CallerOrdering {
    draw_ids: Vec<String>,
}

impl CallerOrdering {
    pub fn new(draw_ids: Vec<String>) -> Result<Self, StudyError> {
        if draw_ids.is_empty() {
            return Err(StudyError::EmptyOrdering);
        }
        let mut seen = BTreeSet::new();
        for draw_id in &draw_ids {
            if draw_id.is_empty() {
                return Err(StudyError::EmptyDrawIdentity);
            }
            if !seen.insert(draw_id.clone()) {
                return Err(StudyError::DuplicateDrawIdentity(draw_id.clone()));
            }
        }
        Ok(Self { draw_ids })
    }

    pub fn draw_ids(&self) -> &[String] {
        &self.draw_ids
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum StudyProfile {
    Opaque {
        depth_write: bool,
    },
    Cutout {
        threshold: CutoutThreshold,
        comparison: ThresholdComparison,
        depth_write: bool,
    },
    Blend {
        depth_write: bool,
        ordering: CallerOrdering,
    },
}

impl StudyProfile {
    pub const fn opaque(depth_write: bool) -> Self {
        Self::Opaque { depth_write }
    }

    pub const fn cutout(
        threshold: CutoutThreshold,
        comparison: ThresholdComparison,
        depth_write: bool,
    ) -> Self {
        Self::Cutout {
            threshold,
            comparison,
            depth_write,
        }
    }

    pub fn blend(depth_write: bool, ordering: Option<Vec<String>>) -> Result<Self, StudyError> {
        let ordering = ordering.ok_or(StudyError::MissingBlendOrdering)?;
        Ok(Self::Blend {
            depth_write,
            ordering: CallerOrdering::new(ordering)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FragmentAction {
    Keep,
    Discard,
    Blend,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FragmentObservation {
    pub action: FragmentAction,
    pub source_alpha: f32,
    pub writes_depth: bool,
    pub resulting_rgba: Option<[f32; 4]>,
}

pub fn evaluate_fragment(
    profile: &StudyProfile,
    source: [u8; 4],
    destination: [u8; 4],
) -> FragmentObservation {
    let source = normalize(source);
    let destination = normalize(destination);
    match profile {
        StudyProfile::Opaque { depth_write } => FragmentObservation {
            action: FragmentAction::Keep,
            source_alpha: source[3],
            writes_depth: *depth_write,
            resulting_rgba: Some(source),
        },
        StudyProfile::Cutout {
            threshold,
            comparison,
            depth_write,
        } => {
            let discard = match comparison {
                ThresholdComparison::DiscardBelow => source[3] < threshold.get(),
                ThresholdComparison::DiscardAtOrBelow => source[3] <= threshold.get(),
            };
            if discard {
                FragmentObservation {
                    action: FragmentAction::Discard,
                    source_alpha: source[3],
                    writes_depth: false,
                    resulting_rgba: None,
                }
            } else {
                FragmentObservation {
                    action: FragmentAction::Keep,
                    source_alpha: source[3],
                    writes_depth: *depth_write,
                    resulting_rgba: Some(source),
                }
            }
        }
        StudyProfile::Blend { depth_write, .. } => {
            let inverse = 1.0 - source[3];
            FragmentObservation {
                action: FragmentAction::Blend,
                source_alpha: source[3],
                writes_depth: *depth_write,
                resulting_rgba: Some([
                    source[0] * source[3] + destination[0] * inverse,
                    source[1] * source[3] + destination[1] * inverse,
                    source[2] * source[3] + destination[2] * inverse,
                    source[3] + destination[3] * inverse,
                ]),
            }
        }
    }
}

fn normalize(rgba: [u8; 4]) -> [f32; 4] {
    rgba.map(|channel| channel as f32 / 255.0)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StudyDraw {
    pub id: &'static str,
    pub translation: [f32; 3],
    pub rotation_degrees: [f32; 3],
    pub scale: [f32; 2],
    pub depth: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SceneCase {
    pub id: &'static str,
    pub fixture: FixtureId,
    pub draws: Vec<StudyDraw>,
    pub variable: &'static str,
}

/// Fixed Slice 4 submission evidence shared by the native and browser
/// realizations. This is corpus-local manifest data, not renderer vocabulary.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InteractionManifest {
    pub viewport: [u32; 2],
    pub panels: [[f32; 2]; 3],
    pub panel_scale: [f32; 2],
    pub backing_depth: f32,
    pub cutout_depth: f32,
    pub blend_left_depth: f32,
    pub blend_right_depth: f32,
    pub binary_fixture_fingerprint_blake3: String,
    pub mixed_fixture_fingerprint_blake3: String,
    pub submissions: Vec<InteractionSubmission>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InteractionSubmission {
    pub id: &'static str,
    pub fixture: Option<FixtureId>,
    pub profile: &'static str,
    pub depth_shape: &'static str,
}

pub fn interaction_manifest() -> InteractionManifest {
    let fixture_fingerprint = |id| {
        fixtures()
            .into_iter()
            .find(|fixture| fixture.id() == id)
            .expect("fixed alpha fixture exists")
            .fingerprint_blake3()
    };
    InteractionManifest {
        viewport: VIEWPORT,
        panels: INTERACTION_PANELS,
        panel_scale: INTERACTION_PANEL_SCALE,
        backing_depth: INTERACTION_BACKGROUND_DEPTH,
        cutout_depth: INTERACTION_FOREGROUND_DEPTH,
        blend_left_depth: INTERACTION_BLEND_LEFT_DEPTH,
        blend_right_depth: INTERACTION_BLEND_RIGHT_DEPTH,
        binary_fixture_fingerprint_blake3: fixture_fingerprint(FixtureId::BinaryMask),
        mixed_fixture_fingerprint_blake3: fixture_fingerprint(FixtureId::MixedAlpha),
        submissions: vec![
            interaction_submission("cutout-backing", None, "opaque", "flat-backing"),
            interaction_submission(
                "cutout-foreground",
                Some(FixtureId::BinaryMask),
                "cutout-discard-below-depth-write",
                "flat-foreground",
            ),
            interaction_submission("blend-backing", None, "opaque", "flat-backing"),
            interaction_submission(
                "blend-foreground",
                Some(FixtureId::MixedAlpha),
                "blend-no-depth-write",
                "flat-foreground",
            ),
            interaction_submission("crossing-backing", None, "opaque", "flat-backing"),
            interaction_submission(
                "crossing-blend",
                Some(FixtureId::MixedAlpha),
                "blend-depth-write",
                "sloped-left-near-right-far",
            ),
            interaction_submission(
                "crossing-cutout",
                Some(FixtureId::BinaryMask),
                "cutout-discard-below-depth-write",
                "flat-foreground",
            ),
        ],
    }
}

pub fn interaction_manifest_fingerprint() -> Result<String, serde_json::Error> {
    let json = serde_json::to_vec(&interaction_manifest())?;
    Ok(blake3::hash(&json).to_hex().to_string())
}

const fn interaction_submission(
    id: &'static str,
    fixture: Option<FixtureId>,
    profile: &'static str,
    depth_shape: &'static str,
) -> InteractionSubmission {
    InteractionSubmission {
        id,
        fixture,
        profile,
        depth_shape,
    }
}

pub fn scene_cases() -> Vec<SceneCase> {
    vec![
        scene(
            "same-texture-three-profiles",
            FixtureId::MixedAlpha,
            &[
                draw("opaque", [-2.0, 0.0, 0.0], [0.0; 3], 0.5),
                draw("cutout", [0.0, 0.0, 0.0], [0.0; 3], 0.5),
                draw("blend", [2.0, 0.0, 0.0], [0.0; 3], 0.5),
            ],
            "profile-only",
        ),
        scene(
            "cutout-over-opaque",
            FixtureId::BinaryMask,
            &[
                draw("background", [0.0; 3], [0.0; 3], 0.75),
                draw("cutout", [0.0; 3], [0.0; 3], 0.25),
            ],
            "categorical-foreground",
        ),
        scene(
            "blend-over-opaque",
            FixtureId::ContinuousGradient,
            &[
                draw("background", [0.0; 3], [0.0; 3], 0.75),
                draw("blend", [0.0; 3], [0.0; 3], 0.25),
            ],
            "continuous-foreground-and-depth-write",
        ),
        scene(
            "overlapping-blend-back-to-front",
            FixtureId::MixedAlpha,
            &[
                draw("background", [0.0; 3], [0.0; 3], 0.9),
                draw("far-blend", [0.0; 3], [0.0; 3], 0.65),
                draw("near-blend", [0.0; 3], [0.0; 3], 0.35),
            ],
            "caller-order",
        ),
        scene(
            "overlapping-blend-front-to-back",
            FixtureId::MixedAlpha,
            &[
                draw("background", [0.0; 3], [0.0; 3], 0.9),
                draw("near-blend", [0.0; 3], [0.0; 3], 0.35),
                draw("far-blend", [0.0; 3], [0.0; 3], 0.65),
            ],
            "reversed-caller-order",
        ),
        scene(
            "cutout-blend-intersection",
            FixtureId::MixedAlpha,
            &[
                draw("background", [0.0; 3], [0.0; 3], 0.9),
                draw("blend", [0.0; 3], [0.0; 3], 0.55),
                draw("cutout", [0.0; 3], [0.0, 0.0, 45.0], 0.45),
            ],
            "capability-interaction",
        ),
        scene(
            "identical-depth-overlap",
            FixtureId::MixedAlpha,
            &[
                draw("background", [0.0; 3], [0.0; 3], 0.9),
                draw("first", [0.0; 3], [0.0; 3], 0.5),
                draw("second", [0.0; 3], [0.0; 3], 0.5),
            ],
            "depth-comparison-not-alpha-inference",
        ),
    ]
}

fn scene(
    id: &'static str,
    fixture: FixtureId,
    draws: &[StudyDraw],
    variable: &'static str,
) -> SceneCase {
    SceneCase {
        id,
        fixture,
        draws: draws.to_vec(),
        variable,
    }
}

const fn draw(
    id: &'static str,
    translation: [f32; 3],
    rotation_degrees: [f32; 3],
    depth: f32,
) -> StudyDraw {
    StudyDraw {
        id,
        translation,
        rotation_degrees,
        scale: [1.0, 1.0],
        depth,
    }
}

pub fn scene_fingerprint(case: &SceneCase) -> Result<String, serde_json::Error> {
    let json = serde_json::to_vec(case)?;
    Ok(blake3::hash(&json).to_hex().to_string())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureObservation {
    id: FixtureId,
    label: &'static str,
    width: u32,
    height: u32,
    fingerprint_blake3: String,
    alpha_distribution: BTreeMap<u8, usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneObservation {
    case: SceneCase,
    fingerprint_blake3: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FragmentSampleObservation {
    fixture: FixtureId,
    source_index: usize,
    source_rgba8: [u8; 4],
    profile_label: &'static str,
    profile: StudyProfile,
    fragment: FragmentObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StudyFailureKind {
    NonFiniteThreshold,
    ThresholdOutOfRange,
    MissingBlendOrdering,
    EmptyOrdering,
    EmptyDrawIdentity,
    DuplicateDrawIdentity,
    DimensionsOverflow,
    MalformedRgba8,
    Serialization,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailureObservation {
    case: &'static str,
    kind: StudyFailureKind,
    diagnostic: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyReport {
    schema_version: u32,
    viewport: [u32; 2],
    fixtures: Vec<FixtureObservation>,
    scenes: Vec<SceneObservation>,
    fragment_samples: Vec<FragmentSampleObservation>,
    failure_observations: Vec<FailureObservation>,
}

pub fn study_report() -> Result<StudyReport, StudyError> {
    let fixtures = fixtures()
        .into_iter()
        .map(|fixture| {
            fixture.validate()?;
            Ok(FixtureObservation {
                id: fixture.id(),
                label: fixture.id().label(),
                width: fixture.width(),
                height: fixture.height(),
                fingerprint_blake3: fixture.fingerprint_blake3(),
                alpha_distribution: fixture.alpha_distribution(),
            })
        })
        .collect::<Result<Vec<_>, StudyError>>()?;
    let scenes = scene_cases()
        .into_iter()
        .map(|case| {
            let fingerprint_blake3 = scene_fingerprint(&case)?;
            Ok(SceneObservation {
                case,
                fingerprint_blake3,
            })
        })
        .collect::<Result<Vec<_>, StudyError>>()?;
    Ok(StudyReport {
        schema_version: 1,
        viewport: VIEWPORT,
        fixtures,
        scenes,
        fragment_samples: fragment_samples()?,
        failure_observations: baseline_failure_observations(),
    })
}

fn fragment_samples() -> Result<Vec<FragmentSampleObservation>, StudyError> {
    let threshold = CutoutThreshold::new(INTERIOR_THRESHOLD)?;
    let profiles = vec![
        ("opaque-depth-write", StudyProfile::opaque(true)),
        (
            "cutout-discard-below-depth-write",
            StudyProfile::cutout(threshold, ThresholdComparison::DiscardBelow, true),
        ),
        (
            "cutout-discard-at-or-below-depth-write",
            StudyProfile::cutout(threshold, ThresholdComparison::DiscardAtOrBelow, true),
        ),
        (
            "blend-no-depth-write",
            StudyProfile::blend(false, Some(vec!["background".into(), "source".into()]))?,
        ),
        (
            "blend-depth-write",
            StudyProfile::blend(true, Some(vec!["background".into(), "source".into()]))?,
        ),
    ];
    let mut samples = Vec::new();
    for fixture in fixtures().into_iter().filter(|fixture| {
        matches!(
            fixture.id(),
            FixtureId::ThresholdBoundary | FixtureId::MixedAlpha
        )
    }) {
        for (source_index, source) in fixture.rgba8().chunks_exact(4).enumerate() {
            let source_rgba8 = [source[0], source[1], source[2], source[3]];
            for (profile_label, profile) in &profiles {
                samples.push(FragmentSampleObservation {
                    fixture: fixture.id(),
                    source_index,
                    source_rgba8,
                    profile_label,
                    profile: profile.clone(),
                    fragment: evaluate_fragment(profile, source_rgba8, [16, 32, 64, 255]),
                });
            }
        }
    }
    Ok(samples)
}

fn baseline_failure_observations() -> Vec<FailureObservation> {
    vec![
        failure("threshold-nan", CutoutThreshold::new(f32::NAN).unwrap_err()),
        failure(
            "threshold-positive-infinity",
            CutoutThreshold::new(f32::INFINITY).unwrap_err(),
        ),
        failure(
            "threshold-negative",
            CutoutThreshold::new(-0.01).unwrap_err(),
        ),
        failure(
            "threshold-above-one",
            CutoutThreshold::new(1.01).unwrap_err(),
        ),
        failure(
            "blend-missing-order",
            StudyProfile::blend(false, None).unwrap_err(),
        ),
        failure(
            "blend-empty-order",
            StudyProfile::blend(false, Some(Vec::new())).unwrap_err(),
        ),
        failure(
            "blend-empty-draw-identity",
            StudyProfile::blend(false, Some(vec![String::new()])).unwrap_err(),
        ),
        failure(
            "blend-duplicate-draw-identity",
            StudyProfile::blend(false, Some(vec!["same".into(), "same".into()])).unwrap_err(),
        ),
        failure("rgba8-length", validate_rgba8(2, 2, &[0; 15]).unwrap_err()),
        failure(
            "rgba8-dimensions-overflow",
            validate_rgba8(u32::MAX, u32::MAX, &[]).unwrap_err(),
        ),
    ]
}

fn failure(case: &'static str, error: StudyError) -> FailureObservation {
    FailureObservation {
        case,
        kind: error.kind(),
        diagnostic: error.to_string(),
    }
}

pub fn study_report_json() -> Result<String, StudyError> {
    Ok(serde_json::to_string_pretty(&study_report()?)?)
}

#[derive(Debug, Error)]
pub enum StudyError {
    #[error("cutout threshold must be finite")]
    NonFiniteThreshold,
    #[error("cutout threshold {value} is outside inclusive range 0..=1")]
    ThresholdOutOfRange { value: f32 },
    #[error("blend profile requires explicit caller ordering")]
    MissingBlendOrdering,
    #[error("caller ordering cannot be empty")]
    EmptyOrdering,
    #[error("caller ordering contains an empty draw identity")]
    EmptyDrawIdentity,
    #[error("caller ordering repeats draw identity `{0}`")]
    DuplicateDrawIdentity(String),
    #[error("RGBA8 dimensions {width}x{height} overflow byte-count calculation")]
    DimensionsOverflow { width: u32, height: u32 },
    #[error("RGBA8 payload for {width}x{height} requires {expected} bytes, received {actual}")]
    MalformedRgba8 {
        width: u32,
        height: u32,
        expected: usize,
        actual: usize,
    },
    #[error("failed to serialize study evidence: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl StudyError {
    pub const fn kind(&self) -> StudyFailureKind {
        match self {
            Self::NonFiniteThreshold => StudyFailureKind::NonFiniteThreshold,
            Self::ThresholdOutOfRange { .. } => StudyFailureKind::ThresholdOutOfRange,
            Self::MissingBlendOrdering => StudyFailureKind::MissingBlendOrdering,
            Self::EmptyOrdering => StudyFailureKind::EmptyOrdering,
            Self::EmptyDrawIdentity => StudyFailureKind::EmptyDrawIdentity,
            Self::DuplicateDrawIdentity(_) => StudyFailureKind::DuplicateDrawIdentity,
            Self::DimensionsOverflow { .. } => StudyFailureKind::DimensionsOverflow,
            Self::MalformedRgba8 { .. } => StudyFailureKind::MalformedRgba8,
            Self::Serialization(_) => StudyFailureKind::Serialization,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    // Corpus-only reference compositor for the fixed LessEqual-depth cases.
    // It establishes the question the visual fixtures exercise; it does not
    // specify a renderer-owned ordering or depth API.
    fn composite_layers(profile: &StudyProfile, layers: &[(f32, [u8; 4])]) -> [f32; 4] {
        let mut destination = [0.0, 0.0, 0.0, 1.0];
        let mut stored_depth = 1.0;
        for (depth, source) in layers {
            if *depth > stored_depth {
                continue;
            }
            let observation = evaluate_fragment(profile, *source, denormalize(destination));
            if let Some(result) = observation.resulting_rgba {
                destination = result;
                if observation.writes_depth {
                    stored_depth = *depth;
                }
            }
        }
        destination
    }

    fn denormalize(rgba: [f32; 4]) -> [u8; 4] {
        rgba.map(|channel| (channel * 255.0).round() as u8)
    }

    #[test]
    fn fixture_matrix_is_complete_and_well_formed() {
        let fixtures = fixtures();
        assert_eq!(fixtures.len(), 6);
        assert_eq!(
            fixtures
                .iter()
                .map(Rgba8Fixture::id)
                .collect::<BTreeSet<_>>()
                .len(),
            6
        );
        for fixture in fixtures {
            fixture.validate().unwrap();
            assert_eq!(
                fixture.alpha_distribution().values().sum::<usize>(),
                (fixture.width() * fixture.height()) as usize
            );
        }
    }

    #[test]
    fn fixture_fingerprints_match_the_retained_manifest() {
        let expected = BTreeMap::from([
            (
                FixtureId::OpaqueControl,
                "3678aa1245e5e36ceb1f5b59dbc60e6da129db027ef6df452f600c644e99d129",
            ),
            (
                FixtureId::BinaryMask,
                "37dc5c494f2394b7c7c99eca6cc800f039975fb6add1e48868fbc965657fa48e",
            ),
            (
                FixtureId::ThresholdBoundary,
                "62558ade2bf5d4ca32c79d234ce4f282f37adf01b551a52907fb60c26df69e2d",
            ),
            (
                FixtureId::ContinuousGradient,
                "7e57ab1608b24e89af1dda5c1ff51cfe9f8e74fe9d063a42cd1b371debbff6bd",
            ),
            (
                FixtureId::MixedAlpha,
                "2d82b95538bf2af33e88a9eb1bd1a2de73e9a1a15d3305f1268c048e7c9fc4dd",
            ),
            (
                FixtureId::ColoredTransparent,
                "cc4f041a142a01cff05acf6c8967921cde5c7a56ebb367f656e6ec8d190ca572",
            ),
        ]);
        for fixture in fixtures() {
            assert_eq!(
                fixture.fingerprint_blake3(),
                expected[&fixture.id()],
                "fixture {} drifted from its retained source identity",
                fixture.id().label()
            );
        }
    }

    #[test]
    fn continuous_gradient_contains_every_alpha_byte_once() {
        let fixture = fixtures()
            .into_iter()
            .find(|fixture| fixture.id() == FixtureId::ContinuousGradient)
            .unwrap();
        assert_eq!(fixture.alpha_distribution().len(), 256);
        assert!(fixture
            .alpha_distribution()
            .values()
            .all(|count| *count == 1));
    }

    #[test]
    fn identical_source_alpha_has_distinct_declared_semantics() {
        let source = [255, 128, 32, 64];
        let destination = [16, 32, 64, 255];
        let opaque = StudyProfile::opaque(true);
        let cutout = StudyProfile::cutout(
            CutoutThreshold::new(INTERIOR_THRESHOLD).unwrap(),
            ThresholdComparison::DiscardBelow,
            true,
        );
        let blend = StudyProfile::blend(false, Some(ids(&["background", "foreground"]))).unwrap();

        assert_eq!(
            evaluate_fragment(&opaque, source, destination).action,
            FragmentAction::Keep
        );
        assert_eq!(
            evaluate_fragment(&cutout, source, destination).action,
            FragmentAction::Discard
        );
        assert_eq!(
            evaluate_fragment(&blend, source, destination).action,
            FragmentAction::Blend
        );
    }

    #[test]
    fn threshold_comparison_keeps_the_equal_case_distinct() {
        let threshold = CutoutThreshold::new(INTERIOR_THRESHOLD).unwrap();
        let below = StudyProfile::cutout(threshold, ThresholdComparison::DiscardBelow, true);
        let at_or_below =
            StudyProfile::cutout(threshold, ThresholdComparison::DiscardAtOrBelow, true);
        let equal = [255, 255, 255, 128];
        assert_eq!(
            evaluate_fragment(&below, equal, [0; 4]).action,
            FragmentAction::Keep
        );
        assert_eq!(
            evaluate_fragment(&at_or_below, equal, [0; 4]).action,
            FragmentAction::Discard
        );
    }

    #[test]
    fn zero_and_one_thresholds_keep_their_explicit_boundary_meanings() {
        let transparent = [255, 255, 255, 0];
        let opaque = [255, 255, 255, 255];
        let zero = CutoutThreshold::new(0.0).unwrap();
        let one = CutoutThreshold::new(1.0).unwrap();

        let discard_below_zero =
            StudyProfile::cutout(zero, ThresholdComparison::DiscardBelow, true);
        let discard_at_or_below_zero =
            StudyProfile::cutout(zero, ThresholdComparison::DiscardAtOrBelow, true);
        assert_eq!(
            evaluate_fragment(&discard_below_zero, transparent, [0; 4]).action,
            FragmentAction::Keep
        );
        assert_eq!(
            evaluate_fragment(&discard_at_or_below_zero, transparent, [0; 4]).action,
            FragmentAction::Discard
        );

        let discard_below_one = StudyProfile::cutout(one, ThresholdComparison::DiscardBelow, true);
        let discard_at_or_below_one =
            StudyProfile::cutout(one, ThresholdComparison::DiscardAtOrBelow, true);
        assert_eq!(
            evaluate_fragment(&discard_below_one, opaque, [0; 4]).action,
            FragmentAction::Keep
        );
        assert_eq!(
            evaluate_fragment(&discard_at_or_below_one, opaque, [0; 4]).action,
            FragmentAction::Discard
        );
    }

    #[test]
    fn discarded_cutout_fragment_never_writes_depth() {
        let profile = StudyProfile::cutout(
            CutoutThreshold::new(INTERIOR_THRESHOLD).unwrap(),
            ThresholdComparison::DiscardBelow,
            true,
        );
        let observation = evaluate_fragment(&profile, [255, 0, 0, 0], [0; 4]);
        assert_eq!(observation.action, FragmentAction::Discard);
        assert!(!observation.writes_depth);
        assert_eq!(observation.resulting_rgba, None);
    }

    #[test]
    fn opaque_keeps_colored_zero_alpha_texels_visible() {
        let observation = evaluate_fragment(&StudyProfile::opaque(true), [255, 0, 0, 0], [0; 4]);
        assert_eq!(observation.action, FragmentAction::Keep);
        assert!(observation.writes_depth);
        assert!(observation.resulting_rgba.is_some());
    }

    #[test]
    fn invalid_thresholds_are_rejected_without_clamping() {
        assert!(matches!(
            CutoutThreshold::new(f32::NAN),
            Err(StudyError::NonFiniteThreshold)
        ));
        assert!(matches!(
            CutoutThreshold::new(f32::INFINITY),
            Err(StudyError::NonFiniteThreshold)
        ));
        assert!(matches!(
            CutoutThreshold::new(-0.01),
            Err(StudyError::ThresholdOutOfRange { .. })
        ));
        assert!(matches!(
            CutoutThreshold::new(1.01),
            Err(StudyError::ThresholdOutOfRange { .. })
        ));
    }

    #[test]
    fn blend_requires_nonempty_unique_caller_ordering() {
        assert!(matches!(
            StudyProfile::blend(false, None),
            Err(StudyError::MissingBlendOrdering)
        ));
        assert!(matches!(
            StudyProfile::blend(false, Some(vec![])),
            Err(StudyError::EmptyOrdering)
        ));
        assert!(matches!(
            StudyProfile::blend(false, Some(ids(&["same", "same"]))),
            Err(StudyError::DuplicateDrawIdentity(_))
        ));
    }

    #[test]
    fn malformed_rgba8_is_rejected() {
        assert!(matches!(
            validate_rgba8(2, 2, &[0; 15]),
            Err(StudyError::MalformedRgba8 {
                expected: 16,
                actual: 15,
                ..
            })
        ));
    }

    #[test]
    fn reversed_blend_scenes_retain_distinct_fingerprints() {
        let cases = scene_cases();
        let back_to_front = cases
            .iter()
            .find(|case| case.id == "overlapping-blend-back-to-front")
            .unwrap();
        let front_to_back = cases
            .iter()
            .find(|case| case.id == "overlapping-blend-front-to-back")
            .unwrap();
        assert_ne!(
            scene_fingerprint(back_to_front).unwrap(),
            scene_fingerprint(front_to_back).unwrap()
        );
        for id in ["background", "near-blend", "far-blend"] {
            let first = back_to_front
                .draws
                .iter()
                .find(|draw| draw.id == id)
                .unwrap();
            let second = front_to_back
                .draws
                .iter()
                .find(|draw| draw.id == id)
                .unwrap();
            assert_eq!(first.translation, second.translation);
            assert_eq!(first.rotation_degrees, second.rotation_degrees);
            assert_eq!(first.scale, second.scale);
            assert_eq!(first.depth, second.depth);
        }
        assert_ne!(
            back_to_front
                .draws
                .iter()
                .map(|draw| draw.id)
                .collect::<Vec<_>>(),
            front_to_back
                .draws
                .iter()
                .map(|draw| draw.id)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn opaque_and_cutout_are_depth_order_invariant_while_blend_is_not() {
        let background = (0.9, [16, 32, 64, 255]);
        let far = (0.65, [255, 0, 0, 128]);
        let near = (0.35, [0, 255, 0, 128]);
        let far_then_near = [background, far, near];
        let near_then_far = [background, near, far];

        let opaque = StudyProfile::opaque(true);
        assert_eq!(
            composite_layers(&opaque, &far_then_near),
            composite_layers(&opaque, &near_then_far),
            "opaque coverage remains governed by the nearer depth, not submission order"
        );

        let cutout = StudyProfile::cutout(
            CutoutThreshold::new(INTERIOR_THRESHOLD).unwrap(),
            ThresholdComparison::DiscardBelow,
            true,
        );
        assert_eq!(
            composite_layers(&cutout, &far_then_near),
            composite_layers(&cutout, &near_then_far),
            "retained cutout coverage remains governed by the nearer depth"
        );

        let blend = StudyProfile::blend(false, Some(ids(&["far", "near"]))).unwrap();
        assert_ne!(
            composite_layers(&blend, &far_then_near),
            composite_layers(&blend, &near_then_far),
            "continuous blending remains sensitive to caller submission order"
        );
    }

    #[test]
    fn scene_fingerprints_match_the_retained_manifest() {
        let expected = BTreeMap::from([
            (
                "same-texture-three-profiles",
                "1ca0df91e92939a737f72364b785069edd14165e5a8d47963067840c7ea95da2",
            ),
            (
                "cutout-over-opaque",
                "86fc9dc54299fa0a1c78c6d4646326dd88d07336fdb48a6d2cae86345ab4b794",
            ),
            (
                "blend-over-opaque",
                "41343caddde50643d69e5e8f83273f83159cec87377ea21411947fc69659ac83",
            ),
            (
                "overlapping-blend-back-to-front",
                "3e8dd5abc1a3d0b97f55cfbd557a31d74ea96b9a1139c28ec4ade41775e72a5f",
            ),
            (
                "overlapping-blend-front-to-back",
                "a49604bdb4d6b053174d9ac420f06bdf6fd8cd05bfcdeb655e449a5f060ea6d3",
            ),
            (
                "cutout-blend-intersection",
                "f129d02267efa29405a5bed436fcdac306e640baafd3eeeef2eb6f35d69fd196",
            ),
            (
                "identical-depth-overlap",
                "4ab5d287b04098be61acb1b27af6fd392ec3b00729e80dd9971cfd93bf0992fc",
            ),
        ]);
        for case in scene_cases() {
            assert_eq!(
                scene_fingerprint(&case).unwrap(),
                expected[case.id],
                "scene {} drifted from its retained identity",
                case.id
            );
        }
    }

    #[test]
    fn interaction_manifest_is_fingerprint_locked() {
        assert_eq!(
            interaction_manifest_fingerprint().unwrap(),
            "0a99c714c258bac7f91eb5dd39748651abca8db96bfc1a410d823a18d2c23d93",
            "Slice 4 source fixtures, layout, depths, or submission order drifted"
        );
    }

    #[test]
    fn study_report_is_deterministic() {
        assert_eq!(study_report_json().unwrap(), study_report_json().unwrap());
    }

    #[test]
    fn report_classifies_every_boundary_and_mixed_texel_under_each_candidate() {
        let samples = fragment_samples().unwrap();
        assert_eq!(samples.len(), 50);
        assert_eq!(
            samples
                .iter()
                .map(|sample| sample.profile_label)
                .collect::<BTreeSet<_>>()
                .len(),
            5
        );
        assert!(samples.iter().any(|sample| {
            sample.source_rgba8[3] == 128
                && sample.profile_label == "cutout-discard-below-depth-write"
                && sample.fragment.action == FragmentAction::Keep
        }));
        assert!(samples.iter().any(|sample| {
            sample.source_rgba8[3] == 128
                && sample.profile_label == "cutout-discard-at-or-below-depth-write"
                && sample.fragment.action == FragmentAction::Discard
        }));
    }

    #[test]
    fn report_retains_typed_negative_observations() {
        let failures = baseline_failure_observations();
        assert_eq!(failures.len(), 10);
        let kinds = failures
            .iter()
            .map(|failure| failure.kind)
            .collect::<BTreeSet<_>>();
        assert!(kinds.contains(&StudyFailureKind::NonFiniteThreshold));
        assert!(kinds.contains(&StudyFailureKind::ThresholdOutOfRange));
        assert!(kinds.contains(&StudyFailureKind::MissingBlendOrdering));
        assert!(kinds.contains(&StudyFailureKind::MalformedRgba8));
        assert!(kinds.contains(&StudyFailureKind::DimensionsOverflow));
    }
}
