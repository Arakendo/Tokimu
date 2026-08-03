//! Bounded MilkDrop 1-style source inspection for corpus evidence.
//!
//! This library intentionally parses only a selected key/value and equation
//! subset. It retains source locations, evaluates a small pure scalar subset,
//! and classifies unsupported constructs; it does not compile shaders, resolve
//! textures, or own renderer resources.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod custom_shape;
mod custom_wave;
mod runtime;
mod shader_inspection;

pub use custom_shape::{
    lower_selected_custom_shapes, resolve_selected_custom_shapes, MilkDropCustomShape,
    MilkDropCustomShapeError, MilkDropCustomShapeFrame, MAX_CUSTOM_SHAPE_SIDES,
};
pub use custom_wave::{
    lower_selected_custom_waves, resolve_selected_custom_waves, MilkDropCustomWave,
    MilkDropCustomWaveError, MilkDropCustomWaveFrame, MilkDropCustomWaveSampleSource,
    MAX_CUSTOM_WAVE_SAMPLES,
};
pub use runtime::{
    MilkDropClassicFrameControls, MilkDropSelectedRuntime, MilkDropSelectedRuntimeError,
};
pub use shader_inspection::{
    inspect_shader_entries, MilkDropShaderFeature, MilkDropShaderInspection, MilkDropShaderStage,
    MilkDropShaderTranslationBlocker, MilkDropShaderTranslationDisposition,
};

pub const MAX_PRESET_BYTES: usize = 64 * 1024;
pub const MAX_PRESET_LINES: usize = 2_048;
pub const MAX_SECTIONS: usize = 32;
pub const MAX_ENTRIES: usize = 512;
pub const MAX_KEY_BYTES: usize = 128;
pub const MAX_VALUE_BYTES: usize = 4 * 1024;
pub const MAX_EQUATION_OPERATIONS: usize = 128;

/// One source location retained by the corpus parser.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MilkDropSourceLocation {
    pub line: usize,
}

/// The selected construct classification for a preset entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MilkDropConstruct {
    ScalarParameter,
    InitEquation,
    PerFrameEquation,
    PerPixelEquation,
    SelectedCustomWaveParameter,
    SelectedCustomShapeParameter,
    UnsupportedCustomWave,
    UnsupportedCustomShape,
    UnsupportedWarpShader,
    UnsupportedCompositeShader,
    UnsupportedUnknown,
}

/// A source-preserving MilkDrop 1-style key/value entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MilkDropPresetEntry {
    pub key: String,
    pub value: String,
    pub construct: MilkDropConstruct,
    pub location: MilkDropSourceLocation,
}

/// A named source section and its ordered entries.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MilkDropPresetSection {
    pub name: String,
    pub location: MilkDropSourceLocation,
    pub entries: Vec<MilkDropPresetEntry>,
}

/// Parsed source evidence for the selected MilkDrop 1-style subset.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MilkDropPresetDocument {
    pub schema: String,
    pub sections: Vec<MilkDropPresetSection>,
    /// Entries recognized by the parser but deliberately not executed by the
    /// current scalar-only runtime.
    pub deferred_entries: usize,
    /// Entries outside the selected source subset. They remain visible in the
    /// structural artifact rather than being silently dropped.
    pub unsupported_entries: usize,
}

impl MilkDropPresetDocument {
    pub fn parse(source: &str) -> Result<Self, MilkDropParseError> {
        if source.len() > MAX_PRESET_BYTES {
            return Err(MilkDropParseError::TooManyBytes {
                actual: source.len(),
                maximum: MAX_PRESET_BYTES,
            });
        }

        let mut sections = Vec::new();
        let mut active_section = None;
        let mut entries = 0_usize;
        let mut deferred_entries = 0_usize;
        let mut unsupported_entries = 0_usize;

        for (index, raw_line) in source.lines().enumerate() {
            let line = index + 1;
            if line > MAX_PRESET_LINES {
                return Err(MilkDropParseError::TooManyLines {
                    actual: line,
                    maximum: MAX_PRESET_LINES,
                });
            }
            let trimmed = raw_line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with(';') {
                continue;
            }

            if trimmed.starts_with('[') {
                let name = parse_section_name(trimmed, line)?;
                if sections.len() >= MAX_SECTIONS {
                    return Err(MilkDropParseError::TooManySections {
                        maximum: MAX_SECTIONS,
                    });
                }
                sections.push(MilkDropPresetSection {
                    name,
                    location: MilkDropSourceLocation { line },
                    entries: Vec::new(),
                });
                active_section = Some(sections.len() - 1);
                continue;
            }

            let section_index =
                active_section.ok_or(MilkDropParseError::EntryBeforeSection { line })?;
            let (key, value) = parse_entry(trimmed, line)?;
            if entries >= MAX_ENTRIES {
                return Err(MilkDropParseError::TooManyEntries {
                    maximum: MAX_ENTRIES,
                });
            }
            let section_name = &sections[section_index].name;
            let construct = classify_construct(section_name, &key);
            if is_deferred(&construct) {
                deferred_entries += 1;
            } else if is_unsupported(&construct) {
                unsupported_entries += 1;
            }
            sections[section_index].entries.push(MilkDropPresetEntry {
                key,
                value,
                construct,
                location: MilkDropSourceLocation { line },
            });
            entries += 1;
        }

        if sections.is_empty() {
            return Err(MilkDropParseError::MissingSection);
        }

        Ok(Self {
            schema: "tokimu-milkdrop-inspection-v1".to_owned(),
            sections,
            deferred_entries,
            unsupported_entries,
        })
    }

