use std::{fs, path::Path};

use crate::{
    CgmAttribute, CgmAttributeValue, CgmClipIndicator, CgmColor, CgmColorSelectionMode,
    CgmDiagnostic, CgmDiagnosticCode, CgmElement, CgmEncoding, CgmError, CgmInspection,
    CgmMetafileDescriptor, CgmPartition, CgmPicture, CgmPictureControlState, CgmPictureDescriptor,
    CgmPolygonSetEdgeFlag, CgmPolygonSetRecord, CgmPresentationState, CgmPrimitive,
    CgmPrimitiveKind, CgmResult, CgmScalingMode, CgmVdcExtent, CgmVdcType, DelimiterElement,
    ElementSupport,
};

const LONG_FORM_LENGTH: usize = 31;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeLimits {
    pub max_input_bytes: usize,
    pub max_elements: usize,
    pub max_parameter_bytes: usize,
    pub max_partitions_per_element: usize,
}

impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 16 * 1024 * 1024,
            max_elements: 100_000,
            max_parameter_bytes: 8 * 1024 * 1024,
            max_partitions_per_element: 4_096,
        }
    }
}

pub fn inspect_binary_cgm_file(
    path: impl AsRef<Path>,
    limits: DecodeLimits,
) -> CgmResult<CgmInspection> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|source| CgmError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    inspect_binary_cgm(&bytes, limits)
}

pub fn inspect_binary_cgm(bytes: &[u8], limits: DecodeLimits) -> CgmResult<CgmInspection> {
    if bytes.len() > limits.max_input_bytes {
        return Err(CgmError::InputTooLarge {
            actual: bytes.len(),
            limit: limits.max_input_bytes,
        });
    }

    let first_word = read_u16(bytes, 0, "binary element header")?;
    let first_class = (first_word >> 12) as u8;
    let first_id = ((first_word >> 5) & 0x7f) as u8;
    if first_class != 0 || first_id != 1 {
        return Err(CgmError::UnsupportedEncoding {
            class: first_class,
            id: first_id,
        });
    }

    let mut elements = Vec::new();
    let mut diagnostics = Vec::new();
    let mut lifecycle = Lifecycle::BeforeMetafile;
    let mut metafile_name = None;
    let mut metafile = CgmMetafileDescriptor::default();
    let mut pictures = Vec::new();
    let mut current_picture: Option<PictureBuilder> = None;
    let mut offset = 0;

    while offset < bytes.len() {
        if elements.len() >= limits.max_elements {
            return Err(CgmError::ElementLimit {
                limit: limits.max_elements,
            });
        }

        let mut element = decode_element(bytes, offset, elements.len(), limits)?;
        let parameters = parameter_bytes(bytes, &element)?;
        let delimiter = element.delimiter;

        if let Some(delimiter) = delimiter {
            apply_delimiter(
                delimiter,
                &element,
                &parameters,
                &mut lifecycle,
                &mut metafile_name,
                &mut current_picture,
                &mut pictures,
            )?;
        } else if apply_descriptor(
            &element,
            &parameters,
            lifecycle,
            &mut metafile,
            &mut current_picture,
        )? {
            element.support = ElementSupport::Descriptor;
        } else if apply_control(
            &element,
            &parameters,
            lifecycle,
            &metafile,
            &mut current_picture,
        )? {
            element.support = ElementSupport::Control;
        } else if apply_attribute(
            &element,
            &parameters,
            lifecycle,
            &metafile,
            &mut current_picture,
        )? {
            element.support = ElementSupport::Attribute;
        } else if apply_primitive(
            &element,
            &parameters,
            lifecycle,
            &metafile,
            &mut current_picture,
        )? {
            element.support = ElementSupport::Primitive;
        } else {
            diagnostics.push(CgmDiagnostic {
                code: CgmDiagnosticCode::UnsupportedElement,
                source_offset: element.source_offset,
                class: element.class,
                id: element.id,
                picture: current_picture.as_ref().map(|picture| picture.name.clone()),
                message: format!(
                    "CGM class {} element {} is not decoded by the lifecycle profile",
                    element.class, element.id
                ),
            });
        }

        offset = element.source_offset + element.encoded_length;
        elements.push(element);

        if delimiter == Some(DelimiterElement::EndMetafile) {
            let trailing = &bytes[offset..];
            let valid_record_padding = trailing.is_empty()
                || (trailing.len() == 2 && trailing.iter().all(|byte| *byte == 0));
            if !valid_record_padding {
                return Err(CgmError::TrailingData {
                    count: trailing.len(),
                });
            }
            break;
        }
    }

    if lifecycle != Lifecycle::Ended {
        return Err(CgmError::MissingEndMetafile);
    }

    Ok(CgmInspection {
        encoding: CgmEncoding::Binary,
        source_bytes: bytes.len(),
        trailing_padding_bytes: bytes.len() - offset,
        metafile_name: metafile_name.unwrap_or_default(),
        metafile,
        elements,
        pictures,
        diagnostics,
    })
}

pub fn parameter_bytes(bytes: &[u8], element: &CgmElement) -> CgmResult<Vec<u8>> {
    let mut parameters = Vec::with_capacity(element.parameter_length);
    for partition in &element.partitions {
        let end = partition
            .parameter_offset
            .checked_add(partition.parameter_length)
            .ok_or(CgmError::Truncated {
                offset: partition.parameter_offset,
                context: "element parameters",
            })?;
        let slice = bytes
            .get(partition.parameter_offset..end)
            .ok_or(CgmError::Truncated {
                offset: partition.parameter_offset,
                context: "element parameters",
            })?;
        parameters.extend_from_slice(slice);
    }
    Ok(parameters)
}

