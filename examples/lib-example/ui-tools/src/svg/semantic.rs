use xml_tools::{XmlAttribute, XmlEvent, XmlSpan};

use super::transform::{parse_svg_transform, parse_svg_transform_numbers, SvgAffine};
use super::{
    SvgColor, SvgFillRule, SvgPreserveAspectRatio, SvgStrokeLinecap, SvgStrokeLinejoin,
    SvgVectorRecord,
};

impl SvgVectorRecord {
    pub(super) fn from_presentation(
        path: crate::VectorPath,
        source_bounds: Option<([f32; 2], [f32; 2])>,
        transformed_bounds: Option<([f32; 2], [f32; 2])>,
        presentation: SvgPresentationState,
        source_span: XmlSpan,
    ) -> Self {
        Self {
            path,
            clip_path: None,
            source_bounds,
            transformed_bounds,
            fill: presentation.fill,
            stroke: presentation.stroke,
            fill_color: presentation.fill_color,
            stroke_color: presentation.stroke_color,
            fill_opacity: presentation.fill_opacity,
            stroke_opacity: presentation.stroke_opacity,
            opacity: presentation.opacity,
            fill_rule: presentation.fill_rule,
            stroke_width: presentation.stroke_width,
            stroke_linecap: presentation.stroke_linecap,
            stroke_linejoin: presentation.stroke_linejoin,
            stroke_miterlimit: presentation.stroke_miterlimit,
            stroke_dasharray: presentation.stroke_dasharray,
            stroke_dashoffset: presentation.stroke_dashoffset,
            source_span,
        }
    }
}

/// The limited inherited SVG presentation profile currently admitted by the
/// importer. This is deliberately paint intent, not a CSS implementation.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SvgPresentationState {
    pub(super) fill: bool,
    pub(super) stroke: bool,
    pub(super) fill_color: Option<SvgColor>,
    pub(super) stroke_color: Option<SvgColor>,
    pub(super) color: SvgColor,
    pub(super) fill_opacity: f32,
    pub(super) stroke_opacity: f32,
    pub(super) opacity: f32,
    pub(super) fill_rule: SvgFillRule,
    pub(super) stroke_width: f32,
    pub(super) stroke_linecap: SvgStrokeLinecap,
    pub(super) stroke_linejoin: SvgStrokeLinejoin,
    pub(super) stroke_miterlimit: f32,
    pub(super) stroke_dasharray: Option<Vec<f32>>,
    pub(super) stroke_dashoffset: f32,
    pub(super) clip_path: Option<String>,
    pub(super) transform: SvgAffine,
}

impl Default for SvgPresentationState {
    fn default() -> Self {
        Self {
            fill: true,
            stroke: false,
            fill_color: Some(SvgColor::Rgba([0.0, 0.0, 0.0, 1.0])),
            stroke_color: None,
            color: SvgColor::Rgba([0.0, 0.0, 0.0, 1.0]),
            fill_opacity: 1.0,
            stroke_opacity: 1.0,
            opacity: 1.0,
            fill_rule: SvgFillRule::NonZero,
            stroke_width: 1.0,
            stroke_linecap: SvgStrokeLinecap::Butt,
            stroke_linejoin: SvgStrokeLinejoin::Miter,
            stroke_miterlimit: 4.0,
            stroke_dasharray: None,
            stroke_dashoffset: 0.0,
            clip_path: None,
            transform: SvgAffine::IDENTITY,
        }
    }
}

