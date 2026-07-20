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
                    Ok(*value)
                }
                _ => Err(format!("incomplete {active} command at token {index}")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let relative = active.is_ascii_lowercase();
        let upper = active.to_ascii_uppercase();
        let command_value = match upper {
            'M' => SvgPathCommand::MoveTo { relative, x: values[0], y: values[1] },
            'L' => SvgPathCommand::LineTo { relative, x: values[0], y: values[1] },
            'H' => SvgPathCommand::HorizontalTo { relative, x: values[0] },
            'V' => SvgPathCommand::VerticalTo { relative, y: values[0] },
            'C' => SvgPathCommand::CubicTo { relative, values: values.try_into().unwrap() },
            'S' => SvgPathCommand::SmoothCubicTo { relative, values: values.try_into().unwrap() },
            'Q' => SvgPathCommand::QuadraticTo { relative, values: values.try_into().unwrap() },
            'T' => SvgPathCommand::SmoothQuadraticTo { relative, values: values.try_into().unwrap() },
            'A' => SvgPathCommand::ArcTo { relative, values: values.try_into().unwrap() },
            _ => unreachable!(),
        };
        result.push(command_value);
        if upper == 'M' {
            command = Some(if relative { 'l' } else { 'L' });
        }
    }
    Ok(result)
}

pub fn flatten_path(commands: &[SvgPathCommand], subdivisions: usize) -> Vec<Vec<[f32; 2]>> {
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
                if points.len() > 1 { paths.push(std::mem::take(&mut points)); }
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
                if current != start { points.push(start); }
                current = start;
                last_cubic_control = None;
                last_quadratic_control = None;
            }
            SvgPathCommand::SmoothCubicTo { relative, values } => {
                let p0 = current;
                let p1 = last_cubic_control.map(|control| [2.0 * p0[0] - control[0], 2.0 * p0[1] - control[1]]).unwrap_or(p0);
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
                let p1 = last_quadratic_control.map(|control| [2.0 * p0[0] - control[0], 2.0 * p0[1] - control[1]]).unwrap_or(p0);
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
                let arc = arc_points(current, end, values[0], values[1], values[2], values[3] != 0.0, values[4] != 0.0, steps);
                points.extend(arc.into_iter().skip(1));
                current = end;
                last_cubic_control = None;
                last_quadratic_control = None;
            }
        }
    }
    if points.len() > 1 { paths.push(points); }
    paths
}

/// Converts flattened SVG centerlines into a triangle-list stroke mesh.
/// `width` is the half-width in the caller's coordinate system.
pub fn stroke_paths(paths: &[Vec<[f32; 2]>], width: f32) -> Vec<[f32; 3]> {
    paths.iter().flat_map(|path| stroke_polyline(path, width)).collect()
}

