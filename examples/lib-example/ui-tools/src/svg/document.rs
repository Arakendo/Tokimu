use std::collections::{HashMap, HashSet};

use xml_tools::{parse_xml_events, XmlAttribute, XmlEvent, XmlOptions, XmlSourceId};

use super::path::flatten_path;
use super::primitives::{parse_svg_point_numbers, svg_rectangle};
use super::semantic::{
    is_svg_geometry_element, is_svg_name, is_unadmitted_svg_feature,
    normalize_svg_point_with_aspect, parse_svg_root_preserve_aspect_ratio, parse_svg_root_view_box,
    svg_attribute_value, svg_number_attribute, svg_semantic_events, validate_svg_view_box,
    SvgSemanticEvent, SvgSemanticFrame,
};
use super::{
    parse_path, SvgImportDiagnostic, SvgPreserveAspectRatio, SvgVectorRecord, SvgViewportSource,
};

type Bounds = Option<([f32; 2], [f32; 2])>;

fn svg_fragment_reference(attributes: &[XmlAttribute]) -> Option<&str> {
    attributes.iter().find_map(|attribute| {
        let is_href = attribute.name.local_name == "href"
            && (attribute.name.namespace_uri.is_none()
                || attribute.name.namespace_uri.as_deref() == Some("http://www.w3.org/1999/xlink"));
        is_href.then(|| attribute.value.trim())
    })
}

fn svg_definition_id(attributes: &[XmlAttribute]) -> Option<&str> {
    attributes.iter().find_map(|attribute| {
        (attribute.name.namespace_uri.is_none() && attribute.name.local_name == "id")
            .then(|| attribute.value.trim())
            .filter(|value| !value.is_empty())
    })
}

fn svg_use_has_unadmitted_overrides(attributes: &[XmlAttribute]) -> bool {
    attributes.iter().any(|attribute| {
        let local_name = attribute.name.local_name.as_str();
        local_name != "href"
            && local_name != "id"
            && (attribute.name.namespace_uri.is_none()
                || attribute.name.namespace_uri.as_deref() == Some("http://www.w3.org/1999/xlink"))
    })
}

fn active_clip_path_id(element_stack: &[SvgSemanticFrame]) -> Option<String> {
    element_stack
        .iter()
        .rev()
        .find(|frame| {
            is_svg_name(&frame.element.name) && frame.element.name.local_name == "clipPath"
        })
        .and_then(|frame| svg_definition_id(&frame.element.attributes))
        .map(str::to_owned)
}

fn bounds_of_points(points: &[[f32; 2]]) -> Bounds {
    points.iter().copied().fold(None, |bounds, point| {
        Some(match bounds {
            Some((min, max)) => (
                [min[0].min(point[0]), min[1].min(point[1])],
                [max[0].max(point[0]), max[1].max(point[1])],
            ),
            None => (point, point),
        })
    })
}

fn union_bounds(first: Bounds, second: Bounds) -> Bounds {
    match (first, second) {
        (Some((first_min, first_max)), Some((second_min, second_max))) => Some((
            [
                first_min[0].min(second_min[0]),
                first_min[1].min(second_min[1]),
            ],
            [
                first_max[0].max(second_max[0]),
                first_max[1].max(second_max[1]),
            ],
        )),
        (Some(bounds), None) | (None, Some(bounds)) => Some(bounds),
        (None, None) => None,
    }
}

fn normalize_points(
    points: &[[f32; 2]],
    transform: super::transform::SvgAffine,
    view_box: [f32; 4],
    preserve_aspect_ratio: SvgPreserveAspectRatio,
) -> (Vec<[f32; 2]>, Bounds, Bounds) {
    let transformed = points
        .iter()
        .copied()
        .map(|point| transform.apply(point))
        .collect::<Vec<_>>();
    let source_bounds = bounds_of_points(points);
    let transformed_bounds = bounds_of_points(&transformed);
    let normalized = transformed
        .iter()
        .copied()
        .map(|point| normalize_svg_point_with_aspect(point, view_box, preserve_aspect_ratio))
        .collect();
    (normalized, source_bounds, transformed_bounds)
}