impl SvgPresentationState {
    pub(super) fn inherit_and_apply(self, attributes: &[XmlAttribute]) -> Result<Self, String> {
        let mut next = self;
        if let Some(value) = svg_attribute_value(attributes, "color") {
            if value != "inherit" {
                next.color = parse_svg_color(value)?;
            }
        }
        if let Some(value) = svg_attribute_value(attributes, "fill") {
            if value != "inherit" {
                next.fill_color = parse_svg_paint(value, next.color)?;
                next.fill = next.fill_color.is_some();
            }
        }
        if let Some(value) = svg_attribute_value(attributes, "stroke") {
            if value != "inherit" {
                next.stroke_color = parse_svg_paint(value, next.color)?;
                next.stroke = next.stroke_color.is_some();
            }
        }
        if let Some(value) = svg_attribute_value(attributes, "fill-opacity") {
            next.fill_opacity = parse_svg_opacity("fill-opacity", value)?;
        }
        if let Some(value) = svg_attribute_value(attributes, "stroke-opacity") {
            next.stroke_opacity = parse_svg_opacity("stroke-opacity", value)?;
        }
        if let Some(value) = svg_attribute_value(attributes, "opacity") {
            next.opacity = parse_svg_opacity("opacity", value)?;
        }
        if let Some(value) = svg_attribute_value(attributes, "fill-rule") {
            match value {
                "inherit" => {}
                "evenodd" => next.fill_rule = SvgFillRule::EvenOdd,
                _ => next.fill_rule = SvgFillRule::NonZero,
            }
        }
        if let Some(value) = svg_attribute_value(attributes, "stroke-width") {
            let width = value.parse::<f32>().map_err(|_| {
                format!("SVG attribute 'stroke-width' contains invalid number '{value}'")
            })?;
            if !width.is_finite() || width < 0.0 {
                return Err(format!(
                    "SVG attribute 'stroke-width' must be finite and non-negative, received '{value}'"
                ));
            }
            next.stroke_width = width;
        }
        if let Some(value) = svg_attribute_value(attributes, "stroke-linecap") {
            next.stroke_linecap = match value {
                "butt" => SvgStrokeLinecap::Butt,
                "round" => SvgStrokeLinecap::Round,
                "square" => SvgStrokeLinecap::Square,
                "inherit" => next.stroke_linecap,
                _ => return Err(format!("SVG stroke-linecap value '{value}' is unsupported")),
            };
        }
        if let Some(value) = svg_attribute_value(attributes, "stroke-linejoin") {
            next.stroke_linejoin = match value {
                "miter" => SvgStrokeLinejoin::Miter,
                "round" => SvgStrokeLinejoin::Round,
                "bevel" => SvgStrokeLinejoin::Bevel,
                "inherit" => next.stroke_linejoin,
                _ => {
                    return Err(format!(
                        "SVG stroke-linejoin value '{value}' is unsupported"
                    ))
                }
            };
        }
        if let Some(value) = svg_attribute_value(attributes, "stroke-miterlimit") {
            let limit = value.parse::<f32>().map_err(|_| {
                format!("SVG attribute 'stroke-miterlimit' contains invalid number '{value}'")
            })?;
            if !limit.is_finite() || limit <= 0.0 {
                return Err(format!(
                    "SVG attribute 'stroke-miterlimit' must be finite and positive, received '{value}'"
                ));
            }
            next.stroke_miterlimit = limit;
        }
        if let Some(value) = svg_attribute_value(attributes, "stroke-dasharray") {
            if value != "inherit" {
                next.stroke_dasharray = parse_svg_dasharray(value)?;
            }
        }
        if let Some(value) = svg_attribute_value(attributes, "stroke-dashoffset") {
            let offset = value.parse::<f32>().map_err(|_| {
                format!("SVG attribute 'stroke-dashoffset' contains invalid number '{value}'")
            })?;
            if !offset.is_finite() {
                return Err(format!(
                    "SVG attribute 'stroke-dashoffset' must be finite, received '{value}'"
                ));
            }
            next.stroke_dashoffset = offset;
        }
        if let Some(value) = svg_attribute_value(attributes, "clip-path") {
            next.clip_path = parse_svg_clip_path(value)?;
        }
        if let Some(value) = svg_unqualified_attribute_value(attributes, "transform") {
            if value != "inherit" {
                next.transform = next.transform.compose(parse_svg_transform(value)?);
            }
        }
        Ok(next)
    }
}

fn parse_svg_opacity(attribute: &str, value: &str) -> Result<f32, String> {
    let opacity = value
        .parse::<f32>()
        .map_err(|_| format!("SVG attribute '{attribute}' contains invalid number '{value}'"))?;
    if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
        return Err(format!(
            "SVG attribute '{attribute}' must be finite and between 0 and 1, received '{value}'"
        ));
    }
    Ok(opacity)
}

fn parse_svg_paint(value: &str, current_color: SvgColor) -> Result<Option<SvgColor>, String> {
    if value.eq_ignore_ascii_case("none") {
        return Ok(None);
    }
    if value.eq_ignore_ascii_case("currentcolor") {
        return Ok(Some(current_color));
    }
    Ok(Some(parse_svg_color(value)?))
}

