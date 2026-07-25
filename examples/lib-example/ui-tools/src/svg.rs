use std::{error::Error, fmt};

use xml_tools::{
    parse_xml_events, XmlAttribute, XmlDiagnostic, XmlEvent, XmlOptions, XmlSourceId, XmlSpan,
};

/// The SVG pipeline boundary that produced an import diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgImportStage {
    Xml,
    Svg,
}

/// Selects the coordinate bounds used to normalize SVG user-space geometry.
///
/// This importer intentionally resolves only `viewBox` coordinates. Physical
/// viewport sizing (`width`, `height`, and `preserveAspectRatio`) remains an
/// embedding/rendering decision outside this initial profile.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SvgViewportSource {
    /// Keep the established embedding path: the caller owns normalization
    /// bounds and the document's root `viewBox` is not interpreted.
    Caller([f32; 4]),
    /// Read and validate the root SVG element's `viewBox` attribute.
    DocumentViewBox,
}

/// Structured SVG import failure that preserves XML diagnostics instead of
/// reducing them to formatted strings at the importer boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SvgImportDiagnostic {
    pub stage: SvgImportStage,
    pub message: String,
    pub span: Option<XmlSpan>,
    pub related_span: Option<XmlSpan>,
    pub can_continue: bool,
    pub xml: Option<XmlDiagnostic>,
}

impl SvgImportDiagnostic {
    fn svg(span: Option<XmlSpan>, message: impl Into<String>) -> Self {
        Self {
            stage: SvgImportStage::Svg,
            message: message.into(),
            span,
            related_span: None,
            can_continue: false,
            xml: None,
        }
    }
}

impl From<XmlDiagnostic> for SvgImportDiagnostic {
    fn from(diagnostic: XmlDiagnostic) -> Self {
        Self {
            stage: SvgImportStage::Xml,
            message: diagnostic.message.clone(),
            span: diagnostic.span,
            related_span: diagnostic.related_span,
            can_continue: diagnostic.can_continue,
            xml: Some(diagnostic),
        }
    }
}

impl fmt::Display for SvgImportDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.xml {
            Some(diagnostic) => write!(
                formatter,
                "SVG XML syntax error {:?}/{:?} at {:?}: {}",
                diagnostic.category, diagnostic.code, self.span, self.message
            ),
            None => write!(
                formatter,
                "SVG semantic error at {:?}: {}",
                self.span, self.message
            ),
        }
    }
}

impl Error for SvgImportDiagnostic {}

