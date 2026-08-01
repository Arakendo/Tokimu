use super::*;
use crate::{UiBitmapTextMetricsProvider, UiTextMetricsProvider};

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

#[test]
fn raster_metrics_adapter_preserves_provider_neutral_contract() {
    let bytes = include_bytes!("../../fixtures/NotoSans-Regular.otf").to_vec();
    let rasterizer = UiFontRasterizer::from_bytes(bytes).expect("checked-in OTF fixture");
    let provider = rasterizer.metrics_provider(24.0);
    let measure = provider.measure("Ag").expect("OTF metrics");

    assert_eq!(provider.pixels(), 24.0);
    assert!(measure.advance > 0.0);
    assert!(measure.ascent > 0.0);
    assert!(measure.descent < 0.0);
    assert!(measure.visible_bounds.is_some());
    assert!(measure.diagnostics.is_empty());
}

#[test]
fn built_in_ttf_and_otf_providers_share_layout_contracts() {
    let built_in = UiBitmapTextMetricsProvider::new(24.0);
    let ttf = UiFontRasterizer::from_bytes(
        include_bytes!("../../../../../third-party/fonts/inter/docs/font-files/InterVariable.ttf")
            .to_vec(),
    )
    .expect("checked-in Inter TTF");
    let otf = UiFontRasterizer::from_bytes(
        include_bytes!("../../fixtures/NotoSans-Regular.otf").to_vec(),
    )
    .expect("checked-in Noto OTF");

    let providers: [&dyn UiTextMetricsProvider; 3] = [
        &built_in,
        &ttf.metrics_provider(24.0),
        &otf.metrics_provider(24.0),
    ];

    for provider in providers {
        let measure = provider.measure("Provider 0123").expect("text metrics");
        assert!(measure.advance.is_finite() && measure.advance > 0.0);
        assert!(measure.ascent.is_finite() && measure.ascent > 0.0);
        assert!(measure.descent.is_finite());
        assert!(measure.line_gap.is_finite() && measure.line_gap >= 0.0);
        let visible = measure.visible_bounds.expect("visible text bounds");
        assert!(visible.size[0].is_finite() && visible.size[0] > 0.0);
        assert!(visible.size[1].is_finite() && visible.size[1] > 0.0);
        assert!(measure.diagnostics.is_empty());

        let multiline = provider
            .measure("Wide provider line\nshort")
            .expect("multiline text metrics");
        assert!(multiline.advance >= provider.measure("short").unwrap().advance);
        assert!(
            multiline.ascent + multiline.descent.abs() > measure.ascent + measure.descent.abs()
        );
    }
}