    pub fn to_structural_json(&self) -> Result<String, MilkDropParseError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| MilkDropParseError::Serialization(error.to_string()))
    }

    pub fn entry_count(&self) -> usize {
        self.sections
            .iter()
            .map(|section| section.entries.len())
            .sum()
    }
}

/// Deterministic scalar state produced by the bounded equation evaluator.
///
/// The state contains only values explicitly assigned by selected equations.
/// It has no ambient time, random, file, network, device, or renderer access.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MilkDropEvaluationState {
    pub variables: BTreeMap<String, f64>,
}

impl MilkDropEvaluationState {
    pub fn value(&self, name: &str) -> Option<f64> {
        self.variables.get(name).copied()
    }
}

/// Evaluate selected initialization or per-frame scalar equations in source
/// order. Per-pixel equations intentionally remain presentation-lowering work.
pub fn evaluate_selected_equations(
    document: &MilkDropPresetDocument,
    phase: MilkDropEvaluationPhase,
    state: &mut MilkDropEvaluationState,
) -> Result<usize, MilkDropEvaluationError> {
    let selected_construct = match phase {
        MilkDropEvaluationPhase::Initialization => MilkDropConstruct::InitEquation,
        MilkDropEvaluationPhase::PerFrame => MilkDropConstruct::PerFrameEquation,
    };
    let mut evaluated = 0;

    for section in &document.sections {
        for entry in &section.entries {
            if entry.construct != selected_construct {
                continue;
            }
            for assignment in parse_assignments(&entry.value, entry.location.line)? {
                let value = ExpressionParser::new(
                    assignment.expression,
                    &state.variables,
                    entry.location.line,
                )
                .evaluate()?;
                if !value.is_finite() {
                    return Err(MilkDropEvaluationError::NonFiniteResult {
                        line: entry.location.line,
                        target: assignment.target.to_owned(),
                    });
                }
                state.variables.insert(assignment.target.to_owned(), value);
                evaluated += 1;
            }
        }
    }

    Ok(evaluated)
}

/// The explicit, pure equation phases currently admitted by the corpus.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MilkDropEvaluationPhase {
    Initialization,
    PerFrame,
}

/// Tokimu's explicit defaults for the selected classic visualizer parameters.
///
/// These are compatibility inputs for the corpus subset, not a claim that all
/// historical MilkDrop implementations used identical defaults.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MilkDropCompatibilityDefaults {
    pub rating: f32,
    pub decay: f32,
    pub gamma_adjustment: f32,
    pub zoom: f32,
    pub zoom_exponent: f32,
    pub rotation: f32,
    pub warp_amount: f32,
    pub warp_animation_speed: f32,
    pub video_echo_alpha: f32,
    pub video_echo_zoom: f32,
    pub video_echo_orientation: u8,
}

impl Default for MilkDropCompatibilityDefaults {
    fn default() -> Self {
        Self {
            rating: 0.0,
            decay: 0.98,
            gamma_adjustment: 1.0,
            zoom: 1.0,
            zoom_exponent: 1.0,
            rotation: 0.0,
            warp_amount: 1.0,
            warp_animation_speed: 1.0,
            video_echo_alpha: 0.0,
            video_echo_zoom: 1.0,
            video_echo_orientation: 0,
        }
    }
}

/// Resolved selected scalar parameters together with the keys that replaced a
/// Tokimu corpus default.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MilkDropResolvedParameters {
    pub values: MilkDropCompatibilityDefaults,
    pub explicitly_set: Vec<String>,
}

