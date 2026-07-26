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
