use xml_tools::{parse_xml_events, XmlEvent, XmlOptions, XmlSourceId};

use super::path::flatten_path;
use super::primitives::{parse_svg_point_numbers, svg_rectangle};
use super::semantic::{
    is_svg_geometry_element, is_svg_name, is_unadmitted_svg_feature, normalize_svg_point,
    parse_svg_root_view_box, svg_attribute_value, svg_number_attribute, svg_semantic_events,
    validate_svg_view_box, SvgSemanticEvent, SvgSemanticFrame,
};
use super::{parse_path, SvgImportDiagnostic, SvgVectorRecord, SvgViewportSource};

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
    let mut paths = Vec::<(usize, SvgVectorRecord)>::new();
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
                    }
                }
                let inherited_presentation = element_stack
                    .last()
                    .map(|frame| frame.presentation)
                    .unwrap_or_default();
                let presentation = if is_svg_name(&element.name) {
                    inherited_presentation
                        .inherit_and_apply(&element.attributes)
                        .map_err(|message| SvgImportDiagnostic::svg(Some(element.span), message))?
                } else {
                    inherited_presentation
                };
                let is_geometry = is_svg_geometry_element(&element);
                if !is_geometry && is_unadmitted_svg_feature(&element) {
                    return Err(SvgImportDiagnostic::svg(
                        Some(element.span),
                        format!(
                            "SVG element '{}' is outside the admitted importer profile",
                            element.name.local_name
                        ),
                    ));
                }
                element_stack.push(SvgSemanticFrame {
                    element: element.clone(),
                    presentation,
                });
                if !is_geometry {
                    continue;
                }
                (element, presentation)
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
        let (element, presentation) = element;
        let view_box = resolved_view_box.expect("an admitted SVG root resolves a viewBox");
        let attributes = &element.attributes;
        let points = match element.name.local_name.as_str() {
            "path" => {
                let Some(data) = svg_attribute_value(attributes, "d") else {
                    continue;
                };
                let commands = parse_path(data)
                    .map_err(|message| SvgImportDiagnostic::svg(Some(element.span), message))?;
                let contours = flatten_path(&commands, subdivisions)
                    .into_iter()
                    .filter(|points| points.len() > 1)
                    .map(|points| {
                        let mut points = points
                            .into_iter()
                            .map(|point| {
                                normalize_svg_point(presentation.transform.apply(point), view_box)
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
                    paths.push((
                        element.span.start,
                        SvgVectorRecord::from_presentation(
                            crate::VectorPath::new(contours),
                            presentation,
                            element.span,
                        ),
                    ));
                }
                continue;
            }
            "circle" => {
                let Some(radius) = svg_number_attribute(attributes, "r") else {
                    continue;
                };
                // SVG defaults omitted circle centers to the origin.
                let cx = svg_number_attribute(attributes, "cx").unwrap_or(0.0);
                let cy = svg_number_attribute(attributes, "cy").unwrap_or(0.0);
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
                                normalize_svg_point(
                                    presentation.transform.apply([
                                        cx + radius * angle.cos(),
                                        cy + radius * angle.sin(),
                                    ]),
                                    view_box,
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                }
            }
            "ellipse" => {
                let (Some(rx), Some(ry)) = (
                    svg_number_attribute(attributes, "rx"),
                    svg_number_attribute(attributes, "ry"),
                ) else {
                    continue;
                };
                // SVG defaults omitted ellipse centers to the origin.
                let cx = svg_number_attribute(attributes, "cx").unwrap_or(0.0);
                let cy = svg_number_attribute(attributes, "cy").unwrap_or(0.0);
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
                                normalize_svg_point(
                                    presentation
                                        .transform
                                        .apply([cx + rx * angle.cos(), cy + ry * angle.sin()]),
                                    view_box,
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                }
            }
            "line" => {
                let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                    svg_number_attribute(attributes, "x1"),
                    svg_number_attribute(attributes, "y1"),
                    svg_number_attribute(attributes, "x2"),
                    svg_number_attribute(attributes, "y2"),
                ) else {
                    continue;
                };
                Some(vec![
                    normalize_svg_point(presentation.transform.apply([x1, y1]), view_box),
                    normalize_svg_point(presentation.transform.apply([x2, y2]), view_box),
                ])
            }
            "polyline" | "polygon" => {
                let Some(values) = svg_attribute_value(attributes, "points") else {
                    continue;
                };
                let numbers = parse_svg_point_numbers(values, &element.name.local_name)
                    .map_err(|message| SvgImportDiagnostic::svg(Some(element.span), message))?;
                let mut points = numbers
                    .chunks_exact(2)
                    .map(|pair| {
                        normalize_svg_point(
                            presentation.transform.apply([pair[0], pair[1]]),
                            view_box,
                        )
                    })
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
                    svg_number_attribute(attributes, "x"),
                    svg_number_attribute(attributes, "y"),
                    svg_number_attribute(attributes, "width"),
                    svg_number_attribute(attributes, "height"),
                ) else {
                    continue;
                };
                if width < 0.0 || height < 0.0 {
                    return Err(SvgImportDiagnostic::svg(
                        Some(element.span),
                        "SVG rectangle width and height must not be negative",
                    ));
                }
                let raw_rx = svg_number_attribute(attributes, "rx");
                let raw_ry = svg_number_attribute(attributes, "ry");
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
                        .map(|point| {
                            normalize_svg_point(presentation.transform.apply(point), view_box)
                        })
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
            let points = if closed {
                points[..points.len() - 1].to_vec()
            } else {
                points
            };
            paths.push((
                element.span.start,
                SvgVectorRecord::from_presentation(
                    crate::VectorPath::new(vec![crate::VectorContour::new(points, closed)]),
                    presentation,
                    element.span,
                ),
            ));
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
