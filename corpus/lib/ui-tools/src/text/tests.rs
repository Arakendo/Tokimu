use super::*;

#[test]
fn bitmap_layout_respects_center_alignment() {
    let spec = UiTextSpec::new("AA", UiRect::new([0.0, 0.0], [0.6, 0.2]), UiTextRole::Body)
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
    let quads = layout_bitmap_text(&spec, 0.09);

    let min_x = quads
        .iter()
        .map(|quad| quad.center[0] - quad.size[0] * 0.5)
        .fold(f32::INFINITY, f32::min);
    let max_x = quads
        .iter()
        .map(|quad| quad.center[0] + quad.size[0] * 0.5)
        .fold(f32::NEG_INFINITY, f32::max);

    assert!(min_x < 0.0);
    assert!(max_x > 0.0);
}

#[test]
fn bitmap_layout_uses_visible_ink_for_end_alignment() {
    let spec = UiTextSpec::new("AA", UiRect::new([0.0, 0.0], [0.4, 0.2]), UiTextRole::Body)
        .with_alignment(UiTextAlign::End, UiTextAlign::Center);
    let quads = layout_bitmap_text(&spec, 0.09);
    let max_x = quads
        .iter()
        .map(|quad| quad.center[0] + quad.size[0] * 0.5)
        .fold(f32::NEG_INFINITY, f32::max);
    let right = spec.rect.center[0] + spec.rect.size[0] * 0.5;

    assert!((max_x - right).abs() < 0.01);
}

#[test]
fn bitmap_layout_can_align_end_using_advance_width() {
    let spec = UiTextSpec::new("AA", UiRect::new([0.0, 0.0], [0.4, 0.2]), UiTextRole::Body)
        .with_alignment(UiTextAlign::End, UiTextAlign::Center)
        .with_alignment_basis(UiTextAlignmentBasis::Advance);
    let quads = layout_bitmap_text(&spec, 0.09);
    let max_x = quads
        .iter()
        .map(|quad| quad.center[0] + quad.size[0] * 0.5)
        .fold(f32::NEG_INFINITY, f32::max);
    let right = spec.rect.center[0] + spec.rect.size[0] * 0.5;
    let cell = bitmap_cell(0.09);

    assert!((right - max_x - cell * 0.5).abs() < 0.01);
}

#[test]
fn bitmap_layout_clips_to_nonzero_rect() {
    let spec = UiTextSpec::new(
        "AAAAAA",
        UiRect::new([0.0, 0.0], [0.12, 0.08]),
        UiTextRole::Caption,
    )
    .with_alignment(UiTextAlign::Start, UiTextAlign::Center)
    .with_overflow(UiTextOverflow::Clip);
    let quads = layout_bitmap_text(&spec, 0.06);

    assert!(!quads.is_empty());
    assert!(quads.iter().all(|quad| spec.rect.contains(quad.center)));
}

#[test]
fn ellipsis_overflow_truncates_to_the_available_width() {
    let spec = UiTextSpec::new(
        "A LONG LABEL",
        UiRect::new([0.0, 0.0], [0.12, 0.08]),
        UiTextRole::Button,
    )
    .with_overflow(UiTextOverflow::Ellipsis);

    let glyphs = layout_bitmap_text(&spec, 0.03);
    assert!(!glyphs.is_empty());
    assert!(glyphs.iter().all(|glyph| spec.rect.contains(glyph.center)));
}

#[test]
fn deferred_overflow_emits_no_glyphs_but_preserves_fit_evidence() {
    let spec = UiTextSpec::new(
        "THIS REQUEST DOES NOT FIT",
        UiRect::new([0.0, 0.0], [0.12, 0.08]),
        UiTextRole::Status,
    )
    .with_overflow(UiTextOverflow::Defer);

    let report = spec.headless_report(0.05);

    assert!(layout_bitmap_text(&spec, 0.05).is_empty());
    assert!(report.fit.horizontal_overflow);
    assert_eq!(report.glyph_count, 0);
}

#[test]
fn deferred_text_still_renders_when_the_complete_request_fits() {
    let spec = UiTextSpec::new(
        "OK",
        UiRect::new([0.0, 0.0], [0.4, 0.2]),
        UiTextRole::Status,
    )
    .with_overflow(UiTextOverflow::Defer);

    assert!(!layout_bitmap_text(&spec, 0.05).is_empty());
    assert!(spec.headless_report(0.05).fit.fits());
}