/// Extracts SVG geometry into the shared provider-neutral path model.
///
/// This is an intentionally small migration adapter over the existing parser:
/// flattened contours retain explicit closure when the parser emitted a
/// repeated endpoint. SVG styling and topology beyond that contract remain
/// importer concerns.
/// Extracts SVG geometry while preserving XML- and SVG-stage diagnostics.
///
/// Callers that need an actionable boundary should use this form. The
/// string-returning convenience API below delegates here so the normal parsing
/// implementation remains singular.
pub fn parse_svg_document_vector_records_with_xml_options(
    svg: &str,
    subdivisions: usize,
    view_box: [f32; 4],
    xml_options: XmlOptions,
) -> Result<Vec<SvgVectorRecord>, SvgImportDiagnostic> {
    parse_svg_document_vector_records_with_viewport(
        svg,
        subdivisions,
        SvgViewportSource::Caller(view_box),
        xml_options,
    )
}

/// Extracts SVG geometry with an explicit caller or root-document `viewBox`
/// policy. Physical viewport sizing is intentionally outside this
/// coordinate-only importer profile.
pub fn parse_svg_document_vector_records_with_viewport(
    svg: &str,
    subdivisions: usize,
    viewport_source: SvgViewportSource,
    xml_options: XmlOptions,
) -> Result<Vec<SvgVectorRecord>, SvgImportDiagnostic> {
    let events = parse_xml_events(XmlSourceId::new(0), svg, xml_options)
        .map_err(SvgImportDiagnostic::from)?;
    parse_svg_document_vector_records_from_xml_events(&events, subdivisions, viewport_source)
}

