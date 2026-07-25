//! Corpus case contracts and producer-stage selection.

use crate::CorpusStage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphCase {
    pub id: &'static str,
    pub character: char,
}

impl GlyphCase {
    pub const fn new(id: &'static str, character: char) -> Self {
        Self { id, character }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyntheticCase {
    pub id: &'static str,
    pub description: &'static str,
    pub expected_failure: bool,
}

impl SyntheticCase {
    pub const fn new(id: &'static str, description: &'static str) -> Self {
        Self {
            id,
            description,
            expected_failure: false,
        }
    }

    pub const fn expected_failure(id: &'static str, description: &'static str) -> Self {
        Self {
            id,
            description,
            expected_failure: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgCase {
    pub id: &'static str,
    pub file_name: &'static str,
    pub description: &'static str,
}

impl SvgCase {
    pub const fn new(id: &'static str, file_name: &'static str, description: &'static str) -> Self {
        Self {
            id,
            file_name,
            description,
        }
    }
}

/// A deliberately small, self-contained SVG document used to exercise an SVG
/// semantic boundary without inheriting unrelated third-party fixture scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyntheticSvgCase {
    pub id: &'static str,
    pub description: &'static str,
    pub source: &'static str,
}

impl SyntheticSvgCase {
    pub const fn new(id: &'static str, description: &'static str, source: &'static str) -> Self {
        Self {
            id,
            description,
            source,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct W3cSvgCase {
    pub id: &'static str,
    pub file_name: &'static str,
    pub description: &'static str,
    pub expectation: W3cSvgExpectation,
    pub source: W3cSvgSource,
}

/// The expected result for a deliberately admitted W3C source fixture.
///
/// W3C files often cover one desired behavior while also carrying unrelated
/// SVG features such as embedded fonts or text. Those fixtures remain useful
/// provenance and XML-stage evidence, but must not masquerade as structural
/// SVG passes until the full source fits the admitted profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum W3cSvgExpectation {
    StructuralPass,
    UnsupportedProfile,
}

/// Locates a W3C-related fixture without confusing a reduced geometry fixture
/// with a verbatim upstream conformance document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum W3cSvgSource {
    UpstreamSvg,
    DerivedProfileFixture,
}

impl W3cSvgCase {
    pub const fn new(id: &'static str, file_name: &'static str, description: &'static str) -> Self {
        Self {
            id,
            file_name,
            description,
            expectation: W3cSvgExpectation::StructuralPass,
            source: W3cSvgSource::UpstreamSvg,
        }
    }

    pub const fn expected_unsupported_profile(
        id: &'static str,
        file_name: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            id,
            file_name,
            description,
            expectation: W3cSvgExpectation::UnsupportedProfile,
            source: W3cSvgSource::UpstreamSvg,
        }
    }

    pub const fn derived_profile_fixture(
        id: &'static str,
        file_name: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            id,
            file_name,
            description,
            expectation: W3cSvgExpectation::StructuralPass,
            source: W3cSvgSource::DerivedProfileFixture,
        }
    }

    pub(crate) const fn producer(self) -> &'static str {
        match self.source {
            W3cSvgSource::UpstreamSvg => "svg/w3c",
            W3cSvgSource::DerivedProfileFixture => "svg/w3c-derived",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiCase {
    pub id: &'static str,
    pub description: &'static str,
}

impl UiCase {
    pub const fn new(id: &'static str, description: &'static str) -> Self {
        Self { id, description }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorpusCase {
    Glyph(GlyphCase),
    Synthetic(SyntheticCase),
    Svg(SvgCase),
    SyntheticSvg(SyntheticSvgCase),
    W3cSvg(W3cSvgCase),
    Ui(UiCase),
}

impl CorpusCase {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Glyph(case) => case.id,
            Self::Synthetic(case) => case.id,
            Self::Svg(case) => case.id,
            Self::SyntheticSvg(case) => case.id,
            Self::W3cSvg(case) => case.id,
            Self::Ui(case) => case.id,
        }
    }

    pub const fn selected_stages(self) -> &'static [CorpusStage] {
        match self {
            Self::Glyph(_) => &GLYPH_STAGES,
            Self::Svg(_) | Self::SyntheticSvg(_) | Self::W3cSvg(_) => &SVG_STAGES,
            Self::Synthetic(_) | Self::Ui(_) => &PATH_STAGES,
        }
    }
}

pub(crate) const GLYPH_STAGES: [CorpusStage; 4] = [
    CorpusStage::Source,
    CorpusStage::Outline,
    CorpusStage::Vector,
    CorpusStage::Mesh,
];
pub(crate) const PATH_STAGES: [CorpusStage; 3] =
    [CorpusStage::Source, CorpusStage::Vector, CorpusStage::Mesh];
pub(crate) const SVG_STAGES: [CorpusStage; 4] = [
    CorpusStage::Source,
    CorpusStage::Xml,
    CorpusStage::Vector,
    CorpusStage::Mesh,
];
