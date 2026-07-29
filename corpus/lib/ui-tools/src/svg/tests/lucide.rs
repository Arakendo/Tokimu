use super::super::*;

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
    let mesh = stroke_paths(&paths, 1.0 / 32.0);
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