/// Resolve selected classic scalar parameters without observing renderer or
/// platform state. Duplicate admitted keys are rejected so a corpus result has
/// one unambiguous source value per parameter.
pub fn resolve_selected_parameters(
    document: &MilkDropPresetDocument,
) -> Result<MilkDropResolvedParameters, MilkDropEvaluationError> {
    let mut values = MilkDropCompatibilityDefaults::default();
    let mut explicitly_set = BTreeSet::new();

    for section in &document.sections {
        for entry in &section.entries {
            if entry.construct != MilkDropConstruct::ScalarParameter {
                continue;
            }
            if !explicitly_set.insert(entry.key.clone()) {
                return Err(MilkDropEvaluationError::DuplicateScalarParameter {
                    line: entry.location.line,
                    key: entry.key.clone(),
                });
            }
            let value = parse_finite_scalar(&entry.value, entry.location.line)?;
            match entry.key.as_str() {
                "frating" => values.rating = value,
                "fdecay" => values.decay = value,
                "fgammaadj" => values.gamma_adjustment = value,
                "fzoom" => values.zoom = value,
                "fzoomexp" => values.zoom_exponent = value,
                "frot" => values.rotation = value,
                "fwarpamount" => values.warp_amount = value,
                "fwarpanimspeed" => values.warp_animation_speed = value,
                "fvideoechoalpha" => values.video_echo_alpha = value,
                "fvideoechozoom" => values.video_echo_zoom = value,
                "nvideoechoorientation" => {
                    if value.fract() != 0.0 || !(0.0..=3.0).contains(&value) {
                        return Err(MilkDropEvaluationError::InvalidEchoOrientation {
                            line: entry.location.line,
                            value: entry.value.clone(),
                        });
                    }
                    values.video_echo_orientation = value as u8;
                }
                _ => unreachable!("scalar parameter classification must remain selected"),
            }
        }
    }

    Ok(MilkDropResolvedParameters {
        values,
        explicitly_set: explicitly_set.into_iter().collect(),
    })
}

fn parse_finite_scalar(value: &str, line: usize) -> Result<f32, MilkDropEvaluationError> {
    let parsed = value.trim().parse::<f32>().map_err(|_| {
        MilkDropEvaluationError::InvalidScalarParameter {
            line,
            value: value.to_owned(),
        }
    })?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(MilkDropEvaluationError::InvalidScalarParameter {
            line,
            value: value.to_owned(),
        })
    }
}

struct Assignment<'a> {
    target: &'a str,
    expression: &'a str,
}

fn parse_assignments(
    value: &str,
    line: usize,
) -> Result<Vec<Assignment<'_>>, MilkDropEvaluationError> {
    let assignments = value
        .split(';')
        .filter(|assignment| !assignment.trim().is_empty())
        .map(|assignment| parse_assignment(assignment, line))
        .collect::<Result<Vec<_>, _>>()?;
    if assignments.is_empty() {
        return Err(MilkDropEvaluationError::EmptyExpression { line });
    }
    Ok(assignments)
}

fn parse_assignment(value: &str, line: usize) -> Result<Assignment<'_>, MilkDropEvaluationError> {
    let value = value.trim();
    let Some((target, expression)) = value.split_once('=') else {
        return Err(MilkDropEvaluationError::MissingAssignment { line });
    };
    let target = target.trim();
    let expression = expression.trim();
    if !is_identifier(target) {
        return Err(MilkDropEvaluationError::InvalidTarget {
            line,
            target: target.to_owned(),
        });
    }
    if expression.is_empty() {
        return Err(MilkDropEvaluationError::EmptyExpression { line });
    }
    Ok(Assignment { target, expression })
}

fn is_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some(character) if character.is_ascii_alphabetic() || character == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

struct ExpressionParser<'a> {
    source: &'a str,
    cursor: usize,
    variables: &'a BTreeMap<String, f64>,
    line: usize,
    operations: usize,
}

impl<'a> ExpressionParser<'a> {
    fn new(source: &'a str, variables: &'a BTreeMap<String, f64>, line: usize) -> Self {
        Self {
            source,
            cursor: 0,
            variables,
            line,
            operations: 0,
        }
    }

    fn evaluate(mut self) -> Result<f64, MilkDropEvaluationError> {
        let value = self.parse_sum()?;
        self.skip_whitespace();
        if self.cursor != self.source.len() {
            return Err(self.unexpected());
        }
        Ok(value)
    }

