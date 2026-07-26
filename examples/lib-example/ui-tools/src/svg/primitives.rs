pub(super) fn parse_svg_point_numbers(values: &str, element: &str) -> Result<Vec<f32>, String> {
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

pub(super) fn svg_rectangle(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rx: f32,
    ry: f32,
) -> Vec<[f32; 2]> {
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
pub(super) fn stroke_paths(paths: &[Vec<[f32; 2]>], width: f32) -> Vec<[f32; 3]> {
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
