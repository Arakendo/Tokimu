use super::*;

#[test]
fn rectangle_is_one_closed_contour_with_expected_bounds() {
    let path = PathBuilder::new().rect([-2.0, -1.0], [4.0, 3.0]).build();

    assert_eq!(path.contours.len(), 1);
    assert!(path.contours[0].closed);
    assert_eq!(path.contours[0].points.len(), 4);
    assert_eq!(path.bounds(), Some(([-2.0, -1.0], [2.0, 2.0])));
}

#[test]
fn axis_aligned_clip_intersects_closed_polygon_contours() {
    let path = VectorPath::new(vec![VectorContour::new(
        vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
        true,
    )]);

    let clipped = clip_path_to_axis_aligned_rect(&path, [0.5, 0.25], [1.5, 1.75])
        .expect("rectangle clipping should produce a contour");
    assert_eq!(clipped.contours.len(), 1);
    assert_eq!(clipped.bounds(), Some(([0.5, 0.25], [1.5, 1.75])));
}

#[test]
fn axis_aligned_clip_rejects_open_contours() {
    let path = VectorPath::new(vec![VectorContour::new(
        vec![[0.0, 0.0], [2.0, 2.0]],
        false,
    )]);

    let error = clip_path_to_axis_aligned_rect(&path, [0.0, 0.0], [1.0, 1.0])
        .expect_err("open strokes require a separate clipping policy");
    assert!(error.contains("closed polygon"));
}

#[test]
fn axis_aligned_clip_drops_disjoint_contours_and_preserves_intersecting_ones() {
    let path = VectorPath::new(vec![
        VectorContour::new(
            vec![[10.0, 10.0], [12.0, 10.0], [12.0, 12.0], [10.0, 12.0]],
            true,
        ),
        VectorContour::new(vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]], true),
    ]);

    let clipped = clip_path_to_axis_aligned_rect(&path, [0.5, 0.5], [1.5, 1.5])
        .expect("disjoint contours should be discarded without failing the clip");
    assert_eq!(clipped.contours.len(), 1);
    assert_eq!(clipped.bounds(), Some(([0.5, 0.5], [1.5, 1.5])));
}

#[test]
fn axis_aligned_clip_rejects_non_positive_bounds() {
    let path = PathBuilder::new().rect([0.0, 0.0], [1.0, 1.0]).build();

    let error = clip_path_to_axis_aligned_rect(&path, [1.0, 0.0], [1.0, 1.0])
        .expect_err("zero-sized clip bounds should be rejected");
    assert!(error.contains("finite positive bounds"));
}

#[test]
fn convex_clip_intersects_a_triangle() {
    let path = PathBuilder::new().rect([0.0, 0.0], [4.0, 4.0]).build();
    let clip = VectorPath::new(vec![VectorContour::new(
        vec![[1.0, 0.0], [4.0, 1.0], [1.0, 4.0]],
        true,
    )]);

    let clipped = clip_path_to_convex_polygon(&path, &clip)
        .expect("a convex polygon clip should produce finite geometry");
    assert_eq!(clipped.contours.len(), 1);
    assert_eq!(clipped.bounds(), Some(([1.0, 0.0], [4.0, 4.0])));
}

#[test]
fn convex_clip_intersects_a_many_sided_circle_approximation() {
    let path = PathBuilder::new().rect([-2.0, -2.0], [4.0, 4.0]).build();
    let clip = VectorPath::new(vec![VectorContour::new(
        (0..16)
            .map(|index| {
                let angle = index as f32 * std::f32::consts::TAU / 16.0;
                [angle.cos(), angle.sin()]
            })
            .collect(),
        true,
    )]);

    let clipped = clip_path_to_convex_polygon(&path, &clip)
        .expect("a convex circular approximation should clip a rectangle");
    let points = &clipped.contours[0].points;
    assert!(points.len() >= 16);
    assert!(points
        .iter()
        .all(|point| point[0].abs() <= 1.001 && point[1].abs() <= 1.001));
    assert!(clipped.bounds().is_some());
}