#[test]
fn scale_down_preserves_complete_text_inside_its_bounds() {
    let bounds = UiRect::new([0.0, 0.0], [0.24, 0.08]);
    let spec = UiTextSpec::new("SCALE DOWN", bounds, UiTextRole::Button)
        .with_overflow(UiTextOverflow::ScaleDown);

    let glyphs = layout_bitmap_text(&spec, 0.08);
    let layout = spec.bitmap_layout(0.08);

    assert!(!glyphs.is_empty());
    assert!(glyphs.iter().all(|glyph| bounds.contains(glyph.center)));
    assert!(layout.measure.ascent < bitmap_glyph_height(0.08));
    assert!(spec.headless_report(0.08).fit.horizontal_overflow);
}

#[test]
fn bitmap_layout_keeps_start_aligned_glyphs_inside_bounds() {
    let spec = UiTextSpec::new("A", UiRect::new([0.0, 0.0], [0.12, 0.12]), UiTextRole::Body)
        .with_alignment(UiTextAlign::Start, UiTextAlign::Start)
        .with_overflow(UiTextOverflow::Clip);
    let quads = layout_bitmap_text(&spec, 0.06);

    assert!(!quads.is_empty());
    assert!(quads.iter().all(|quad| {
        let left = quad.center[0] - quad.size[0] * 0.5;
        let top = quad.center[1] + quad.size[1] * 0.5;
        left >= spec.rect.center[0] - spec.rect.size[0] * 0.5
            && top <= spec.rect.center[1] + spec.rect.size[1] * 0.5
    }));
}

#[test]
fn bitmap_layout_wraps_words_into_multiple_lines() {
    let spec = UiTextSpec::new(
        "BUILD SETTINGS",
        UiRect::new([0.0, 0.0], [0.16, 0.4]),
        UiTextRole::Body,
    )
    .with_alignment(UiTextAlign::Start, UiTextAlign::Start)
    .with_overflow(UiTextOverflow::Wrap);
    let quads = layout_bitmap_text(&spec, 0.06);
    let distinct_rows = quads
        .iter()
        .map(|quad| (quad.center[1] * 1000.0).round() as i32)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(distinct_rows.len() > 7);
    assert!(quads.iter().all(|quad| {
        quad.center[0] >= spec.rect.center[0] - spec.rect.size[0] * 0.5
            && quad.center[0] <= spec.rect.center[0] + spec.rect.size[0] * 0.5
    }));
}

#[test]
fn bitmap_layout_honors_explicit_newlines() {
    let spec = UiTextSpec::new(
        "A\nB",
        UiRect::new([0.0, 0.0], [0.2, 0.4]),
        UiTextRole::Body,
    )
    .with_overflow(UiTextOverflow::Wrap);
    let quads = layout_bitmap_text(&spec, 0.06);
    let rows = quads
        .iter()
        .map(|quad| (quad.center[1] * 1000.0).round() as i32)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(rows.len() > 7);
}

#[test]
fn bitmap_layout_resolves_start_alignment_by_text_direction() {
    let ltr = UiTextSpec::new("AB", UiRect::new([0.0, 0.0], [0.4, 0.2]), UiTextRole::Body)
        .with_alignment(UiTextAlign::Start, UiTextAlign::Center)
        .with_direction(UiTextDirection::Ltr);
    let rtl = ltr.clone().with_direction(UiTextDirection::Rtl);
    let ltr_quads = layout_bitmap_text(&ltr, 0.06);
    let rtl_quads = layout_bitmap_text(&rtl, 0.06);
    let ltr_min = ltr_quads
        .iter()
        .map(|quad| quad.center[0] - quad.size[0] * 0.5)
        .fold(f32::INFINITY, f32::min);
    let rtl_max = rtl_quads
        .iter()
        .map(|quad| quad.center[0] + quad.size[0] * 0.5)
        .fold(f32::NEG_INFINITY, f32::max);

    assert!(ltr_min < -0.15);
    assert!(rtl_max > 0.15);
}

