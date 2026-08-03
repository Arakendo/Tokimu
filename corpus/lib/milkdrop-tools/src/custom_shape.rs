//! Bounded extraction of selected MilkDrop custom-shape scalar properties.
//!
//! Only literal properties in real `[shape_N]` sections are admitted here.
//! `shapecode_N`, texture resolution, and renderer blend policy remain visible
//! unsupported constructs rather than being silently approximated.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{section_index, MilkDropConstruct, MilkDropPresetDocument, MilkDropSourceLocation};

pub const MAX_CUSTOM_SHAPE_SIDES: u8 = 100;
const MIN_CUSTOM_SHAPE_SIDES: u8 = 3;

/// Provider-neutral description of one selected literal MilkDrop custom shape.
///
/// The selected subset is a static convex polygon. It owns no mesh, shader,
/// blend state, texture, or custom-code execution.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MilkDropCustomShape {
    pub index: u8,
    pub enabled: bool,
    pub sides: u8,
    pub additive: bool,
    pub thick_outline: bool,
    pub textured: bool,
    /// Normalized center in `[0, 1]` presentation coordinates.
    pub center: [f32; 2],
    /// Normalized radius in `[0, 1]` presentation coordinates.
    pub radius: f32,
    /// Clockwise rotation in radians.
    pub angle_radians: f32,
    pub color: [f32; 4],
}

/// A renderer-neutral polygon lowered from one selected custom shape.
///
/// Vertices are normalized to the presentation unit square. Consumers choose
/// fill, outline, blend, and mesh execution policy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MilkDropCustomShapeFrame {
    pub shape: MilkDropCustomShape,
    pub points: Vec<[f32; 2]>,
}

