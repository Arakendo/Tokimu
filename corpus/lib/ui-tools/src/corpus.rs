/// Stable version marker for the shared presentation corpus.
pub const TEXT_CORPUS_VERSION: &str = "text-corpus-v1";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UiTextCorpusGroup {
    PresentationRole,
    Coverage,
    MetricTorture,
    NaturalText,
    AlignmentAndBounds,
    CoverageAndFallback,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiTextCorpusSample {
    pub id: &'static str,
    pub group: UiTextCorpusGroup,
    pub text: &'static str,
}

pub const TEXT_CORPUS: &[UiTextCorpusSample] = &[
    UiTextCorpusSample {
        id: "title",
        group: UiTextCorpusGroup::PresentationRole,
        text: "Tokimu Text Corpus",
    },
    UiTextCorpusSample {
        id: "subtitle",
        group: UiTextCorpusGroup::PresentationRole,
        text: "Semantic text presentation reference",
    },
    UiTextCorpusSample {
        id: "body",
        group: UiTextCorpusGroup::PresentationRole,
        text: "The quick brown fox jumps over the lazy dog.",
    },
    UiTextCorpusSample {
        id: "caption",
        group: UiTextCorpusGroup::PresentationRole,
        text: "Small supporting text",
    },
    UiTextCorpusSample {
        id: "button",
        group: UiTextCorpusGroup::PresentationRole,
        text: "Continue",
    },
    UiTextCorpusSample {
        id: "status",
        group: UiTextCorpusGroup::PresentationRole,
        text: "Ready",
    },
    UiTextCorpusSample {
        id: "uppercase",
        group: UiTextCorpusGroup::Coverage,
        text: "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    },
    UiTextCorpusSample {
        id: "lowercase",
        group: UiTextCorpusGroup::Coverage,
        text: "abcdefghijklmnopqrstuvwxyz",
    },
    UiTextCorpusSample {
        id: "digits",
        group: UiTextCorpusGroup::Coverage,
        text: "0123456789",
    },
    UiTextCorpusSample {
        id: "punctuation",
        group: UiTextCorpusGroup::Coverage,
        text: "! ? + - = / \\ : ; , . () [] {} <> | _ ~ @ # % & *",
    },
    UiTextCorpusSample {
        id: "capital-i",
        group: UiTextCorpusGroup::MetricTorture,
        text: "IIIIIIIIIIII",
    },
    UiTextCorpusSample {
        id: "capital-w",
        group: UiTextCorpusGroup::MetricTorture,
        text: "WWWWWWWWWW",
    },
    UiTextCorpusSample {
        id: "zeros",
        group: UiTextCorpusGroup::MetricTorture,
        text: "0000000000",
    },
    UiTextCorpusSample {
        id: "ones",
        group: UiTextCorpusGroup::MetricTorture,
        text: "1111111111",
    },
    UiTextCorpusSample {
        id: "periods",
        group: UiTextCorpusGroup::MetricTorture,
        text: "...........",
    },
    UiTextCorpusSample {
        id: "lowercase-l",
        group: UiTextCorpusGroup::MetricTorture,
        text: "lllllllllll",
    },
    UiTextCorpusSample {
        id: "lowercase-m",
        group: UiTextCorpusGroup::MetricTorture,
        text: "mmmmmmmmmmm",
    },
    UiTextCorpusSample {
        id: "av-pair",
        group: UiTextCorpusGroup::MetricTorture,
        text: "AVAVAVAVAV",
    },
    UiTextCorpusSample {
        id: "to-pair",
        group: UiTextCorpusGroup::MetricTorture,
        text: "ToToToToTo",
    },
    UiTextCorpusSample {
        id: "wa-pair",
        group: UiTextCorpusGroup::MetricTorture,
        text: "WaWaWaWaWa",
    },
    UiTextCorpusSample {
        id: "sphinx",
        group: UiTextCorpusGroup::NaturalText,
        text: "Sphinx of black quartz, judge my vow.",
    },
    UiTextCorpusSample {
        id: "liquor-jugs",
        group: UiTextCorpusGroup::NaturalText,
        text: "Pack my box with five dozen liquor jugs. 0123456789",
    },
    UiTextCorpusSample {
        id: "hamburgefontsiv",
        group: UiTextCorpusGroup::NaturalText,
        text: "Hamburgefontsiv",
    },
    UiTextCorpusSample {
        id: "start",
        group: UiTextCorpusGroup::AlignmentAndBounds,
        text: "START ALIGNED",
    },
    UiTextCorpusSample {
        id: "center",
        group: UiTextCorpusGroup::AlignmentAndBounds,
        text: "CENTER ALIGNED",
    },
    UiTextCorpusSample {
        id: "end",
        group: UiTextCorpusGroup::AlignmentAndBounds,
        text: "END ALIGNED",
    },
    UiTextCorpusSample {
        id: "clipped",
        group: UiTextCorpusGroup::AlignmentAndBounds,
        text: "CLIPPED TEXT SAMPLE",
    },
    UiTextCorpusSample {
        id: "accented-latin",
        group: UiTextCorpusGroup::CoverageAndFallback,
        text: "Café naïve résumé",
    },
    UiTextCorpusSample {
        id: "greek",
        group: UiTextCorpusGroup::CoverageAndFallback,
        text: "Greek: Alpha Beta Gamma",
    },
    UiTextCorpusSample {
        id: "fallback",
        group: UiTextCorpusGroup::CoverageAndFallback,
        text: "Fallback: こんにちは Привет 中文",
    },
];

pub fn samples(group: UiTextCorpusGroup) -> impl Iterator<Item = &'static UiTextCorpusSample> {
    TEXT_CORPUS
        .iter()
        .filter(move |sample| sample.group == group)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_has_stable_version_and_unique_ids() {
        assert_eq!(TEXT_CORPUS_VERSION, "text-corpus-v1");
        for (index, sample) in TEXT_CORPUS.iter().enumerate() {
            assert!(!sample.id.is_empty());
            assert!(!TEXT_CORPUS[..index]
                .iter()
                .any(|previous| previous.id == sample.id));
        }
    }

    #[test]
    fn corpus_contains_each_required_group() {
        for group in [
            UiTextCorpusGroup::PresentationRole,
            UiTextCorpusGroup::Coverage,
            UiTextCorpusGroup::MetricTorture,
            UiTextCorpusGroup::NaturalText,
            UiTextCorpusGroup::AlignmentAndBounds,
            UiTextCorpusGroup::CoverageAndFallback,
        ] {
            assert!(samples(group).next().is_some());
        }
    }
}
