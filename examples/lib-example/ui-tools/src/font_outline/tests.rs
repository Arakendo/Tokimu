use super::*;
use crate::{UiFontRasterizer, VectorPath};

fn noto_fixture() -> UiFontRasterizer {
    UiFontRasterizer::from_bytes(include_bytes!("../../fixtures/NotoSans-Regular.otf").to_vec())
        .expect("checked-in OTF fixture")
}

fn prepared_inter_fixture() -> Option<UiFontRasterizer> {
    let source =
        crate::UiFontSource::from_prepared_corpus("inter", crate::UiFontFormat::Ttf).ok()?;
    UiFontRasterizer::from_bytes(source.bytes).ok()
}

fn prepared_provider_fixture(provider: &str, candidates: &[&str]) -> Option<UiFontRasterizer> {
    candidates.iter().find_map(|filename| {
        let path = format!("../../../../target/glyph-corpus/fonts/{provider}/{filename}");
        let bytes = std::fs::read(path).ok()?;
        UiFontRasterizer::from_bytes(bytes).ok()
    })
}

#[test]
fn extracts_provider_neutral_otf_outline() {
    let outline = noto_fixture().outline('A').expect("A outline");

    assert_eq!(outline.character, 'A');
    assert!(outline.units_per_em > 0.0);
    assert!(!outline.contours.is_empty());
    assert!(outline.contours.iter().all(|contour| contour.closed));
    assert!(outline.is_finite());
}

#[test]
fn prepared_ttf_uses_the_same_outline_contract() {
    let Some(font) = prepared_inter_fixture() else {
        return;
    };
    let outline = font.outline('A').expect("Inter A outline");

    assert!(outline.is_finite());
    assert!(!outline.contours.is_empty());
    assert!(outline.contours.iter().all(|contour| contour.closed));
}

#[test]
fn prepared_font_providers_share_the_outline_contract() {
    let providers = [
        (
            "inter",
            ["Inter[opsz,wght].ttf", "Inter-Regular.ttf"].as_slice(),
        ),
        (
            "jetbrains-mono",
            ["JetBrainsMono-Regular.otf", "JetBrainsMono-Regular.ttf"].as_slice(),
        ),
        (
            "noto",
            ["NotoSans-VF.ttf", "NotoSans-Regular.ttf"].as_slice(),
        ),
    ];

    for (provider, candidates) in providers {
        let Some(font) = prepared_provider_fixture(provider, candidates) else {
            continue;
        };
        let outline = font
            .outline('A')
            .unwrap_or_else(|error| panic!("{provider} A outline failed: {error:?}"));

        assert!(outline.is_finite(), "{provider} outline must be finite");
        assert!(!outline.contours.is_empty(), "{provider} needs contours");
        assert!(
            outline.contours.iter().all(|contour| contour.closed),
            "{provider} contours must be closed"
        );
    }
}

#[test]
fn preserves_multiple_contours_for_counter_glyph() {
    let outline = noto_fixture().outline('O').expect("O outline");

    assert!(outline.contours.len() >= 2);
    assert!(outline.contours.iter().all(|contour| contour.closed));
}

#[test]
fn preserves_native_curve_segments() {
    let outline = noto_fixture().outline('S').expect("S outline");

    assert!(outline.contours.iter().any(|contour| {
        contour.segments.iter().any(|segment| {
            matches!(
                segment,
                UiGlyphOutlineSegment::QuadTo { .. } | UiGlyphOutlineSegment::CubicTo { .. }
            )
        })
    }));
}

#[test]
fn whitespace_reports_missing_outline() {
    let error = noto_fixture().outline(' ').expect_err("space has no ink");

    assert_eq!(error.kind, UiGlyphOutlineDiagnosticKind::MissingOutline);
    assert_eq!(error.character, ' ');
}