    fn parse_sum(&mut self) -> Result<f64, MilkDropEvaluationError> {
        let mut value = self.parse_product()?;
        loop {
            if self.consume('+') {
                value += self.parse_product()?;
                self.count_operation()?;
            } else if self.consume('-') {
                value -= self.parse_product()?;
                self.count_operation()?;
            } else {
                return Ok(value);
            }
        }
    }

    fn parse_product(&mut self) -> Result<f64, MilkDropEvaluationError> {
        let mut value = self.parse_factor()?;
        loop {
            if self.consume('*') {
                value *= self.parse_factor()?;
                self.count_operation()?;
            } else if self.consume('/') {
                let divisor = self.parse_factor()?;
                if divisor == 0.0 {
                    return Err(MilkDropEvaluationError::DivisionByZero { line: self.line });
                }
                value /= divisor;
                self.count_operation()?;
            } else {
                return Ok(value);
            }
        }
    }

    fn parse_factor(&mut self) -> Result<f64, MilkDropEvaluationError> {
        if self.consume('-') {
            self.count_operation()?;
            return Ok(-self.parse_factor()?);
        }
        if self.consume('(') {
            let value = self.parse_sum()?;
            if !self.consume(')') {
                return Err(self.unexpected());
            }
            return Ok(value);
        }
        if let Some(number) = self.parse_number()? {
            return Ok(number);
        }
        let identifier = self.parse_identifier().ok_or_else(|| self.unexpected())?;
        if self.consume('(') {
            if !matches!(identifier, "sin" | "cos" | "abs") {
                return Err(MilkDropEvaluationError::UnsupportedFunction {
                    line: self.line,
                    function: identifier.to_owned(),
                });
            }
            let value = self.parse_sum()?;
            if !self.consume(')') {
                return Err(self.unexpected());
            }
            self.count_operation()?;
            return match identifier {
                "sin" => Ok(value.sin()),
                "cos" => Ok(value.cos()),
                "abs" => Ok(value.abs()),
                _ => unreachable!("function membership was validated before argument parsing"),
            };
        }
        self.variables.get(identifier).copied().ok_or_else(|| {
            MilkDropEvaluationError::UnknownVariable {
                line: self.line,
                variable: identifier.to_owned(),
            }
        })
    }

    fn parse_number(&mut self) -> Result<Option<f64>, MilkDropEvaluationError> {
        self.skip_whitespace();
        let remaining = &self.source[self.cursor..];
        let Some(first) = remaining.bytes().next() else {
            return Ok(None);
        };
        if !first.is_ascii_digit() && first != b'.' {
            return Ok(None);
        }
        let mut length = 0;
        let mut exponent_sign_allowed = false;
        for character in remaining.bytes() {
            let accepted = character.is_ascii_digit()
                || character == b'.'
                || character == b'e'
                || character == b'E'
                || (exponent_sign_allowed && matches!(character, b'+' | b'-'));
            if !accepted {
                break;
            }
            exponent_sign_allowed = matches!(character, b'e' | b'E');
            length += 1;
        }
        let token = &remaining[..length];
        let value = token
            .parse::<f64>()
            .map_err(|_| MilkDropEvaluationError::InvalidNumber {
                line: self.line,
                value: token.to_owned(),
            })?;
        self.cursor += length;
        Ok(Some(value))
    }

    fn parse_identifier(&mut self) -> Option<&'a str> {
        self.skip_whitespace();
        let start = self.cursor;
        let remaining = &self.source[start..];
        let mut characters = remaining.char_indices();
        match characters.next() {
            Some((_, character)) if character.is_ascii_alphabetic() || character == '_' => {}
            _ => return None,
        }
        let mut end = remaining.len();
        for (index, character) in characters {
            if !character.is_ascii_alphanumeric() && character != '_' {
                end = index;
                break;
            }
        }
        self.cursor += end;
        Some(&remaining[..end])
    }

    fn consume(&mut self, character: char) -> bool {
        self.skip_whitespace();
        if self.source[self.cursor..].starts_with(character) {
            self.cursor += character.len_utf8();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(character) = self.source[self.cursor..].chars().next() {
            if !character.is_whitespace() {
                break;
            }
            self.cursor += character.len_utf8();
        }
    }

    fn count_operation(&mut self) -> Result<(), MilkDropEvaluationError> {
        self.operations += 1;
        if self.operations > MAX_EQUATION_OPERATIONS {
            return Err(MilkDropEvaluationError::TooManyOperations {
                line: self.line,
                maximum: MAX_EQUATION_OPERATIONS,
            });
        }
        Ok(())
    }

    fn unexpected(&self) -> MilkDropEvaluationError {
        MilkDropEvaluationError::UnexpectedToken {
            line: self.line,
            offset: self.cursor,
        }
    }
}