fn decode_element(
    bytes: &[u8],
    offset: usize,
    index: usize,
    limits: DecodeLimits,
) -> CgmResult<CgmElement> {
    let word = read_u16(bytes, offset, "binary element header")?;
    let class = (word >> 12) as u8;
    let id = ((word >> 5) & 0x7f) as u8;
    let short_length = (word & 0x1f) as usize;
    let mut partitions = Vec::new();
    let mut parameter_length = 0usize;
    let mut cursor = offset + 2;

    if short_length < LONG_FORM_LENGTH {
        require_range(bytes, cursor, short_length, "short-form parameters")?;
        partitions.push(CgmPartition {
            parameter_offset: cursor,
            parameter_length: short_length,
            continues: false,
        });
        parameter_length = short_length;
        cursor = padded_end(bytes, cursor, short_length, "short-form parameter padding")?;
    } else {
        loop {
            if partitions.len() >= limits.max_partitions_per_element {
                return Err(CgmError::PartitionLimit {
                    offset,
                    limit: limits.max_partitions_per_element,
                });
            }

            let partition_word = read_u16(bytes, cursor, "long-form partition header")?;
            cursor += 2;
            let continues = (partition_word & 0x8000) != 0;
            let partition_length = (partition_word & 0x7fff) as usize;
            require_range(
                bytes,
                cursor,
                partition_length,
                "long-form partition parameters",
            )?;
            parameter_length =
                parameter_length
                    .checked_add(partition_length)
                    .ok_or(CgmError::ParameterLimit {
                        offset,
                        actual: usize::MAX,
                        limit: limits.max_parameter_bytes,
                    })?;
            if parameter_length > limits.max_parameter_bytes {
                return Err(CgmError::ParameterLimit {
                    offset,
                    actual: parameter_length,
                    limit: limits.max_parameter_bytes,
                });
            }

            partitions.push(CgmPartition {
                parameter_offset: cursor,
                parameter_length: partition_length,
                continues,
            });
            cursor = padded_end(
                bytes,
                cursor,
                partition_length,
                "long-form partition padding",
            )?;
            if !continues {
                break;
            }
        }
    }

    if parameter_length > limits.max_parameter_bytes {
        return Err(CgmError::ParameterLimit {
            offset,
            actual: parameter_length,
            limit: limits.max_parameter_bytes,
        });
    }

    let delimiter = (class == 0)
        .then(|| DelimiterElement::from_id(id))
        .flatten();
    let support = if delimiter.is_some() {
        ElementSupport::Lifecycle
    } else {
        ElementSupport::Unsupported
    };

    Ok(CgmElement {
        index,
        source_offset: offset,
        encoded_length: cursor - offset,
        header_length: partitions
            .first()
            .map_or(2, |partition| partition.parameter_offset - offset),
        class,
        id,
        parameter_length,
        partitions,
        support,
        delimiter,
    })
}

#[allow(clippy::too_many_arguments)]
fn apply_delimiter(
    delimiter: DelimiterElement,
    element: &CgmElement,
    parameters: &[u8],
    lifecycle: &mut Lifecycle,
    metafile_name: &mut Option<String>,
    current_picture: &mut Option<PictureBuilder>,
    pictures: &mut Vec<CgmPicture>,
) -> CgmResult<()> {
    match delimiter {
        DelimiterElement::BeginMetafile => {
            expect_state(*lifecycle, Lifecycle::BeforeMetafile, element)?;
            *metafile_name = Some(decode_string(parameters, element.source_offset)?);
            *lifecycle = Lifecycle::Metafile;
        }
        DelimiterElement::EndMetafile => {
            expect_state(*lifecycle, Lifecycle::Metafile, element)?;
            *lifecycle = Lifecycle::Ended;
        }
        DelimiterElement::BeginPicture => {
            expect_state(*lifecycle, Lifecycle::Metafile, element)?;
            let name = decode_string(parameters, element.source_offset)?;
            *current_picture = Some(PictureBuilder {
                name,
                begin_element: element.index,
                body_element: None,
                descriptor: CgmPictureDescriptor::default(),
                controls: CgmPictureControlState::default(),
                state: CgmPresentationState::default(),
                attributes: Vec::new(),
                primitives: Vec::new(),
            });
            *lifecycle = Lifecycle::PictureDescriptor;
        }
        DelimiterElement::BeginPictureBody => {
            expect_state(*lifecycle, Lifecycle::PictureDescriptor, element)?;
            let picture = current_picture
                .as_mut()
                .ok_or_else(|| CgmError::InvalidLifecycle {
                    offset: element.source_offset,
                    reason: "BEGIN PICTURE BODY has no active picture".to_owned(),
                })?;
            picture.body_element = Some(element.index);
            *lifecycle = Lifecycle::PictureBody;
        }
        DelimiterElement::EndPicture => {
            if !matches!(
                lifecycle,
                Lifecycle::PictureDescriptor | Lifecycle::PictureBody
            ) {
                return Err(CgmError::InvalidLifecycle {
                    offset: element.source_offset,
                    reason: format!("END PICTURE is invalid while in state {lifecycle:?}"),
                });
            }
            let picture = current_picture
                .take()
                .ok_or_else(|| CgmError::InvalidLifecycle {
                    offset: element.source_offset,
                    reason: "END PICTURE has no active picture".to_owned(),
                })?;
            pictures.push(CgmPicture {
                name: picture.name,
                begin_element: picture.begin_element,
                body_element: picture.body_element,
                end_element: element.index,
                descriptor: picture.descriptor,
                controls: picture.controls,
                attributes: picture.attributes,
                primitives: picture.primitives,
            });
            *lifecycle = Lifecycle::Metafile;
        }
    }
    Ok(())
}