#[test]
fn adapter_scales_and_positions_without_using_outline_bounds() {
    let outline = noto_fixture().outline('A').expect("A outline");
    let unshifted = outline
        .to_vector_path(UiGlyphVectorOptions::new(32.0, [0.0, 0.0], 0.25))
        .expect("unshifted vector path");
    let shifted = outline
        .to_vector_path(UiGlyphVectorOptions::new(32.0, [10.0, 20.0], 0.25))
        .expect("shifted vector path");
    let original_bounds = unshifted.bounds().expect("unshifted bounds");
    let shifted_bounds = shifted.bounds().expect("shifted bounds");

    assert_eq!(shifted.contours.len(), outline.contours.len());
    assert!((shifted_bounds.0[0] - original_bounds.0[0] - 10.0).abs() < 1.0e-3);
    assert!((shifted_bounds.0[1] - original_bounds.0[1] - 20.0).abs() < 1.0e-3);
    assert!(shifted.is_finite());
}

#[test]
fn adapter_preserves_counter_contours_and_closure() {
    let outline = noto_fixture().outline('O').expect("O outline");
    let path = outline
        .to_vector_path(UiGlyphVectorOptions::new(48.0, [0.0, 0.0], 0.2))
        .expect("vector path");

    assert!(path.contours.len() >= 2);
    assert!(path.contours.iter().all(|contour| contour.closed));
    assert!(path
        .contours
        .iter()
        .all(|contour| contour.points.len() >= 3));
}

#[test]
fn smaller_tolerance_produces_at_least_as_many_curve_points() {
    let outline = noto_fixture().outline('S').expect("S outline");
    let coarse = outline
        .to_vector_path(UiGlyphVectorOptions::new(64.0, [0.0, 0.0], 1.0))
        .expect("coarse vector path");
    let fine = outline
        .to_vector_path(UiGlyphVectorOptions::new(64.0, [0.0, 0.0], 0.1))
        .expect("fine vector path");
    let point_count = |path: &VectorPath| {
        path.contours
            .iter()
            .map(|contour| contour.points.len())
            .sum::<usize>()
    };

    assert!(point_count(&fine) >= point_count(&coarse));
}

#[test]
fn adapter_rejects_invalid_presentation_policy() {
    let outline = noto_fixture().outline('A').expect("A outline");
    let error = outline
        .to_vector_path(UiGlyphVectorOptions::new(0.0, [0.0, 0.0], 0.25))
        .expect_err("zero scale must fail");

    assert_eq!(error.kind, UiGlyphVectorDiagnosticKind::InvalidScale);
}

#[test]
fn fill_topology_reports_counter_and_convex_glyphs_before_tessellation() {
    let options = UiGlyphVectorOptions::new(48.0, [0.0, 0.0], 0.2);
    assert_eq!(
        noto_fixture().outline('O').unwrap().fill_topology(options),
        Ok(UiGlyphFillTopology::MultipleContours)
    );
    assert_eq!(
        noto_fixture().outline('A').unwrap().fill_topology(options),
        Ok(UiGlyphFillTopology::MultipleContours)
    );
}

#[test]
fn counter_glyph_can_use_general_fill_adapter() {
    let outline = noto_fixture().outline('O').expect("O outline");
    let path = outline
        .to_vector_path(UiGlyphVectorOptions::new(64.0, [0.0, 0.0], 0.2))
        .expect("O vector path");
    let triangles = crate::tessellate_general_fill(&path).expect("O fill");

    assert!(!triangles.is_empty());
    assert!(triangles
        .iter()
        .all(|point| { point[0].is_finite() && point[1].is_finite() }));

    let (min, max) = path.bounds().expect("O path bounds");
    assert!(triangles.iter().all(|point| {
        point[0] >= min[0] - POINT_EPSILON
            && point[0] <= max[0] + POINT_EPSILON
            && point[1] >= min[1] - POINT_EPSILON
            && point[1] <= max[1] + POINT_EPSILON
    }));
}

