use super::*;

fn prepared_inter_bytes() -> Option<Vec<u8>> {
    std::fs::read("../../../../target/glyph-corpus/fonts/inter/Inter[opsz,wght].ttf")
        .or_else(|_| std::fs::read("../../../../target/glyph-corpus/fonts/inter/Inter-Regular.ttf"))
        .ok()
}

#[test]
fn rasterized_glyph_has_coverage_and_advance() {
    let Some(bytes) = prepared_inter_bytes() else {
        return;
    };
    let rasterizer = UiFontRasterizer::from_bytes(bytes).unwrap();
    let glyph = rasterizer.rasterize('A', 32.0);
    assert!(glyph.advance > 0.0);
    assert!(glyph.alpha.iter().any(|coverage| *coverage > 0));
    assert!(glyph.alpha.contains(&0));
}

#[test]
fn rasterized_text_reports_visible_ink_metrics() {
    let Some(bytes) = prepared_inter_bytes() else {
        return;
    };
    let rasterizer = UiFontRasterizer::from_bytes(bytes).unwrap();
    let text = rasterizer.rasterize_text("Ag", 32.0);

    assert!(text.width > 0);
    assert!(text.height > 0);
    assert!(text.ascent > 0.0);
    assert!(text.descent < 0.0);
    assert!(text.left <= 0.0);
    assert!(text.top < text.ascent);
    assert_eq!(text.baseline, 0.0);

    let metrics = text.metrics();
    assert_eq!(metrics.width, text.width as f32);
    assert_eq!(metrics.line_gap, 0.0);
}

#[test]
fn tracking_changes_placement_without_changing_glyph_metrics() {
    let Some(bytes) = prepared_inter_bytes() else {
        return;
    };
    let rasterizer = UiFontRasterizer::from_bytes(bytes).unwrap();
    let normal = rasterizer.layout("AA", 32.0);
    let tracked = rasterizer.layout_with_tracking("AA", 32.0, 3.0);

    assert!(tracked.width > normal.width);
    assert_eq!(
        tracked.glyphs[0].glyph.advance,
        normal.glyphs[0].glyph.advance
    );
    assert_eq!(
        tracked.glyphs[1].glyph.advance,
        normal.glyphs[1].glyph.advance
    );
    assert_eq!(tracked.glyphs[0].pen_x, normal.glyphs[0].pen_x);
    assert_eq!(tracked.glyphs[1].pen_x, normal.glyphs[1].pen_x + 3.0);
}

#[test]
fn multiline_layout_preserves_explicit_leading() {
    let Some(bytes) = prepared_inter_bytes() else {
        return;
    };
    let rasterizer = UiFontRasterizer::from_bytes(bytes).unwrap();
    let block = rasterizer.layout_lines(&["A", "g"], 32.0, 4.0);

    assert_eq!(block.lines.len(), 2);
    assert_eq!(block.line_gap, 4.0);
    assert!(block.baselines[0] > block.baselines[1]);
    assert!(block.width > 0.0);
}

#[test]
fn checked_in_noto_fixture_is_loadable_without_prepared_corpus() {
    let bytes = include_bytes!("../../fixtures/NotoSans-Regular.otf").to_vec();
    let rasterizer = UiFontRasterizer::from_bytes(bytes).expect("checked-in OTF fixture");
    let bitmap = rasterizer.rasterize_text("Noto 0123", 24.0);

    assert!(bitmap.width > 0);
    assert!(bitmap.height > 0);
    assert!(!bitmap.alpha.is_empty());
}
