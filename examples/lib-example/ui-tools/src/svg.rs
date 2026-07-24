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
}

impl SvgVectorRecord {
    fn new(path: crate::VectorPath, tag: &str) -> Self {
        let fill = svg_paint_value(tag, "fill").is_none_or(|value| value != "none");
        let stroke = svg_paint_value(tag, "stroke").is_some_and(|value| value != "none");
        let fill_rule = match svg_paint_value(tag, "fill-rule").as_deref() {
            Some("evenodd") => SvgFillRule::EvenOdd,
            _ => SvgFillRule::NonZero,
        };
        Self {
            path,
            fill,
            stroke,
            fill_rule,
        }
    }
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

/// Extracts SVG geometry into normalized, flattened polylines.
///
/// This intentionally supports the small geometry vocabulary used by Lucide
/// and similar icon providers. Document loading remains the caller's concern;
/// this function owns interpretation and coordinate normalization.
#[cfg(test)]
fn parse_svg_document_paths(
    svg: &str,
    subdivisions: usize,
    view_box: [f32; 4],
) -> Result<Vec<Vec<[f32; 2]>>, String> {
    let [view_x, view_y, view_width, view_height] = view_box;
    if view_width <= 0.0 || view_height <= 0.0 {
        return Err("SVG viewBox must have positive dimensions".into());
    }
    let normalize = |point: [f32; 2]| {
        [
            (point[0] - view_x) / view_width - 0.5,
            0.5 - (point[1] - view_y) / view_height,
        ]
    };
    let mut paths = Vec::new();

    let mut remainder = svg;
    while let Some(start) = find_svg_element_start(remainder, "path") {
        remainder = &remainder[start..];
        let Some(end) = svg_tag_end(remainder) else {
            break;
        };
        let tag = &remainder[..=end];
        if let Some(data) = svg_attribute_text(tag, "d") {
            let commands = parse_path(&data)?;
            paths.extend(
                flatten_path(&commands, subdivisions)
                    .into_iter()
                    .filter(|path| path.len() > 1)
                    .map(|path| path.into_iter().map(normalize).collect()),
            );
        }
        remainder = &remainder[end + 1..];
    }

    for element in ["circle", "rect", "line", "polyline", "polygon"] {
        let mut remainder = svg;
        while let Some(start) = remainder.find(&format!("<{element}")) {
            remainder = &remainder[start..];
            let Some(end) = remainder.find('>') else {
                break;
            };
            let tag = &remainder[..=end];
            let path = match element {
                "circle" => {
                    let (Some(cx), Some(cy), Some(radius)) = (
                        svg_attribute(tag, "cx"),
                        svg_attribute(tag, "cy"),
                        svg_attribute(tag, "r"),
                    ) else {
                        remainder = &remainder[end + 1..];
                        continue;
                    };
                    if radius < 0.0 {
                        return Err("SVG circle radius must not be negative".into());
                    }
                    Some(
                        (0..=subdivisions.max(16))
                            .map(|index| {
                                let angle = index as f32 * std::f32::consts::TAU
                                    / subdivisions.max(16) as f32;
                                normalize([cx + radius * angle.cos(), cy + radius * angle.sin()])
                            })
                            .collect(),
                    )
                }
                "line" => {
                    let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                        svg_attribute(tag, "x1"),
                        svg_attribute(tag, "y1"),
                        svg_attribute(tag, "x2"),
                        svg_attribute(tag, "y2"),
                    ) else {
                        remainder = &remainder[end + 1..];
                        continue;
                    };
                    Some(vec![normalize([x1, y1]), normalize([x2, y2])])
                }
                "polyline" | "polygon" => {
                    let Some(values) = svg_attribute_text(tag, "points") else {
                        remainder = &remainder[end + 1..];
                        continue;
                    };
                    let numbers = parse_svg_point_numbers(&values, element)?;
                    let mut points = numbers
                        .chunks_exact(2)
                        .map(|pair| normalize([pair[0], pair[1]]))
                        .collect::<Vec<_>>();
                    if element == "polygon" && points.first() != points.last() {
                        if let Some(first) = points.first().copied() {
                            points.push(first);
                        }
                    }
                    Some(points)
                }
                "rect" => {
                    let (Some(x), Some(y), Some(width), Some(height)) = (
                        svg_attribute(tag, "x"),
                        svg_attribute(tag, "y"),
                        svg_attribute(tag, "width"),
                        svg_attribute(tag, "height"),
                    ) else {
                        remainder = &remainder[end + 1..];
                        continue;
                    };
                    if width < 0.0 || height < 0.0 {
                        return Err("SVG rectangle width and height must not be negative".into());
                    }
                    let raw_rx = svg_attribute(tag, "rx");
                    let raw_ry = svg_attribute(tag, "ry");
                    if raw_rx.is_some_and(|value| value < 0.0)
                        || raw_ry.is_some_and(|value| value < 0.0)
                    {
                        return Err("SVG rectangle corner radii must not be negative".into());
                    }
                    let rx = raw_rx.unwrap_or(0.0).min(width * 0.5);
                    let ry = raw_ry.unwrap_or(rx).min(height * 0.5);
                    Some(
                        svg_rectangle(x, y, width, height, rx, ry)
                            .into_iter()
                            .map(normalize)
                            .collect(),
                    )
                }
                _ => None,
            };
            if let Some(path) = path.filter(|path: &Vec<[f32; 2]>| path.len() > 1) {
                paths.push(path);
            }
            remainder = &remainder[end + 1..];
        }
    }
    Ok(paths)
}