#[test]
fn convex_clip_rejects_concave_clip_geometry() {
    let path = PathBuilder::new().rect([0.0, 0.0], [4.0, 4.0]).build();
    let clip = VectorPath::new(vec![VectorContour::new(
        vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [2.0, 2.0], [0.0, 4.0]],
        true,
    )]);

    let error = clip_path_to_convex_polygon(&path, &clip)
        .expect_err("concave clip geometry remains outside this bounded slice");
    assert!(error.contains("convex polygon"));
}

#[test]
fn rounded_rectangle_is_finite_and_closed() {
    let path = PathBuilder::new()
        .rounded_rect([-1.0, -0.5], [2.0, 1.0], 0.25)
        .build();

    assert_eq!(path.contours.len(), 1);
    assert!(path.contours[0].closed);
    assert!(path.is_finite());
    assert_eq!(path.contours[0].points.len(), 32);
}

#[test]
fn builder_preserves_open_contours() {
    let path = PathBuilder::new()
        .move_to([0.0, 0.0])
        .line_to([1.0, 0.0])
        .build();

    assert_eq!(
        path.contours,
        vec![VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0]], false)]
    );
}

#[test]
fn close_marks_only_the_current_contour_closed() {
    let path = PathBuilder::new()
        .move_to([0.0, 0.0])
        .line_to([1.0, 0.0])
        .close()
        .move_to([2.0, 0.0])
        .line_to([3.0, 0.0])
        .build();

    assert_eq!(path.contours.len(), 2);
    assert!(path.contours[0].closed);
    assert!(!path.contours[1].closed);
}

#[test]
fn convex_fill_tessellates_a_rectangle_with_consistent_winding() {
    let path = PathBuilder::new().rect([0.0, 0.0], [2.0, 1.0]).build();
    let triangles = tessellate_convex_fill(&path).unwrap();

    assert_eq!(triangles.len(), 6);
    assert!(triangles.chunks_exact(3).all(|triangle| {
        cross(
            subtract(triangle[1], triangle[0]),
            subtract(triangle[2], triangle[0]),
        ) > 0.0
    }));
}

#[test]
fn convex_fill_accepts_a_rounded_rectangle() {
    let path = PathBuilder::new()
        .rounded_rect([0.0, 0.0], [2.0, 1.0], 0.2)
        .build();

    let triangles = tessellate_convex_fill(&path).unwrap();

    assert_eq!(triangles.len(), (path.contours[0].points.len() - 2) * 3);
    assert!(triangles
        .iter()
        .all(|point| { point[0].is_finite() && point[1].is_finite() }));
}

#[test]
fn convex_fill_normalizes_reversed_winding() {
    let path = VectorPath::new(vec![VectorContour::new(
        vec![[0.0, 0.0], [0.0, 1.0], [2.0, 1.0], [2.0, 0.0]],
        true,
    )]);

    let triangles = tessellate_convex_fill(&path).unwrap();

    assert_eq!(triangles.len(), 6);
    assert!(triangles.chunks_exact(3).all(|triangle| {
        cross(
            subtract(triangle[1], triangle[0]),
            subtract(triangle[2], triangle[0]),
        ) > 0.0
    }));
}

#[test]
fn convex_fill_accepts_a_repeated_closing_point() {
    let path = VectorPath::new(vec![VectorContour::new(
        vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]],
        true,
    )]);

    assert_eq!(tessellate_convex_fill(&path).unwrap().len(), 3);
}

#[test]
fn convex_fill_rejects_unsupported_topology() {
    let open = PathBuilder::new()
        .move_to([0.0, 0.0])
        .line_to([1.0, 0.0])
        .line_to([0.0, 1.0])
        .build();
    let concave = VectorPath::new(vec![VectorContour::new(
        vec![[0.0, 0.0], [2.0, 0.0], [1.0, 0.5], [2.0, 1.0], [0.0, 1.0]],
        true,
    )]);

    assert!(tessellate_convex_fill(&open).is_err());
    assert!(tessellate_convex_fill(&concave).is_err());
}

