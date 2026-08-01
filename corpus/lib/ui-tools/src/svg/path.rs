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
        } else if character.is_ascii_digit()
            || matches!(character, '.' | 'e' | 'E')
            || (matches!(character, '-' | '+') && (number.ends_with('e') || number.ends_with('E')))
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

pub(super) fn flatten_path(commands: &[SvgPathCommand], subdivisions: usize) -> Vec<Vec<[f32; 2]>> {
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
                    ArcParameters {
                        radii: [values[0], values[1]],
                        rotation: values[2],
                        large_arc: values[3] != 0.0,
                        sweep: values[4] != 0.0,
                    },
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

struct ArcParameters {
    radii: [f32; 2],
    rotation: f32,
    large_arc: bool,
    sweep: bool,
}

fn arc_points(
    start: [f32; 2],
    end: [f32; 2],
    parameters: ArcParameters,
    steps: usize,
) -> Vec<[f32; 2]> {
    let [mut rx, mut ry] = parameters.radii;
    if start == end || rx == 0.0 || ry == 0.0 {
        return vec![start, end];
    }
    let phi = parameters.rotation.to_radians();
    let (sin_phi, cos_phi) = phi.sin_cos();
    rx = rx.abs();
    ry = ry.abs();
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
    let sign = if parameters.large_arc == parameters.sweep {
        -1.0
    } else {
        1.0
    };
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
    if !parameters.sweep && delta > 0.0 {
        delta -= std::f32::consts::TAU;
    }
    if parameters.sweep && delta < 0.0 {
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