impl MilkDropCustomShape {
    fn defaults(index: u8) -> Self {
        Self {
            index,
            enabled: true,
            sides: 4,
            additive: false,
            thick_outline: false,
            textured: false,
            center: [0.5, 0.5],
            radius: 0.1,
            angle_radians: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Resolves bounded literal custom-shape properties in source order.
pub fn resolve_selected_custom_shapes(
    document: &MilkDropPresetDocument,
) -> Result<Vec<MilkDropCustomShape>, MilkDropCustomShapeError> {
    let mut shapes = Vec::new();

    for section in &document.sections {
        let Some(index) = section_index(&section.name, "shape_") else {
            continue;
        };
        let mut shape = MilkDropCustomShape::defaults(index);
        let mut seen = BTreeSet::new();
        let mut selected = false;

        for entry in &section.entries {
            if entry.construct != MilkDropConstruct::SelectedCustomShapeParameter {
                continue;
            }
            selected = true;
            if !seen.insert(entry.key.as_str()) {
                return Err(MilkDropCustomShapeError::DuplicateProperty {
                    line: entry.location.line,
                    key: entry.key.clone(),
                });
            }
            apply_property(&mut shape, &entry.key, &entry.value, entry.location)?;
        }

        if selected {
            shapes.push(shape);
        }
    }

    Ok(shapes)
}

/// Lowers selected literal custom shapes into normalized convex-polygon points.
///
/// Disabled shapes are omitted. The provider deliberately does not choose a
/// fill rule or generate a mesh. A requested texture is rejected explicitly
/// until a resolved asset contract exists; rendering a solid substitute would
/// silently change preset meaning.
pub fn lower_selected_custom_shapes(
    shapes: &[MilkDropCustomShape],
) -> Result<Vec<MilkDropCustomShapeFrame>, MilkDropCustomShapeError> {
    shapes
        .iter()
        .filter(|shape| shape.enabled)
        .map(|shape| {
            if shape.textured {
                return Err(MilkDropCustomShapeError::TextureResolutionRequired {
                    shape_index: shape.index,
                });
            }
            let points = (0..shape.sides)
                .map(|index| {
                    let angle = shape.angle_radians
                        + std::f32::consts::TAU * index as f32 / shape.sides as f32;
                    [
                        shape.center[0] + angle.cos() * shape.radius,
                        shape.center[1] - angle.sin() * shape.radius,
                    ]
                })
                .collect();
            Ok(MilkDropCustomShapeFrame {
                shape: shape.clone(),
                points,
            })
        })
        .collect()
}

fn apply_property(
    shape: &mut MilkDropCustomShape,
    key: &str,
    value: &str,
    location: MilkDropSourceLocation,
) -> Result<(), MilkDropCustomShapeError> {
    match key {
        "enabled" => shape.enabled = parse_flag(key, value, location.line)?,
        "sides" => {
            let sides = parse_u8(key, value, location.line)?;
            if !(MIN_CUSTOM_SHAPE_SIDES..=MAX_CUSTOM_SHAPE_SIDES).contains(&sides) {
                return Err(MilkDropCustomShapeError::OutOfRange {
                    line: location.line,
                    key: key.to_owned(),
                    value: value.to_owned(),
                    minimum: f32::from(MIN_CUSTOM_SHAPE_SIDES),
                    maximum: f32::from(MAX_CUSTOM_SHAPE_SIDES),
                });
            }
            shape.sides = sides;
        }
        "additive" => shape.additive = parse_flag(key, value, location.line)?,
        "thickoutline" => shape.thick_outline = parse_flag(key, value, location.line)?,
        "textured" => shape.textured = parse_flag(key, value, location.line)?,
        "x" => shape.center[0] = parse_range(key, value, location.line, 0.0, 1.0)?,
        "y" => shape.center[1] = parse_range(key, value, location.line, 0.0, 1.0)?,
        "rad" => shape.radius = parse_range(key, value, location.line, 0.0, 1.0)?,
        "ang" => {
            shape.angle_radians = parse_range(
                key,
                value,
                location.line,
                -std::f32::consts::TAU,
                std::f32::consts::TAU,
            )?
        }
        "r" => shape.color[0] = parse_range(key, value, location.line, 0.0, 1.0)?,
        "g" => shape.color[1] = parse_range(key, value, location.line, 0.0, 1.0)?,
        "b" => shape.color[2] = parse_range(key, value, location.line, 0.0, 1.0)?,
        "a" => shape.color[3] = parse_range(key, value, location.line, 0.0, 1.0)?,
        _ => unreachable!("selected custom-shape classification must remain synchronized"),
    }
    Ok(())
}

fn parse_flag(key: &str, value: &str, line: usize) -> Result<bool, MilkDropCustomShapeError> {
    match parse_finite(key, value, line)? {
        0.0 => Ok(false),
        1.0 => Ok(true),
        _ => Err(MilkDropCustomShapeError::InvalidFlag {
            line,
            key: key.to_owned(),
            value: value.to_owned(),
        }),
    }
}

fn parse_u8(key: &str, value: &str, line: usize) -> Result<u8, MilkDropCustomShapeError> {
    let parsed = parse_finite(key, value, line)?;
    if parsed.fract() != 0.0 || !(0.0..=f32::from(u8::MAX)).contains(&parsed) {
        return Err(MilkDropCustomShapeError::InvalidInteger {
            line,
            key: key.to_owned(),
            value: value.to_owned(),
        });
    }
    Ok(parsed as u8)
}

fn parse_range(
    key: &str,
    value: &str,
    line: usize,
    minimum: f32,
    maximum: f32,
) -> Result<f32, MilkDropCustomShapeError> {
    let parsed = parse_finite(key, value, line)?;
    if !(minimum..=maximum).contains(&parsed) {
        return Err(MilkDropCustomShapeError::OutOfRange {
            line,
            key: key.to_owned(),
            value: value.to_owned(),
            minimum,
            maximum,
        });
    }
    Ok(parsed)
}

fn parse_finite(key: &str, value: &str, line: usize) -> Result<f32, MilkDropCustomShapeError> {
    let parsed =
        value
            .trim()
            .parse::<f32>()
            .map_err(|_| MilkDropCustomShapeError::InvalidValue {
                line,
                key: key.to_owned(),
                value: value.to_owned(),
            })?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(MilkDropCustomShapeError::InvalidValue {
            line,
            key: key.to_owned(),
            value: value.to_owned(),
        })
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum MilkDropCustomShapeError {
    #[error("MilkDrop custom shape {shape_index} requests a texture, but no texture-resolution provider is admitted")]
    TextureResolutionRequired { shape_index: u8 },
    #[error("MilkDrop custom-shape property `{key}` is declared more than once; second declaration is at line {line}")]
    DuplicateProperty { line: usize, key: String },
    #[error("MilkDrop custom-shape property `{key}` at line {line} is not a finite numeric value `{value}`")]
    InvalidValue {
        line: usize,
        key: String,
        value: String,
    },
    #[error("MilkDrop custom-shape property `{key}` at line {line} must be an integer, received `{value}`")]
    InvalidInteger {
        line: usize,
        key: String,
        value: String,
    },
    #[error("MilkDrop custom-shape property `{key}` at line {line} must be zero or one, received `{value}`")]
    InvalidFlag {
        line: usize,
        key: String,
        value: String,
    },
    #[error("MilkDrop custom-shape property `{key}` at line {line} must be between {minimum} and {maximum}, received `{value}`")]
    OutOfRange {
        line: usize,
        key: String,
        value: String,
        minimum: f32,
        maximum: f32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MilkDropPresetDocument;

    #[test]
    fn resolves_and_lowers_a_literal_convex_shape_without_executing_code() {
        let document = MilkDropPresetDocument::parse(
            "[shape_2]\nenabled=1\nsides=5\nadditive=1\nthickoutline=1\ntextured=0\nx=0.25\ny=0.75\nrad=0.2\nang=0.5\nr=0.2\ng=0.4\nb=0.6\na=0.8\nshapecode_2=rad=rad*2;",
        )
        .unwrap();
        let shapes = resolve_selected_custom_shapes(&document).unwrap();

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].index, 2);
        assert_eq!(shapes[0].sides, 5);
        assert!(shapes[0].additive);
        assert!(shapes[0].thick_outline);
        assert_eq!(document.unsupported_entries, 1);

        let frames = lower_selected_custom_shapes(&shapes).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].points.len(), 5);
        assert!(frames[0]
            .points
            .iter()
            .flatten()
            .all(|value| value.is_finite()));
    }

    #[test]
    fn custom_shape_rejects_out_of_range_side_counts_at_the_source_line() {
        let document = MilkDropPresetDocument::parse("[shape_0]\nsides=2").unwrap();
        assert!(matches!(
            resolve_selected_custom_shapes(&document),
            Err(MilkDropCustomShapeError::OutOfRange { line: 2, .. })
        ));
    }

    #[test]
    fn textured_shape_requires_a_future_texture_resolution_provider() {
        let document = MilkDropPresetDocument::parse("[shape_3]\ntextured=1").unwrap();
        let shapes = resolve_selected_custom_shapes(&document).unwrap();

        assert!(matches!(
            lower_selected_custom_shapes(&shapes),
            Err(MilkDropCustomShapeError::TextureResolutionRequired { shape_index: 3 })
        ));
    }
}