fn stroke_polyline(points: &[[f32; 2]], width: f32) -> Vec<[f32; 3]> {
    if points.len() < 2 { return Vec::new(); }
    let closed = points.first() == points.last();
    let count = if closed { points.len() - 1 } else { points.len() };
    if count < 2 { return Vec::new(); }
    if !closed && count == 2 {
        let dx = points[1][0] - points[0][0];
        let dy = points[1][1] - points[0][1];
        if (dx * dx + dy * dy).sqrt() < width * 2.0 {
            let center = [(points[0][0] + points[1][0]) * 0.5, (points[0][1] + points[1][1]) * 0.5];
            let mut dot = Vec::new();
            add_round_cap(&mut dot, center, width);
            return dot;
        }
    }
    let mut offsets = Vec::with_capacity(count);
    for index in 0..count {
        let point = points[index];
        let previous = if index == 0 { if closed { points[count - 1] } else { points[1] } } else { points[index - 1] };
        let next = if index + 1 == count { if closed { points[0] } else { points[count - 2] } } else { points[index + 1] };
        let incoming = normalize([point[0] - previous[0], point[1] - previous[1]]);
        let outgoing = normalize([next[0] - point[0], next[1] - point[1]]);
        let incoming_normal = perp(incoming);
        let outgoing_normal = perp(outgoing);
        let offset = if !closed && index == 0 { scale(outgoing_normal, width) } else if !closed && index + 1 == count { scale(incoming_normal, width) } else {
            let sum = normalize([incoming_normal[0] + outgoing_normal[0], incoming_normal[1] + outgoing_normal[1]]);
            let denominator = sum[0] * outgoing_normal[0] + sum[1] * outgoing_normal[1];
            if denominator.abs() < 0.25 { scale(outgoing_normal, width) } else { scale(sum, (width / denominator).clamp(-width * 4.0, width * 4.0)) }
        };
        offsets.push(offset);
    }
    let mut positions = Vec::with_capacity(count * 12);
    let segments = if closed { count } else { count - 1 };
    for index in 0..segments {
        let next = (index + 1) % count;
        let left_a = [points[index][0] + offsets[index][0], points[index][1] + offsets[index][1], 0.0];
        let right_a = [points[index][0] - offsets[index][0], points[index][1] - offsets[index][1], 0.0];
        let left_b = [points[next][0] + offsets[next][0], points[next][1] + offsets[next][1], 0.0];
        let right_b = [points[next][0] - offsets[next][0], points[next][1] - offsets[next][1], 0.0];
        positions.extend([left_a, right_a, left_b, right_a, right_b, left_b]);
    }
    if !closed {
        add_round_cap(&mut positions, points[0], width);
        add_round_cap(&mut positions, points[count - 1], width);
    }
    positions
}

fn normalize(vector: [f32; 2]) -> [f32; 2] {
    let length = (vector[0] * vector[0] + vector[1] * vector[1]).sqrt();
    if length <= f32::EPSILON { [0.0, 0.0] } else { [vector[0] / length, vector[1] / length] }
}

fn scale(vector: [f32; 2], amount: f32) -> [f32; 2] { [vector[0] * amount, vector[1] * amount] }

fn perp(vector: [f32; 2]) -> [f32; 2] { [-vector[1], vector[0]] }

fn add_round_cap(positions: &mut Vec<[f32; 3]>, point: [f32; 2], width: f32) {
    // Keep caps coplanar with the stroke strip. A separate depth offset can
    // make tiny cap geometry disappear when the renderer depth-tests 2D work.
    const JOIN_Z: f32 = 0.0;
    for index in 0..12 {
        let a = index as f32 * std::f32::consts::TAU / 12.0;
        let b = (index + 1) as f32 * std::f32::consts::TAU / 12.0;
        positions.extend([
            [point[0], point[1], JOIN_Z],
            [point[0] + a.cos() * width, point[1] + a.sin() * width, JOIN_Z],
            [point[0] + b.cos() * width, point[1] + b.sin() * width, JOIN_Z],
        ]);
    }
}

fn point(relative: bool, current: [f32; 2], x: f32, y: f32) -> [f32; 2] {
    if relative { [current[0] + x, current[1] + y] } else { [x, y] }
}

fn cubic(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2], t: f32) -> [f32; 2] {
    let u = 1.0 - t;
    [
        u.powi(3) * a[0] + 3.0 * u.powi(2) * t * b[0] + 3.0 * u * t.powi(2) * c[0] + t.powi(3) * d[0],
        u.powi(3) * a[1] + 3.0 * u.powi(2) * t * b[1] + 3.0 * u * t.powi(2) * c[1] + t.powi(3) * d[1],
    ]
}

fn quadratic(a: [f32; 2], b: [f32; 2], c: [f32; 2], t: f32) -> [f32; 2] {
    let u = 1.0 - t;
    [u * u * a[0] + 2.0 * u * t * b[0] + t * t * c[0], u * u * a[1] + 2.0 * u * t * b[1] + t * t * c[1]]
}

