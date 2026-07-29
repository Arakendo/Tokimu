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
    ExpectedInvalidInput,
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

    pub const fn expected_invalid_input(
        id: &'static str,
        file_name: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            id,
            file_name,
            description,
            expectation: W3cSvgExpectation::ExpectedInvalidInput,
            source: W3cSvgSource::DerivedProfileFixture,
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

/// A pinned WebCGM source fixture admitted for structural producer evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CgmCase {
    pub id: &'static str,
    pub file_name: &'static str,
    pub description: &'static str,
    pub expectation: CgmExpectation,
}

impl CgmCase {
    pub const fn new(id: &'static str, file_name: &'static str, description: &'static str) -> Self {
        Self {
            id,
            file_name,
            description,
            expectation: CgmExpectation::VectorPass,
        }
    }

    pub const fn expected_unsupported_lowering(
        id: &'static str,
        file_name: &'static str,
        description: &'static str,
        kind: &'static str,
    ) -> Self {
        Self {
            id,
            file_name,
            description,
            expectation: CgmExpectation::ExpectedUnsupportedLowering { kind },
        }
    }

    /// Registers a fixture whose current value is source-format evidence only.
    ///
    /// This is intentionally not an expected vector failure: no vector
    /// lowering was requested for the case, so the selected pipeline ends at
    /// inspection rather than claiming an unimplemented geometric contract.
    pub const fn source_only(
        id: &'static str,
        file_name: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            id,
            file_name,
            description,
            expectation: CgmExpectation::SourceOnly,
        }
    }

    pub(crate) const fn selected_stages(self) -> &'static [CorpusStage] {
        match self.expectation {
            CgmExpectation::SourceOnly => &CGM_SOURCE_STAGES,
            CgmExpectation::VectorPass | CgmExpectation::ExpectedUnsupportedLowering { .. } => {
                &CGM_STAGES
            }
        }
    }
}

/// The selected CGM profile may intentionally stop before a source primitive
/// has a provider-neutral representation. This preserves the diagnostic
/// boundary as corpus evidence instead of omitting the fixture entirely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CgmExpectation {
    SourceOnly,
    VectorPass,
    ExpectedUnsupportedLowering { kind: &'static str },
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
    Cgm(CgmCase),
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
            Self::Cgm(case) => case.id,
        }
    }

    pub const fn selected_stages(self) -> &'static [CorpusStage] {
        match self {
            Self::Glyph(_) => &GLYPH_STAGES,
            Self::Svg(_) | Self::SyntheticSvg(_) | Self::W3cSvg(_) => &SVG_STAGES,
            Self::Synthetic(_) | Self::Ui(_) => &PATH_STAGES,
            Self::Cgm(case) => case.selected_stages(),
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
pub(crate) const CGM_STAGES: [CorpusStage; 2] = [CorpusStage::Source, CorpusStage::Vector];
pub(crate) const CGM_SOURCE_STAGES: [CorpusStage; 1] = [CorpusStage::Source];