#[test]
fn counter_corpus_glyphs_lower_through_general_fill() {
    let font = noto_fixture();
    for character in ['B', 'P', 'Q', 'a', 'e', 'g', '0', '8', '@'] {
        let outline = font.outline(character).expect("counter glyph outline");
        let path = outline
            .to_vector_path(UiGlyphVectorOptions::new(56.0, [0.0, 0.0], 0.2))
            .expect("counter glyph vector path");
        let triangles = crate::tessellate_general_fill(&path)
            .unwrap_or_else(|error| panic!("{character} fill failed: {error}"));
        assert!(
            !triangles.is_empty(),
            "{character} should produce fill geometry"
        );
        assert!(triangles
            .iter()
            .all(|point| point[0].is_finite() && point[1].is_finite()));
    }
}

#[test]
fn prepared_inter_hard_edge_glyphs_preserve_contour_area() {
    let Some(font) = prepared_inter_fixture() else {
        return;
    };
    for character in [
        '%', '&', '2', '4', 'F', 'K', 'M', 'N', 'W', 'X', 'Z', 'h', 'k', 'r',
    ] {
        let path = font
            .outline(character)
            .expect("Inter outline")
            .to_vector_path(UiGlyphVectorOptions::new(96.0, [0.0, 0.0], 0.2))
            .expect("vector path");
        let contour_area = path
            .contours
            .iter()
            .map(|contour| polygon_area(&contour.points))
            .sum::<f32>()
            .abs();
        let triangles = crate::tessellate_general_fill(&path)
            .unwrap_or_else(|error| panic!("{character} fill failed: {error}"));
        let tessellated_area = triangles
            .chunks_exact(3)
            .map(|triangle| triangle_area(triangle[0], triangle[1], triangle[2]).abs())
            .sum::<f32>();
        let winding_signs = triangles
            .chunks_exact(3)
            .map(|triangle| triangle_area(triangle[0], triangle[1], triangle[2]).signum())
            .filter(|sign| *sign != 0.0)
            .collect::<Vec<_>>();
        let relative_area_error = (tessellated_area - contour_area).abs() / contour_area;
        assert!(
            relative_area_error < 0.08,
            "{character} fill area drifted by {relative_area_error}"
        );
        assert!(
            winding_signs.iter().all(|sign| *sign == winding_signs[0]),
            "{character} produced mixed triangle winding: {winding_signs:?}"
        );
        assert_mesh_matches_non_zero_fill(character, &path, &triangles);
    }
}

#[test]
fn prepared_inter_regression_glyphs_preserve_positioned_mesh_coverage() {
    let Some(font) = prepared_inter_fixture() else {
        return;
    };

    for character in ['F', 'K', 'k', 'M', 'e'] {
        let outline = font
            .outline(character)
            .unwrap_or_else(|error| panic!("{character} outline failed: {error:?}"));
        assert!(outline.is_finite(), "{character} outline must be finite");
        assert!(!outline.contours.is_empty(), "{character} needs contours");
        assert!(
            outline.contours.iter().all(|contour| contour.closed),
            "{character} contours must be closed"
        );

        let path = outline
            .to_vector_path(UiGlyphVectorOptions::new(96.0, [0.0, 0.0], 0.2))
            .unwrap_or_else(|error| panic!("{character} vector conversion failed: {error:?}"));
        let layout = font.layout(&character.to_string(), 96.0);
        let positioned = layout
            .glyphs
            .first()
            .unwrap_or_else(|| panic!("{character} produced no positioned glyph"));
        let triangles = font
            .tessellate_positioned_glyph(positioned, 96.0, 1.0, [0.0, 0.0], 0.2)
            .unwrap_or_else(|error| panic!("{character} tessellation failed: {error:?}"));

        assert!(!triangles.is_empty(), "{character} needs fill geometry");
        assert!(triangles
            .iter()
            .all(|point| point[0].is_finite() && point[1].is_finite()));
        assert_mesh_matches_fill_rule(character, &path, &triangles, true);
    }
}

