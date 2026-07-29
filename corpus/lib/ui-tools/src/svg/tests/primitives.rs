use super::super::*;

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
        .all(|point| { (-0.51..=0.51).contains(&point[0]) && (-0.51..=0.51).contains(&point[1]) }));
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