fn parse_section_name(line: &str, location: usize) -> Result<String, MilkDropParseError> {
    if !line.ends_with(']') {
        return Err(MilkDropParseError::UnterminatedSection { line: location });
    }
    let name = line[1..line.len() - 1].trim();
    if name.is_empty() {
        return Err(MilkDropParseError::EmptySection { line: location });
    }
    if name.len() > MAX_KEY_BYTES {
        return Err(MilkDropParseError::SectionNameTooLong {
            line: location,
            maximum: MAX_KEY_BYTES,
        });
    }
    Ok(name.to_owned())
}

fn parse_entry(line: &str, location: usize) -> Result<(String, String), MilkDropParseError> {
    let Some((raw_key, raw_value)) = line.split_once('=') else {
        return Err(MilkDropParseError::MissingAssignment { line: location });
    };
    let key = raw_key.trim();
    let value = raw_value.trim();
    if key.is_empty() {
        return Err(MilkDropParseError::EmptyKey { line: location });
    }
    if value.is_empty() {
        return Err(MilkDropParseError::EmptyValue { line: location });
    }
    if key.len() > MAX_KEY_BYTES {
        return Err(MilkDropParseError::KeyTooLong {
            line: location,
            maximum: MAX_KEY_BYTES,
        });
    }
    if value.len() > MAX_VALUE_BYTES {
        return Err(MilkDropParseError::ValueTooLong {
            line: location,
            maximum: MAX_VALUE_BYTES,
        });
    }
    Ok((key.to_ascii_lowercase(), value.to_owned()))
}

fn classify_construct(section_name: &str, key: &str) -> MilkDropConstruct {
    if is_selected_custom_wave_section(section_name) && is_selected_custom_wave_key(key) {
        MilkDropConstruct::SelectedCustomWaveParameter
    } else if is_selected_custom_shape_section(section_name) && is_selected_custom_shape_key(key) {
        MilkDropConstruct::SelectedCustomShapeParameter
    } else if key.starts_with("per_frame_init_") {
        MilkDropConstruct::InitEquation
    } else if key.starts_with("per_frame_") {
        MilkDropConstruct::PerFrameEquation
    } else if key.starts_with("per_pixel_") {
        MilkDropConstruct::PerPixelEquation
    } else if key.starts_with("wavecode_") || key.starts_with("wave_") {
        MilkDropConstruct::UnsupportedCustomWave
    } else if key.starts_with("shapecode_") || key.starts_with("shape_") {
        MilkDropConstruct::UnsupportedCustomShape
    } else if key.starts_with("warp_shader_") {
        MilkDropConstruct::UnsupportedWarpShader
    } else if key.starts_with("comp_shader_") {
        MilkDropConstruct::UnsupportedCompositeShader
    } else if is_known_scalar_parameter(key) {
        MilkDropConstruct::ScalarParameter
    } else {
        MilkDropConstruct::UnsupportedUnknown
    }
}

fn is_selected_custom_wave_section(section_name: &str) -> bool {
    section_index(section_name, "wave_").is_some()
}

fn is_selected_custom_shape_section(section_name: &str) -> bool {
    section_index(section_name, "shape_").is_some()
}

pub(crate) fn section_index(section_name: &str, prefix: &str) -> Option<u8> {
    let suffix = section_name.strip_prefix(prefix)?;
    suffix.parse::<u8>().ok()
}

pub(crate) fn is_selected_custom_wave_key(key: &str) -> bool {
    matches!(
        key,
        "enabled"
            | "samples"
            | "bspectrum"
            | "busedots"
            | "bdrawthick"
            | "badditive"
            | "scaling"
            | "r"
            | "g"
            | "b"
            | "a"
            | "x"
            | "y"
    )
}

fn is_selected_custom_shape_key(key: &str) -> bool {
    matches!(
        key,
        "enabled"
            | "sides"
            | "additive"
            | "thickoutline"
            | "textured"
            | "x"
            | "y"
            | "rad"
            | "ang"
            | "r"
            | "g"
            | "b"
            | "a"
    )
}

fn is_known_scalar_parameter(key: &str) -> bool {
    matches!(
        key,
        "frating"
            | "fdecay"
            | "fgammaadj"
            | "fzoom"
            | "fzoomexp"
            | "frot"
            | "fwarpamount"
            | "fwarpanimspeed"
            | "fvideoechoalpha"
            | "fvideoechozoom"
            | "nvideoechoorientation"
    )
}