#[test]
fn positioned_glyph_adapter_uses_layout_pen_position() {
    let font = noto_fixture();
    let layout = font.layout("AA", 48.0);
    let first = font
        .tessellate_positioned_glyph(&layout.glyphs[0], 48.0, 0.01, [-1.0, 0.5], 0.2)
        .expect("first positioned glyph");
    let second = font
        .tessellate_positioned_glyph(&layout.glyphs[1], 48.0, 0.01, [-1.0, 0.5], 0.2)
        .expect("second positioned glyph");

    assert!(!first.is_empty());
    assert!(!second.is_empty());
    let first_min_x = first
        .iter()
        .map(|point| point[0])
        .fold(f32::INFINITY, f32::min);
    let second_min_x = second
        .iter()
        .map(|point| point[0])
        .fold(f32::INFINITY, f32::min);
    let expected_delta = (layout.glyphs[1].pen_x - layout.glyphs[0].pen_x) * 0.01;
    assert!(second_min_x > first_min_x);
    assert!((second_min_x - first_min_x - expected_delta).abs() < 0.0005);
    assert!(first
        .iter()
        .chain(second.iter())
        .all(|point| point[0].is_finite() && point[1].is_finite()));
}

#[test]
fn positioned_glyph_scales_across_ui_sizes_without_invalid_geometry() {
    let font = noto_fixture();
    let mut triangle_counts = Vec::new();
    for pixels in [24.0_f32, 56.0, 96.0] {
        let layout = font.layout("O", pixels);
        let triangles = font
            .tessellate_positioned_glyph(&layout.glyphs[0], pixels, 0.01, [0.0, 0.0], 0.2)
            .unwrap_or_else(|error| panic!("{pixels}px glyph failed: {error:?}"));

        assert!(!triangles.is_empty(), "{pixels}px glyph should be visible");
        assert!(triangles
            .iter()
            .all(|point| { point[0].is_finite() && point[1].is_finite() }));
        triangle_counts.push(triangles.len() / 3);
    }

    assert!(triangle_counts[1] >= triangle_counts[0]);
    assert!(triangle_counts[2] >= triangle_counts[1]);
}

#[test]
fn positioned_glyph_output_scale_changes_geometry_not_layout_input() {
    let font = noto_fixture();
    let layout = font.layout("O", 48.0);
    let small = font
        .tessellate_positioned_glyph(&layout.glyphs[0], 48.0, 0.01, [0.0, 0.0], 0.2)
        .expect("small output scale");
    let large = font
        .tessellate_positioned_glyph(&layout.glyphs[0], 48.0, 0.02, [0.0, 0.0], 0.2)
        .expect("large output scale");

    let bounds = |points: &[[f32; 2]]| {
        points.iter().fold(
            ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]),
            |(mut min, mut max), point| {
                min[0] = min[0].min(point[0]);
                min[1] = min[1].min(point[1]);
                max[0] = max[0].max(point[0]);
                max[1] = max[1].max(point[1]);
                (min, max)
            },
        )
    };
    let (small_min, small_max) = bounds(&small);
    let (large_min, large_max) = bounds(&large);
    let small_width = small_max[0] - small_min[0];
    let large_width = large_max[0] - large_min[0];

    assert!(small_width > 0.0);
    assert!((large_width / small_width - 2.0).abs() < 0.05);
    assert_eq!(layout.glyphs[0].pen_x, 0.0);
}

#[test]
fn positioned_glyph_tolerance_is_in_output_units() {
    let font = noto_fixture();
    let layout = font.layout("O", 48.0);
    let low_scale = font
        .tessellate_positioned_glyph(&layout.glyphs[0], 48.0, 0.01, [0.0, 0.0], 0.2)
        .expect("low-scale glyph");
    let high_scale = font
        .tessellate_positioned_glyph(&layout.glyphs[0], 48.0, 0.02, [0.0, 0.0], 0.4)
        .expect("high-scale glyph");

    assert!(!low_scale.is_empty());
    assert!(!high_scale.is_empty());
    assert!(high_scale.len() >= low_scale.len());
}