/// Records picture-body control elements without applying source-format paint
/// semantics in the provider-neutral vector layer.
fn apply_control(
    element: &CgmElement,
    parameters: &[u8],
    lifecycle: Lifecycle,
    metafile: &CgmMetafileDescriptor,
    current_picture: &mut Option<PictureBuilder>,
) -> CgmResult<bool> {
    if lifecycle != Lifecycle::PictureBody || element.class != 3 {
        return Ok(false);
    }

    let picture = current_picture
        .as_mut()
        .ok_or_else(|| CgmError::InvalidLifecycle {
            offset: element.source_offset,
            reason: "picture body control has no active picture".to_owned(),
        })?;
    match element.id {
        // CLIP RECTANGLE: two VDC points. Preserve their original order for
        // the same orientation reasons as the picture VDC extent.
        5 => {
            let points = decode_fixed_integer_vdc_points(parameters, metafile, element, 2)?;
            if points[0][0] == points[1][0] || points[0][1] == points[1][1] {
                return Err(CgmError::InvalidVdcExtent {
                    offset: element.source_offset,
                    reason: "clip rectangle has zero width or height".to_owned(),
                });
            }
            picture.controls.clip_rectangle = Some(CgmVdcExtent {
                first: points[0],
                second: points[1],
            });
            Ok(true)
        }
        // CLIP INDICATOR: CGM encodes 0 as OFF and 1 as ON.
        6 => {
            let value = read_parameter_u16(parameters, element, "clip indicator")?;
            picture.controls.clip_indicator = Some(match value {
                0 => CgmClipIndicator::Off,
                1 => CgmClipIndicator::On,
                _ => {
                    return Err(CgmError::InvalidPrimitive {
                        offset: element.source_offset,
                        reason: format!("unsupported clip indicator value {value}"),
                    })
                }
            });
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Records the small, explicit attribute profile exercised by the selected
/// WebCGM fixtures. This is intentionally source-state only: primitive
/// lowering will later decide how a snapshot of these mutations becomes
/// provider-neutral presentation geometry.
fn apply_attribute(
    element: &CgmElement,
    parameters: &[u8],
    lifecycle: Lifecycle,
    metafile: &CgmMetafileDescriptor,
    current_picture: &mut Option<PictureBuilder>,
) -> CgmResult<bool> {
    if lifecycle != Lifecycle::PictureBody || element.class != 5 {
        return Ok(false);
    }

    let value = match element.id {
        3 => Some(CgmAttributeValue::LineWidth {
            bytes: parameters.to_vec(),
        }),
        4 => Some(CgmAttributeValue::LineColor {
            color: decode_color(parameters, metafile, current_picture, element)?,
        }),
        22 => Some(CgmAttributeValue::InteriorStyle {
            value: read_parameter_u16(parameters, element, "interior style")?,
        }),
        23 => Some(CgmAttributeValue::FillColor {
            color: decode_color(parameters, metafile, current_picture, element)?,
        }),
        28 => Some(CgmAttributeValue::EdgeWidth {
            bytes: parameters.to_vec(),
        }),
        29 => Some(CgmAttributeValue::EdgeColor {
            color: decode_color(parameters, metafile, current_picture, element)?,
        }),
        30 => Some(CgmAttributeValue::EdgeVisibility {
            visible: read_parameter_u16(parameters, element, "edge visibility")? != 0,
        }),
        37 => Some(CgmAttributeValue::LineCap {
            bytes: parameters.to_vec(),
        }),
        38 => Some(CgmAttributeValue::LineJoin {
            value: read_parameter_u16(parameters, element, "line join")?,
        }),
        _ => None,
    };

    let Some(value) = value else {
        return Ok(false);
    };
    let picture = current_picture
        .as_mut()
        .ok_or_else(|| CgmError::InvalidLifecycle {
            offset: element.source_offset,
            reason: "picture body attribute has no active picture".to_owned(),
        })?;
    picture.state.apply(&value);
    picture.attributes.push(CgmAttribute {
        source_element: element.index,
        source_offset: element.source_offset,
        value,
    });
    Ok(true)
}

fn decode_color(
    parameters: &[u8],
    _metafile: &CgmMetafileDescriptor,
    current_picture: &Option<PictureBuilder>,
    element: &CgmElement,
) -> CgmResult<CgmColor> {
    let picture = current_picture
        .as_ref()
        .ok_or_else(|| CgmError::InvalidLifecycle {
            offset: element.source_offset,
            reason: "colour attribute has no active picture".to_owned(),
        })?;
    Ok(match picture.descriptor.color_selection_mode {
        CgmColorSelectionMode::Indexed => CgmColor::Indexed(parameters.to_vec()),
        CgmColorSelectionMode::Direct => CgmColor::Direct(parameters.to_vec()),
    })
}

/// Decodes the first selected CGM geometry forms without lowering them into a
/// shared vector path. The source type remains visible in `CgmPrimitiveKind`
/// until the later primitive-lowering slice earns a provider-neutral contract.
fn apply_primitive(
    element: &CgmElement,
    parameters: &[u8],
    lifecycle: Lifecycle,
    metafile: &CgmMetafileDescriptor,
    current_picture: &mut Option<PictureBuilder>,
) -> CgmResult<bool> {
    if lifecycle != Lifecycle::PictureBody || element.class != 4 {
        return Ok(false);
    }
    let kind = match element.id {
        1 => CgmPrimitiveKind::Polyline {
            points: decode_integer_vdc_points(parameters, metafile, element)?,
        },
        7 => CgmPrimitiveKind::Polygon {
            points: decode_integer_vdc_points(parameters, metafile, element)?,
        },
        8 => CgmPrimitiveKind::PolygonSet {
            records: decode_polygon_set_records(parameters, metafile, element)?,
        },
        11 => {
            let points = decode_fixed_integer_vdc_points(parameters, metafile, element, 2)?;
            CgmPrimitiveKind::Rectangle {
                first: points[0],
                second: points[1],
            }
        }
        12 => {
            let values = decode_fixed_i16_values(parameters, metafile, element, 3)?;
            CgmPrimitiveKind::Circle {
                center: [values[0], values[1]],
                radius: values[2],
            }
        }
        17 => {
            let points = decode_fixed_integer_vdc_points(parameters, metafile, element, 3)?;
            CgmPrimitiveKind::Ellipse {
                center: points[0],
                first_axis: points[1],
                second_axis: points[2],
            }
        }
        15 => {
            let values = decode_fixed_i16_values(parameters, metafile, element, 7)?;
            CgmPrimitiveKind::CircularArc {
                center: [values[0], values[1]],
                start_vector: [values[2], values[3]],
                end_vector: [values[4], values[5]],
                radius: values[6],
            }
        }
        18 => {
            let points = decode_fixed_integer_vdc_points(parameters, metafile, element, 5)?;
            CgmPrimitiveKind::EllipticalArc {
                center: points[0],
                first_axis: points[1],
                second_axis: points[2],
                start_vector: points[3],
                end_vector: points[4],
            }
        }
        _ => return Ok(false),
    };
    let picture = current_picture
        .as_mut()
        .ok_or_else(|| CgmError::InvalidLifecycle {
            offset: element.source_offset,
            reason: "picture body primitive has no active picture".to_owned(),
        })?;
    picture.primitives.push(CgmPrimitive {
        source_element: element.index,
        source_offset: element.source_offset,
        attribute_count: picture.attributes.len(),
        state: picture.state.clone(),
        controls: picture.controls.clone(),
        kind,
    });
    Ok(true)
}

fn decode_polygon_set_records(
    parameters: &[u8],
    metafile: &CgmMetafileDescriptor,
    element: &CgmElement,
) -> CgmResult<Vec<CgmPolygonSetRecord>> {
    if metafile.vdc_type != CgmVdcType::Integer || metafile.integer_precision != 16 {
        return Err(CgmError::UnsupportedIntegerPrecision {
            offset: element.source_offset,
            value: metafile.integer_precision,
        });
    }
    const RECORD_BYTES: usize = 6;
    if !parameters.len().is_multiple_of(RECORD_BYTES) {
        return Err(CgmError::InvalidPrimitive {
            offset: element.source_offset,
            reason: format!(
                "expected 16-bit polygon-set point/flag records, found {} parameter bytes",
                parameters.len()
            ),
        });
    }
    let mut records = Vec::with_capacity(parameters.len() / RECORD_BYTES);
    for record in parameters.chunks_exact(RECORD_BYTES) {
        let edge = match u16::from_be_bytes([record[4], record[5]]) {
            0 => CgmPolygonSetEdgeFlag::Invisible,
            1 => CgmPolygonSetEdgeFlag::Visible,
            2 => CgmPolygonSetEdgeFlag::CloseInvisible,
            3 => CgmPolygonSetEdgeFlag::CloseVisible,
            value => {
                return Err(CgmError::InvalidPrimitive {
                    offset: element.source_offset,
                    reason: format!("unsupported polygon-set edge flag {value}"),
                });
            }
        };
        records.push(CgmPolygonSetRecord {
            point: [
                i16::from_be_bytes([record[0], record[1]]) as i32,
                i16::from_be_bytes([record[2], record[3]]) as i32,
            ],
            edge,
        });
    }
    if records.is_empty() {
        return Err(CgmError::InvalidPrimitive {
            offset: element.source_offset,
            reason: "polygon set has no point/flag records".to_owned(),
        });
    }
    Ok(records)
}

fn decode_integer_vdc_points(
    parameters: &[u8],
    metafile: &CgmMetafileDescriptor,
    element: &CgmElement,
) -> CgmResult<Vec<[i32; 2]>> {
    if metafile.vdc_type != CgmVdcType::Integer || metafile.integer_precision != 16 {
        return Err(CgmError::UnsupportedIntegerPrecision {
            offset: element.source_offset,
            value: metafile.integer_precision,
        });
    }
    if !parameters.len().is_multiple_of(4) {
        return Err(CgmError::InvalidPrimitive {
            offset: element.source_offset,
            reason: format!(
                "expected 16-bit VDC coordinate pairs, found {} parameter bytes",
                parameters.len()
            ),
        });
    }
    let points = parameters
        .chunks_exact(4)
        .map(|pair| {
            [
                i16::from_be_bytes([pair[0], pair[1]]) as i32,
                i16::from_be_bytes([pair[2], pair[3]]) as i32,
            ]
        })
        .collect::<Vec<_>>();
    if points.is_empty() {
        return Err(CgmError::InvalidPrimitive {
            offset: element.source_offset,
            reason: "primitive has no VDC points".to_owned(),
        });
    }
    Ok(points)
}

fn decode_fixed_integer_vdc_points(
    parameters: &[u8],
    metafile: &CgmMetafileDescriptor,
    element: &CgmElement,
    point_count: usize,
) -> CgmResult<Vec<[i32; 2]>> {
    let expected_values = point_count
        .checked_mul(2)
        .ok_or_else(|| CgmError::InvalidPrimitive {
            offset: element.source_offset,
            reason: "fixed coordinate count overflowed".to_owned(),
        })?;
    let values = decode_fixed_i16_values(parameters, metafile, element, expected_values)?;
    Ok(values
        .chunks_exact(2)
        .map(|pair| [pair[0], pair[1]])
        .collect())
}

fn decode_fixed_i16_values(
    parameters: &[u8],
    metafile: &CgmMetafileDescriptor,
    element: &CgmElement,
    value_count: usize,
) -> CgmResult<Vec<i32>> {
    if metafile.vdc_type != CgmVdcType::Integer || metafile.integer_precision != 16 {
        return Err(CgmError::UnsupportedIntegerPrecision {
            offset: element.source_offset,
            value: metafile.integer_precision,
        });
    }
    let expected_bytes = value_count
        .checked_mul(2)
        .ok_or_else(|| CgmError::InvalidPrimitive {
            offset: element.source_offset,
            reason: "fixed parameter length overflowed".to_owned(),
        })?;
    if parameters.len() != expected_bytes {
        return Err(CgmError::InvalidPrimitive {
            offset: element.source_offset,
            reason: format!(
                "expected {expected_bytes} bytes for {value_count} integer values, found {}",
                parameters.len()
            ),
        });
    }
    Ok(parameters
        .chunks_exact(2)
        .map(|pair| i16::from_be_bytes([pair[0], pair[1]]) as i32)
        .collect())
}

/// Resolves the small descriptor profile needed before primitive parameters
/// have coordinate meaning. This deliberately stops before attributes and
/// primitive lowering; those remain later CGM-owned stages.
fn apply_descriptor(
    element: &CgmElement,
    parameters: &[u8],
    lifecycle: Lifecycle,
    metafile: &mut CgmMetafileDescriptor,
    current_picture: &mut Option<PictureBuilder>,
) -> CgmResult<bool> {
    match (element.class, element.id) {
        // VDC TYPE is a metafile descriptor. The selected v1 cases use
        // integer coordinates, which we can parse without inventing a real
        // number policy before the corpus has required one.
        (1, 3) if lifecycle == Lifecycle::Metafile => {
            let value = read_parameter_u16(parameters, element, "VDC type")?;
            metafile.vdc_type = match value {
                0 => CgmVdcType::Integer,
                1 => {
                    return Err(CgmError::UnsupportedVdcType {
                        offset: element.source_offset,
                        value,
                    })
                }
                _ => {
                    return Err(CgmError::UnsupportedVdcType {
                        offset: element.source_offset,
                        value,
                    })
                }
            };
            Ok(true)
        }
        (1, 4) if lifecycle == Lifecycle::Metafile => {
            let value = read_parameter_u16(parameters, element, "integer precision")?;
            if value != 16 {
                return Err(CgmError::UnsupportedIntegerPrecision {
                    offset: element.source_offset,
                    value,
                });
            }
            metafile.integer_precision = value;
            Ok(true)
        }
        (1, 7) if lifecycle == Lifecycle::Metafile => {
            let value = read_parameter_u16(parameters, element, "color precision")?;
            if value != 8 {
                return Err(CgmError::UnsupportedColorPrecision {
                    offset: element.source_offset,
                    kind: "direct color",
                    value,
                });
            }
            metafile.color_precision = value;
            Ok(true)
        }
        (1, 8) if lifecycle == Lifecycle::Metafile => {
            let value = read_parameter_u16(parameters, element, "color index precision")?;
            if value != 8 {
                return Err(CgmError::UnsupportedColorPrecision {
                    offset: element.source_offset,
                    kind: "color index",
                    value,
                });
            }
            metafile.color_index_precision = value;
            Ok(true)
        }
        (2, 1) if lifecycle == Lifecycle::PictureDescriptor => {
            let picture = active_picture_descriptor(element, current_picture)?;
            let mode = read_parameter_u16(parameters, element, "scaling mode")?;
            picture.scaling_mode = match mode {
                0 => CgmScalingMode::Abstract,
                1 => CgmScalingMode::Metric,
                _ => {
                    return Err(CgmError::InvalidLifecycle {
                        offset: element.source_offset,
                        reason: format!("unsupported scaling mode {mode}"),
                    });
                }
            };
            picture.metric_scale_bytes = (mode == 1).then(|| parameters[2..].to_vec());
            Ok(true)
        }
        (2, 2) if lifecycle == Lifecycle::PictureDescriptor => {
            let picture = active_picture_descriptor(element, current_picture)?;
            let mode = read_parameter_u16(parameters, element, "color selection mode")?;
            picture.color_selection_mode = match mode {
                0 => CgmColorSelectionMode::Indexed,
                1 => CgmColorSelectionMode::Direct,
                _ => {
                    return Err(CgmError::InvalidLifecycle {
                        offset: element.source_offset,
                        reason: format!("unsupported color selection mode {mode}"),
                    });
                }
            };
            Ok(true)
        }
        (2, 6) if lifecycle == Lifecycle::PictureDescriptor => {
            if metafile.vdc_type != CgmVdcType::Integer || metafile.integer_precision != 16 {
                return Err(CgmError::UnsupportedIntegerPrecision {
                    offset: element.source_offset,
                    value: metafile.integer_precision,
                });
            }
            if parameters.len() != 8 {
                return Err(CgmError::InvalidVdcExtent {
                    offset: element.source_offset,
                    reason: format!(
                        "expected four 16-bit coordinates, found {} bytes",
                        parameters.len()
                    ),
                });
            }
            let coordinates = parameters
                .chunks_exact(2)
                .map(|pair| i16::from_be_bytes([pair[0], pair[1]]) as i32)
                .collect::<Vec<_>>();
            let extent = CgmVdcExtent {
                first: [coordinates[0], coordinates[1]],
                second: [coordinates[2], coordinates[3]],
            };
            if extent.first[0] == extent.second[0] || extent.first[1] == extent.second[1] {
                return Err(CgmError::InvalidVdcExtent {
                    offset: element.source_offset,
                    reason: "extent has zero width or height".to_owned(),
                });
            }
            active_picture_descriptor(element, current_picture)?.vdc_extent = Some(extent);
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn active_picture_descriptor<'a>(
    element: &CgmElement,
    current_picture: &'a mut Option<PictureBuilder>,
) -> CgmResult<&'a mut CgmPictureDescriptor> {
    current_picture
        .as_mut()
        .map(|picture| &mut picture.descriptor)
        .ok_or_else(|| CgmError::InvalidLifecycle {
            offset: element.source_offset,
            reason: "picture descriptor has no active picture".to_owned(),
        })
}

fn read_parameter_u16(
    parameters: &[u8],
    element: &CgmElement,
    name: &'static str,
) -> CgmResult<u16> {
    let value = parameters
        .get(..2)
        .ok_or_else(|| CgmError::InvalidLifecycle {
            offset: element.source_offset,
            reason: format!("{name} requires a 16-bit parameter"),
        })?;
    Ok(u16::from_be_bytes([value[0], value[1]]))
}

fn decode_string(parameters: &[u8], element_offset: usize) -> CgmResult<String> {
    let Some((&first, remaining)) = parameters.split_first() else {
        return Ok(String::new());
    };
    if first == 255 {
        return Err(CgmError::InvalidString {
            offset: element_offset,
            reason: "extended CGM strings are outside the initial profile".to_owned(),
        });
    }

    let length = first as usize;
    let bytes = remaining
        .get(..length)
        .ok_or_else(|| CgmError::InvalidString {
            offset: element_offset,
            reason: format!(
                "declared string length {length} exceeds {} available bytes",
                remaining.len()
            ),
        })?;
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

fn expect_state(actual: Lifecycle, expected: Lifecycle, element: &CgmElement) -> CgmResult<()> {
    if actual != expected {
        return Err(CgmError::InvalidLifecycle {
            offset: element.source_offset,
            reason: format!(
                "{:?} requires state {expected:?}, found {actual:?}",
                element.delimiter
            ),
        });
    }
    Ok(())
}

fn padded_end(
    bytes: &[u8],
    start: usize,
    length: usize,
    context: &'static str,
) -> CgmResult<usize> {
    let end = start.checked_add(length).ok_or(CgmError::Truncated {
        offset: start,
        context,
    })?;
    let padded = end.checked_add(length & 1).ok_or(CgmError::Truncated {
        offset: end,
        context,
    })?;
    require_range(bytes, start, padded - start, context)?;
    Ok(padded)
}

fn require_range(
    bytes: &[u8],
    start: usize,
    length: usize,
    context: &'static str,
) -> CgmResult<()> {
    let end = start.checked_add(length).ok_or(CgmError::Truncated {
        offset: start,
        context,
    })?;
    if end > bytes.len() {
        return Err(CgmError::Truncated {
            offset: start,
            context,
        });
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize, context: &'static str) -> CgmResult<u16> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or(CgmError::Truncated { offset, context })?;
    Ok(u16::from_be_bytes([value[0], value[1]]))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Lifecycle {
    BeforeMetafile,
    Metafile,
    PictureDescriptor,
    PictureBody,
    Ended,
}

#[derive(Debug)]
struct PictureBuilder {
    name: String,
    begin_element: usize,
    body_element: Option<usize>,
    descriptor: CgmPictureDescriptor,
    controls: CgmPictureControlState,
    state: CgmPresentationState,
    attributes: Vec<CgmAttribute>,
    primitives: Vec<CgmPrimitive>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(class: u16, id: u16, length: u16) -> [u8; 2] {
        ((class << 12) | (id << 5) | length).to_be_bytes()
    }

    fn element(class: u16, id: u16, parameters: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(header(class, id, parameters.len() as u16));
        bytes.extend(parameters);
        if !parameters.len().is_multiple_of(2) {
            bytes.push(0);
        }
        bytes
    }

    fn minimal_document() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(element(0, 1, &[4, b't', b'e', b's', b't']));
        bytes.extend(element(0, 3, &[3, b'o', b'n', b'e']));
        bytes.extend(element(0, 4, &[]));
        // Polymarker remains outside the first primitive profile and keeps
        // this document useful for unsupported-element diagnostics.
        bytes.extend(element(4, 3, &[0, 1, 0, 2]));
        bytes.extend(element(0, 5, &[]));
        bytes.extend(element(0, 2, &[]));
        bytes
    }

    fn stateful_two_picture_document() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(element(0, 1, &[4, b't', b'e', b's', b't']));
        bytes.extend(element(1, 3, &[0, 0]));
        bytes.extend(element(1, 4, &[0, 16]));
        bytes.extend(element(0, 3, &[3, b'o', b'n', b'e']));
        bytes.extend(element(2, 1, &[0, 0]));
        bytes.extend(element(2, 2, &[0, 1]));
        bytes.extend(element(2, 6, &[0, 0, 3, 232, 3, 232, 0, 0]));
        bytes.extend(element(0, 4, &[]));
        bytes.extend(element(3, 5, &[0, 0, 0, 0, 3, 232, 3, 232]));
        bytes.extend(element(3, 6, &[0, 1]));
        bytes.extend(element(5, 3, &[0, 3]));
        bytes.extend(element(4, 1, &[0, 0, 0, 0, 0, 1, 0, 1]));
        bytes.extend(element(0, 5, &[]));
        bytes.extend(element(0, 3, &[3, b't', b'w', b'o']));
        bytes.extend(element(0, 4, &[]));
        bytes.extend(element(4, 1, &[0, 0, 0, 0, 0, 1, 0, 1]));
        bytes.extend(element(0, 5, &[]));
        bytes.extend(element(0, 2, &[]));
        bytes
    }

    #[test]
    fn inspects_lifecycle_and_preserves_unsupported_elements() {
        let inspection = inspect_binary_cgm(&minimal_document(), DecodeLimits::default())
            .expect("minimal binary CGM should inspect");

        assert_eq!(inspection.metafile_name, "test");
        assert_eq!(inspection.pictures.len(), 1);
        assert_eq!(inspection.pictures[0].name, "one");
        assert_eq!(inspection.elements.len(), 6);
        assert_eq!(inspection.diagnostics.len(), 1);
        assert_eq!(inspection.diagnostics[0].class, 4);
        assert_eq!(inspection.diagnostics[0].id, 3);
        assert_eq!(inspection.diagnostics[0].picture.as_deref(), Some("one"));
    }

    #[test]
    fn rejects_non_binary_signature() {
        let error = inspect_binary_cgm(b"BEGMF sample", DecodeLimits::default())
            .expect_err("clear-text signature must not enter the binary decoder");
        assert!(matches!(error, CgmError::UnsupportedEncoding { .. }));
    }

    #[test]
    fn rejects_truncated_short_parameters() {
        let bytes = [header(0, 1, 4), [1, b'x']].concat();
        let error = inspect_binary_cgm(&bytes, DecodeLimits::default())
            .expect_err("truncated parameters must fail");
        assert!(matches!(error, CgmError::Truncated { .. }));
    }

    #[test]
    fn rejects_oversized_input_before_parsing() {
        let limits = DecodeLimits {
            max_input_bytes: 3,
            ..DecodeLimits::default()
        };
        let error = inspect_binary_cgm(&minimal_document(), limits)
            .expect_err("oversized input must fail before parsing");
        assert!(matches!(error, CgmError::InputTooLarge { .. }));
    }

    #[test]
    fn decodes_partitioned_long_form_parameters() {
        let mut bytes = Vec::new();
        bytes.extend(header(0, 1, 31));
        bytes.extend(0x8003u16.to_be_bytes());
        bytes.extend([1, b'a', 0]);
        bytes.push(0);
        bytes.extend(0x0002u16.to_be_bytes());
        bytes.extend([b'b', b'c']);
        bytes.extend(element(0, 2, &[]));

        let inspection = inspect_binary_cgm(&bytes, DecodeLimits::default())
            .expect("partitioned element should inspect");
        assert_eq!(inspection.elements[0].partitions.len(), 2);
        assert_eq!(
            parameter_bytes(&bytes, &inspection.elements[0]).expect("parameters"),
            [1, b'a', 0, b'b', b'c']
        );
    }

    #[test]
    fn rejects_invalid_lifecycle_order() {
        let mut bytes = Vec::new();
        bytes.extend(element(0, 1, &[1, b'x']));
        bytes.extend(element(0, 4, &[]));
        let error = inspect_binary_cgm(&bytes, DecodeLimits::default())
            .expect_err("picture body before picture must fail");
        assert!(matches!(error, CgmError::InvalidLifecycle { .. }));
    }

    #[test]
    fn accepts_two_byte_zero_record_padding() {
        let mut bytes = minimal_document();
        bytes.extend([0, 0]);

        let inspection = inspect_binary_cgm(&bytes, DecodeLimits::default())
            .expect("two-byte zero record padding should be preserved");
        assert_eq!(inspection.trailing_padding_bytes, 2);
    }

    #[test]
    fn rejects_nonzero_trailing_data() {
        let mut bytes = minimal_document();
        bytes.extend([0, 1]);

        let error = inspect_binary_cgm(&bytes, DecodeLimits::default())
            .expect_err("nonzero trailing data must not be treated as alignment padding");
        assert!(matches!(error, CgmError::TrailingData { count: 2 }));
    }

    #[test]
    fn rejects_truncated_long_form_partition() {
        let mut bytes = Vec::new();
        bytes.extend(header(0, 1, 31));
        bytes.extend(0x0004u16.to_be_bytes());
        bytes.extend([1, b'x']);

        let error = inspect_binary_cgm(&bytes, DecodeLimits::default())
            .expect_err("a truncated long-form partition must fail");
        assert!(matches!(error, CgmError::Truncated { .. }));
    }

    #[test]
    fn enforces_partition_limit() {
        let mut bytes = Vec::new();
        bytes.extend(header(0, 1, 31));
        bytes.extend(0x8002u16.to_be_bytes());
        bytes.extend([1, b'x']);
        bytes.extend(0x0002u16.to_be_bytes());
        bytes.extend([0, 0]);
        let limits = DecodeLimits {
            max_partitions_per_element: 1,
            ..DecodeLimits::default()
        };

        let error = inspect_binary_cgm(&bytes, limits)
            .expect_err("a continued element must respect the partition limit");
        assert!(matches!(error, CgmError::PartitionLimit { limit: 1, .. }));
    }

    #[test]
    fn enforces_parameter_limit() {
        let limits = DecodeLimits {
            max_parameter_bytes: 2,
            ..DecodeLimits::default()
        };

        let error = inspect_binary_cgm(&minimal_document(), limits)
            .expect_err("element parameters must respect the configured limit");
        assert!(matches!(
            error,
            CgmError::ParameterLimit {
                actual: 5,
                limit: 2,
                ..
            }
        ));
    }

    #[test]
    fn preserves_source_extent_orientation_and_resets_picture_state() {
        let inspection =
            inspect_binary_cgm(&stateful_two_picture_document(), DecodeLimits::default())
                .expect("descriptor profile should inspect");

        assert_eq!(inspection.metafile.vdc_type, CgmVdcType::Integer);
        assert_eq!(inspection.metafile.integer_precision, 16);
        assert_eq!(inspection.pictures.len(), 2);
        assert_eq!(
            inspection.pictures[0].descriptor.vdc_extent,
            Some(CgmVdcExtent {
                first: [0, 1000],
                second: [1000, 0],
            })
        );
        assert_eq!(
            inspection.pictures[0].descriptor.color_selection_mode,
            CgmColorSelectionMode::Direct
        );
        assert_eq!(
            inspection.pictures[1].descriptor.vdc_extent, None,
            "a second picture must not inherit the prior picture extent"
        );
        assert_eq!(
            inspection.pictures[1].descriptor.color_selection_mode,
            CgmColorSelectionMode::Indexed,
            "a second picture must start from CGM descriptor defaults"
        );
        assert_eq!(
            inspection.pictures[0].primitives[0].state.line_width,
            Some(vec![0, 3]),
            "the first picture should retain its explicit drawing state"
        );
        assert_eq!(
            inspection.pictures[1].primitives[0].state.line_width, None,
            "a second picture must not inherit the first picture's drawing state"
        );
        assert_eq!(
            inspection.pictures[0].controls.clip_rectangle,
            Some(CgmVdcExtent {
                first: [0, 0],
                second: [1000, 1000],
            })
        );
        assert_eq!(
            inspection.pictures[0].controls.clip_indicator,
            Some(CgmClipIndicator::On)
        );
        assert_eq!(
            inspection.pictures[0].primitives[0].controls, inspection.pictures[0].controls,
            "a primitive must snapshot the active picture-body controls"
        );
        assert_eq!(
            inspection.pictures[1].controls,
            CgmPictureControlState::default(),
            "a second picture must not inherit the first picture's controls"
        );
        assert_eq!(
            inspection.pictures[1].primitives[0].controls,
            CgmPictureControlState::default(),
            "a primitive must not inherit controls from an earlier picture"
        );
    }

    #[test]
    fn rejects_real_vdc_coordinates_before_primitive_interpretation() {
        let mut bytes = Vec::new();
        bytes.extend(element(0, 1, &[1, b'x']));
        bytes.extend(element(1, 3, &[0, 1]));

        let error = inspect_binary_cgm(&bytes, DecodeLimits::default())
            .expect_err("real VDC coordinates must remain explicit until supported");
        assert!(matches!(
            error,
            CgmError::UnsupportedVdcType { value: 1, .. }
        ));
    }

    #[test]
    fn rejects_unsupported_color_precisions_before_color_interpretation() {
        for (id, kind) in [(7, "direct color"), (8, "color index")] {
            let mut bytes = Vec::new();
            bytes.extend(element(0, 1, &[1, b'x']));
            bytes.extend(element(1, id, &[0, 16]));

            let error = inspect_binary_cgm(&bytes, DecodeLimits::default())
                .expect_err("unadmitted color precision must fail at the CGM boundary");
            assert!(matches!(
                error,
                CgmError::UnsupportedColorPrecision {
                    kind: actual_kind,
                    value: 16,
                    ..
                } if actual_kind == kind
            ));
        }
    }

    #[test]
    fn snapshots_explicit_attribute_state_at_each_primitive() {
        let mut bytes = Vec::new();
        bytes.extend(element(0, 1, &[1, b'x']));
        bytes.extend(element(0, 3, &[3, b'p', b'i', b'c']));
        bytes.extend(element(0, 4, &[]));
        bytes.extend(element(5, 3, &[0, 1]));
        bytes.extend(element(4, 1, &[0, 2, 0, 3, 0, 4, 0, 5]));
        bytes.extend(element(5, 3, &[0, 2]));
        bytes.extend(element(4, 1, &[0, 6, 0, 7, 0, 8, 0, 9]));
        bytes.extend(element(0, 5, &[]));
        bytes.extend(element(0, 2, &[]));

        let inspection = inspect_binary_cgm(&bytes, DecodeLimits::default())
            .expect("integer polyline should inspect");
        let primitives = &inspection.pictures[0].primitives;
        assert_eq!(primitives.len(), 2);
        assert_eq!(primitives[0].attribute_count, 1);
        assert_eq!(primitives[0].state.line_width, Some(vec![0, 1]));
        assert_eq!(
            primitives[0].kind,
            CgmPrimitiveKind::Polyline {
                points: vec![[2, 3], [4, 5]],
            }
        );
        assert_eq!(primitives[1].attribute_count, 2);
        assert_eq!(primitives[1].state.line_width, Some(vec![0, 2]));
        assert_eq!(
            primitives[1].kind,
            CgmPrimitiveKind::Polyline {
                points: vec![[6, 7], [8, 9]],
            }
        );
    }

    #[test]
    fn normalizes_against_source_order_without_erasing_axis_direction() {
        let extent = CgmVdcExtent {
            first: [0, 1000],
            second: [1000, 0],
        };

        assert_eq!(extent.normalize([0, 1000]), Some([0.0, 0.0]));
        assert_eq!(extent.normalize([1000, 0]), Some([1.0, 1.0]));
        assert_eq!(extent.normalize([500, 500]), Some([0.5, 0.5]));
        assert_eq!(
            CgmVdcExtent {
                first: [0, 0],
                second: [0, 1],
            }
            .normalize([0, 0]),
            None
        );
    }

    #[test]
    fn rejects_malformed_polygon_set_record_cadence() {
        let mut bytes = Vec::new();
        bytes.extend(element(0, 1, &[1, b'x']));
        bytes.extend(element(0, 3, &[3, b'p', b'i', b'c']));
        bytes.extend(element(0, 4, &[]));
        // POLYGON SET records are two 16-bit VDC coordinates plus one
        // 16-bit edge flag. Eight bytes cannot encode whole records.
        bytes.extend(element(4, 8, &[0, 0, 0, 0, 0, 1, 0, 1]));

        let error = inspect_binary_cgm(&bytes, DecodeLimits::default())
            .expect_err("partial polygon-set records must fail during CGM decoding");
        assert!(matches!(error, CgmError::InvalidPrimitive { .. }));
    }

    #[test]
    fn rejects_unknown_polygon_set_edge_flags_without_inventing_topology() {
        let mut bytes = Vec::new();
        bytes.extend(element(0, 1, &[1, b'x']));
        bytes.extend(element(0, 3, &[3, b'p', b'i', b'c']));
        bytes.extend(element(0, 4, &[]));
        // The four admitted flags are invisible, visible, close-invisible,
        // and close-visible. A fifth value must not acquire guessed meaning.
        bytes.extend(element(4, 8, &[0, 0, 0, 0, 0, 4]));

        let error = inspect_binary_cgm(&bytes, DecodeLimits::default())
            .expect_err("unknown polygon-set flags must fail during CGM decoding");
        assert!(matches!(error, CgmError::InvalidPrimitive { .. }));
    }

    #[test]
    fn rejects_unknown_clip_indicator_without_creating_geometry() {
        let mut bytes = Vec::new();
        bytes.extend(element(0, 1, &[1, b'x']));
        bytes.extend(element(0, 3, &[3, b'p', b'i', b'c']));
        bytes.extend(element(0, 4, &[]));
        bytes.extend(element(3, 6, &[0, 2]));

        let error = inspect_binary_cgm(&bytes, DecodeLimits::default())
            .expect_err("unknown clip indicators must not silently select a policy");
        assert!(matches!(error, CgmError::InvalidPrimitive { .. }));
    }
}