#[test]
fn convex_fill_rejects_degenerate_and_multi_contour_paths() {
    let degenerate = PathBuilder::new().rect([0.0, 0.0], [0.0, 1.0]).build();
    let multi = VectorPath::new(vec![
        VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]], true),
        VectorContour::new(vec![[2.0, 2.0], [3.0, 2.0], [2.0, 3.0]], true),
    ]);
    let non_finite = VectorPath::new(vec![VectorContour::new(
        vec![[0.0, 0.0], [f32::NAN, 0.0], [0.0, 1.0]],
        true,
    )]);

    assert!(tessellate_convex_fill(&degenerate).is_err());
    assert!(tessellate_convex_fill(&multi).is_err());
    assert!(tessellate_convex_fill(&non_finite).is_err());
}

#[test]
fn convex_fill_validation_reports_support_without_geometry() {
    let supported = VectorPath::new(vec![VectorContour::new(
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        true,
    )]);
    let unsupported = VectorPath::new(vec![VectorContour::new(
        vec![[0.0, 0.0], [1.0, 0.0], [0.5, 0.25], [0.0, 1.0]],
        true,
    )]);

    assert!(validate_convex_fill(&supported).is_ok());
    assert!(validate_convex_fill(&unsupported).is_err());
}

#[test]
fn general_fill_tessellates_a_concave_contour() {
    let path = VectorPath::new(vec![VectorContour::new(
        vec![
            [0.0, 0.0],
            [3.0, 0.0],
            [3.0, 1.0],
            [1.0, 1.0],
            [1.0, 3.0],
            [0.0, 3.0],
        ],
        true,
    )]);

    let triangles = tessellate_general_fill(&path).expect("concave fill");

    assert!(!triangles.is_empty());
    assert!(triangles
        .iter()
        .all(|point| { point[0].is_finite() && point[1].is_finite() }));
}

#[test]
fn general_fill_ignores_duplicate_outline_points() {
    let path = VectorPath::new(vec![VectorContour::new(
        vec![
            [-1.0, -1.0],
            [1.0, -1.0],
            [1.0, -1.0],
            [1.0, 1.0],
            [-1.0, 1.0],
            [-1.0, -1.0],
        ],
        true,
    )]);
    let triangles = tessellate_general_fill(&path).expect("duplicate points are harmless");
    assert_eq!(triangles.len(), 6);
}

#[test]
fn general_fill_ignores_collinear_flattening_samples() {
    let path = VectorPath::new(vec![VectorContour::new(
        vec![
            [0.0, 0.0],
            [0.5, 0.0],
            [1.0, 0.0],
            [1.0, 0.5],
            [1.0, 1.0],
            [0.0, 1.0],
        ],
        true,
    )]);

    let triangles = tessellate_general_fill(&path)
        .expect("collinear samples from a flattened contour are harmless");

    assert_eq!(triangles.len(), 6);
}

#[test]
fn simple_fill_allows_vertices_on_candidate_ear_edges() {
    let points = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [2.0, 1.0], [0.0, 1.0]];

    let triangles =
        tessellate_simple_loop(&points).expect("a boundary vertex should not block a valid ear");

    assert_eq!(triangles.len(), 6);
}

#[test]
fn font_fill_preserves_thin_single_contours() {
    // A slash-like glyph can be valid at font scale while falling below the
    // robust fill tessellator's tolerance after output scaling.
    let path = VectorPath::new(vec![VectorContour::new(
        vec![[0.0, 0.0], [0.0015, 1.0], [0.0030, 1.0], [0.0015, 0.0]],
        true,
    )]);

    let triangles = tessellate_font_fill_with_rule(&path, VectorFillRule::EvenOdd)
        .expect("thin font contours should remain drawable");

    assert!(!triangles.is_empty());
    assert!(triangles.iter().flatten().all(|value| value.is_finite()));
}