#[test]
fn headless_layout_produces_stable_bounds_without_renderer_state() {
    let spec = UiTextSpec::new(
        "HEADLESS TEXT\nSECOND LINE",
        UiRect::new([0.0, 0.0], [0.7, 0.4]),
        UiTextRole::Body,
    )
    .with_alignment(UiTextAlign::Start, UiTextAlign::Start)
    .with_overflow(UiTextOverflow::Wrap);

    let first = layout_bitmap_text(&spec, 0.05);
    let second = layout_bitmap_text(&spec, 0.05);

    assert!(!first.is_empty());
    assert_eq!(first, second);
    assert!(first.iter().all(|quad| spec.rect.contains(quad.center)));
}

#[test]
fn headless_report_describes_the_same_layout_consumed_by_rendering() {
    let spec = UiTextSpec::new(
        "REPORT\nREADY",
        UiRect::new([0.0, 0.0], [0.5, 0.3]),
        UiTextRole::Status,
    )
    .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
    let report = spec.headless_report(0.05);
    let rendered = layout_bitmap_text(&spec, 0.05);

    assert_eq!(report.text, "REPORT\nREADY");
    assert_eq!(report.line_count, 2);
    assert_eq!(report.glyph_count, rendered.len());
    assert!(report.visible_bounds.is_some());
    assert!(report.fit.fits());
}

#[test]
fn headless_report_exposes_hidden_horizontal_overflow() {
    let spec = UiTextSpec::new(
        "THIS LABEL IS TOO WIDE",
        UiRect::new([0.0, 0.0], [0.18, 0.10]),
        UiTextRole::Status,
    )
    .with_overflow(UiTextOverflow::Ellipsis);

    let report = spec.headless_report(0.04);

    assert!(report.fit.horizontal_overflow);
    assert!(!report.fit.vertical_overflow);
    assert!(!report.fit.fits());
}

#[test]
fn headless_report_exposes_wrapped_vertical_overflow() {
    let spec = UiTextSpec::new(
        "ONE TWO THREE FOUR FIVE SIX",
        UiRect::new([0.0, 0.0], [0.18, 0.08]),
        UiTextRole::Body,
    )
    .with_overflow(UiTextOverflow::Wrap);

    let report = spec.headless_report(0.04);

    assert!(report.fit.horizontal_overflow);
    assert!(report.fit.vertical_overflow);
    assert!(!report.fit.fits());
}

#[test]
fn bitmap_layout_handles_empty_spaces_and_punctuation_deterministically() {
    let bounds = UiRect::new([0.0, 0.0], [0.5, 0.2]);
    let empty = layout_bitmap_text(&UiTextSpec::new("", bounds, UiTextRole::Body), 0.05);
    let spaces = layout_bitmap_text(&UiTextSpec::new("   ", bounds, UiTextRole::Body), 0.05);
    let punctuation = layout_bitmap_text(&UiTextSpec::new("!?.,", bounds, UiTextRole::Body), 0.05);

    assert!(empty.is_empty());
    assert!(spaces.is_empty());
    assert!(!punctuation.is_empty());
    assert_eq!(
        empty,
        layout_bitmap_text(&UiTextSpec::new("", bounds, UiTextRole::Body), 0.05)
    );
}

#[test]
fn bitmap_layout_keeps_zero_ink_unknown_text_on_a_stable_policy() {
    let spec = UiTextSpec::new(
        "\u{1f600}",
        UiRect::new([0.0, 0.0], [0.4, 0.2]),
        UiTextRole::Body,
    )
    .with_missing_glyph_policy(UiMissingGlyphPolicy::Report);

    let first = layout_bitmap_text(&spec, 0.05);
    let second = layout_bitmap_text(&spec, 0.05);

    assert_eq!(first, second);
}

#[test]
fn bitmap_layout_exposes_provider_neutral_lines_and_baselines() {
    let spec = UiTextSpec::new(
        "FIRST\nSECOND",
        UiRect::new([0.0, 0.0], [0.6, 0.4]),
        UiTextRole::Body,
    )
    .with_alignment(UiTextAlign::Start, UiTextAlign::Center);
    let layout = spec.bitmap_layout(0.05);

    assert_eq!(layout.line_count(), 2);
    assert!(layout.lines[0].advance > 0.0);
    assert!(layout.lines[0].baseline > layout.lines[1].baseline);
    assert_eq!(layout.lines[0].origin[0], layout.lines[1].origin[0]);
}