fn arc_points(
    start: [f32; 2], end: [f32; 2], rx: f32, ry: f32, rotation: f32,
    large_arc: bool, sweep: bool, steps: usize,
) -> Vec<[f32; 2]> {
    if start == end || rx == 0.0 || ry == 0.0 { return vec![start, end]; }
    let phi = rotation.to_radians();
    let (sin_phi, cos_phi) = phi.sin_cos();
    let mut rx = rx.abs();
    let mut ry = ry.abs();
    let dx = (start[0] - end[0]) * 0.5;
    let dy = (start[1] - end[1]) * 0.5;
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;
    let radii_scale = (x1p * x1p / (rx * rx) + y1p * y1p / (ry * ry)).sqrt().max(1.0);
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
    let vector = |x: f32, y: f32| [ (x - cxp) / rx, (y - cyp) / ry ];
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
    if !sweep && delta > 0.0 { delta -= std::f32::consts::TAU; }
    if sweep && delta < 0.0 { delta += std::f32::consts::TAU; }
    let mut points: Vec<_> = (0..=steps).map(|index| {
        let t = start_angle + delta * index as f32 / steps as f32;
        [center[0] + rx * cos_phi * t.cos() - ry * sin_phi * t.sin(), center[1] + rx * sin_phi * t.cos() + ry * cos_phi * t.sin()]
    }).collect();
    if let Some(last) = points.last_mut() { *last = end; }
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
        if character.is_ascii_alphabetic() {
            flush(&mut tokens, &mut number);
            tokens.push(SvgToken::Command(character));
        } else if character == '.' && number.contains('.') && !number.contains('e') && !number.contains('E') {
            flush(&mut tokens, &mut number);
            number.push(character);
        } else if character.is_ascii_digit() || matches!(character, '.' | 'e' | 'E') {
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
    use super::{flatten_path, parse_path, stroke_paths, tokenize_path, SvgPathCommand, SvgToken};

    #[test]
    fn preserves_compact_signed_numbers() {
        assert_eq!(
            tokenize_path("M20 6 9 17l-5-5"),
            vec![
                SvgToken::Command('M'), SvgToken::Number(20.0), SvgToken::Number(6.0),
                SvgToken::Number(9.0), SvgToken::Number(17.0), SvgToken::Command('l'),
                SvgToken::Number(-5.0), SvgToken::Number(-5.0),
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
        assert!(mesh.iter().all(|vertex| vertex.iter().all(|value| value.is_finite())));
    }

    #[test]
    fn lucide_astroid_arc_geometry_stays_inside_viewbox() {
        let data = "M12.983 21.186a1 1 0 0 1-1.966 0 10 10 0 0 0-8.203-8.203 1 1 0 0 1 0-1.966 10 10 0 0 0 8.203-8.203 1 1 0 0 1 1.966 0 10 10 0 0 0 8.203 8.203 1 1 0 0 1 0 1.966 10 10 0 0 0-8.203 8.203";
        let commands = parse_path(data).expect("astroid path should parse");
        let points = flatten_path(&commands, 12).into_iter().next().expect("astroid path should flatten");
        let min_x = points.iter().map(|point| point[0]).fold(f32::INFINITY, f32::min);
        let max_x = points.iter().map(|point| point[0]).fold(f32::NEG_INFINITY, f32::max);
        let min_y = points.iter().map(|point| point[1]).fold(f32::INFINITY, f32::min);
        let max_y = points.iter().map(|point| point[1]).fold(f32::NEG_INFINITY, f32::max);
        assert!(min_x >= -0.01 && max_x <= 24.01 && min_y >= -0.01 && max_y <= 24.01, "bounds x={min_x}..{max_x}, y={min_y}..{max_y}");
    }

    #[test]
    fn tiny_lucide_control_stroke_produces_a_cap() {
        let commands = parse_path("M6 8h.01").expect("tiny Lucide path should parse");
        let paths = flatten_path(&commands, 32);
        let mesh = stroke_paths(&paths, 1.0 / 32.0);
        assert!(!mesh.is_empty());
        assert!(mesh.iter().all(|vertex| vertex[2] == 0.0));
    }
}