#[test]
fn general_fill_preserves_self_intersecting_font_outline_extents() {
    let points = vec![
        [0.0050781253, 0.0],
        [0.057910156, 0.14550781],
        [0.079394534, 0.14550781],
        [0.13291016, 0.0],
        [0.113378905, 0.0],
        [0.08251953, 0.08632813],
        [0.075927734, 0.10629883],
        [0.06679688, 0.13681641],
        [0.0703125, 0.13681641],
        [0.061181642, 0.10590821],
        [0.05478516, 0.08632813],
        [0.024804687, 0.0],
        [0.0050781253, 0.0],
    ];
    let expected_min_x = points
        .iter()
        .map(|point| point[0])
        .fold(f32::INFINITY, f32::min);
    let path = VectorPath::new(vec![VectorContour::new(points, true)]);

    let triangles = tessellate_general_fill(&path).expect("self-intersecting font fill");
    let actual_min_x = triangles
        .iter()
        .map(|point| point[0])
        .fold(f32::INFINITY, f32::min);

    assert!(
        (actual_min_x - expected_min_x).abs() <= 1.0e-6,
        "expected min x {expected_min_x}, got {actual_min_x}"
    );
}

#[test]
fn general_fill_tessellates_a_counter_with_multiple_contours() {
    let path = VectorPath::new(vec![
        VectorContour::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]], true),
        VectorContour::new(vec![[1.0, 1.0], [1.0, 3.0], [3.0, 3.0], [3.0, 1.0]], true),
    ]);

    let triangles = tessellate_general_fill(&path).expect("counter fill");

    assert!(!triangles.is_empty());
    assert!(triangles
        .iter()
        .all(|point| { point[0].is_finite() && point[1].is_finite() }));
}

#[test]
fn general_fill_supports_explicit_fill_rules() {
    let path = VectorPath::new(vec![
        VectorContour::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]], true),
        VectorContour::new(vec![[1.0, 1.0], [1.0, 3.0], [3.0, 3.0], [3.0, 1.0]], true),
    ]);

    let non_zero = tessellate_general_fill_with_rule(&path, VectorFillRule::NonZero)
        .expect("non-zero counter fill");
    let even_odd = tessellate_general_fill_with_rule(&path, VectorFillRule::EvenOdd)
        .expect("even-odd counter fill");

    assert!(!non_zero.is_empty());
    assert!(!even_odd.is_empty());
    assert!(non_zero
        .iter()
        .chain(even_odd.iter())
        .all(|point| point[0].is_finite() && point[1].is_finite()));
}

#[test]
fn general_fill_accepts_reversed_inner_contour_winding() {
    let path = VectorPath::new(vec![
        VectorContour::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]], true),
        VectorContour::new(vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]], true),
    ]);

    let triangles = tessellate_general_fill_with_rule(&path, VectorFillRule::NonZero)
        .expect("reversed inner contour fill");

    assert!(!triangles.is_empty());
    assert!(triangles
        .iter()
        .all(|point| point[0].is_finite() && point[1].is_finite()));
}

#[test]
fn stroke_uses_explicit_open_and_closed_contour_state() {
    let open = VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0]], false);
    let closed = VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]], true);

    assert_eq!(tessellate_stroke(&open, 0.1).len(), 78);
    assert_eq!(tessellate_stroke(&closed, 0.1).len(), 24);
}