/// Extracts SVG geometry into the shared provider-neutral path model.
///
/// This is an intentionally small migration adapter over the existing parser:
/// flattened contours retain explicit closure when the parser emitted a
/// repeated endpoint. SVG styling and topology beyond that contract remain
/// importer concerns.
pub fn parse_svg_document_vector_records(
    svg: &str,
    subdivisions: usize,
    view_box: [f32; 4],
) -> Result<Vec<SvgVectorRecord>, String> {
    let [view_x, view_y, view_width, view_height] = view_box;
    if view_width <= 0.0 || view_height <= 0.0 {
        return Err("SVG viewBox must have positive dimensions".into());
    }
    let normalize = |point: [f32; 2]| {
        [
            (point[0] - view_x) / view_width - 0.5,
            0.5 - (point[1] - view_y) / view_height,
        ]
    };
    let mut paths = Vec::<(usize, SvgVectorRecord)>::new();

    // Read `d` only from actual path tags. Splitting the whole document on
    // `d="` also matches the suffix of `id="` and can feed metadata into the
    // path parser as if it were SVG geometry.
    let mut remainder = svg;
    while let Some(start) = find_svg_element_start(remainder, "path") {
        let source_offset = svg.len() - remainder.len() + start;
        remainder = &remainder[start..];
        let Some(end) = svg_tag_end(remainder) else {
            break;
        };
        let tag = &remainder[..=end];
        if let Some(data) = svg_attribute_text(tag, "d") {
            let commands = parse_path(&data)?;
            let contours = flatten_path(&commands, subdivisions)
                .into_iter()
                .filter(|points| points.len() > 1)
                .map(|points| {
                    let mut points = points.into_iter().map(normalize).collect::<Vec<_>>();
                    let closed = points.len() > 1 && points.first() == points.last();
                    if closed {
                        points.pop();
                    }
                    crate::VectorContour::new(points, closed)
                })
                .collect::<Vec<_>>();
            if !contours.is_empty() {
                paths.push((
                    source_offset,
                    SvgVectorRecord::new(crate::VectorPath::new(contours), tag),
                ));
            }
        }
        remainder = &remainder[end + 1..];
    }

    for element in ["circle", "rect", "line", "polyline", "polygon"] {
        let mut remainder = svg;
        while let Some(start) = find_svg_element_start(remainder, element) {
            let source_offset = svg.len() - remainder.len() + start;
            remainder = &remainder[start..];
            let Some(end) = svg_tag_end(remainder) else {
                break;
            };
            let tag = &remainder[..=end];
            let points = match element {
                "circle" => {
                    let (Some(cx), Some(cy), Some(radius)) = (
                        svg_attribute(tag, "cx"),
                        svg_attribute(tag, "cy"),
                        svg_attribute(tag, "r"),
                    ) else {
                        remainder = &remainder[end + 1..];
                        continue;
                    };
                    if radius < 0.0 {
                        return Err("SVG circle radius must not be negative".into());
                    }
                    Some(
                        (0..=subdivisions.max(16))
                            .map(|index| {
                                let angle = index as f32 * std::f32::consts::TAU
                                    / subdivisions.max(16) as f32;
                                normalize([cx + radius * angle.cos(), cy + radius * angle.sin()])
                            })
                            .collect::<Vec<_>>(),
                    )
                }
                "line" => {
                    let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                        svg_attribute(tag, "x1"),
                        svg_attribute(tag, "y1"),
                        svg_attribute(tag, "x2"),
                        svg_attribute(tag, "y2"),
                    ) else {
                        remainder = &remainder[end + 1..];
                        continue;
                    };
                    Some(vec![normalize([x1, y1]), normalize([x2, y2])])
                }
                "polyline" | "polygon" => {
                    let Some(values) = svg_attribute_text(tag, "points") else {
                        remainder = &remainder[end + 1..];
                        continue;
                    };
                    let numbers = parse_svg_point_numbers(&values, element)?;
                    let mut points = numbers
                        .chunks_exact(2)
                        .map(|pair| normalize([pair[0], pair[1]]))
                        .collect::<Vec<_>>();
                    if element == "polygon" && points.first() != points.last() {
                        if let Some(first) = points.first().copied() {
                            points.push(first);
                        }
                    }
                    Some(points)
                }
                "rect" => {
                    let (Some(x), Some(y), Some(width), Some(height)) = (
                        svg_attribute(tag, "x"),
                        svg_attribute(tag, "y"),
                        svg_attribute(tag, "width"),
                        svg_attribute(tag, "height"),
                    ) else {
                        remainder = &remainder[end + 1..];
                        continue;
                    };
                    if width < 0.0 || height < 0.0 {
                        return Err("SVG rectangle width and height must not be negative".into());
                    }
                    let raw_rx = svg_attribute(tag, "rx");
                    let raw_ry = svg_attribute(tag, "ry");
                    if raw_rx.is_some_and(|value| value < 0.0)
                        || raw_ry.is_some_and(|value| value < 0.0)
                    {
                        return Err("SVG rectangle corner radii must not be negative".into());
                    }
                    let rx = raw_rx.unwrap_or(0.0).min(width * 0.5);
                    let ry = raw_ry.unwrap_or(rx).min(height * 0.5);
                    Some(
                        svg_rectangle(x, y, width, height, rx, ry)
                            .into_iter()
                            .map(normalize)
                            .collect(),
                    )
                }
                _ => None,
            };
            if let Some(points) = points.filter(|points: &Vec<[f32; 2]>| points.len() > 1) {
                let closed = matches!(element, "circle" | "rect" | "polygon");
                let points = if closed {
                    points[..points.len() - 1].to_vec()
                } else {
                    points
                };
                paths.push((
                    source_offset,
                    SvgVectorRecord::new(
                        crate::VectorPath::new(vec![crate::VectorContour::new(points, closed)]),
                        tag,
                    ),
                ));
            }
            remainder = &remainder[end + 1..];
        }
    }

    paths.sort_by_key(|(source_offset, _)| *source_offset);
    Ok(paths.into_iter().map(|(_, record)| record).collect())
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
    if numbers.len() % 2 != 0 {
        return Err(format!(
            "SVG {element} points attribute contains an unmatched coordinate"
        ));
    }
    Ok(numbers)
}

