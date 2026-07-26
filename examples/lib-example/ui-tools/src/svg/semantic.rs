use xml_tools::{XmlAttribute, XmlEvent, XmlSpan};

use super::transform::{parse_svg_transform, parse_svg_transform_numbers, SvgAffine};
use super::{SvgFillRule, SvgVectorRecord};

impl SvgVectorRecord {
    pub(super) fn from_presentation(
        path: crate::VectorPath,
        presentation: SvgPresentationState,
        source_span: XmlSpan,
    ) -> Self {
        Self {
            path,
            fill: presentation.fill,
            stroke: presentation.stroke,
            fill_rule: presentation.fill_rule,
            source_span,
        }
    }
}

/// The limited inherited SVG presentation profile currently admitted by the
/// importer. This is deliberately paint intent, not a CSS implementation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SvgPresentationState {
    pub(super) fill: bool,
    pub(super) stroke: bool,
    pub(super) fill_rule: SvgFillRule,
    pub(super) transform: SvgAffine,
}

impl Default for SvgPresentationState {
    fn default() -> Self {
        Self {
            fill: true,
            stroke: false,
            fill_rule: SvgFillRule::NonZero,
            transform: SvgAffine::IDENTITY,
        }
    }
}

impl SvgPresentationState {
    pub(super) fn inherit_and_apply(self, attributes: &[XmlAttribute]) -> Result<Self, String> {
        let mut next = self;
        if let Some(value) = svg_attribute_value(attributes, "fill") {
            if value != "inherit" {
                next.fill = value != "none";
            }
        }
        if let Some(value) = svg_attribute_value(attributes, "stroke") {
            if value != "inherit" {
                next.stroke = value != "none";
            }
        }
        if let Some(value) = svg_attribute_value(attributes, "fill-rule") {
            match value {
                "inherit" => {}
                "evenodd" => next.fill_rule = SvgFillRule::EvenOdd,
                _ => next.fill_rule = SvgFillRule::NonZero,
            }
        }
        if let Some(value) = svg_unqualified_attribute_value(attributes, "transform") {
            if value != "inherit" {
                next.transform = next.transform.compose(parse_svg_transform(value)?);
            }
        }
        Ok(next)
    }
}

#[derive(Clone, Debug)]
pub(super) struct SvgElement {
    pub(super) name: xml_tools::ExpandedName,
    pub(super) attributes: Vec<XmlAttribute>,
    pub(super) span: XmlSpan,
}

#[derive(Clone, Debug)]
pub(super) struct SvgSemanticFrame {
    pub(super) element: SvgElement,
    pub(super) presentation: SvgPresentationState,
}

#[derive(Clone, Debug)]
pub(super) enum SvgSemanticEvent {
    Start(SvgElement),
    End {
        name: xml_tools::ExpandedName,
        span: XmlSpan,
    },
}

const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

pub(super) fn svg_semantic_events(events: &[XmlEvent]) -> Vec<SvgSemanticEvent> {
    let mut semantic_events = Vec::new();
    for event in events {
        match event {
            XmlEvent::StartElement {
                name,
                attributes,
                span,
                ..
            } => semantic_events.push(SvgSemanticEvent::Start(SvgElement {
                name: name.clone(),
                attributes: attributes.clone(),
                span: *span,
            })),
            XmlEvent::EndElement { name, span, .. } => {
                semantic_events.push(SvgSemanticEvent::End {
                    name: name.clone(),
                    span: *span,
                })
            }
            XmlEvent::Text { .. }
            | XmlEvent::Comment { .. }
            | XmlEvent::ProcessingInstruction { .. } => {}
        }
    }
    semantic_events
}

pub(super) fn is_svg_name(name: &xml_tools::ExpandedName) -> bool {
    name.namespace_uri
        .as_deref()
        .is_none_or(|namespace| namespace == SVG_NAMESPACE)
}

pub(super) fn is_svg_geometry_element(element: &SvgElement) -> bool {
    is_svg_name(&element.name)
        && matches!(
            element.name.local_name.as_str(),
            "path" | "circle" | "ellipse" | "line" | "polyline" | "polygon" | "rect"
        )
}

/// Features with established SVG meaning that need an explicit ownership and
/// corpus admission decision before this importer may claim to support them.
pub(super) fn is_unadmitted_svg_feature(element: &SvgElement) -> bool {
    is_svg_name(&element.name)
        && matches!(
            element.name.local_name.as_str(),
            "text"
                | "textPath"
                | "tspan"
                | "defs"
                | "use"
                | "clipPath"
                | "mask"
                | "linearGradient"
                | "radialGradient"
                | "pattern"
                | "filter"
                | "image"
                | "animate"
                | "animateMotion"
                | "animateTransform"
                | "script"
        )
}

pub(super) fn svg_attribute_value<'a>(
    attributes: &'a [XmlAttribute],
    name: &str,
) -> Option<&'a str> {
    if let Some(value) = svg_unqualified_attribute_value(attributes, name) {
        return Some(value);
    }
    let style = svg_unqualified_attribute_value(attributes, "style")?;
    style.split(';').find_map(|declaration| {
        let (property, value) = declaration.split_once(':')?;
        property
            .trim()
            .eq_ignore_ascii_case(name)
            .then(|| value.trim())
    })
}

pub(super) fn svg_unqualified_attribute_value<'a>(
    attributes: &'a [XmlAttribute],
    name: &str,
) -> Option<&'a str> {
    attributes.iter().find_map(|attribute| {
        (attribute.name.namespace_uri.is_none() && attribute.name.local_name == name)
            .then(|| attribute.value.trim())
    })
}

pub(super) fn svg_number_attribute(attributes: &[XmlAttribute], name: &str) -> Option<f32> {
    svg_attribute_value(attributes, name)?.parse().ok()
}

pub(super) fn validate_svg_view_box(view_box: [f32; 4]) -> Result<[f32; 4], String> {
    let [_, _, width, height] = view_box;
    if !view_box.iter().all(|value| value.is_finite()) || width <= 0.0 || height <= 0.0 {
        return Err("SVG viewBox must contain finite values with positive dimensions".into());
    }
    Ok(view_box)
}

pub(super) fn parse_svg_root_view_box(attributes: &[XmlAttribute]) -> Result<[f32; 4], String> {
    let Some(value) = svg_unqualified_attribute_value(attributes, "viewBox") else {
        return Err("SVG root requires a viewBox for the DocumentViewBox policy".into());
    };
    let values = parse_svg_transform_numbers(value, "viewBox")?;
    let view_box = match values.as_slice() {
        [x, y, width, height] => [*x, *y, *width, *height],
        _ => return Err("SVG root viewBox requires exactly four numbers".into()),
    };
    validate_svg_view_box(view_box)
}

pub(super) fn normalize_svg_point(point: [f32; 2], view_box: [f32; 4]) -> [f32; 2] {
    let [view_x, view_y, view_width, view_height] = view_box;
    [
        (point[0] - view_x) / view_width - 0.5,
        0.5 - (point[1] - view_y) / view_height,
    ]
}