#[test]
fn positioned_glyph_adapter_requires_explicit_font_size() {
    let font = noto_fixture();
    let layout = font.layout("A", 48.0);
    let error = font
        .tessellate_positioned_glyph(&layout.glyphs[0], 0.0, 0.01, [0.0, 0.0], 0.2)
        .expect_err("zero font size must fail");

    assert_eq!(error.kind, UiGlyphVectorDiagnosticKind::InvalidScale);
}

#[test]
fn positioned_glyph_adapter_preserves_missing_outline_diagnostic() {
    let font = noto_fixture();
    let layout = font.layout(" ", 48.0);
    let error = font
        .tessellate_positioned_glyph(&layout.glyphs[0], 48.0, 0.01, [0.0, 0.0], 0.2)
        .expect_err("whitespace has no outline");

    assert_eq!(error.kind, UiGlyphVectorDiagnosticKind::MissingOutline);
}

#[test]
fn fill_topology_keeps_simple_glyphs_on_the_existing_contract() {
    let topology = noto_fixture()
        .outline('-')
        .expect("hyphen outline")
        .fill_topology(UiGlyphVectorOptions::new(48.0, [0.0, 0.0], 0.2))
        .expect("topology classification");

    assert_eq!(topology, UiGlyphFillTopology::SingleConvexContour);
}

fn polygon_area(points: &[[f32; 2]]) -> f32 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(left, right)| left[0] * right[1] - right[0] * left[1])
        .sum::<f32>()
        * 0.5
}

fn triangle_area(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])) * 0.5
}

fn assert_mesh_matches_non_zero_fill(character: char, path: &VectorPath, triangles: &[[f32; 2]]) {
    assert_mesh_matches_fill_rule(character, path, triangles, false);
}

fn assert_mesh_matches_fill_rule(
    character: char,
    path: &VectorPath,
    triangles: &[[f32; 2]],
    even_odd: bool,
) {
    let (min, max) = path.bounds().expect("glyph bounds");
    let mut mismatches = 0usize;
    for row in 0..32 {
        for column in 0..32 {
            let point = [
                min[0] + (column as f32 + 0.5) / 32.0 * (max[0] - min[0]),
                min[1] + (row as f32 + 0.5) / 32.0 * (max[1] - min[1]),
            ];
            let winding = path
                .contours
                .iter()
                .map(|contour| winding_number(point, &contour.points))
                .sum::<i32>();
            let source_filled = if even_odd {
                winding.unsigned_abs() % 2 == 1
            } else {
                winding != 0
            };
            let mesh_filled = triangles.chunks_exact(3).any(|triangle| {
                point_in_triangle_sample(point, triangle[0], triangle[1], triangle[2])
            });
            mismatches += usize::from(source_filled != mesh_filled);
        }
    }
    assert!(
        mismatches <= 2,
        "{character} mesh disagrees with source fill at {mismatches}/1024 samples"
    );
}

fn winding_number(point: [f32; 2], polygon: &[[f32; 2]]) -> i32 {
    polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
        .fold(0, |winding, (start, end)| {
            let side = triangle_area(*start, *end, point);
            if start[1] <= point[1] && end[1] > point[1] && side > 0.0 {
                winding + 1
            } else if start[1] > point[1] && end[1] <= point[1] && side < 0.0 {
                winding - 1
            } else {
                winding
            }
        })
}

fn point_in_triangle_sample(point: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    let ab = triangle_area(a, b, point);
    let bc = triangle_area(b, c, point);
    let ca = triangle_area(c, a, point);
    (ab >= -1.0e-5 && bc >= -1.0e-5 && ca >= -1.0e-5)
        || (ab <= 1.0e-5 && bc <= 1.0e-5 && ca <= 1.0e-5)
}
