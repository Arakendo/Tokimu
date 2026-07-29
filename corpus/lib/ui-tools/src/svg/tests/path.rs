use super::super::*;

#[test]
fn preserves_compact_signed_numbers() {
    assert_eq!(
        tokenize_path("M20 6 9 17l-5-5"),
        vec![
            SvgToken::Command('M'),
            SvgToken::Number(20.0),
            SvgToken::Number(6.0),
            SvgToken::Number(9.0),
            SvgToken::Number(17.0),
            SvgToken::Command('l'),
            SvgToken::Number(-5.0),
            SvgToken::Number(-5.0),
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
fn parses_implicit_repeated_move_and_line_arguments() {
    let commands = parse_path("M0 0 10 0 10 10 l5 0 0 5").unwrap();

    assert!(matches!(commands[0], SvgPathCommand::MoveTo { .. }));
    assert!(matches!(commands[1], SvgPathCommand::LineTo { .. }));
    assert!(matches!(commands[2], SvgPathCommand::LineTo { .. }));
    assert!(matches!(commands[3], SvgPathCommand::LineTo { .. }));
    assert!(matches!(commands[4], SvgPathCommand::LineTo { .. }));
}

#[test]
fn tokenizes_scientific_notation_without_confusing_the_exponent_for_a_command() {
    let commands = parse_path("M1e1 2E1 l-5e-1 .5").unwrap();

    assert_eq!(commands.len(), 2);
    assert!(matches!(
        commands[0],
        SvgPathCommand::MoveTo {
            x: 10.0,
            y: 20.0,
            ..
        }
    ));
    assert!(matches!(
        commands[1],
        SvgPathCommand::LineTo {
            relative: true,
            x: -0.5,
            y: 0.5,
            ..
        }
    ));
}

#[test]
fn flattening_resolves_relative_horizontal_and_vertical_commands() {
    let commands = parse_path("M2 3 h8 v4 h-8 z").unwrap();
    let paths = flatten_path(&commands, 8);

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].first().copied(), Some([2.0, 3.0]));
    assert_eq!(paths[0].last().copied(), Some([2.0, 3.0]));
    assert!(paths[0].contains(&[10.0, 3.0]));
    assert!(paths[0].contains(&[10.0, 7.0]));
}

#[test]
fn flattening_keeps_closed_and_following_relative_subpaths_separate() {
    let commands = parse_path("M10 10 l10 0 l0 10 z m5 5 l5 0").unwrap();
    let paths = flatten_path(&commands, 8);

    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0].first(), paths[0].last());
    assert_ne!(paths[1].first(), paths[1].last());
    assert_eq!(paths[1].first().copied(), Some([15.0, 15.0]));
}

#[test]
fn smooth_quadratic_control_does_not_leak_across_a_new_subpath() {
    let commands = parse_path("M0 0 Q10 10 20 0 T40 0 M0 20 T20 20").unwrap();
    let paths = flatten_path(&commands, 8);

    assert_eq!(paths.len(), 2);
    assert!(paths[0].iter().any(|point| point[1] > 0.0));
    assert!(paths[1]
        .iter()
        .all(|point| (point[1] - 20.0).abs() < 1.0e-5));
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
fn degenerate_arc_radii_reduce_to_a_line_without_non_finite_points() {
    let commands = parse_path("M0 0 A0 9 45 0 1 12 8").unwrap();
    let paths = flatten_path(&commands, 8);

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], vec![[0.0, 0.0], [12.0, 8.0]]);
}

#[test]
fn rotated_arc_preserves_endpoints_and_finite_geometry() {
    let commands = parse_path("M10 20 A18 7 37 1 0 70 55").unwrap();
    let paths = flatten_path(&commands, 16);

    assert_eq!(paths.len(), 1);
    assert!(paths[0]
        .iter()
        .all(|point| point[0].is_finite() && point[1].is_finite()));
    assert_eq!(paths[0].first().copied(), Some([10.0, 20.0]));
    assert_eq!(paths[0].last().copied(), Some([70.0, 55.0]));
}

#[test]
fn malformed_path_commands_return_diagnostics() {
    assert!(parse_path("M0 0 L").is_err());
    assert!(parse_path("M0 0 R10 10").is_err());
    assert!(parse_path("0 0 L10 10").is_err());
}

#[test]
fn path_commands_reject_non_finite_numbers() {
    let error =
        parse_path("M0 0 L1e39 1").expect_err("overflowing SVG path coordinates must be rejected");

    assert!(error.contains("non-finite L coordinate"));
}

#[test]
fn arc_commands_reject_non_binary_flags() {
    let error = parse_path("M0 0 A4 4 0 2 0 8 8").expect_err("SVG arc flags must be binary values");

    assert!(error.contains("invalid A arc flag"));
}