fn is_unsupported(construct: &MilkDropConstruct) -> bool {
    matches!(
        construct,
        MilkDropConstruct::UnsupportedCustomWave
            | MilkDropConstruct::UnsupportedCustomShape
            | MilkDropConstruct::UnsupportedWarpShader
            | MilkDropConstruct::UnsupportedCompositeShader
            | MilkDropConstruct::UnsupportedUnknown
    )
}

fn is_deferred(construct: &MilkDropConstruct) -> bool {
    matches!(construct, MilkDropConstruct::PerPixelEquation)
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum MilkDropParseError {
    #[error("MilkDrop source contains {actual} bytes; maximum is {maximum}")]
    TooManyBytes { actual: usize, maximum: usize },
    #[error("MilkDrop source contains more than {maximum} lines")]
    TooManyLines { actual: usize, maximum: usize },
    #[error("MilkDrop source contains more than {maximum} sections")]
    TooManySections { maximum: usize },
    #[error("MilkDrop source contains more than {maximum} entries")]
    TooManyEntries { maximum: usize },
    #[error("MilkDrop source must begin entries within a named section; entry at line {line}")]
    EntryBeforeSection { line: usize },
    #[error("MilkDrop source contains no named section")]
    MissingSection,
    #[error("MilkDrop section at line {line} is missing a closing bracket")]
    UnterminatedSection { line: usize },
    #[error("MilkDrop section at line {line} has an empty name")]
    EmptySection { line: usize },
    #[error("MilkDrop section name at line {line} exceeds {maximum} bytes")]
    SectionNameTooLong { line: usize, maximum: usize },
    #[error("MilkDrop entry at line {line} is missing an equals assignment")]
    MissingAssignment { line: usize },
    #[error("MilkDrop entry at line {line} has an empty key")]
    EmptyKey { line: usize },
    #[error("MilkDrop entry at line {line} has an empty value")]
    EmptyValue { line: usize },
    #[error("MilkDrop key at line {line} exceeds {maximum} bytes")]
    KeyTooLong { line: usize, maximum: usize },
    #[error("MilkDrop value at line {line} exceeds {maximum} bytes")]
    ValueTooLong { line: usize, maximum: usize },
    #[error("could not serialize MilkDrop inspection artifact: {0}")]
    Serialization(String),
}

/// Evaluation failures remain tied to their source line and never fall back to
/// host-language evaluation or ambient state.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum MilkDropEvaluationError {
    #[error("MilkDrop scalar parameter at line {line} is not a finite numeric value `{value}`")]
    InvalidScalarParameter { line: usize, value: String },
    #[error("MilkDrop scalar parameter `{key}` is declared more than once; second declaration is at line {line}")]
    DuplicateScalarParameter { line: usize, key: String },
    #[error("MilkDrop video echo orientation at line {line} must be an integer from 0 to 3, received `{value}`")]
    InvalidEchoOrientation { line: usize, value: String },
    #[error("MilkDrop equation at line {line} is missing an assignment")]
    MissingAssignment { line: usize },
    #[error("MilkDrop equation at line {line} has an invalid assignment target `{target}`")]
    InvalidTarget { line: usize, target: String },
    #[error("MilkDrop equation at line {line} has an empty expression")]
    EmptyExpression { line: usize },
    #[error("MilkDrop equation at line {line} contains an unexpected token at byte {offset}")]
    UnexpectedToken { line: usize, offset: usize },
    #[error("MilkDrop equation at line {line} contains invalid number `{value}`")]
    InvalidNumber { line: usize, value: String },
    #[error("MilkDrop equation at line {line} references unknown variable `{variable}`")]
    UnknownVariable { line: usize, variable: String },
    #[error("MilkDrop equation at line {line} uses unsupported function `{function}`")]
    UnsupportedFunction { line: usize, function: String },
    #[error("MilkDrop equation at line {line} divides by zero")]
    DivisionByZero { line: usize },
    #[error("MilkDrop equation at line {line} exceeds {maximum} operations")]
    TooManyOperations { line: usize, maximum: usize },
    #[error("MilkDrop equation at line {line} assigned non-finite value to `{target}`")]
    NonFiniteResult { line: usize, target: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"
        [preset00]
        fRating=3.0
        fDecay=0.97
        per_frame_init_1=q1=0;
        per_frame_1=q1=q1+1;
        per_pixel_1=zoom=1.0+0.02*sin(rad);
        wavecode_0=sample=sample*0.5;
        comp_shader_1=float4 main() { return 0; }
        unknown_thing=42
    "#;

    #[test]
    fn selected_constructs_preserve_source_order_and_locations() {
        let document = MilkDropPresetDocument::parse(FIXTURE).unwrap();
        assert_eq!(document.sections.len(), 1);
        assert_eq!(document.entry_count(), 8);
        assert_eq!(document.sections[0].entries[0].key, "frating");
        assert_eq!(
            document.sections[0].entries[3].construct,
            MilkDropConstruct::PerFrameEquation
        );
        assert_eq!(
            document.sections[0].entries[5].construct,
            MilkDropConstruct::UnsupportedCustomWave
        );
        assert_eq!(document.deferred_entries, 1);
        assert_eq!(document.unsupported_entries, 3);
        assert!(
            document.sections[0].entries[0].location.line
                < document.sections[0].entries[7].location.line
        );
    }

    #[test]
    fn malformed_source_reports_the_owning_line() {
        assert_eq!(
            MilkDropPresetDocument::parse("[preset00]\nfDecay").unwrap_err(),
            MilkDropParseError::MissingAssignment { line: 2 }
        );
        assert_eq!(
            MilkDropPresetDocument::parse("fDecay=0.9").unwrap_err(),
            MilkDropParseError::EntryBeforeSection { line: 1 }
        );
    }

    #[test]
    fn serializes_deterministically() {
        let document = MilkDropPresetDocument::parse(FIXTURE).unwrap();
        assert_eq!(
            document.to_structural_json().unwrap(),
            document.to_structural_json().unwrap()
        );
    }

    #[test]
    fn selected_equations_evaluate_in_source_order_without_ambient_state() {
        let document = MilkDropPresetDocument::parse(FIXTURE).unwrap();
        let mut state = MilkDropEvaluationState::default();

        assert_eq!(
            evaluate_selected_equations(
                &document,
                MilkDropEvaluationPhase::Initialization,
                &mut state,
            )
            .unwrap(),
            1
        );
        assert_eq!(state.value("q1"), Some(0.0));

        assert_eq!(
            evaluate_selected_equations(&document, MilkDropEvaluationPhase::PerFrame, &mut state)
                .unwrap(),
            1
        );
        assert_eq!(state.value("q1"), Some(1.0));
    }

    #[test]
    fn evaluator_rejects_unknown_variables_deterministically() {
        let document = MilkDropPresetDocument::parse("[preset00]\nper_frame_1=q1=q2+1;").unwrap();
        assert_eq!(
            evaluate_selected_equations(
                &document,
                MilkDropEvaluationPhase::PerFrame,
                &mut MilkDropEvaluationState::default(),
            )
            .unwrap_err(),
            MilkDropEvaluationError::UnknownVariable {
                line: 2,
                variable: "q2".to_owned(),
            }
        );
    }

    #[test]
    fn evaluator_rejects_ambient_style_symbols_and_functions() {
        let unknown_variable =
            MilkDropPresetDocument::parse("[preset00]\nper_frame_1=q1=wall_clock;").unwrap();
        assert_eq!(
            evaluate_selected_equations(
                &unknown_variable,
                MilkDropEvaluationPhase::PerFrame,
                &mut MilkDropEvaluationState::default(),
            )
            .unwrap_err(),
            MilkDropEvaluationError::UnknownVariable {
                line: 2,
                variable: "wall_clock".to_owned(),
            }
        );

        let unsupported_function =
            MilkDropPresetDocument::parse("[preset00]\nper_frame_1=q1=random();").unwrap();
        assert_eq!(
            evaluate_selected_equations(
                &unsupported_function,
                MilkDropEvaluationPhase::PerFrame,
                &mut MilkDropEvaluationState::default(),
            )
            .unwrap_err(),
            MilkDropEvaluationError::UnsupportedFunction {
                line: 2,
                function: "random".to_owned(),
            }
        );
    }

    #[test]
    fn evaluator_preserves_assignment_order_within_one_source_entry() {
        let document =
            MilkDropPresetDocument::parse("[preset00]\nper_frame_1=q1=2; q2=q1*3; q1=q2+1;")
                .unwrap();
        let mut state = MilkDropEvaluationState::default();

        assert_eq!(
            evaluate_selected_equations(&document, MilkDropEvaluationPhase::PerFrame, &mut state)
                .unwrap(),
            3
        );
        assert_eq!(state.value("q1"), Some(7.0));
        assert_eq!(state.value("q2"), Some(6.0));
    }

    #[test]
    fn selected_parameters_apply_explicit_defaults_and_source_values() {
        let document = MilkDropPresetDocument::parse(FIXTURE).unwrap();
        let parameters = resolve_selected_parameters(&document).unwrap();
        assert_eq!(parameters.values.rating, 3.0);
        assert_eq!(parameters.values.decay, 0.97);
        assert_eq!(parameters.values.zoom, 1.0);
        assert_eq!(parameters.values.video_echo_alpha, 0.0);
        assert_eq!(parameters.explicitly_set, vec!["fdecay", "frating"]);
    }

    #[test]
    fn selected_parameters_resolve_every_admitted_scalar_key() {
        let document = MilkDropPresetDocument::parse(
            "[preset00]\n\
             fRating=1\n\
             fDecay=0.91\n\
             fGammaAdj=1.25\n\
             fZoom=1.5\n\
             fZoomExp=2\n\
             fRot=-0.25\n\
             fWarpAmount=0.75\n\
             fWarpAnimSpeed=3\n\
             fVideoEchoAlpha=0.5\n\
             fVideoEchoZoom=1.125\n\
             nVideoEchoOrientation=3",
        )
        .unwrap();

        let parameters = resolve_selected_parameters(&document).unwrap();
        assert_eq!(parameters.values.rating, 1.0);
        assert_eq!(parameters.values.decay, 0.91);
        assert_eq!(parameters.values.gamma_adjustment, 1.25);
        assert_eq!(parameters.values.zoom, 1.5);
        assert_eq!(parameters.values.zoom_exponent, 2.0);
        assert_eq!(parameters.values.rotation, -0.25);
        assert_eq!(parameters.values.warp_amount, 0.75);
        assert_eq!(parameters.values.warp_animation_speed, 3.0);
        assert_eq!(parameters.values.video_echo_alpha, 0.5);
        assert_eq!(parameters.values.video_echo_zoom, 1.125);
        assert_eq!(parameters.values.video_echo_orientation, 3);
        assert_eq!(
            parameters.explicitly_set,
            vec![
                "fdecay",
                "fgammaadj",
                "frating",
                "frot",
                "fvideoechoalpha",
                "fvideoechozoom",
                "fwarpamount",
                "fwarpanimspeed",
                "fzoom",
                "fzoomexp",
                "nvideoechoorientation",
            ]
        );
    }

    #[test]
    fn selected_parameters_reject_non_finite_and_invalid_orientation_values() {
        let non_finite = MilkDropPresetDocument::parse("[preset00]\nfDecay=NaN").unwrap();
        assert_eq!(
            resolve_selected_parameters(&non_finite).unwrap_err(),
            MilkDropEvaluationError::InvalidScalarParameter {
                line: 2,
                value: "NaN".to_owned(),
            }
        );

        let fractional_orientation =
            MilkDropPresetDocument::parse("[preset00]\nnVideoEchoOrientation=1.5").unwrap();
        assert_eq!(
            resolve_selected_parameters(&fractional_orientation).unwrap_err(),
            MilkDropEvaluationError::InvalidEchoOrientation {
                line: 2,
                value: "1.5".to_owned(),
            }
        );

        let out_of_range_orientation =
            MilkDropPresetDocument::parse("[preset00]\nnVideoEchoOrientation=4").unwrap();
        assert_eq!(
            resolve_selected_parameters(&out_of_range_orientation).unwrap_err(),
            MilkDropEvaluationError::InvalidEchoOrientation {
                line: 2,
                value: "4".to_owned(),
            }
        );
    }

    #[test]
    fn duplicate_scalar_parameters_are_not_silently_overwritten() {
        let document = MilkDropPresetDocument::parse("[preset00]\nfZoom=1\nfZoom=2").unwrap();
        assert!(matches!(
            resolve_selected_parameters(&document),
            Err(MilkDropEvaluationError::DuplicateScalarParameter { .. })
        ));
    }

    #[test]
    fn parser_rejects_bounded_input_before_retaining_unbounded_source() {
        let oversized_source = "x".repeat(MAX_PRESET_BYTES + 1);
        assert!(matches!(
            MilkDropPresetDocument::parse(&oversized_source),
            Err(MilkDropParseError::TooManyBytes { .. })
        ));

        let too_many_sections = (0..=MAX_SECTIONS)
            .map(|index| format!("[preset{index}]"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(matches!(
            MilkDropPresetDocument::parse(&too_many_sections),
            Err(MilkDropParseError::TooManySections { .. })
        ));

        let too_many_entries = format!(
            "[preset00]\n{}",
            (0..=MAX_ENTRIES)
                .map(|index| format!("future_key_{index}=1"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert!(matches!(
            MilkDropPresetDocument::parse(&too_many_entries),
            Err(MilkDropParseError::TooManyEntries { .. })
        ));
    }
}