#[test]
fn short_open_stroke_preserves_both_endpoints() {
    let contour = VectorContour::new(vec![[0.0, 0.0], [0.1, 0.0]], false);
    let mesh = tessellate_stroke(&contour, 0.1);

    assert!(!mesh.is_empty());
    let min_x = mesh
        .iter()
        .map(|vertex| vertex[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = mesh
        .iter()
        .map(|vertex| vertex[0])
        .fold(f32::NEG_INFINITY, f32::max);

    assert!(
        min_x <= -0.099,
        "short stroke lost its first endpoint: {min_x}"
    );
    assert!(
        max_x >= 0.199,
        "short stroke lost its second endpoint: {max_x}"
    );
}

#[test]
fn stroke_caps_change_only_the_open_end_treatment() {
    let contour = VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0]], false);
    let style = |cap| VectorStrokeStyle {
        half_width: 0.1,
        cap,
        join: VectorStrokeJoin::Miter,
        miter_limit: 4.0,
    };

    let butt =
        tessellate_stroke_with_style(&contour, style(VectorStrokeCap::Butt)).expect("butt cap");
    let round =
        tessellate_stroke_with_style(&contour, style(VectorStrokeCap::Round)).expect("round cap");
    let square =
        tessellate_stroke_with_style(&contour, style(VectorStrokeCap::Square)).expect("square cap");

    assert_eq!(butt.len(), 6);
    assert_eq!(round.len(), 78);
    assert_eq!(square.len(), 6);
    assert!(square.iter().any(|vertex| vertex[0] < -0.099));
    assert!(square.iter().any(|vertex| vertex[0] > 1.099));
}

#[test]
fn connected_stroke_joins_produce_geometry() {
    let contour = VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], false);
    let style = VectorStrokeStyle {
        half_width: 0.1,
        cap: VectorStrokeCap::Butt,
        join: VectorStrokeJoin::Round,
        miter_limit: 4.0,
    };

    let round = tessellate_stroke_with_style(&contour, style).expect("round join");
    let bevel = tessellate_stroke_with_style(
        &contour,
        VectorStrokeStyle {
            join: VectorStrokeJoin::Bevel,
            ..style
        },
    )
    .expect("bevel join");
    assert!(round.len() > 6);
    assert!(bevel.len() > 6);
}

#[test]
fn round_join_uses_the_short_arc_for_a_right_turn() {
    let contour = VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, -1.0]], false);
    let style = VectorStrokeStyle {
        half_width: 0.1,
        cap: VectorStrokeCap::Butt,
        join: VectorStrokeJoin::Round,
        miter_limit: 4.0,
    };

    let mesh = tessellate_stroke_with_style(&contour, style).expect("right-hand round join");
    assert_eq!(mesh.len(), 30);
}

#[test]
fn miter_limit_controls_sharp_join_extent() {
    let contour = VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0], [1.1, 0.0175]], false);
    let style = |miter_limit| VectorStrokeStyle {
        half_width: 0.1,
        cap: VectorStrokeCap::Butt,
        join: VectorStrokeJoin::Miter,
        miter_limit,
    };
    let constrained = tessellate_stroke_with_style(&contour, style(0.5)).expect("low miter");
    let extended = tessellate_stroke_with_style(&contour, style(4.0)).expect("high miter");

    assert_ne!(
        constrained, extended,
        "miter limit did not affect the sharp join"
    );
}

#[test]
fn closed_stroke_normalizes_a_repeated_endpoint_without_inference() {
    let repeated = VectorContour::new(
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]],
        true,
    );
    let explicit = VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]], true);

    assert_eq!(
        tessellate_stroke(&repeated, 0.1).len(),
        tessellate_stroke(&explicit, 0.1).len()
    );
}

#[test]
fn path_collection_strokes_all_contours_in_order() {
    let paths = vec![
        VectorPath::new(vec![VectorContour::new(
            vec![[0.0, 0.0], [1.0, 0.0]],
            false,
        )]),
        VectorPath::new(vec![VectorContour::new(
            vec![[0.0, 1.0], [1.0, 1.0]],
            false,
        )]),
    ];

    let mesh = tessellate_path_strokes(&paths, 0.1);

    assert_eq!(
        mesh.len(),
        tessellate_stroke(&paths[0].contours[0], 0.1).len() * 2
    );
    assert!(mesh
        .iter()
        .all(|vertex| vertex.iter().all(|value| value.is_finite())));
}
