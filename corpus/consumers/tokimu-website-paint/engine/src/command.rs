use crate::{DocumentError, EditableRasterDocument, Rgba8};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

pub const MAX_STROKE_DIAMETER: u8 = 64;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PixelPoint {
    pub x: u32,
    pub y: u32,
}

/// Signed canvas coordinates used by shape commands before clipping them to a
/// document. Freehand input remains document-local because it is sampled from
/// an already bounded painting surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CanvasPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PixelBounds {
    pub min: PixelPoint,
    pub max: PixelPoint,
}

impl PixelBounds {
    fn from_point(point: PixelPoint) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    fn include(&mut self, point: PixelPoint) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum PaintCommand {
    PencilStroke {
        points: Vec<PixelPoint>,
        color: Rgba8,
    },
    BrushStroke {
        points: Vec<PixelPoint>,
        color: Rgba8,
        diameter: u8,
    },
    EraseStroke {
        points: Vec<PixelPoint>,
    },
    FloodFill {
        origin: PixelPoint,
        replacement: Rgba8,
    },
    DrawLine {
        start: CanvasPoint,
        end: CanvasPoint,
        color: Rgba8,
    },
    DrawRectangle {
        start: CanvasPoint,
        end: CanvasPoint,
        outline: Rgba8,
        fill: Option<Rgba8>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditObservation {
    pub schema: u32,
    pub command: &'static str,
    pub changed_pixels: usize,
    pub changed_bounds: Option<PixelBounds>,
    pub revision: u64,
    pub dirty: bool,
    pub no_op: bool,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum CommandError {
    #[error("a stroke requires at least one point")]
    EmptyStroke,
    #[error("stroke diameter must be in 1..={maximum}, got {actual}")]
    InvalidStrokeDiameter { actual: u8, maximum: u8 },
    #[error("command point ({x}, {y}) is outside document bounds {width}x{height}")]
    PointOutOfBounds {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    #[error(transparent)]
    Document(#[from] DocumentError),
}

pub fn sample_color(
    document: &EditableRasterDocument,
    point: PixelPoint,
) -> Result<Rgba8, CommandError> {
    validate_point(document, point)?;
    document.pixel(point.x, point.y).map_err(CommandError::from)
}

pub fn apply_command(
    document: &mut EditableRasterDocument,
    command: &PaintCommand,
) -> Result<EditObservation, CommandError> {
    match command {
        PaintCommand::PencilStroke { points, color } => {
            apply_stroke(document, points, *color, "pencil-stroke")
        }
        PaintCommand::BrushStroke {
            points,
            color,
            diameter,
        } => apply_brush_stroke(document, points, *color, *diameter),
        PaintCommand::EraseStroke { points } => {
            apply_stroke(document, points, Rgba8::TRANSPARENT, "erase-stroke")
        }
        PaintCommand::FloodFill {
            origin,
            replacement,
        } => apply_flood_fill(document, *origin, *replacement),
        PaintCommand::DrawLine { start, end, color } => {
            apply_clipped_line(document, *start, *end, *color, "draw-line")
        }
        PaintCommand::DrawRectangle {
            start,
            end,
            outline,
            fill,
        } => apply_rectangle(document, *start, *end, *outline, *fill),
    }
}

fn apply_brush_stroke(
    document: &mut EditableRasterDocument,
    points: &[PixelPoint],
    color: Rgba8,
    diameter: u8,
) -> Result<EditObservation, CommandError> {
    if points.is_empty() {
        return Err(CommandError::EmptyStroke);
    }
    if diameter == 0 || diameter > MAX_STROKE_DIAMETER {
        return Err(CommandError::InvalidStrokeDiameter {
            actual: diameter,
            maximum: MAX_STROKE_DIAMETER,
        });
    }
    for point in points {
        validate_point(document, *point)?;
    }

    let mut centerline = BTreeSet::new();
    centerline.insert(points[0]);
    for segment in points.windows(2) {
        rasterize_line(segment[0], segment[1], &mut centerline);
    }

    let radius = i32::from(diameter) / 2;
    let mut covered = BTreeSet::new();
    for center in centerline {
        let center_x = center.x as i32;
        let center_y = center.y as i32;
        for y in (center_y - radius)..=(center_y + radius) {
            for x in (center_x - radius)..=(center_x + radius) {
                let delta_x = x - center_x;
                let delta_y = y - center_y;
                if delta_x * delta_x + delta_y * delta_y <= radius * radius
                    && x >= 0
                    && y >= 0
                    && x < document.width() as i32
                    && y < document.height() as i32
                {
                    covered.insert(PixelPoint {
                        x: x as u32,
                        y: y as u32,
                    });
                }
            }
        }
    }
    apply_covered_points(document, covered, color, "brush-stroke")
}

fn apply_clipped_line(
    document: &mut EditableRasterDocument,
    start: CanvasPoint,
    end: CanvasPoint,
    color: Rgba8,
    command: &'static str,
) -> Result<EditObservation, CommandError> {
    let Some((start, end)) = clip_line_to_document(document, start, end) else {
        return Ok(commit_observation(document, command, 0, None));
    };

    let mut covered = BTreeSet::new();
    rasterize_signed_line(start, end, &mut covered);
    apply_covered_points(document, covered, color, command)
}

fn apply_rectangle(
    document: &mut EditableRasterDocument,
    start: CanvasPoint,
    end: CanvasPoint,
    outline: Rgba8,
    fill: Option<Rgba8>,
) -> Result<EditObservation, CommandError> {
    let min_x = start.x.min(end.x).max(0);
    let min_y = start.y.min(end.y).max(0);
    let max_x = start.x.max(end.x).min(document.width() as i32 - 1);
    let max_y = start.y.max(end.y).min(document.height() as i32 - 1);
    if min_x > max_x || min_y > max_y {
        return Ok(commit_observation(document, "draw-rectangle", 0, None));
    }

    let mut covered = BTreeMap::new();
    if let Some(fill) = fill {
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                covered.insert(
                    PixelPoint {
                        x: x as u32,
                        y: y as u32,
                    },
                    fill,
                );
            }
        }
    }
    for x in min_x..=max_x {
        covered.insert(
            PixelPoint {
                x: x as u32,
                y: min_y as u32,
            },
            outline,
        );
        covered.insert(
            PixelPoint {
                x: x as u32,
                y: max_y as u32,
            },
            outline,
        );
    }
    for y in min_y..=max_y {
        covered.insert(
            PixelPoint {
                x: min_x as u32,
                y: y as u32,
            },
            outline,
        );
        covered.insert(
            PixelPoint {
                x: max_x as u32,
                y: y as u32,
            },
            outline,
        );
    }

    apply_colored_points(document, covered, "draw-rectangle")
}

fn apply_stroke(
    document: &mut EditableRasterDocument,
    points: &[PixelPoint],
    color: Rgba8,
    command: &'static str,
) -> Result<EditObservation, CommandError> {
    if points.is_empty() {
        return Err(CommandError::EmptyStroke);
    }
    for point in points {
        validate_point(document, *point)?;
    }

    let mut covered = BTreeSet::new();
    covered.insert(points[0]);
    for segment in points.windows(2) {
        rasterize_line(segment[0], segment[1], &mut covered);
    }

    let mut changed_pixels = 0;
    let mut changed_bounds = None;
    for point in covered {
        if document.replace_pixel_if_different(point.x, point.y, color)? {
            changed_pixels += 1;
            include_changed(&mut changed_bounds, point);
        }
    }

    Ok(commit_observation(
        document,
        command,
        changed_pixels,
        changed_bounds,
    ))
}

fn apply_flood_fill(
    document: &mut EditableRasterDocument,
    origin: PixelPoint,
    replacement: Rgba8,
) -> Result<EditObservation, CommandError> {
    validate_point(document, origin)?;
    let target = sample_color(document, origin)?;
    if target == replacement {
        return Ok(commit_observation(document, "flood-fill", 0, None));
    }

    let width = document.width() as usize;
    let height = document.height() as usize;
    let mut visited = vec![false; width * height];
    let mut pending = VecDeque::from([origin]);
    let mut changed_pixels = 0;
    let mut changed_bounds = None;

    while let Some(point) = pending.pop_front() {
        let index = point.y as usize * width + point.x as usize;
        if visited[index] {
            continue;
        }
        visited[index] = true;
        if sample_color(document, point)? != target {
            continue;
        }

        if document.replace_pixel_if_different(point.x, point.y, replacement)? {
            changed_pixels += 1;
            include_changed(&mut changed_bounds, point);
        }

        if point.x > 0 {
            pending.push_back(PixelPoint {
                x: point.x - 1,
                y: point.y,
            });
        }
        if point.x + 1 < document.width() {
            pending.push_back(PixelPoint {
                x: point.x + 1,
                y: point.y,
            });
        }
        if point.y > 0 {
            pending.push_back(PixelPoint {
                x: point.x,
                y: point.y - 1,
            });
        }
        if point.y + 1 < document.height() {
            pending.push_back(PixelPoint {
                x: point.x,
                y: point.y + 1,
            });
        }
    }

    Ok(commit_observation(
        document,
        "flood-fill",
        changed_pixels,
        changed_bounds,
    ))
}

fn validate_point(
    document: &EditableRasterDocument,
    point: PixelPoint,
) -> Result<(), CommandError> {
    if point.x >= document.width() || point.y >= document.height() {
        return Err(CommandError::PointOutOfBounds {
            x: point.x,
            y: point.y,
            width: document.width(),
            height: document.height(),
        });
    }
    Ok(())
}

fn rasterize_line(start: PixelPoint, end: PixelPoint, output: &mut BTreeSet<PixelPoint>) {
    rasterize_signed_line(
        CanvasPoint {
            x: start.x as i32,
            y: start.y as i32,
        },
        CanvasPoint {
            x: end.x as i32,
            y: end.y as i32,
        },
        output,
    );
}

fn rasterize_signed_line(start: CanvasPoint, end: CanvasPoint, output: &mut BTreeSet<PixelPoint>) {
    let mut x = i64::from(start.x);
    let mut y = i64::from(start.y);
    let end_x = i64::from(end.x);
    let end_y = i64::from(end.y);
    let delta_x = (end_x - x).abs();
    let step_x = if x < end_x { 1 } else { -1 };
    let delta_y = -(end_y - y).abs();
    let step_y = if y < end_y { 1 } else { -1 };
    let mut error = delta_x + delta_y;

    loop {
        output.insert(PixelPoint {
            x: x as u32,
            y: y as u32,
        });
        if x == end_x && y == end_y {
            break;
        }
        let doubled_error = 2 * error;
        if doubled_error >= delta_y {
            error += delta_y;
            x += step_x;
        }
        if doubled_error <= delta_x {
            error += delta_x;
            y += step_y;
        }
    }
}

fn apply_covered_points(
    document: &mut EditableRasterDocument,
    covered: BTreeSet<PixelPoint>,
    color: Rgba8,
    command: &'static str,
) -> Result<EditObservation, CommandError> {
    apply_colored_points(
        document,
        covered.into_iter().map(|point| (point, color)).collect(),
        command,
    )
}

fn apply_colored_points(
    document: &mut EditableRasterDocument,
    covered: BTreeMap<PixelPoint, Rgba8>,
    command: &'static str,
) -> Result<EditObservation, CommandError> {
    let mut changed_pixels = 0;
    let mut changed_bounds = None;
    for (point, color) in covered {
        if document.replace_pixel_if_different(point.x, point.y, color)? {
            changed_pixels += 1;
            include_changed(&mut changed_bounds, point);
        }
    }
    Ok(commit_observation(
        document,
        command,
        changed_pixels,
        changed_bounds,
    ))
}

fn clip_line_to_document(
    document: &EditableRasterDocument,
    start: CanvasPoint,
    end: CanvasPoint,
) -> Option<(CanvasPoint, CanvasPoint)> {
    const LEFT: u8 = 1;
    const RIGHT: u8 = 2;
    const TOP: u8 = 4;
    const BOTTOM: u8 = 8;

    fn out_code(point: CanvasPoint, max_x: i32, max_y: i32) -> u8 {
        let mut code = 0;
        if point.x < 0 {
            code |= LEFT;
        } else if point.x > max_x {
            code |= RIGHT;
        }
        if point.y < 0 {
            code |= TOP;
        } else if point.y > max_y {
            code |= BOTTOM;
        }
        code
    }

    let max_x = document.width() as i32 - 1;
    let max_y = document.height() as i32 - 1;
    let mut start = start;
    let mut end = end;
    loop {
        let start_code = out_code(start, max_x, max_y);
        let end_code = out_code(end, max_x, max_y);
        if start_code == 0 && end_code == 0 {
            return Some((start, end));
        }
        if start_code & end_code != 0 {
            return None;
        }

        let outside = if start_code != 0 {
            start_code
        } else {
            end_code
        };
        let delta_x = i64::from(end.x) - i64::from(start.x);
        let delta_y = i64::from(end.y) - i64::from(start.y);
        let clipped = if outside & TOP != 0 {
            CanvasPoint {
                x: (i64::from(start.x) + delta_x * -i64::from(start.y) / delta_y) as i32,
                y: 0,
            }
        } else if outside & BOTTOM != 0 {
            CanvasPoint {
                x: (i64::from(start.x)
                    + delta_x * (i64::from(max_y) - i64::from(start.y)) / delta_y)
                    as i32,
                y: max_y,
            }
        } else if outside & RIGHT != 0 {
            CanvasPoint {
                x: max_x,
                y: (i64::from(start.y)
                    + delta_y * (i64::from(max_x) - i64::from(start.x)) / delta_x)
                    as i32,
            }
        } else {
            CanvasPoint {
                x: 0,
                y: (i64::from(start.y) + delta_y * -i64::from(start.x) / delta_x) as i32,
            }
        };
        if outside == start_code {
            start = clipped;
        } else {
            end = clipped;
        }
    }
}

fn include_changed(bounds: &mut Option<PixelBounds>, point: PixelPoint) {
    if let Some(bounds) = bounds {
        bounds.include(point);
    } else {
        *bounds = Some(PixelBounds::from_point(point));
    }
}

fn commit_observation(
    document: &mut EditableRasterDocument,
    command: &'static str,
    changed_pixels: usize,
    changed_bounds: Option<PixelBounds>,
) -> EditObservation {
    if changed_pixels > 0 {
        document.commit_edit();
    }
    EditObservation {
        schema: 1,
        command,
        changed_pixels,
        changed_bounds,
        revision: document.revision(),
        dirty: document.is_dirty(),
        no_op: changed_pixels == 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_command, sample_color, CanvasPoint, CommandError, PaintCommand, PixelBounds,
        PixelPoint, MAX_STROKE_DIAMETER,
    };
    use crate::{DocumentConfig, EditableRasterDocument, Rgba8};

    const BLACK: Rgba8 = Rgba8 {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    const WHITE: Rgba8 = Rgba8 {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 255,
    };

    fn document() -> EditableRasterDocument {
        EditableRasterDocument::blank(5, 5, BLACK, DocumentConfig::default()).unwrap()
    }

    #[test]
    fn pencil_stroke_is_deterministic_and_commits_one_transaction() {
        let command = PaintCommand::PencilStroke {
            points: vec![PixelPoint { x: 0, y: 0 }, PixelPoint { x: 4, y: 4 }],
            color: WHITE,
        };
        let mut first = document();
        let mut second = document();

        let observation = apply_command(&mut first, &command).unwrap();
        apply_command(&mut second, &command).unwrap();

        assert_eq!(observation.changed_pixels, 5);
        assert_eq!(
            observation.changed_bounds,
            Some(PixelBounds {
                min: PixelPoint { x: 0, y: 0 },
                max: PixelPoint { x: 4, y: 4 }
            })
        );
        assert_eq!(observation.revision, 1);
        assert!(observation.dirty);
        assert_eq!(
            first.observation().pixel_fingerprint,
            second.observation().pixel_fingerprint
        );
    }

    #[test]
    fn eraser_and_sample_are_provider_neutral_document_operations() {
        let mut document =
            EditableRasterDocument::blank(2, 1, WHITE, DocumentConfig::default()).unwrap();
        let observation = apply_command(
            &mut document,
            &PaintCommand::EraseStroke {
                points: vec![PixelPoint { x: 1, y: 0 }],
            },
        )
        .unwrap();

        assert_eq!(observation.command, "erase-stroke");
        assert_eq!(
            sample_color(&document, PixelPoint { x: 0, y: 0 }).unwrap(),
            WHITE
        );
        assert_eq!(
            sample_color(&document, PixelPoint { x: 1, y: 0 }).unwrap(),
            Rgba8::TRANSPARENT
        );
    }

    #[test]
    fn fill_is_exact_four_connected_and_non_recursive() {
        let mut document = document();
        apply_command(
            &mut document,
            &PaintCommand::PencilStroke {
                points: vec![PixelPoint { x: 2, y: 0 }, PixelPoint { x: 2, y: 4 }],
                color: WHITE,
            },
        )
        .unwrap();
        let observation = apply_command(
            &mut document,
            &PaintCommand::FloodFill {
                origin: PixelPoint { x: 0, y: 2 },
                replacement: Rgba8::TRANSPARENT,
            },
        )
        .unwrap();

        assert_eq!(observation.changed_pixels, 10);
        assert_eq!(
            sample_color(&document, PixelPoint { x: 3, y: 2 }).unwrap(),
            BLACK
        );
        assert_eq!(
            sample_color(&document, PixelPoint { x: 2, y: 2 }).unwrap(),
            WHITE
        );
    }

    #[test]
    fn fill_stress_remains_iterative_for_an_odd_sized_document() {
        let width = 63;
        let height = 47;
        let mut document =
            EditableRasterDocument::blank(width, height, BLACK, DocumentConfig::default()).unwrap();

        let observation = apply_command(
            &mut document,
            &PaintCommand::FloodFill {
                origin: PixelPoint { x: 0, y: 0 },
                replacement: WHITE,
            },
        )
        .unwrap();

        assert_eq!(observation.changed_pixels, (width * height) as usize);
        assert_eq!(
            sample_color(
                &document,
                PixelPoint {
                    x: width - 1,
                    y: height - 1
                }
            )
            .unwrap(),
            WHITE
        );
    }

    #[test]
    fn no_op_does_not_change_revision_or_dirty_state() {
        let mut document = document();
        let observation = apply_command(
            &mut document,
            &PaintCommand::PencilStroke {
                points: vec![PixelPoint { x: 1, y: 1 }],
                color: BLACK,
            },
        )
        .unwrap();

        assert!(observation.no_op);
        assert_eq!(observation.changed_pixels, 0);
        assert_eq!(observation.revision, 0);
        assert!(!observation.dirty);
    }

    #[test]
    fn line_clips_crossing_canvas_edges_without_writing_outside_the_document() {
        let mut document = document();
        let observation = apply_command(
            &mut document,
            &PaintCommand::DrawLine {
                start: CanvasPoint { x: -10, y: 2 },
                end: CanvasPoint { x: 10, y: 2 },
                color: WHITE,
            },
        )
        .unwrap();

        assert_eq!(observation.command, "draw-line");
        assert_eq!(observation.changed_pixels, 5);
        assert_eq!(
            observation.changed_bounds,
            Some(PixelBounds {
                min: PixelPoint { x: 0, y: 2 },
                max: PixelPoint { x: 4, y: 2 },
            })
        );
        for x in 0..5 {
            assert_eq!(
                sample_color(&document, PixelPoint { x, y: 2 }).unwrap(),
                WHITE
            );
        }
    }

    #[test]
    fn rectangle_clips_and_applies_fill_before_its_outline() {
        let mut document = document();
        let observation = apply_command(
            &mut document,
            &PaintCommand::DrawRectangle {
                start: CanvasPoint { x: -2, y: 1 },
                end: CanvasPoint { x: 2, y: 6 },
                outline: WHITE,
                fill: Some(Rgba8::TRANSPARENT),
            },
        )
        .unwrap();

        assert_eq!(observation.command, "draw-rectangle");
        assert_eq!(observation.changed_pixels, 12);
        assert_eq!(
            sample_color(&document, PixelPoint { x: 1, y: 2 }).unwrap(),
            Rgba8::TRANSPARENT
        );
        assert_eq!(
            sample_color(&document, PixelPoint { x: 0, y: 2 }).unwrap(),
            WHITE
        );
        assert_eq!(
            sample_color(&document, PixelPoint { x: 2, y: 4 }).unwrap(),
            WHITE
        );
    }

    #[test]
    fn brush_diameter_is_bounded_and_clips_at_the_document_edge() {
        let mut document = document();
        let observation = apply_command(
            &mut document,
            &PaintCommand::BrushStroke {
                points: vec![PixelPoint { x: 0, y: 0 }],
                color: WHITE,
                diameter: 3,
            },
        )
        .unwrap();

        assert_eq!(observation.changed_pixels, 3);
        assert_eq!(
            sample_color(&document, PixelPoint { x: 0, y: 0 }).unwrap(),
            WHITE
        );
        assert_eq!(
            sample_color(&document, PixelPoint { x: 1, y: 0 }).unwrap(),
            WHITE
        );
        assert_eq!(
            sample_color(&document, PixelPoint { x: 0, y: 1 }).unwrap(),
            WHITE
        );

        let before = document.observation().pixel_fingerprint;
        assert_eq!(
            apply_command(
                &mut document,
                &PaintCommand::BrushStroke {
                    points: vec![PixelPoint { x: 1, y: 1 }],
                    color: WHITE,
                    diameter: MAX_STROKE_DIAMETER + 1,
                },
            ),
            Err(CommandError::InvalidStrokeDiameter {
                actual: MAX_STROKE_DIAMETER + 1,
                maximum: MAX_STROKE_DIAMETER,
            })
        );
        assert_eq!(document.observation().pixel_fingerprint, before);
    }
}