fn parse_svg_color(value: &str) -> Result<SvgColor, String> {
    let rgba = match value.to_ascii_lowercase().as_str() {
        "black" => [0.0, 0.0, 0.0, 1.0],
        "white" => [1.0, 1.0, 1.0, 1.0],
        "red" => [1.0, 0.0, 0.0, 1.0],
        "green" => [0.0, 0.5, 0.0, 1.0],
        "blue" => [0.0, 0.0, 1.0, 1.0],
        "transparent" => [0.0, 0.0, 0.0, 0.0],
        value if value.len() == 4 && value.starts_with('#') => [
            hex_component(&value[1..2])?,
            hex_component(&value[2..3])?,
            hex_component(&value[3..4])?,
            1.0,
        ],
        value if value.len() == 7 && value.starts_with('#') => [
            hex_component(&value[1..3])?,
            hex_component(&value[3..5])?,
            hex_component(&value[5..7])?,
            1.0,
        ],
        _ => {
            return Err(format!(
            "SVG solid color '{value}' is unsupported; expected a bounded named color or hex value"
        ))
        }
    };
    Ok(SvgColor::Rgba(rgba))
}

fn hex_component(value: &str) -> Result<f32, String> {
    let byte = u8::from_str_radix(value, 16)
        .map_err(|_| format!("SVG color component '{value}' is not hexadecimal"))?;
    let expanded = if value.len() == 1 {
        u16::from(byte) * 17
    } else {
        u16::from(byte)
    };
    Ok(f32::from(expanded) / 255.0)
}

fn parse_svg_dasharray(value: &str) -> Result<Option<Vec<f32>>, String> {
    if value.eq_ignore_ascii_case("none") || value.eq_ignore_ascii_case("inherit") {
        return Ok(None);
    }

    let mut values = Vec::new();
    for token in value.split(|character: char| character == ',' || character.is_ascii_whitespace())
    {
        if token.is_empty() {
            continue;
        }
        let number = token.parse::<f32>().map_err(|_| {
            format!("SVG attribute 'stroke-dasharray' contains invalid number '{token}'")
        })?;
        if !number.is_finite() || number < 0.0 {
            return Err(format!(
                "SVG attribute 'stroke-dasharray' values must be finite and non-negative, received '{token}'"
            ));
        }
        values.push(number);
    }

    values.retain(|value| *value > 0.0);
    if values.is_empty() {
        return Err("SVG stroke-dasharray must contain a positive length".into());
    }
    if values.len() % 2 == 1 {
        let repeated = values.clone();
        values.extend(repeated);
    }
    Ok(Some(values))
}

