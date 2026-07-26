use super::{tokenize_path, SvgToken};

/// A compact SVG affine transform in the standard `[a b c d e f]` form.
/// It remains private to SVG lowering: `VectorPath` contains only final,
/// provider-neutral coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SvgAffine {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl SvgAffine {
    pub(super) const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub(super) fn apply(self, point: [f32; 2]) -> [f32; 2] {
        [
            self.a * point[0] + self.c * point[1] + self.e,
            self.b * point[0] + self.d * point[1] + self.f,
        ]
    }

    /// Extends this parent/list transform with a local SVG transform.
    /// With column vectors, the resulting transform applies `local` first and
    /// then this transform, matching nested SVG coordinate systems.
    pub(super) fn compose(self, local: Self) -> Self {
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

pub(super) fn parse_svg_transform(value: &str) -> Result<SvgAffine, String> {
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
            "skewX" | "skewY" => match values.as_slice() {
                [degrees] => {
                    let tangent = degrees.to_radians().tan();
                    if !tangent.is_finite() {
                        return Err(format!(
                            "SVG transform '{name}' produces a non-finite coefficient"
                        ));
                    }
                    if name == "skewX" {
                        SvgAffine {
                            c: tangent,
                            ..SvgAffine::IDENTITY
                        }
                    } else {
                        SvgAffine {
                            b: tangent,
                            ..SvgAffine::IDENTITY
                        }
                    }
                }
                _ => return Err(format!("SVG transform '{name}' requires one number")),
            },
            _ => return Err(format!("SVG transform '{name}' is unsupported")),
        };
        transform = transform.compose(function);
        remainder = remainder[arguments_end + 1..].trim_start();
    }

    Ok(transform)
}

pub(super) fn parse_svg_transform_numbers(value: &str, function: &str) -> Result<Vec<f32>, String> {
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