fn svg_attribute(tag: &str, name: &str) -> Option<f32> {
    let (start, quote) = svg_attribute_value_start(tag, name)?;
    let end = tag[start..].find(quote)? + start;
    tag[start..end].parse().ok()
}

fn find_svg_element_start(svg: &str, name: &str) -> Option<usize> {
    let needle = format!("<{name}");
    let mut offset = 0;
    while let Some(found) = svg[offset..].find(&needle) {
        let start = offset + found;
        let comment_start = svg[..start].rfind("<!--");
        let comment_end = svg[..start].rfind("-->");
        let in_comment = match (comment_start, comment_end) {
            (Some(comment_start), Some(comment_end)) => comment_start > comment_end,
            (Some(_), None) => true,
            _ => false,
        };
        if in_comment {
            let Some(end) = svg[start..].find("-->") else {
                return None;
            };
            offset = start + end + 3;
            continue;
        }
        let after_name = start + needle.len();
        let boundary = svg[after_name..].chars().next();
        if boundary.is_some_and(|character| {
            character.is_ascii_whitespace() || matches!(character, '>' | '/')
        }) {
            return Some(start);
        }
        offset = after_name;
    }
    None
}

fn svg_tag_end(svg: &str) -> Option<usize> {
    let mut quote = None;
    for (index, character) in svg.char_indices() {
        match (quote, character) {
            (None, '\'' | '"') => quote = Some(character),
            (Some(active), character) if active == character => quote = None,
            (None, '>') => return Some(index),
            _ => {}
        }
    }
    None
}