fn parse_svg_clip_path(value: &str) -> Result<Option<String>, String> {
    if value.eq_ignore_ascii_case("none") || value.eq_ignore_ascii_case("inherit") {
        return Ok(None);
    }
    let Some(id) = value
        .strip_prefix("url(#")
        .and_then(|value| value.strip_suffix(')'))
        .filter(|id| !id.trim().is_empty())
    else {
        return Err(format!(
            "SVG clip-path value '{value}' is unsupported; expected 'none' or a local url(#id)"
        ));
    };
    Ok(Some(id.trim().to_owned()))
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
    pub(super) render_geometry: bool,
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
    let style = svg_unqualified_attribute_value(attributes, "style");
    if let Some(value) = style.and_then(|style| {
        style.split(';').rev().find_map(|declaration| {
            let (property, value) = declaration.split_once(':')?;
            property
                .trim()
                .eq_ignore_ascii_case(name)
                .then(|| value.trim())
        })
    }) {
        return Some(value);
    }
    svg_unqualified_attribute_value(attributes, name)
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

pub(super) fn svg_number_attribute(
    attributes: &[XmlAttribute],
    name: &str,
) -> Result<Option<f32>, String> {
    let Some(value) = svg_attribute_value(attributes, name) else {
        return Ok(None);
    };
    let number = value
        .parse::<f32>()
        .map_err(|_| format!("SVG attribute '{name}' contains invalid number '{value}'"))?;
    if !number.is_finite() {
        return Err(format!(
            "SVG attribute '{name}' contains non-finite number '{value}'"
        ));
    }
    Ok(Some(number))
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

pub(super) fn parse_svg_root_preserve_aspect_ratio(
    attributes: &[XmlAttribute],
) -> Result<SvgPreserveAspectRatio, String> {
    let Some(value) = svg_unqualified_attribute_value(attributes, "preserveAspectRatio") else {
        return Ok(SvgPreserveAspectRatio::XMidYMidMeet);
    };
    let tokens = value.split_ascii_whitespace().collect::<Vec<_>>();
    if tokens.first() == Some(&"defer") {
        return Err("SVG preserveAspectRatio 'defer' is unsupported".into());
    }
    let value = tokens.first().copied().unwrap_or("xMidYMid");
    if tokens.get(1).is_some_and(|mode| *mode == "slice") {
        return Err("SVG preserveAspectRatio slice mode is not admitted yet".into());
    }
    if value == "none" {
        return Ok(SvgPreserveAspectRatio::None);
    }
    let aspect = match value {
        "xMinYMin" => SvgPreserveAspectRatio::XMinYMinMeet,
        "xMidYMin" => SvgPreserveAspectRatio::XMidYMinMeet,
        "xMaxYMin" => SvgPreserveAspectRatio::XMaxYMinMeet,
        "xMinYMid" => SvgPreserveAspectRatio::XMinYMidMeet,
        "xMidYMid" => SvgPreserveAspectRatio::XMidYMidMeet,
        "xMaxYMid" => SvgPreserveAspectRatio::XMaxYMidMeet,
        "xMinYMax" => SvgPreserveAspectRatio::XMinYMaxMeet,
        "xMidYMax" => SvgPreserveAspectRatio::XMidYMaxMeet,
        "xMaxYMax" => SvgPreserveAspectRatio::XMaxYMaxMeet,
        _ => {
            return Err(format!(
                "SVG preserveAspectRatio value '{value}' is unsupported"
            ))
        }
    };
    Ok(aspect)
}

pub(super) fn normalize_svg_point_with_aspect(
    point: [f32; 2],
    view_box: [f32; 4],
    aspect: SvgPreserveAspectRatio,
) -> [f32; 2] {
    let [view_x, view_y, view_width, view_height] = view_box;
    if aspect == SvgPreserveAspectRatio::None {
        return [
            (point[0] - view_x) / view_width - 0.5,
            0.5 - (point[1] - view_y) / view_height,
        ];
    }
    let scale = (1.0 / view_width).min(1.0 / view_height);
    let width = view_width * scale;
    let height = view_height * scale;
    let x_offset = match aspect {
        SvgPreserveAspectRatio::XMinYMinMeet
        | SvgPreserveAspectRatio::XMinYMidMeet
        | SvgPreserveAspectRatio::XMinYMaxMeet => -0.5,
        SvgPreserveAspectRatio::XMidYMinMeet
        | SvgPreserveAspectRatio::XMidYMidMeet
        | SvgPreserveAspectRatio::XMidYMaxMeet => -width * 0.5,
        SvgPreserveAspectRatio::XMaxYMinMeet
        | SvgPreserveAspectRatio::XMaxYMidMeet
        | SvgPreserveAspectRatio::XMaxYMaxMeet => 0.5 - width,
        SvgPreserveAspectRatio::None => unreachable!(),
    };
    let y_top = match aspect {
        SvgPreserveAspectRatio::XMinYMinMeet
        | SvgPreserveAspectRatio::XMidYMinMeet
        | SvgPreserveAspectRatio::XMaxYMinMeet => 0.5,
        SvgPreserveAspectRatio::XMinYMidMeet
        | SvgPreserveAspectRatio::XMidYMidMeet
        | SvgPreserveAspectRatio::XMaxYMidMeet => height * 0.5,
        SvgPreserveAspectRatio::XMinYMaxMeet
        | SvgPreserveAspectRatio::XMidYMaxMeet
        | SvgPreserveAspectRatio::XMaxYMaxMeet => -0.5 + height,
        SvgPreserveAspectRatio::None => unreachable!(),
    };
    [
        x_offset + (point[0] - view_x) * scale,
        y_top - (point[1] - view_y) * scale,
    ]
}