/// Lowers an already-parsed XML event stream through the SVG semantic profile.
///
/// This lets corpus tooling retain XML-stage evidence and lower the exact same
/// parse result without running a second XML parser. XML diagnostics remain
/// the caller's responsibility because malformed input cannot produce events.
/// The current profile admits only `svg` and `g` containers plus `path`,
/// `circle`, `line`, `polyline`, `polygon`, and `rect` geometry. Known SVG
/// features outside that profile diagnose explicitly rather than silently
/// claiming browser-level SVG support.
pub fn parse_svg_document_vector_records_from_xml_events(
    xml_events: &[XmlEvent],
    subdivisions: usize,
    viewport_source: SvgViewportSource,
) -> Result<Vec<SvgVectorRecord>, SvgImportDiagnostic> {
    let mut resolved_view_box = match viewport_source {
        SvgViewportSource::Caller(view_box) => Some(
            validate_svg_view_box(view_box)
                .map_err(|message| SvgImportDiagnostic::svg(None, message))?,
        ),
        SvgViewportSource::DocumentViewBox => None,
    };
    let mut preserve_aspect_ratio = SvgPreserveAspectRatio::None;
    let mut paths = Vec::<(usize, SvgVectorRecord)>::new();
    let mut definitions = HashMap::<String, SvgVectorRecord>::new();
    let mut clip_paths = HashMap::<String, crate::VectorPath>::new();
    let mut definition_ids = HashSet::<String>::new();
    let mut element_stack = Vec::<SvgSemanticFrame>::new();
    let mut saw_root = false;

    for event in svg_semantic_events(xml_events) {
        let element = match event {
            SvgSemanticEvent::Start(element) => {
                if element_stack.is_empty() {
                    saw_root = true;
                    if element.name.local_name != "svg" || !is_svg_name(&element.name) {
                        return Err(SvgImportDiagnostic::svg(
                            Some(element.span),
                            "SVG document root must be an unqualified or SVG-namespaced <svg> element",
                        ));
                    }
                    if viewport_source == SvgViewportSource::DocumentViewBox {
                        resolved_view_box =
                            Some(parse_svg_root_view_box(&element.attributes).map_err(
                                |message| SvgImportDiagnostic::svg(Some(element.span), message),
                            )?);
                        preserve_aspect_ratio = parse_svg_root_preserve_aspect_ratio(
                            &element.attributes,
                        )
                        .map_err(|message| SvgImportDiagnostic::svg(Some(element.span), message))?;
                    }
                }
                let inherited_presentation = element_stack
                    .last()
                    .map(|frame| frame.presentation.clone())
                    .unwrap_or_default();
                let presentation = if is_svg_name(&element.name) {
                    inherited_presentation
                        .inherit_and_apply(&element.attributes)
                        .map_err(|message| SvgImportDiagnostic::svg(Some(element.span), message))?
                } else {
                    inherited_presentation
                };
                let parent_renders_geometry = element_stack
                    .last()
                    .is_none_or(|frame| frame.render_geometry);
                let render_geometry = parent_renders_geometry
                    && !(is_svg_name(&element.name) && element.name.local_name == "defs");
                let is_geometry = is_svg_geometry_element(&element);
                let is_use = is_svg_name(&element.name) && element.name.local_name == "use";
                if is_svg_name(&element.name)
                    && element.name.local_name == "clipPath"
                    && element_stack.iter().any(|frame| {
                        is_svg_name(&frame.element.name)
                            && frame.element.name.local_name == "clipPath"
                    })
                {
                    return Err(SvgImportDiagnostic::svg(
                        Some(element.span),
                        "nested SVG clipPath elements are outside the admitted one-level profile",
                    ));
                }
                if !is_geometry && is_unadmitted_svg_feature(&element) {
                    return Err(SvgImportDiagnostic::svg(
                        Some(element.span),
                        format!(
                            "SVG element '{}' is outside the admitted importer profile",
                            element.name.local_name
                        ),
                    ));
                }
                if !render_geometry {
                    if let Some(id) = svg_definition_id(&element.attributes) {
                        if !definition_ids.insert(id.to_owned()) {
                            return Err(SvgImportDiagnostic::svg(
                                Some(element.span),
                                format!("duplicate SVG definition id '{id}'"),
                            ));
                        }
                    }
                }
                element_stack.push(SvgSemanticFrame {
                    element: element.clone(),
                    presentation: presentation.clone(),
                    render_geometry,
                });
                if (!is_geometry && !is_use) || (is_use && !render_geometry) {
                    continue;
                }
                (element, presentation, render_geometry)
            }
            SvgSemanticEvent::End { name, span } => {
                let Some(open) = element_stack.pop() else {
                    return Err(SvgImportDiagnostic::svg(
                        Some(span),
                        "SVG semantic traversal encountered an end element without an open element",
                    ));
                };
                if open.element.name != name {
                    return Err(SvgImportDiagnostic::svg(
                        Some(span),
                        format!(
                            "SVG semantic traversal closed '{}' while '{}' remained open",
                            name.local_name, open.element.name.local_name
                        ),
                    ));
                }
                continue;
            }
        };
        let (element, presentation, render_geometry) = element;
        let view_box = resolved_view_box.expect("an admitted SVG root resolves a viewBox");
        let attributes = &element.attributes;
        let clip_definition_id = active_clip_path_id(&element_stack);
        if element.name.local_name == "use" {
            if svg_use_has_unadmitted_overrides(attributes)
                || presentation.transform != super::transform::SvgAffine::IDENTITY
            {
                return Err(SvgImportDiagnostic::svg(
                    Some(element.span),
                    "SVG <use> overrides and transforms are outside the admitted local-fragment profile",
                ));
            }
            let Some(reference) = svg_fragment_reference(attributes) else {
                return Err(SvgImportDiagnostic::svg(
                    Some(element.span),
                    "SVG <use> requires a local fragment href such as '#shape'",
                ));
            };
            let Some(id) = reference.strip_prefix('#').filter(|id| !id.is_empty()) else {
                return Err(SvgImportDiagnostic::svg(
                    Some(element.span),
                    format!("SVG <use> reference '{reference}' is external; only local fragments are admitted"),
                ));
            };
            let Some(record) = definitions.get(id).cloned() else {
                let message = if definition_ids.contains(id) {
                    format!("SVG <use> target '#{id}' is a cyclic or non-geometric definition")
                } else {
                    format!("SVG <use> target '#{id}' is missing")
                };
                return Err(SvgImportDiagnostic::svg(Some(element.span), message));
            };
            let mut reused = record;
            reused.source_span = element.span;
            if render_geometry {
                paths.push((element.span.start, reused));
            }
            continue;
        }
        let number = |name: &str| {
            svg_number_attribute(attributes, name)
                .map_err(|message| SvgImportDiagnostic::svg(Some(element.span), message))
        };
        let points = match element.name.local_name.as_str() {
            "path" => {
                let Some(data) = svg_attribute_value(attributes, "d") else {
                    continue;
                };
                let commands = parse_path(data)
                    .map_err(|message| SvgImportDiagnostic::svg(Some(element.span), message))?;
                let raw_contours = flatten_path(&commands, subdivisions)
                    .into_iter()
                    .filter(|points| points.len() > 1)
                    .collect::<Vec<_>>();
                let source_bounds = raw_contours
                    .iter()
                    .map(|points| bounds_of_points(points))
                    .fold(None, union_bounds);
                let transformed_contours = raw_contours
                    .iter()
                    .map(|points| {
                        points
                            .iter()
                            .copied()
                            .map(|point| presentation.transform.apply(point))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                let transformed_bounds = transformed_contours
                    .iter()
                    .map(|points| bounds_of_points(points))
                    .fold(None, union_bounds);
                let contours = transformed_contours
                    .into_iter()
                    .map(|points| {
                        let mut points = points
                            .into_iter()
                            .map(|point| {
                                normalize_svg_point_with_aspect(
                                    point,
                                    view_box,
                                    preserve_aspect_ratio,
                                )
                            })
                            .collect::<Vec<_>>();
                        let closed = points.len() > 1 && points.first() == points.last();
                        if closed {
                            points.pop();
                        }
                        crate::VectorContour::new(points, closed)
                    })
                    .collect::<Vec<_>>();
                if !contours.is_empty() {
                    let clip_path_reference = presentation.clip_path.clone();
                    let mut record = SvgVectorRecord::from_presentation(
                        crate::VectorPath::new(contours),
                        source_bounds,
                        transformed_bounds,
                        presentation,
                        element.span,
                    );
                    if let Some(clip_id) = clip_path_reference.as_deref() {
                        let Some(clip_path) = clip_paths.get(clip_id).cloned() else {
                            return Err(SvgImportDiagnostic::svg(
                                Some(element.span),
                                format!(
                                    "SVG clip-path target '#{clip_id}' is missing or not yet defined"
                                ),
                            ));
                        };
                        record.clip_path = Some(clip_path);
                    }
                    if let Some(clip_id) = clip_definition_id {
                        if clip_paths
                            .insert(clip_id.clone(), record.path.clone())
                            .is_some()
                        {
                            return Err(SvgImportDiagnostic::svg(
                                Some(element.span),
                                format!(
                                    "SVG clipPath '#{clip_id}' contains multiple geometric children"
                                ),
                            ));
                        }
                        continue;
                    }
                    if render_geometry {
                        paths.push((element.span.start, record));
                    } else if let Some(id) = svg_definition_id(attributes) {
                        definitions.insert(id.to_owned(), record);
                    }
                }
                continue;
            }
            "circle" => {
                let Some(radius) = number("r")? else {
                    continue;
                };
                // SVG defaults omitted circle centers to the origin.
                let cx = number("cx")?.unwrap_or(0.0);
                let cy = number("cy")?.unwrap_or(0.0);
                if radius < 0.0 {
                    return Err(SvgImportDiagnostic::svg(
                        Some(element.span),
                        "SVG circle radius must not be negative",
                    ));
                }
                if radius <= f32::EPSILON {
                    None
                } else {
                    Some(
                        (0..=subdivisions.max(16))
                            .map(|index| {
                                let angle = index as f32 * std::f32::consts::TAU
                                    / subdivisions.max(16) as f32;
                                [cx + radius * angle.cos(), cy + radius * angle.sin()]
                            })
                            .collect::<Vec<_>>(),
                    )
                }
            }
            "ellipse" => {
                let (Some(rx), Some(ry)) = (number("rx")?, number("ry")?) else {
                    continue;
                };
                // SVG defaults omitted ellipse centers to the origin.
                let cx = number("cx")?.unwrap_or(0.0);
                let cy = number("cy")?.unwrap_or(0.0);
                if rx < 0.0 || ry < 0.0 {
                    return Err(SvgImportDiagnostic::svg(
                        Some(element.span),
                        "SVG ellipse radii must not be negative",
                    ));
                }
                if rx <= f32::EPSILON || ry <= f32::EPSILON {
                    None
                } else {
                    Some(
                        (0..=subdivisions.max(16))
                            .map(|index| {
                                let angle = index as f32 * std::f32::consts::TAU
                                    / subdivisions.max(16) as f32;
                                [cx + rx * angle.cos(), cy + ry * angle.sin()]
                            })
                            .collect::<Vec<_>>(),
                    )
                }
            }
            "line" => {
                let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                    (number("x1")?, number("y1")?, number("x2")?, number("y2")?)
                else {
                    continue;
                };
                Some(vec![[x1, y1], [x2, y2]])
            }
            "polyline" | "polygon" => {
                let Some(values) = svg_attribute_value(attributes, "points") else {
                    continue;
                };
                let numbers = parse_svg_point_numbers(values, &element.name.local_name)
                    .map_err(|message| SvgImportDiagnostic::svg(Some(element.span), message))?;
                let mut points = numbers
                    .chunks_exact(2)
                    .map(|pair| [pair[0], pair[1]])
                    .collect::<Vec<_>>();
                if element.name.local_name == "polygon" && points.first() != points.last() {
                    if let Some(first) = points.first().copied() {
                        points.push(first);
                    }
                }
                Some(points)
            }
            "rect" => {
                let (Some(x), Some(y), Some(width), Some(height)) = (
                    number("x")?,
                    number("y")?,
                    number("width")?,
                    number("height")?,
                ) else {
                    continue;
                };
                if width < 0.0 || height < 0.0 {
                    return Err(SvgImportDiagnostic::svg(
                        Some(element.span),
                        "SVG rectangle width and height must not be negative",
                    ));
                }
                let raw_rx = number("rx")?;
                let raw_ry = number("ry")?;
                if raw_rx.is_some_and(|value| value < 0.0)
                    || raw_ry.is_some_and(|value| value < 0.0)
                {
                    return Err(SvgImportDiagnostic::svg(
                        Some(element.span),
                        "SVG rectangle corner radii must not be negative",
                    ));
                }
                let (rx, ry) = match (raw_rx, raw_ry) {
                    (Some(rx), Some(ry)) => (rx, ry),
                    (Some(rx), None) => (rx, rx),
                    (None, Some(ry)) => (ry, ry),
                    (None, None) => (0.0, 0.0),
                };
                let rx = rx.min(width * 0.5);
                let ry = ry.min(height * 0.5);
                Some(
                    svg_rectangle(x, y, width, height, rx, ry)
                        .into_iter()
                        .collect(),
                )
            }
            _ => unreachable!("only admitted SVG geometry reaches lowering"),
        };
        if let Some(points) = points.filter(|points: &Vec<[f32; 2]>| points.len() > 1) {
            let closed = matches!(
                element.name.local_name.as_str(),
                "circle" | "ellipse" | "rect" | "polygon"
            );
            let (normalized_points, source_bounds, transformed_bounds) = normalize_points(
                &points,
                presentation.transform,
                view_box,
                preserve_aspect_ratio,
            );
            let points = if closed {
                normalized_points[..normalized_points.len() - 1].to_vec()
            } else {
                normalized_points
            };
            let clip_path_reference = presentation.clip_path.clone();
            let mut record = SvgVectorRecord::from_presentation(
                crate::VectorPath::new(vec![crate::VectorContour::new(points, closed)]),
                source_bounds,
                transformed_bounds,
                presentation,
                element.span,
            );
            if let Some(clip_id) = clip_path_reference.as_deref() {
                let Some(clip_path) = clip_paths.get(clip_id).cloned() else {
                    return Err(SvgImportDiagnostic::svg(
                        Some(element.span),
                        format!("SVG clip-path target '#{clip_id}' is missing or not yet defined"),
                    ));
                };
                record.clip_path = Some(clip_path);
            }
            if let Some(clip_id) = clip_definition_id {
                if clip_paths
                    .insert(clip_id.clone(), record.path.clone())
                    .is_some()
                {
                    return Err(SvgImportDiagnostic::svg(
                        Some(element.span),
                        format!("SVG clipPath '#{clip_id}' contains multiple geometric children"),
                    ));
                }
                continue;
            }
            if render_geometry {
                paths.push((element.span.start, record));
            } else if let Some(id) = svg_definition_id(attributes) {
                definitions.insert(id.to_owned(), record);
            }
        }
    }

    if !saw_root {
        return Err(SvgImportDiagnostic::svg(
            None,
            "SVG document contains no root element",
        ));
    }
    if !element_stack.is_empty() {
        return Err(SvgImportDiagnostic::svg(
            element_stack.last().map(|frame| frame.element.span),
            "SVG semantic traversal ended with an open element",
        ));
    }

    paths.sort_by_key(|(source_offset, _)| *source_offset);
    Ok(paths.into_iter().map(|(_, record)| record).collect())
}

/// Extracts SVG geometry into the shared provider-neutral path model.
///
/// This convenience API retains the historic string error surface while the
/// structured variant carries XML categories, codes, and source spans.
pub fn parse_svg_document_vector_records(
    svg: &str,
    subdivisions: usize,
    view_box: [f32; 4],
) -> Result<Vec<SvgVectorRecord>, String> {
    parse_svg_document_vector_records_with_xml_options(
        svg,
        subdivisions,
        view_box,
        XmlOptions::default(),
    )
    .map_err(|diagnostic| diagnostic.to_string())
}

/// Extracts SVG geometry while discarding SVG-specific paint metadata.
pub fn parse_svg_document_vector_paths(
    svg: &str,
    subdivisions: usize,
    view_box: [f32; 4],
) -> Result<Vec<crate::VectorPath>, String> {
    Ok(
        parse_svg_document_vector_records(svg, subdivisions, view_box)?
            .into_iter()
            .map(|record| record.path)
            .collect(),
    )
}

/// Parses SVG geometry and routes only convex single-contour paths through the
/// bounded shared fill tessellator.
///
/// This is intentionally not a general SVG fill implementation. Unsupported
/// topology is returned with the path index so callers can choose a fallback
/// or report an importer diagnostic without silently dropping geometry.
pub fn parse_svg_document_convex_fill_meshes(
    svg: &str,
    subdivisions: usize,
    view_box: [f32; 4],
) -> Result<Vec<Vec<[f32; 2]>>, String> {
    parse_svg_document_vector_paths(svg, subdivisions, view_box)?
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            crate::validate_convex_fill(&path)
                .map_err(|error| format!("SVG fill path {index} is unsupported: {error}"))?;
            crate::tessellate_convex_fill(&path)
        })
        .collect()
}
