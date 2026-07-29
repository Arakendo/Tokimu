//! Corpus catalog access, lookup, and producer dispatch.

use crate::{
    run_cgm_case, run_glyph_case, run_svg_case, run_synthetic_case, run_synthetic_svg_case,
    run_ui_case, run_w3c_svg_case, CaseReport, CgmCase, CorpusCase, GlyphCase, SvgCase,
    SyntheticCase, SyntheticSvgCase, UiCase, W3cSvgCase, ALL_CASES, CGM_CASES, GLYPH_CASES,
    SVG_CASES, SYNTHETIC_CASES, SYNTHETIC_SVG_CASES, UI_CASES, W3C_SVG_CASES,
};

pub fn glyph_cases() -> &'static [GlyphCase] {
    &GLYPH_CASES
}

pub fn find_glyph_case(id: &str) -> Option<GlyphCase> {
    glyph_cases().iter().copied().find(|case| case.id == id)
}

pub fn synthetic_cases() -> &'static [SyntheticCase] {
    &SYNTHETIC_CASES
}

pub fn svg_cases() -> &'static [SvgCase] {
    &SVG_CASES
}

pub fn synthetic_svg_cases() -> &'static [SyntheticSvgCase] {
    &SYNTHETIC_SVG_CASES
}

pub fn w3c_svg_cases() -> &'static [W3cSvgCase] {
    &W3C_SVG_CASES
}

pub fn ui_cases() -> &'static [UiCase] {
    &UI_CASES
}

pub fn cgm_cases() -> &'static [CgmCase] {
    &CGM_CASES
}

pub fn all_cases() -> &'static [CorpusCase] {
    &ALL_CASES
}

pub fn find_case(id: &str) -> Option<CorpusCase> {
    all_cases()
        .iter()
        .copied()
        .find(|case| case.id() == id)
        .or_else(|| {
            w3c_svg_cases()
                .iter()
                .copied()
                .find(|case| case.id == id)
                .map(CorpusCase::W3cSvg)
        })
}

pub fn run_case(case: CorpusCase) -> CaseReport {
    match case {
        CorpusCase::Glyph(case) => run_glyph_case(case),
        CorpusCase::Synthetic(case) => run_synthetic_case(case),
        CorpusCase::Svg(case) => run_svg_case(case),
        CorpusCase::SyntheticSvg(case) => run_synthetic_svg_case(case),
        CorpusCase::W3cSvg(case) => run_w3c_svg_case(case),
        CorpusCase::Ui(case) => run_ui_case(case),
        CorpusCase::Cgm(case) => run_cgm_case(case),
    }
}