fn svg_attribute_text(tag: &str, name: &str) -> Option<String> {
    let (start, quote) = svg_attribute_value_start(tag, name)?;
    let end = tag[start..].find(quote)? + start;
    Some(tag[start..end].to_owned())
}

fn svg_paint_value(tag: &str, name: &str) -> Option<String> {
    if let Some(value) = svg_attribute_text(tag, name) {
        return Some(value.trim().to_ascii_lowercase());
    }
    let style = svg_attribute_text(tag, "style")?;
    style.split(';').find_map(|declaration| {
        let (property, value) = declaration.split_once(':')?;
        (property.trim().eq_ignore_ascii_case(name)).then(|| value.trim().to_ascii_lowercase())
    })
}

fn svg_attribute_value_start(tag: &str, name: &str) -> Option<(usize, char)> {
    let mut offset = 0;
    while let Some(found) = tag[offset..].find(name) {
        let start = offset + found;
        let is_attribute_boundary = start == 0
            || tag[..start]
                .chars()
                .next_back()
                .is_some_and(|character| character.is_ascii_whitespace() || character == '<');
        if is_attribute_boundary {
            let mut cursor = start + name.len();
            while let Some(character) = tag[cursor..].chars().next() {
                if !character.is_ascii_whitespace() {
                    break;
                }
                cursor += character.len_utf8();
            }
            if tag[cursor..].starts_with('=') {
                cursor += 1;
                while let Some(character) = tag[cursor..].chars().next() {
                    if !character.is_ascii_whitespace() {
                        break;
                    }
                    cursor += character.len_utf8();
                }
                let quote = tag[cursor..].chars().next()?;
                if matches!(quote, '\'' | '"') {
                    return Some((cursor + quote.len_utf8(), quote));
                }
            }
        }
        offset = start + name.len();
    }
    None
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
        flatten_path, parse_path, parse_svg_document_convex_fill_meshes, parse_svg_document_paths,
        parse_svg_document_vector_paths, parse_svg_document_vector_records, stroke_paths,
        tokenize_path, SvgPathCommand, SvgToken,
    };

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
    fn legacy_document_adapter_accepts_single_quoted_path_data() {
        let paths = parse_svg_document_paths(
            "<svg><path d='M0 0 L24 0 L24 24 Z' /></svg>",
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect("legacy path adapter should share quoted attribute handling");

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].first().copied(), Some([-0.5, 0.5]));
        assert_eq!(paths[0].last().copied(), Some([-0.5, 0.5]));
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
            r#"<svg><circle cx="12" cy="12" r="4"/><rect x="2" y="3" width="5" height="6"/><line x1="0" y1="0" x2="24" y2="24"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .unwrap();

        assert_eq!(paths.len(), 3);
        assert!(paths[0].contours[0].closed);
        assert!(paths[1].contours[0].closed);
        assert!(!paths[2].contours[0].closed);
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
    fn vector_document_adapter_rejects_unmatched_polyline_coordinates() {
        let error = parse_svg_document_vector_paths(
            r#"<svg><polyline points="0,0 12,12 24"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect_err("an unmatched primitive coordinate must not be discarded");

        assert!(error.contains("unmatched coordinate"));
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
        let paths = parse_svg_document_paths(svg, 8, [0.0, 0.0, 24.0, 24.0]).unwrap();
        assert!(paths.len() >= 4);
        assert!(paths.iter().all(|path| path.len() > 1));
        assert!(paths.iter().flatten().all(|point| {
            (-0.51..=0.51).contains(&point[0]) && (-0.51..=0.51).contains(&point[1])
        }));
    }
}