#[derive(Debug, Clone, PartialEq)]
pub enum SvgToken {
    Command(char),
    Number(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SvgPathCommand {
    MoveTo { relative: bool, x: f32, y: f32 },
    LineTo { relative: bool, x: f32, y: f32 },
    HorizontalTo { relative: bool, x: f32 },
    VerticalTo { relative: bool, y: f32 },
    CubicTo { relative: bool, values: [f32; 6] },
    SmoothCubicTo { relative: bool, values: [f32; 4] },
    QuadraticTo { relative: bool, values: [f32; 4] },
    SmoothQuadraticTo { relative: bool, values: [f32; 2] },
    ArcTo { relative: bool, values: [f32; 7] },
    ClosePath,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgFillRule {
    NonZero,
    EvenOdd,
}

/// SVG element geometry plus the small amount of paint intent needed by an
/// importer. This stays above `VectorPath`: fill/stroke are SVG semantics, not
/// responsibilities of the provider-neutral geometry contract.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgVectorRecord {
    pub path: crate::VectorPath,
    pub fill: bool,
    pub stroke: bool,
    pub fill_rule: SvgFillRule,
    pub source_span: XmlSpan,
}

impl SvgVectorRecord {
    fn from_presentation(
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
struct SvgPresentationState {
    fill: bool,
    stroke: bool,
    fill_rule: SvgFillRule,
    transform: SvgAffine,
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
    fn inherit_and_apply(self, attributes: &[XmlAttribute]) -> Result<Self, String> {
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

/// A compact SVG affine transform in the standard `[a b c d e f]` form.
/// It remains private to SVG lowering: `VectorPath` contains only final,
/// provider-neutral coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
struct SvgAffine {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl SvgAffine {
    const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    fn apply(self, point: [f32; 2]) -> [f32; 2] {
        [
            self.a * point[0] + self.c * point[1] + self.e,
            self.b * point[0] + self.d * point[1] + self.f,
        ]
    }

    /// Extends this parent/list transform with a local SVG transform.
    /// With column vectors, the resulting transform applies `local` first and
    /// then this transform, matching nested SVG coordinate systems.
    fn compose(self, local: Self) -> Self {
        Self {
            a: self.a * local.a + self.c * local.b,
            b: self.b * local.a + self.d * local.b,
            c: self.a * local.c + self.c * local.d,
            d: self.b * local.c + self.d * local.d,
            e: self.a * local.e + self.c * local.f + self.e,
            f: self.b * local.e + self.d * local.f + self.f,
        }
    }
}

#[derive(Clone, Debug)]
struct SvgElement {
    name: xml_tools::ExpandedName,
    attributes: Vec<XmlAttribute>,
    span: XmlSpan,
}

#[derive(Clone, Debug)]
struct SvgSemanticFrame {
    element: SvgElement,
    presentation: SvgPresentationState,
}

#[derive(Clone, Debug)]
enum SvgSemanticEvent {
    Start(SvgElement),
    End {
        name: xml_tools::ExpandedName,
        span: XmlSpan,
    },
}

const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

fn svg_semantic_events(events: &[XmlEvent]) -> Vec<SvgSemanticEvent> {
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

fn is_svg_name(name: &xml_tools::ExpandedName) -> bool {
    name.namespace_uri
        .as_deref()
        .is_none_or(|namespace| namespace == SVG_NAMESPACE)
}

fn is_svg_geometry_element(element: &SvgElement) -> bool {
    is_svg_name(&element.name)
        && matches!(
            element.name.local_name.as_str(),
            "path" | "circle" | "ellipse" | "line" | "polyline" | "polygon" | "rect"
        )
}

/// Features with established SVG meaning that need an explicit ownership and
/// corpus admission decision before this importer may claim to support them.
fn is_unadmitted_svg_feature(element: &SvgElement) -> bool {
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

fn svg_attribute_value<'a>(attributes: &'a [XmlAttribute], name: &str) -> Option<&'a str> {
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

fn svg_unqualified_attribute_value<'a>(
    attributes: &'a [XmlAttribute],
    name: &str,
) -> Option<&'a str> {
    attributes.iter().find_map(|attribute| {
        (attribute.name.namespace_uri.is_none() && attribute.name.local_name == name)
            .then(|| attribute.value.trim())
    })
}

fn parse_svg_transform(value: &str) -> Result<SvgAffine, String> {
    let mut remainder = value.trim();
    let mut transform = SvgAffine::IDENTITY;

    while !remainder.is_empty() {
        let Some(open) = remainder.find('(') else {
            return Err(format!("SVG transform is missing '(' in '{value}'"));
        };
        let name = remainder[..open].trim();
        if name.is_empty()
            || !name
                .chars()
                .all(|character| character.is_ascii_alphabetic())
        {
            return Err(format!(
                "SVG transform has an invalid function name in '{value}'"
            ));
        }
        let arguments_start = open + 1;
        let Some(close_relative) = remainder[arguments_start..].find(')') else {
            return Err(format!("SVG transform '{name}' is missing ')'"));
        };
        let arguments_end = arguments_start + close_relative;
        let values = parse_svg_transform_numbers(&remainder[arguments_start..arguments_end], name)?;
        let function = match name {
            "matrix" => match values.as_slice() {
                [a, b, c, d, e, f] => SvgAffine {
                    a: *a,
                    b: *b,
                    c: *c,
                    d: *d,
                    e: *e,
                    f: *f,
                },
                _ => return Err("SVG matrix transform requires six numbers".into()),
            },
            "translate" => match values.as_slice() {
                [x] => SvgAffine {
                    e: *x,
                    ..SvgAffine::IDENTITY
                },
                [x, y] => SvgAffine {
                    e: *x,
                    f: *y,
                    ..SvgAffine::IDENTITY
                },
                _ => return Err("SVG translate transform requires one or two numbers".into()),
            },
            "scale" => match values.as_slice() {
                [value] => SvgAffine {
                    a: *value,
                    d: *value,
                    ..SvgAffine::IDENTITY
                },
                [x, y] => SvgAffine {
                    a: *x,
                    d: *y,
                    ..SvgAffine::IDENTITY
                },
                _ => return Err("SVG scale transform requires one or two numbers".into()),
            },
            "rotate" => match values.as_slice() {
                [degrees] => svg_rotation(*degrees),
                [degrees, center_x, center_y] => SvgAffine {
                    e: *center_x,
                    f: *center_y,
                    ..SvgAffine::IDENTITY
                }
                .compose(svg_rotation(*degrees))
                .compose(SvgAffine {
                    e: -*center_x,
                    f: -*center_y,
                    ..SvgAffine::IDENTITY
                }),
                _ => return Err("SVG rotate transform requires one or three numbers".into()),
            },
            "skewX" | "skewY" => {
                return Err(format!(
                    "SVG transform '{name}' is outside the current importer profile"
                ))
            }
            _ => return Err(format!("SVG transform '{name}' is unsupported")),
        };
        transform = transform.compose(function);
        remainder = remainder[arguments_end + 1..].trim_start();
    }

    Ok(transform)
}

fn parse_svg_transform_numbers(value: &str, function: &str) -> Result<Vec<f32>, String> {
    if !value.chars().all(|character| {
        character.is_ascii_digit()
            || character.is_ascii_whitespace()
            || matches!(character, ',' | '.' | '+' | '-' | 'e' | 'E')
    }) {
        return Err(format!(
            "SVG transform '{function}' contains unsupported arguments '{value}'"
        ));
    }
    let values = tokenize_path(value)
        .into_iter()
        .map(|token| match token {
            SvgToken::Number(value) if value.is_finite() => Ok(value),
            SvgToken::Number(_) => Err(format!(
                "SVG transform '{function}' contains a non-finite number"
            )),
            SvgToken::Command(_) => Err(format!(
                "SVG transform '{function}' contains an invalid numeric argument"
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.is_empty() && !value.trim().is_empty() {
        return Err(format!(
            "SVG transform '{function}' contains invalid numbers"
        ));
    }
    Ok(values)
}

fn svg_rotation(degrees: f32) -> SvgAffine {
    let (sine, cosine) = degrees.to_radians().sin_cos();
    SvgAffine {
        a: cosine,
        b: sine,
        c: -sine,
        d: cosine,
        e: 0.0,
        f: 0.0,
    }
}

fn svg_number_attribute(attributes: &[XmlAttribute], name: &str) -> Option<f32> {
    svg_attribute_value(attributes, name)?.parse().ok()
}

fn validate_svg_view_box(view_box: [f32; 4]) -> Result<[f32; 4], String> {
    let [_, _, width, height] = view_box;
    if !view_box.iter().all(|value| value.is_finite()) || width <= 0.0 || height <= 0.0 {
        return Err("SVG viewBox must contain finite values with positive dimensions".into());
    }
    Ok(view_box)
}

fn parse_svg_root_view_box(attributes: &[XmlAttribute]) -> Result<[f32; 4], String> {
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

fn normalize_svg_point(point: [f32; 2], view_box: [f32; 4]) -> [f32; 2] {
    let [view_x, view_y, view_width, view_height] = view_box;
    [
        (point[0] - view_x) / view_width - 0.5,
        0.5 - (point[1] - view_y) / view_height,
    ]
}

pub fn parse_path(data: &str) -> Result<Vec<SvgPathCommand>, String> {
    let tokens = tokenize_path(data);
    let mut index = 0;
    let mut command = None;
    let mut result = Vec::new();
    while index < tokens.len() {
        if let SvgToken::Command(value) = tokens[index] {
            command = Some(value);
            index += 1;
        }
        let Some(active) = command else {
            return Err("path data begins with coordinates".into());
        };
        if active.eq_ignore_ascii_case(&'Z') {
            result.push(SvgPathCommand::ClosePath);
            command = None;
            continue;
        }
        let arity = match active.to_ascii_uppercase() {
            'M' | 'L' => 2,
            'H' | 'V' => 1,
            'C' => 6,
            'S' | 'Q' => 4,
            'T' => 2,
            'A' => 7,
            other => return Err(format!("unsupported SVG command: {other}")),
        };
        let values = (0..arity)
            .map(|_| match tokens.get(index) {
                Some(SvgToken::Number(value)) => {
                    index += 1;
                    if value.is_finite() {
                        Ok(*value)
                    } else {
                        Err(format!(
                            "non-finite {active} coordinate at token {}",
                            index - 1
                        ))
                    }
                }
                _ => Err(format!("incomplete {active} command at token {index}")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let relative = active.is_ascii_lowercase();
        let upper = active.to_ascii_uppercase();
        if upper == 'A'
            && ((values[3] != 0.0 && values[3] != 1.0) || (values[4] != 0.0 && values[4] != 1.0))
        {
            return Err(format!("invalid {active} arc flag at token {index}"));
        }
        let command_value = match upper {
            'M' => SvgPathCommand::MoveTo {
                relative,
                x: values[0],
                y: values[1],
            },
            'L' => SvgPathCommand::LineTo {
                relative,
                x: values[0],
                y: values[1],
            },
            'H' => SvgPathCommand::HorizontalTo {
                relative,
                x: values[0],
            },
            'V' => SvgPathCommand::VerticalTo {
                relative,
                y: values[0],
            },
            'C' => SvgPathCommand::CubicTo {
                relative,
                values: values.try_into().unwrap(),
            },
            'S' => SvgPathCommand::SmoothCubicTo {
                relative,
                values: values.try_into().unwrap(),
            },
            'Q' => SvgPathCommand::QuadraticTo {
                relative,
                values: values.try_into().unwrap(),
            },
            'T' => SvgPathCommand::SmoothQuadraticTo {
                relative,
                values: values.try_into().unwrap(),
            },
            'A' => SvgPathCommand::ArcTo {
                relative,
                values: values.try_into().unwrap(),
            },
            _ => unreachable!(),
        };
        result.push(command_value);
        if upper == 'M' {
            command = Some(if relative { 'l' } else { 'L' });
        }
    }
    Ok(result)
}

fn flatten_path(commands: &[SvgPathCommand], subdivisions: usize) -> Vec<Vec<[f32; 2]>> {
    let steps = subdivisions.max(2);
    let mut paths = Vec::new();
    let mut points = Vec::new();
    let mut current = [0.0, 0.0];
    let mut start = [0.0, 0.0];
    let mut last_cubic_control = None;
    let mut last_quadratic_control = None;
    for command in commands {
        match command {
            SvgPathCommand::MoveTo { relative, x, y } => {
                if points.len() > 1 {
                    paths.push(std::mem::take(&mut points));
                }
                current = point(*relative, current, *x, *y);
                start = current;
                points.push(current);
                last_cubic_control = None;
                last_quadratic_control = None;
            }
            SvgPathCommand::LineTo { relative, x, y } => {
                current = point(*relative, current, *x, *y);
                points.push(current);
                last_cubic_control = None;
                last_quadratic_control = None;
            }
            SvgPathCommand::HorizontalTo { relative, x } => {
                current = [if *relative { current[0] + x } else { *x }, current[1]];
                points.push(current);
                last_cubic_control = None;
                last_quadratic_control = None;
            }
            SvgPathCommand::VerticalTo { relative, y } => {
                current = [current[0], if *relative { current[1] + y } else { *y }];
                points.push(current);
                last_cubic_control = None;
                last_quadratic_control = None;
            }
            SvgPathCommand::CubicTo { relative, values } => {
                let p0 = current;
                let p1 = point(*relative, p0, values[0], values[1]);
                let p2 = point(*relative, p0, values[2], values[3]);
                let p3 = point(*relative, p0, values[4], values[5]);
                for index in 1..=steps {
                    let t = index as f32 / steps as f32;
                    points.push(cubic(p0, p1, p2, p3, t));
                }
                current = p3;
                last_cubic_control = Some(p2);
                last_quadratic_control = None;
            }
            SvgPathCommand::QuadraticTo { relative, values } => {
                let p0 = current;
                let p1 = point(*relative, p0, values[0], values[1]);
                let p2 = point(*relative, p0, values[2], values[3]);
                for index in 1..=steps {
                    let t = index as f32 / steps as f32;
                    points.push(quadratic(p0, p1, p2, t));
                }
                current = p2;
                last_quadratic_control = Some(p1);
                last_cubic_control = None;
            }
            SvgPathCommand::ClosePath => {
                if current != start {
                    points.push(start);
                }
                current = start;
                last_cubic_control = None;
                last_quadratic_control = None;
            }
            SvgPathCommand::SmoothCubicTo { relative, values } => {
                let p0 = current;
                let p1 = last_cubic_control
                    .map(|control| [2.0 * p0[0] - control[0], 2.0 * p0[1] - control[1]])
                    .unwrap_or(p0);
                let p2 = point(*relative, p0, values[0], values[1]);
                let p3 = point(*relative, p0, values[2], values[3]);
                for index in 1..=steps {
                    let t = index as f32 / steps as f32;
                    points.push(cubic(p0, p1, p2, p3, t));
                }
                current = p3;
                last_cubic_control = Some(p2);
                last_quadratic_control = None;
            }
            SvgPathCommand::SmoothQuadraticTo { relative, values } => {
                let p0 = current;
                let p1 = last_quadratic_control
                    .map(|control| [2.0 * p0[0] - control[0], 2.0 * p0[1] - control[1]])
                    .unwrap_or(p0);
                let p2 = point(*relative, p0, values[0], values[1]);
                for index in 1..=steps {
                    let t = index as f32 / steps as f32;
                    points.push(quadratic(p0, p1, p2, t));
                }
                current = p2;
                last_quadratic_control = Some(p1);
                last_cubic_control = None;
            }
            SvgPathCommand::ArcTo { relative, values } => {
                let end = point(*relative, current, values[5], values[6]);
                let arc = arc_points(
                    current,
                    end,
                    values[0],
                    values[1],
                    values[2],
                    values[3] != 0.0,
                    values[4] != 0.0,
                    steps,
                );
                points.extend(arc.into_iter().skip(1));
                current = end;
                last_cubic_control = None;
                last_quadratic_control = None;
            }
        }
    }
    if points.len() > 1 {
        paths.push(points);
    }
    paths
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
                                let angle = index as f32
                                    * std::f32::consts::TAU
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
                                let angle = index as f32
                                    * std::f32::consts::TAU
                                    / subdivisions.max(16) as f32;
                                normalize_svg_point(
                                    presentation.transform.apply([
                                        cx + rx * angle.cos(),
                                        cy + ry * angle.sin(),
                                    ]),
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

fn parse_svg_point_numbers(values: &str, element: &str) -> Result<Vec<f32>, String> {
    let numbers = values
        .split(|character: char| character == ',' || character.is_ascii_whitespace())
        .filter(|value| !value.is_empty())
        .map(|value| {
            let number = value.parse::<f32>().map_err(|_| {
                format!("SVG {element} points attribute contains invalid number '{value}'")
            })?;
            if !number.is_finite() {
                return Err(format!(
                    "SVG {element} points attribute contains non-finite number '{value}'"
                ));
            }
            Ok(number)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(numbers)
}

fn svg_rectangle(x: f32, y: f32, width: f32, height: f32, rx: f32, ry: f32) -> Vec<[f32; 2]> {
    if rx <= f32::EPSILON || ry <= f32::EPSILON {
        return vec![
            [x, y],
            [x + width, y],
            [x + width, y + height],
            [x, y + height],
            [x, y],
        ];
    }
    let mut points = Vec::with_capacity(20);
    for (center_x, center_y, start) in [
        (x + rx, y + ry, std::f32::consts::PI),
        (x + width - rx, y + ry, -std::f32::consts::FRAC_PI_2),
        (x + width - rx, y + height - ry, 0.0),
        (x + rx, y + height - ry, std::f32::consts::FRAC_PI_2),
    ] {
        for step in 0..=4 {
            let angle = start + step as f32 * std::f32::consts::FRAC_PI_2 / 4.0;
            points.push([center_x + rx * angle.cos(), center_y + ry * angle.sin()]);
        }
    }
    if let Some(first) = points.first().copied() {
        points.push(first);
    }
    points
}

/// Compatibility adapter from the legacy flattened SVG representation to the
/// provider-neutral vector contour stroke tessellator.
#[cfg(test)]
fn stroke_paths(paths: &[Vec<[f32; 2]>], width: f32) -> Vec<[f32; 3]> {
    paths
        .iter()
        .flat_map(|points| {
            let closed = points.len() > 1 && points.first() == points.last();
            let points = if closed && points.len() > 1 {
                points[..points.len() - 1].to_vec()
            } else {
                points.clone()
            };
            crate::tessellate_stroke(&crate::VectorContour::new(points, closed), width)
        })
        .collect()
}

fn point(relative: bool, current: [f32; 2], x: f32, y: f32) -> [f32; 2] {
    if relative {
        [current[0] + x, current[1] + y]
    } else {
        [x, y]
    }
}

fn cubic(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2], t: f32) -> [f32; 2] {
    let u = 1.0 - t;
    [
        u.powi(3) * a[0]
            + 3.0 * u.powi(2) * t * b[0]
            + 3.0 * u * t.powi(2) * c[0]
            + t.powi(3) * d[0],
        u.powi(3) * a[1]
            + 3.0 * u.powi(2) * t * b[1]
            + 3.0 * u * t.powi(2) * c[1]
            + t.powi(3) * d[1],
    ]
}

fn quadratic(a: [f32; 2], b: [f32; 2], c: [f32; 2], t: f32) -> [f32; 2] {
    let u = 1.0 - t;
    [
        u * u * a[0] + 2.0 * u * t * b[0] + t * t * c[0],
        u * u * a[1] + 2.0 * u * t * b[1] + t * t * c[1],
    ]
}

fn arc_points(
    start: [f32; 2],
    end: [f32; 2],
    rx: f32,
    ry: f32,
    rotation: f32,
    large_arc: bool,
    sweep: bool,
    steps: usize,
) -> Vec<[f32; 2]> {
    if start == end || rx == 0.0 || ry == 0.0 {
        return vec![start, end];
    }
    let phi = rotation.to_radians();
    let (sin_phi, cos_phi) = phi.sin_cos();
    let mut rx = rx.abs();
    let mut ry = ry.abs();
    let dx = (start[0] - end[0]) * 0.5;
    let dy = (start[1] - end[1]) * 0.5;
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;
    let radii_scale = (x1p * x1p / (rx * rx) + y1p * y1p / (ry * ry))
        .sqrt()
        .max(1.0);
    rx *= radii_scale;
    ry *= radii_scale;
    let numerator = (rx * rx * ry * ry - rx * rx * y1p * y1p - ry * ry * x1p * x1p).max(0.0);
    let denominator = rx * rx * y1p * y1p + ry * ry * x1p * x1p;
    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let coefficient = sign * (numerator / denominator.max(f32::EPSILON)).sqrt();
    let cxp = coefficient * (rx * y1p / ry);
    let cyp = coefficient * (-ry * x1p / rx);
    let center = [
        cos_phi * cxp - sin_phi * cyp + (start[0] + end[0]) * 0.5,
        sin_phi * cxp + cos_phi * cyp + (start[1] + end[1]) * 0.5,
    ];
    let vector = |x: f32, y: f32| [(x - cxp) / rx, (y - cyp) / ry];
    let u = vector(x1p, y1p);
    let v = vector(-x1p, -y1p);
    let angle = |a: [f32; 2], b: [f32; 2]| {
        let cross = a[0] * b[1] - a[1] * b[0];
        let dot = a[0] * b[0] + a[1] * b[1];
        // Preserve the sign of the dot product. Clamping it positive folds
        // angles beyond 90 degrees into the wrong quadrant.
        cross.atan2(dot)
    };
    let start_angle = angle([1.0, 0.0], u);
    let mut delta = angle(u, v);
    if !sweep && delta > 0.0 {
        delta -= std::f32::consts::TAU;
    }
    if sweep && delta < 0.0 {
        delta += std::f32::consts::TAU;
    }
    let mut points: Vec<_> = (0..=steps)
        .map(|index| {
            let t = start_angle + delta * index as f32 / steps as f32;
            [
                center[0] + rx * cos_phi * t.cos() - ry * sin_phi * t.sin(),
                center[1] + rx * sin_phi * t.cos() + ry * cos_phi * t.sin(),
            ]
        })
        .collect();
    if let Some(last) = points.last_mut() {
        *last = end;
    }
    points
}

pub fn tokenize_path(data: &str) -> Vec<SvgToken> {
    let mut tokens = Vec::new();
    let mut number = String::new();
    let flush = |tokens: &mut Vec<SvgToken>, number: &mut String| {
        if !number.is_empty() {
            if let Ok(value) = number.parse::<f32>() {
                tokens.push(SvgToken::Number(value));
            }
            number.clear();
        }
    };
    for character in data.chars() {
        let exponent = matches!(character, 'e' | 'E')
            && !number.is_empty()
            && !number.contains('e')
            && !number.contains('E');
        if character.is_ascii_alphabetic() && !exponent {
            flush(&mut tokens, &mut number);
            tokens.push(SvgToken::Command(character));
        } else if character == '.'
            && number.contains('.')
            && !number.contains('e')
            && !number.contains('E')
        {
            flush(&mut tokens, &mut number);
            number.push(character);
        } else if character.is_ascii_digit() || matches!(character, '.' | 'e' | 'E') {
            number.push(character);
        } else if matches!(character, '-' | '+') && (number.ends_with('e') || number.ends_with('E'))
        {
            number.push(character);
        } else if matches!(character, '-' | '+') {
            flush(&mut tokens, &mut number);
            number.push(character);
        } else {
            flush(&mut tokens, &mut number);
        }
    }
    flush(&mut tokens, &mut number);
    tokens
}

#[cfg(test)]
mod tests {
    use super::{
        flatten_path, parse_path, parse_svg_document_convex_fill_meshes,
        parse_svg_document_vector_paths, parse_svg_document_vector_records,
        parse_svg_document_vector_records_from_xml_events,
        parse_svg_document_vector_records_with_viewport,
        parse_svg_document_vector_records_with_xml_options, stroke_paths, tokenize_path,
        SvgImportStage, SvgPathCommand, SvgToken, SvgViewportSource,
    };
    use xml_tools::{parse_xml_events, XmlDiagnosticCode, XmlLimits, XmlOptions, XmlSourceId};

    #[test]
    fn preserves_compact_signed_numbers() {
        assert_eq!(
            tokenize_path("M20 6 9 17l-5-5"),
            vec![
                SvgToken::Command('M'),
                SvgToken::Number(20.0),
                SvgToken::Number(6.0),
                SvgToken::Number(9.0),
                SvgToken::Number(17.0),
                SvgToken::Command('l'),
                SvgToken::Number(-5.0),
                SvgToken::Number(-5.0),
            ]
        );
    }

    #[test]
    fn parses_curve_arc_and_close_commands() {
        let commands = parse_path("M0 0 C1 2 3 4 5 6 A2 3 0 0 1 8 9 Z").unwrap();
        assert!(matches!(commands[1], SvgPathCommand::CubicTo { .. }));
        assert!(matches!(commands[2], SvgPathCommand::ArcTo { .. }));
        assert_eq!(commands[3], SvgPathCommand::ClosePath);
    }

    #[test]
    fn parses_implicit_repeated_move_and_line_arguments() {
        let commands = parse_path("M0 0 10 0 10 10 l5 0 0 5").unwrap();

        assert!(matches!(commands[0], SvgPathCommand::MoveTo { .. }));
        assert!(matches!(commands[1], SvgPathCommand::LineTo { .. }));
        assert!(matches!(commands[2], SvgPathCommand::LineTo { .. }));
        assert!(matches!(commands[3], SvgPathCommand::LineTo { .. }));
        assert!(matches!(commands[4], SvgPathCommand::LineTo { .. }));
    }

    #[test]
    fn tokenizes_scientific_notation_without_confusing_the_exponent_for_a_command() {
        let commands = parse_path("M1e1 2E1 l-5e-1 .5").unwrap();

        assert_eq!(commands.len(), 2);
        assert!(matches!(
            commands[0],
            SvgPathCommand::MoveTo {
                x: 10.0,
                y: 20.0,
                ..
            }
        ));
        assert!(matches!(
            commands[1],
            SvgPathCommand::LineTo {
                relative: true,
                x: -0.5,
                y: 0.5,
                ..
            }
        ));
    }

    #[test]
    fn flattening_resolves_relative_horizontal_and_vertical_commands() {
        let commands = parse_path("M2 3 h8 v4 h-8 z").unwrap();
        let paths = flatten_path(&commands, 8);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].first().copied(), Some([2.0, 3.0]));
        assert_eq!(paths[0].last().copied(), Some([2.0, 3.0]));
        assert!(paths[0].contains(&[10.0, 3.0]));
        assert!(paths[0].contains(&[10.0, 7.0]));
    }

    #[test]
    fn flattening_keeps_closed_and_following_relative_subpaths_separate() {
        let commands = parse_path("M10 10 l10 0 l0 10 z m5 5 l5 0").unwrap();
        let paths = flatten_path(&commands, 8);

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].first(), paths[0].last());
        assert_ne!(paths[1].first(), paths[1].last());
        assert_eq!(paths[1].first().copied(), Some([15.0, 15.0]));
    }

    #[test]
    fn smooth_quadratic_control_does_not_leak_across_a_new_subpath() {
        let commands = parse_path("M0 0 Q10 10 20 0 T40 0 M0 20 T20 20").unwrap();
        let paths = flatten_path(&commands, 8);

        assert_eq!(paths.len(), 2);
        assert!(paths[0].iter().any(|point| point[1] > 0.0));
        assert!(paths[1]
            .iter()
            .all(|point| (point[1] - 20.0).abs() < 1.0e-5));
    }

    #[test]
    fn flattens_cubic_and_closes_subpath() {
        let commands = parse_path("M0 0 C0 1 1 1 1 0 Z").unwrap();
        let paths = flatten_path(&commands, 8);
        assert_eq!(paths.len(), 1);
        assert!(paths[0].len() > 8);
        assert_eq!(paths[0].first(), paths[0].last());
    }

    #[test]
    fn flattens_arc_into_multiple_points() {
        let commands = parse_path("M21 12 A9 9 0 1 1 3 12").unwrap();
        let paths = flatten_path(&commands, 8);
        assert!(paths[0].len() > 8);
        assert!((paths[0].last().unwrap()[0] - 3.0).abs() < 0.01);
    }

    #[test]
    fn degenerate_arc_radii_reduce_to_a_line_without_non_finite_points() {
        let commands = parse_path("M0 0 A0 9 45 0 1 12 8").unwrap();
        let paths = flatten_path(&commands, 8);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![[0.0, 0.0], [12.0, 8.0]]);
    }

    #[test]
    fn rotated_arc_preserves_endpoints_and_finite_geometry() {
        let commands = parse_path("M10 20 A18 7 37 1 0 70 55").unwrap();
        let paths = flatten_path(&commands, 16);

        assert_eq!(paths.len(), 1);
        assert!(paths[0]
            .iter()
            .all(|point| point[0].is_finite() && point[1].is_finite()));
        assert_eq!(paths[0].first().copied(), Some([10.0, 20.0]));
        assert_eq!(paths[0].last().copied(), Some([70.0, 55.0]));
    }

    #[test]
    fn malformed_path_commands_return_diagnostics() {
        assert!(parse_path("M0 0 L").is_err());
        assert!(parse_path("M0 0 R10 10").is_err());
        assert!(parse_path("0 0 L10 10").is_err());
    }

    #[test]
    fn path_commands_reject_non_finite_numbers() {
        let error = parse_path("M0 0 L1e39 1")
            .expect_err("overflowing SVG path coordinates must be rejected");

        assert!(error.contains("non-finite L coordinate"));
    }

    #[test]
    fn arc_commands_reject_non_binary_flags() {
        let error =
            parse_path("M0 0 A4 4 0 2 0 8 8").expect_err("SVG arc flags must be binary values");

        assert!(error.contains("invalid A arc flag"));
    }

    #[test]
    fn vector_document_adapter_preserves_closed_contours() {
        let paths = parse_svg_document_vector_paths(
            r#"<svg><path d="M0 0 L24 0 L24 24 Z"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .unwrap();

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].contours.len(), 1);
        assert!(paths[0].contours[0].closed);
    }

    #[test]
    fn vector_document_adapter_ignores_document_metadata() {
        let paths = parse_svg_document_vector_paths(
            r#"<svg id="svg-root"><path id="path-01" d="M0 0 L24 0 L24 24 Z" /></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect("metadata must not be parsed as path data");

        assert_eq!(paths.len(), 1);
        assert!(paths[0].contours[0].closed);
    }

    #[test]
    fn vector_document_adapter_ignores_geometry_inside_comments() {
        let paths = parse_svg_document_vector_paths(
            r#"<svg>
                <!-- <path d="M0 0 L24 0 L24 24 Z"/><rect x="0" y="0" width="24" height="24"/> -->
                <line x1="0" y1="0" x2="24" y2="24"/>
            </svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect("comments should not be treated as geometry");

        assert_eq!(paths.len(), 1);
        assert!(!paths[0].contours[0].closed);
    }

    #[test]
    fn vector_document_adapter_consumes_decoded_attributes_and_ignores_processing_instructions() {
        let records = parse_svg_document_vector_records_with_xml_options(
            r#"<?xml version="1.0"?><svg><?corpus keep?><path d="M0&#x20;0&#x20;L24&#x20;0"/><?corpus keep?><line x1="0" y1="24" x2="24" y2="24"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect("XML-decoded SVG attributes must reach SVG lowering unchanged");

        assert_eq!(records.len(), 2);
        assert_eq!(
            records[0].path.contours[0].points,
            vec![[-0.5, 0.5], [0.5, 0.5]]
        );
        assert_eq!(
            records[1].path.contours[0].points,
            vec![[-0.5, -0.5], [0.5, -0.5]]
        );
        assert!(records[0].source_span.start < records[1].source_span.start);
    }

    #[test]
    fn vector_document_adapter_does_not_match_element_name_prefixes() {
        let paths = parse_svg_document_vector_paths(
            r#"<svg>
                <pathology d="M0 0 L24 0 L24 24 Z"/>
                <rectangle x="0" y="0" width="24" height="24"/>
            </svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect("unrelated element names should be ignored");

        assert!(paths.is_empty());
    }

    #[test]
    fn vector_document_adapter_handles_gt_inside_quoted_attributes() {
        let paths = parse_svg_document_vector_paths(
            r#"<svg><path data-label="a > b" d="M0 0 L24 0 L24 24 Z" /></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect("quoted attribute text must not terminate the tag early");

        assert_eq!(paths.len(), 1);
        assert!(paths[0].contours[0].closed);
    }

    #[test]
    fn vector_document_adapter_accepts_single_quoted_attributes() {
        let paths = parse_svg_document_vector_paths(
            "<svg><path d='M0 0 L24 0 L24 24 Z' /></svg>",
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect("single-quoted SVG attributes should parse");

        assert_eq!(paths.len(), 1);
        assert!(paths[0].contours[0].closed);
    }

    #[test]
    fn vector_document_adapter_accepts_single_quoted_path_data() {
        let paths = parse_svg_document_vector_paths(
            "<svg><path d='M0 0 L24 0 L24 24 Z' /></svg>",
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect("vector adapter should accept single-quoted path data");

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].contours.len(), 1);
        assert_eq!(
            paths[0].contours[0].points.first().copied(),
            Some([-0.5, 0.5])
        );
        assert!(paths[0].contours[0].closed);
    }

    #[test]
    fn document_adapters_reject_unterminated_path_elements() {
        let svg = r#"<svg><path d="M0 0 L24 0"#;

        let vector_error = parse_svg_document_vector_paths(svg, 8, [0.0, 0.0, 24.0, 24.0])
            .expect_err("vector adapter must reject truncated path markup");
        assert!(vector_error.contains("SVG XML syntax error"));
    }

    #[test]
    fn structured_svg_diagnostics_preserve_xml_and_svg_boundaries() {
        let xml_error = parse_svg_document_vector_records_with_xml_options(
            r#"<svg><path d="M0 0""#,
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect_err("truncated XML must preserve its XML diagnostic");
        assert_eq!(xml_error.stage, SvgImportStage::Xml);
        assert_eq!(
            xml_error.xml.as_ref().map(|diagnostic| diagnostic.code),
            Some(XmlDiagnosticCode::ParserSyntax)
        );
        assert!(xml_error.span.is_some());

        let limit_error = parse_svg_document_vector_records_with_xml_options(
            r#"<svg><path d="M0 0 L24 24"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions {
                limits: XmlLimits {
                    max_input_bytes: 8,
                    ..XmlLimits::default()
                },
            },
        )
        .expect_err("XML resource limits must remain distinguishable");
        assert_eq!(limit_error.stage, SvgImportStage::Xml);
        assert_eq!(
            limit_error.xml.as_ref().map(|diagnostic| diagnostic.code),
            Some(XmlDiagnosticCode::InputTooLarge)
        );

        let namespace_error = parse_svg_document_vector_records_with_xml_options(
            r#"<svg><foreign:path/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect_err("unbound XML prefixes must stop before SVG interpretation");
        assert_eq!(namespace_error.stage, SvgImportStage::Xml);
        assert_eq!(
            namespace_error
                .xml
                .as_ref()
                .map(|diagnostic| diagnostic.code),
            Some(XmlDiagnosticCode::UnboundPrefix)
        );

        let svg_error = parse_svg_document_vector_records_with_xml_options(
            r#"<svg><rect x="0" y="0" width="-1" height="1"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect_err("valid XML with unsupported SVG geometry must be an SVG diagnostic");
        assert_eq!(svg_error.stage, SvgImportStage::Svg);
        assert!(svg_error.xml.is_none());
        assert!(svg_error.span.is_some());
    }

    #[test]
    fn semantic_pass_has_explicit_root_and_namespace_policy() {
        let prefixed = parse_svg_document_vector_records_with_xml_options(
            r#"<svg:svg xmlns:svg="http://www.w3.org/2000/svg"><svg:path d="M0 0 L24 0 L24 24 Z"/></svg:svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect("SVG-prefixed elements must be admitted through expanded names");
        assert_eq!(prefixed.len(), 1);
        assert_eq!(prefixed[0].source_span.source.value(), 0);

        let default_namespaced = parse_svg_document_vector_records_with_xml_options(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0 L24 0 L24 24 Z"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect("default SVG namespaces must be admitted through expanded names");
        assert_eq!(default_namespaced.len(), 1);

        let foreign = parse_svg_document_vector_records_with_xml_options(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:other="urn:other"><other:path d="M0 0 L24 0 L24 24 Z"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect("foreign geometry names must not become SVG paths by local-name collision");
        assert!(foreign.is_empty());

        let invalid_root = parse_svg_document_vector_records_with_xml_options(
            r#"<document><path d="M0 0 L24 0 L24 24 Z"/></document>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect_err("a valid XML non-SVG document must be an SVG-stage diagnostic");
        assert_eq!(invalid_root.stage, SvgImportStage::Svg);
        assert!(invalid_root.xml.is_none());
    }

    #[test]
    fn semantic_profile_diagnoses_unadmitted_svg_features() {
        for element in [
            r#"<text x="1" y="1">not admitted</text>"#,
            r#"<defs><path id="shape" d="M0 0 L24 0"/></defs>"#,
            r#"<clipPath id="clip"><rect width="24" height="24"/></clipPath>"#,
        ] {
            let diagnostic = parse_svg_document_vector_records_with_xml_options(
                &format!("<svg>{element}</svg>"),
                8,
                [0.0, 0.0, 24.0, 24.0],
                XmlOptions::default(),
            )
            .expect_err("unadmitted SVG features must not be silently accepted");
            assert_eq!(diagnostic.stage, SvgImportStage::Svg);
            assert!(diagnostic.span.is_some());
            assert!(diagnostic
                .message
                .contains("outside the admitted importer profile"));
        }
    }

    #[test]
    fn vector_document_adapter_rejects_unterminated_primitive_elements() {
        let error = parse_svg_document_vector_paths(
            r#"<svg><rect x="0" y="0" width="24""#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect_err("vector adapter must reject truncated primitive markup");

        assert!(error.contains("SVG XML syntax error"));
    }

    #[test]
    fn vector_document_adapter_stops_at_the_xml_profile_boundary() {
        let error = parse_svg_document_vector_paths(
            r#"<!DOCTYPE svg SYSTEM "external.dtd"><svg><path d="M0 0 L24 0"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect_err("disabled XML DTD processing must stop before SVG interpretation");

        assert!(error.contains("UnsupportedDocumentType"));
        assert!(error.contains("DOCTYPE declarations"));
    }

    #[test]
    fn vector_document_adapter_accepts_whitespace_around_attribute_equals() {
        let records = parse_svg_document_vector_records(
            r#"<svg><path d = "M0 0 L24 0 L24 24 Z" fill = "none" stroke = "black"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect("whitespace around attribute equals should be accepted");

        assert_eq!(records.len(), 1);
        assert!(!records[0].fill);
        assert!(records[0].stroke);
        assert!(records[0].path.contours[0].closed);
    }

    #[test]
    fn vector_document_adapter_preserves_open_and_multiple_contours() {
        let paths = parse_svg_document_vector_paths(
            r#"<svg><path d="M0 0 L24 0 M0 24 L24 24"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .unwrap();

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].contours.len(), 2);
        assert!(paths[0].contours.iter().all(|contour| !contour.closed));
    }

    #[test]
    fn vector_document_adapter_handles_primitive_elements() {
        let paths = parse_svg_document_vector_paths(
            r#"<svg><circle cx="12" cy="12" r="4"/><ellipse cx="12" cy="12" rx="6" ry="4"/><rect x="2" y="3" width="5" height="6"/><line x1="0" y1="0" x2="24" y2="24"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .unwrap();

        assert_eq!(paths.len(), 4);
        assert!(paths[0].contours[0].closed);
        assert!(paths[1].contours[0].closed);
        assert!(paths[2].contours[0].closed);
        assert!(!paths[3].contours[0].closed);
    }

    #[test]
    fn primitive_elements_reject_negative_dimensions() {
        let circle_error = parse_svg_document_vector_paths(
            r#"<svg><circle cx="12" cy="12" r="-4"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect_err("negative circle radii must be rejected");
        assert!(circle_error.contains("circle radius"));

        let ellipse_error = parse_svg_document_vector_paths(
            r#"<svg><ellipse cx="12" cy="12" rx="6" ry="-4"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect_err("negative ellipse radii must be rejected");
        assert!(ellipse_error.contains("ellipse radii"));

        let rect_error = parse_svg_document_vector_paths(
            r#"<svg><rect x="0" y="0" width="-4" height="8"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect_err("negative rectangle dimensions must be rejected");
        assert!(rect_error.contains("width and height"));

        let radius_error = parse_svg_document_vector_records(
            r#"<svg><rect x="0" y="0" width="8" height="8" rx="-1"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect_err("negative rectangle radii must be rejected");
        assert!(radius_error.contains("corner radii"));
    }

    #[test]
    fn rounded_rect_couples_an_omitted_radius() {
        let records = parse_svg_document_vector_records(
            r#"<svg><rect x="0" y="0" width="12" height="8" ry="2"/><rect x="20" y="0" width="12" height="8" rx="2"/></svg>"#,
            8,
            [0.0, 0.0, 32.0, 8.0],
        )
        .expect("rounded rectangles with one radius should parse");

        assert_eq!(records.len(), 2);
        assert!(records[0].path.contours[0].points.len() > 5);
        assert_eq!(
            records[0].path.contours[0].points.len(),
            records[1].path.contours[0].points.len()
        );
    }

    #[test]
    fn vector_document_adapter_preserves_mixed_element_order() {
        let records = parse_svg_document_vector_records(
            r#"<svg>
                <rect x="0" y="0" width="4" height="4"/>
                <path d="M8 0 L12 0 L12 4 Z"/>
                <line x1="16" y1="0" x2="20" y2="4"/>
            </svg>"#,
            8,
            [0.0, 0.0, 20.0, 4.0],
        )
        .expect("mixed SVG elements should preserve source order");

        assert_eq!(records.len(), 3);
        assert_eq!(records[0].path.contours[0].points[0], [-0.5, 0.5]);
        let path_start = records[1].path.contours[0].points[0];
        let line_start = records[2].path.contours[0].points[0];
        assert!((path_start[0] + 0.1).abs() < 1.0e-6 && (path_start[1] - 0.5).abs() < 1.0e-6);
        assert!((line_start[0] - 0.3).abs() < 1.0e-6 && (line_start[1] - 0.5).abs() < 1.0e-6);
    }

    #[test]
    fn vector_document_adapter_ignores_unmatched_trailing_point_coordinate() {
        let paths = parse_svg_document_vector_paths(
            r#"<svg><polyline points="0,0 12,12 24"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect("an unmatched trailing coordinate is ignored");

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].contours[0].points.len(), 2);
    }

    #[test]
    fn vector_document_adapter_rejects_invalid_polyline_numbers() {
        let error = parse_svg_document_vector_paths(
            r#"<svg><polyline points="0,0 nope,12"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect_err("an invalid primitive coordinate must not be discarded");

        assert!(error.contains("invalid number 'nope'"));
    }

    #[test]
    fn vector_document_adapter_rejects_non_finite_polyline_numbers() {
        let error = parse_svg_document_vector_paths(
            r#"<svg><polyline points="0,0 NaN,12"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect_err("non-finite primitive coordinates must not enter geometry");

        assert!(error.contains("non-finite number 'NaN'"));
    }

    #[test]
    fn vector_document_adapter_rejects_invalid_view_box_dimensions() {
        let error = parse_svg_document_vector_paths(
            r#"<svg><path d="M0 0 L1 1"/></svg>"#,
            8,
            [0.0, 0.0, 0.0, 24.0],
        )
        .expect_err("a zero-width viewBox cannot be normalized");

        assert!(error.contains("positive dimensions"));
    }

    #[test]
    fn vector_document_adapter_normalizes_negative_primitive_coordinates() {
        let records = parse_svg_document_vector_records(
            r#"<svg><line x1="-10" y1="-5" x2="10" y2="5"/></svg>"#,
            8,
            [-10.0, -5.0, 20.0, 10.0],
        )
        .expect("negative source coordinates should normalize");

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].path.contours[0].points[0], [-0.5, 0.5]);
        assert_eq!(records[0].path.contours[0].points[1], [0.5, -0.5]);
    }

    #[test]
    fn vector_records_preserve_fill_and_stroke_intent() {
        let records = parse_svg_document_vector_records(
            r#"<svg>
                <path d="M0 0 L24 0 L24 24 Z" fill="none" stroke="black"/>
                <path d="M1 1 L23 1 L23 23 Z" style="fill: none; stroke: black; fill-rule: evenodd"/>
                <rect x="2" y="2" width="4" height="4"/>
            </svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect("SVG paint metadata should parse");

        assert_eq!(records.len(), 3);
        assert!(!records[0].fill && records[0].stroke);
        assert!(!records[1].fill && records[1].stroke);
        assert_eq!(records[1].fill_rule, super::SvgFillRule::EvenOdd);
        assert!(records[2].fill && !records[2].stroke);
        assert_eq!(records[2].fill_rule, super::SvgFillRule::NonZero);
    }

    #[test]
    fn nested_svg_presentation_state_inherits_and_restores_for_siblings() {
        let records = parse_svg_document_vector_records(
            r#"<svg fill="none" stroke="black" fill-rule="evenodd">
                <g fill="white" style="fill: none; stroke: none">
                    <path d="M0 0 L24 0 L24 24 Z"/>
                </g>
                <path d="M1 1 L23 1 L23 23 Z"/>
                <g fill="none" stroke="none" fill-rule="nonzero">
                    <path d="M2 2 L22 2 L22 22 Z" fill="inherit"/>
                </g>
                <path d="M3 3 L21 3 L21 21 Z"/>
            </svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect("nested SVG presentation state should lower deterministically");

        assert_eq!(records.len(), 4);

        // Presentation attributes take precedence over an inline style in the
        // initial profile, preserving the importer's established behavior.
        assert!(records[0].fill);
        assert!(!records[0].stroke);
        assert_eq!(records[0].fill_rule, super::SvgFillRule::EvenOdd);

        assert!(!records[1].fill && records[1].stroke);
        assert_eq!(records[1].fill_rule, super::SvgFillRule::EvenOdd);

        assert!(!records[2].fill && !records[2].stroke);
        assert_eq!(records[2].fill_rule, super::SvgFillRule::NonZero);

        assert!(!records[3].fill && records[3].stroke);
        assert_eq!(records[3].fill_rule, super::SvgFillRule::EvenOdd);
    }

    #[test]
    fn svg_transforms_compose_through_nested_state_before_normalization() {
        let assert_point = |actual: [f32; 2], expected: [f32; 2]| {
            assert!(
                (actual[0] - expected[0]).abs() < 1.0e-5
                    && (actual[1] - expected[1]).abs() < 1.0e-5,
                "expected {expected:?}, received {actual:?}"
            );
        };
        let records = parse_svg_document_vector_records(
            r#"<svg>
                <g transform="translate(10 20)">
                    <path transform="scale(2)" d="M0 0 L10 0"/>
                    <path transform="rotate(90 10 10)" d="M20 10 L20 20"/>
                </g>
            </svg>"#,
            8,
            [0.0, 0.0, 100.0, 100.0],
        )
        .expect("supported transforms should lower through the SVG state stack");

        assert_eq!(records.len(), 2);
        assert_point(records[0].path.contours[0].points[0], [-0.4, 0.3]);
        assert_point(records[0].path.contours[0].points[1], [-0.2, 0.3]);
        assert_point(records[1].path.contours[0].points[0], [-0.3, 0.1]);
        assert_point(records[1].path.contours[0].points[1], [-0.4, 0.1]);

        let listed = parse_svg_document_vector_records(
            r#"<svg><path transform="translate(10) scale(2)" d="M10 0 L20 0"/></svg>"#,
            8,
            [0.0, 0.0, 100.0, 100.0],
        )
        .expect("transform lists should compose in SVG order");
        assert_point(listed[0].path.contours[0].points[0], [-0.2, 0.5]);
        assert_point(listed[0].path.contours[0].points[1], [0.0, 0.5]);
    }

    #[test]
    fn unsupported_or_malformed_svg_transforms_are_svg_diagnostics() {
        for transform in ["skewX(10)", "translate(nope)", "translate(10"] {
            let diagnostic = parse_svg_document_vector_records_with_xml_options(
                &format!(r#"<svg><path transform="{transform}" d="M0 0 L24 0"/></svg>"#),
                8,
                [0.0, 0.0, 24.0, 24.0],
                XmlOptions::default(),
            )
            .expect_err("unsupported SVG transform syntax must remain visible");
            assert_eq!(diagnostic.stage, SvgImportStage::Svg);
            assert!(diagnostic.xml.is_none());
            assert!(diagnostic.span.is_some());
        }
    }

    #[test]
    fn svg_viewport_policy_distinguishes_caller_bounds_from_root_view_box() {
        let source = r#"<svg viewBox="10 20 20 10" width="200" height="100">
            <line x1="10" y1="20" x2="30" y2="30"/>
        </svg>"#;
        let document = parse_svg_document_vector_records_with_viewport(
            source,
            8,
            SvgViewportSource::DocumentViewBox,
            XmlOptions::default(),
        )
        .expect("the document viewBox should provide coordinate normalization");
        assert_eq!(document[0].path.contours[0].points[0], [-0.5, 0.5]);
        assert_eq!(document[0].path.contours[0].points[1], [0.5, -0.5]);

        let caller = parse_svg_document_vector_records_with_viewport(
            source,
            8,
            SvgViewportSource::Caller([0.0, 0.0, 40.0, 40.0]),
            XmlOptions::default(),
        )
        .expect("the caller viewport must remain an explicit alternate path");
        assert_eq!(caller[0].path.contours[0].points[0], [-0.25, 0.0]);
        assert_eq!(caller[0].path.contours[0].points[1], [0.25, -0.25]);

        let missing = parse_svg_document_vector_records_with_viewport(
            r#"<svg><line x1="0" y1="0" x2="1" y2="1"/></svg>"#,
            8,
            SvgViewportSource::DocumentViewBox,
            XmlOptions::default(),
        )
        .expect_err("document viewBox mode must not invent a coordinate model");
        assert_eq!(missing.stage, SvgImportStage::Svg);
        assert!(missing.span.is_some());
    }

    #[test]
    fn svg_lowering_reuses_an_existing_parser_neutral_event_stream() {
        let source = r#"<svg><path d="M0 0 L24 0 L24 24 Z"/></svg>"#;
        let events = parse_xml_events(XmlSourceId::new(77), source, XmlOptions::default())
            .expect("fixture XML should parse once before SVG lowering");
        let from_events = parse_svg_document_vector_records_from_xml_events(
            &events,
            8,
            SvgViewportSource::Caller([0.0, 0.0, 24.0, 24.0]),
        )
        .expect("SVG lowering should consume existing parser-neutral events");
        let from_source = parse_svg_document_vector_records(source, 8, [0.0, 0.0, 24.0, 24.0])
            .expect("source convenience API should use the same semantic lowering");

        assert_eq!(from_events.len(), from_source.len());
        assert_eq!(from_events[0].path, from_source[0].path);
        assert_eq!(from_events[0].fill, from_source[0].fill);
        assert_eq!(from_events[0].stroke, from_source[0].stroke);
        assert_eq!(from_events[0].fill_rule, from_source[0].fill_rule);
        assert_eq!(from_events[0].source_span.source.value(), 77);
        assert_eq!(from_source[0].source_span.source.value(), 0);
    }

    #[test]
    fn convex_fill_adapter_routes_supported_svg_geometry() {
        let svg = r#"<svg><rect x="0" y="0" width="12" height="12" /></svg>"#;
        let meshes = parse_svg_document_convex_fill_meshes(svg, 8, [0.0, 0.0, 12.0, 12.0])
            .expect("rectangle should use the shared convex fill tessellator");

        assert_eq!(meshes.len(), 1);
        assert_eq!(meshes[0].len(), 6);
    }

    #[test]
    fn convex_fill_adapter_reports_unsupported_svg_topology() {
        let svg = r#"<svg><path d="M 0 0 L 12 0 L 6 3 L 0 12 Z" /></svg>"#;
        let error = parse_svg_document_convex_fill_meshes(svg, 8, [0.0, 0.0, 12.0, 12.0])
            .expect_err("concave fill should be diagnosed");

        assert!(error.contains("SVG fill path 0 is unsupported"));
    }

    #[test]
    fn parses_lucide_activity_path() {
        let data = "M22 12h-2.48a2 2 0 0 0-1.93 1.46l-2.35 8.36a.25.25 0 0 1-.48 0L9.24 2.18a.25.25 0 0 0-.48 0l-2.35 8.36A2 2 0 0 1 4.49 12H2";
        assert_eq!(parse_path(data).map(|_| ()), Ok(()));
    }

    #[test]
    fn lucide_asterisk_stays_three_straight_strokes() {
        let mut paths = Vec::new();
        for data in ["M12 6v12", "M17.196 9 6.804 15", "m6.804 9 10.392 6"] {
            let commands = parse_path(data).expect("asterisk path should parse");
            let flattened = flatten_path(&commands, 12);
            assert_eq!(flattened.len(), 1);
            assert_eq!(flattened[0].len(), 2);
            paths.extend(flattened);
        }
        let mesh = super::stroke_paths(&paths, 1.0 / 32.0);
        assert!(!mesh.is_empty());
        assert!(mesh
            .iter()
            .all(|vertex| vertex.iter().all(|value| value.is_finite())));
    }

    #[test]
    fn lucide_astroid_arc_geometry_stays_inside_viewbox() {
        let data = "M12.983 21.186a1 1 0 0 1-1.966 0 10 10 0 0 0-8.203-8.203 1 1 0 0 1 0-1.966 10 10 0 0 0 8.203-8.203 1 1 0 0 1 1.966 0 10 10 0 0 0 8.203 8.203 1 1 0 0 1 0 1.966 10 10 0 0 0-8.203 8.203";
        let commands = parse_path(data).expect("astroid path should parse");
        let points = flatten_path(&commands, 12)
            .into_iter()
            .next()
            .expect("astroid path should flatten");
        let min_x = points
            .iter()
            .map(|point| point[0])
            .fold(f32::INFINITY, f32::min);
        let max_x = points
            .iter()
            .map(|point| point[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = points
            .iter()
            .map(|point| point[1])
            .fold(f32::INFINITY, f32::min);
        let max_y = points
            .iter()
            .map(|point| point[1])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            min_x >= -0.01 && max_x <= 24.01 && min_y >= -0.01 && max_y <= 24.01,
            "bounds x={min_x}..{max_x}, y={min_y}..{max_y}"
        );
    }

    #[test]
    fn tiny_lucide_control_stroke_produces_a_cap() {
        let commands = parse_path("M6 8h.01").expect("tiny Lucide path should parse");
        let paths = flatten_path(&commands, 32);
        let mesh = stroke_paths(&paths, 1.0 / 32.0);
        assert!(!mesh.is_empty());
        assert!(mesh.iter().all(|vertex| vertex[2] == 0.0));
    }

    #[test]
    fn extracts_svg_primitives_and_normalizes_viewbox() {
        let svg = r#"<svg viewBox="0 0 24 24">
            <path d="M0 0h24v24H0z" />
            <circle cx="12" cy="12" r="4" />
            <line x1="2" y1="3" x2="6" y2="7" />
            <rect x="4" y="5" width="6" height="8" rx="1" />
        </svg>"#;
        let paths = parse_svg_document_vector_paths(svg, 8, [0.0, 0.0, 24.0, 24.0]).unwrap();
        assert!(paths.len() >= 4);
        assert!(paths.iter().all(|path| !path.contours.is_empty()));
        assert!(paths
            .iter()
            .flat_map(|path| &path.contours)
            .flat_map(|contour| &contour.points)
            .all(|point| {
                (-0.51..=0.51).contains(&point[0]) && (-0.51..=0.51).contains(&point[1])
            }));
    }

    #[test]
    fn applies_svg_default_centers_for_circles_and_ellipses() {
        let svg = r#"<svg viewBox="-50 -50 300 300">
            <circle r="10" />
            <circle cx="80" cy="80" r="10" />
            <circle cx="120" cy="120" r="0" />
            <ellipse rx="20" ry="10" />
            <ellipse cx="140" cy="140" rx="20" ry="10" />
            <ellipse cx="180" cy="180" rx="0" ry="10" />
        </svg>"#;
        let paths = parse_svg_document_vector_paths(svg, 8, [-50.0, -50.0, 300.0, 300.0])
            .expect("default shape centers should be accepted");
        assert_eq!(paths.len(), 4);
        assert!(paths.iter().all(|path| path.contours.len() == 1));
    }
}
