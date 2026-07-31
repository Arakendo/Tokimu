//! Bounded execution for the explicitly admitted external raster fixtures.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use screenshot::{write_bmp, write_manifest, Rgba8Image};
use serde::Serialize;

use crate::{
    decode_bmp, decode_jpeg, decode_png, prepare_renderer_texture, DecodeLimits, TextureUse,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RasterCaseArtifact {
    pub schema: u32,
    pub producer: &'static str,
    pub case_id: &'static str,
    pub format: &'static str,
    pub feature: &'static str,
    pub expected_stage: &'static str,
    pub expected: &'static str,
    pub source_fingerprint_algorithm: &'static str,
    pub source_fingerprint: String,
    pub decode_limits: DecodeLimits,
    pub actual: &'static str,
    /// Observed decode duration for this runner invocation. This is evidence,
    /// not a correctness threshold.
    pub decode_elapsed_micros: u64,
    pub decoded_image: Option<crate::DecodedImageArtifact>,
    pub diagnostic: Option<String>,
}

/// A bounded selection over the review cases declared by this corpus.
///
/// Empty fields retain every case. This is intentionally metadata-only: it
/// never searches fixture directories or infers additional test cases.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RasterCaseFilter {
    pub format: Option<String>,
    pub feature: Option<String>,
    pub expected: Option<String>,
    pub expected_stage: Option<String>,
}

struct SelectedCase {
    id: &'static str,
    format: &'static str,
    feature: &'static str,
    source: &'static str,
    expected_stage: &'static str,
    expected: &'static str,
}

const CASES: &[SelectedCase] = &[
    SelectedCase { id: "png-basn0g08", format: "png", feature: "grayscale-eight-bit", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn0g08.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-basn2c08", format: "png", feature: "rgb-eight-bit", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn2c08.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-basn3p08", format: "png", feature: "palette-eight-bit", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn3p08.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-basn4a08", format: "png", feature: "grayscale-alpha-eight-bit", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn4a08.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-basn6a08", format: "png", feature: "rgba-eight-bit", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn6a08.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-tp1n3p08", format: "png", feature: "palette-transparency", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/tp1n3p08.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-xcrn0g04", format: "png", feature: "crc-validation", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/xcrn0g04.png", expected_stage: "decode", expected: "candidate-rejection" },
    SelectedCase { id: "png-basn6a16", format: "png", feature: "sixteen-bit-rgba", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn6a16.png", expected_stage: "profile", expected: "candidate-rejection" },
    SelectedCase { id: "png-basi6a08", format: "png", feature: "adam7-rgba", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basi6a08.png", expected_stage: "profile", expected: "candidate-rejection" },
    SelectedCase { id: "png-f00n2c08", format: "png", feature: "filter-none", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/f00n2c08.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-f01n2c08", format: "png", feature: "filter-sub", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/f01n2c08.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-f02n2c08", format: "png", feature: "filter-up", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/f02n2c08.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-f03n2c08", format: "png", feature: "filter-average", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/f03n2c08.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-f04n2c08", format: "png", feature: "filter-paeth", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/f04n2c08.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-s01n3p01", format: "png", feature: "one-pixel-indexed", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/s01n3p01.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-s07i3p02", format: "png", feature: "adam7-small-indexed", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/s07i3p02.png", expected_stage: "profile", expected: "candidate-rejection" },
    SelectedCase { id: "png-s33n3p04", format: "png", feature: "non-power-of-two-indexed", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/s33n3p04.png", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "png-x00n0g01", format: "png", feature: "empty-image", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/x00n0g01.png", expected_stage: "decode", expected: "candidate-rejection" },
    SelectedCase { id: "png-xlfn0g04", format: "png", feature: "line-feed-corruption", source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/xlfn0g04.png", expected_stage: "decode", expected: "candidate-rejection" },
    SelectedCase { id: "jpeg-baseline-testorig", format: "jpeg", feature: "baseline-ycbcr", source: "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/testorig.jpg", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "jpeg-baseline-testimgint", format: "jpeg", feature: "baseline-ycbcr", source: "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/testimgint.jpg", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "jpeg-baseline-grayscale-square", format: "jpeg", feature: "baseline-grayscale", source: "../../../third-party/fixtures/raster-images/upstream/jpeg-decoder/grayscale_square.jpg", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "jpeg-arithmetic-sequential", format: "jpeg", feature: "arithmetic-sequential", source: "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/testimgari.jpg", expected_stage: "profile", expected: "candidate-rejection" },
    SelectedCase { id: "jpeg-extended-precision", format: "jpeg", feature: "twelve-bit-precision", source: "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/monkey12.jpg", expected_stage: "profile", expected: "candidate-rejection" },
    SelectedCase { id: "bmp-shira-bird8", format: "bmp", feature: "ordinary-bgr", source: "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/shira_bird8.bmp", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "bmp-vgl-6434-0018", format: "bmp", feature: "odd-width-row-padding", source: "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/vgl_6434_0018a.bmp", expected_stage: "decode", expected: "candidate-pass" },
    SelectedCase { id: "bmp-vgl-6548-0026", format: "bmp", feature: "non-four-height", source: "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/vgl_6548_0026a.bmp", expected_stage: "decode", expected: "candidate-pass" },
];

pub fn run_selected_cases() -> Result<Vec<RasterCaseArtifact>, String> {
    run_selected_cases_with_filter(&RasterCaseFilter::default())
}

pub fn run_selected_cases_with_filter(
    filter: &RasterCaseFilter,
) -> Result<Vec<RasterCaseArtifact>, String> {
    CASES
        .iter()
        .filter(|case| case_matches_filter(case, filter))
        .map(run_case)
        .collect()
}

pub fn write_selected_artifacts(output_root: impl AsRef<Path>) -> Result<Vec<PathBuf>, String> {
    write_selected_artifacts_with_filter(output_root, &RasterCaseFilter::default())
}

pub fn write_selected_artifacts_with_filter(
    output_root: impl AsRef<Path>,
    filter: &RasterCaseFilter,
) -> Result<Vec<PathBuf>, String> {
    let output_root = output_root.as_ref();
    let artifacts = run_selected_cases_with_filter(filter)?;
    clear_owned_review_outputs(output_root)?;

    artifacts
        .into_iter()
        .map(|artifact| {
            let path = output_root.join(format!("{}.json", artifact.case_id));
            let parent = path
                .parent()
                .ok_or_else(|| format!("artifact path has no parent: {}", path.display()))?;
            fs::create_dir_all(parent)
                .map_err(|error| format!("create {}: {error}", parent.display()))?;
            let json =
                serde_json::to_string_pretty(&artifact).map_err(|error| error.to_string())?;
            fs::write(&path, format!("{json}\n"))
                .map_err(|error| format!("write {}: {error}", path.display()))?;
            let mut paths = vec![path];
            if artifact.actual == "decoded" {
                let image = decode_case(case_for_id(artifact.case_id)?)?;
                let texture =
                    prepare_renderer_texture(&image, TextureUse::ColorSrgb).map_err(|error| {
                        format!("prepare renderer texture for {}: {error}", artifact.case_id)
                    })?;
                let texture_path = output_root.join(format!("{}.texture.json", artifact.case_id));
                let texture_json = texture
                    .artifact_json()
                    .map_err(|error| format!("serialize {}: {error}", texture_path.display()))?;
                fs::write(&texture_path, texture_json)
                    .map_err(|error| format!("write {}: {error}", texture_path.display()))?;
                let preview_path = output_root.join(format!("{}.bmp", artifact.case_id));
                write_bmp(
                    &preview_path,
                    Rgba8Image {
                        width: image.width,
                        height: image.height,
                        pixels: &image.pixels,
                    },
                )
                .map_err(|error| format!("write {}: {error}", preview_path.display()))?;
                let preview_manifest =
                    output_root.join(format!("{}.preview.txt", artifact.case_id));
                write_manifest(
                    &preview_manifest,
                    &[
                        ("source_stage", "decoded-image"),
                        ("capture_kind", "deterministic-cpu-export"),
                        ("gpu_framebuffer_capture", "false"),
                        ("format", "bmp-bgra8-top-down"),
                        ("pixel_fingerprint", &image.pixel_fingerprint()),
                    ],
                )
                .map_err(|error| format!("write {}: {error}", preview_manifest.display()))?;
                paths.push(texture_path);
                paths.push(preview_path);
                paths.push(preview_manifest);
            }
            Ok(paths)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|groups| groups.into_iter().flatten().collect())
}

/// Clears only generated artifact names owned by the fixed review set.
///
/// A filtered run is a complete view of its declared selection, so stale output
/// from a previous filter must not be mistaken for part of the current review.
/// Fixture files and unrelated output-root entries are intentionally untouched.
fn clear_owned_review_outputs(output_root: &Path) -> Result<(), String> {
    for case in CASES {
        for suffix in [".json", ".texture.json", ".bmp", ".preview.txt"] {
            let path = output_root.join(format!("{}{}", case.id, suffix));
            if path.exists() {
                fs::remove_file(&path)
                    .map_err(|error| format!("remove stale {}: {error}", path.display()))?;
            }
        }
    }
    Ok(())
}

fn case_for_id(id: &str) -> Result<&'static SelectedCase, String> {
    CASES
        .iter()
        .find(|case| case.id == id)
        .ok_or_else(|| format!("unknown static review case {id}"))
}

fn case_matches_filter(case: &SelectedCase, filter: &RasterCaseFilter) -> bool {
    filter
        .format
        .as_deref()
        .is_none_or(|format| format == case.format)
        && filter
            .feature
            .as_deref()
            .is_none_or(|feature| feature == case.feature)
        && filter
            .expected
            .as_deref()
            .is_none_or(|expected| expected == case.expected)
        && filter
            .expected_stage
            .as_deref()
            .is_none_or(|stage| stage == case.expected_stage)
}

fn run_case(case: &SelectedCase) -> Result<RasterCaseArtifact, String> {
    let source = read_case_source(case)?;
    let limits = DecodeLimits::default();
    let started = Instant::now();
    let decoded = decode_source(case, &source, limits);
    let decode_elapsed_micros = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let (actual, decoded_image, diagnostic) = match decoded {
        Ok(image) => (
            "decoded",
            Some(image.artifact().map_err(|error| error.to_string())?),
            None,
        ),
        Err(error) => ("rejected", None, Some(error.to_string())),
    };
    Ok(RasterCaseArtifact {
        schema: 3,
        producer: "raster-image-corpus",
        case_id: case.id,
        format: case.format,
        feature: case.feature,
        expected_stage: case.expected_stage,
        expected: case.expected,
        source_fingerprint_algorithm: "fnv1a64",
        source_fingerprint: source_fingerprint(&source),
        decode_limits: limits,
        actual,
        decode_elapsed_micros,
        decoded_image,
        diagnostic,
    })
}

fn decode_case(case: &SelectedCase) -> Result<crate::DecodedImage, String> {
    let source = read_case_source(case)?;
    decode_source(case, &source, DecodeLimits::default())
}

fn read_case_source(case: &SelectedCase) -> Result<Vec<u8>, String> {
    fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(case.source))
        .map_err(|error| format!("read {}: {error}", case.source))
}

fn decode_source(
    case: &SelectedCase,
    source: &[u8],
    limits: DecodeLimits,
) -> Result<crate::DecodedImage, String> {
    match case.format {
        "png" => decode_png(source, limits),
        "jpeg" => decode_jpeg(source, limits),
        "bmp" => decode_bmp(source, limits),
        _ => unreachable!("selected case formats are static"),
    }
    .map_err(|error| error.to_string())
}

fn source_fingerprint(bytes: &[u8]) -> String {
    let hash = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::{
        run_selected_cases, run_selected_cases_with_filter, write_selected_artifacts_with_filter,
        RasterCaseFilter,
    };

    #[test]
    fn bounded_review_set_matches_its_declared_outcomes() {
        let artifacts = run_selected_cases().expect("selected fixtures should be readable");
        assert_eq!(artifacts.len(), 27);
        for artifact in artifacts {
            assert!(artifact.decode_elapsed_micros < u64::MAX);
            match artifact.expected {
                "candidate-pass" => assert_eq!(artifact.actual, "decoded", "{}", artifact.case_id),
                "candidate-rejection" => {
                    assert_eq!(artifact.actual, "rejected", "{}", artifact.case_id)
                }
                unexpected => panic!("unexpected static expectation {unexpected}"),
            }
        }
    }

    #[test]
    fn review_cases_can_be_filtered_without_discovering_new_inputs() {
        let artifacts = run_selected_cases_with_filter(&RasterCaseFilter {
            format: Some("jpeg".to_owned()),
            expected: Some("candidate-rejection".to_owned()),
            feature: Some("arithmetic-sequential".to_owned()),
            ..RasterCaseFilter::default()
        })
        .expect("selected JPEG fixtures should be readable");

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].case_id, "jpeg-arithmetic-sequential");
        assert_eq!(artifacts[0].feature, "arithmetic-sequential");
    }

    #[test]
    fn decoded_cases_emit_pre_gpu_texture_and_cpu_preview_evidence_for_each_format() {
        let output = std::env::temp_dir().join(format!(
            "tokimu-raster-review-preview-{}",
            std::process::id()
        ));
        for (format, case_id) in [
            ("png", "png-tp1n3p08"),
            ("jpeg", "jpeg-baseline-testorig"),
            ("jpeg", "jpeg-baseline-grayscale-square"),
            ("bmp", "bmp-shira-bird8"),
        ] {
            let paths = write_selected_artifacts_with_filter(
                &output,
                &RasterCaseFilter {
                    format: Some(format.to_owned()),
                    feature: None,
                    expected: Some("candidate-pass".to_owned()),
                    expected_stage: None,
                },
            )
            .expect("decoded review cases should export");

            assert!(
                paths
                    .iter()
                    .any(|path| path.ends_with(format!("{case_id}.texture.json"))),
                "{format} should emit texture preparation evidence"
            );
            let texture = std::fs::read_to_string(output.join(format!("{case_id}.texture.json")))
                .expect("texture preparation artifact should exist");
            assert!(texture.contains("\"artifact_kind\": \"texture-upload-preparation\""));
            assert!(texture.contains("\"gpu_upload_performed\": false"));
            let manifest = std::fs::read_to_string(output.join(format!("{case_id}.preview.txt")))
                .expect("preview manifest should exist");
            assert!(manifest.contains("capture_kind=deterministic-cpu-export"));
            assert!(manifest.contains("gpu_framebuffer_capture=false"));
            assert!(output.join(format!("{case_id}.bmp")).is_file());
        }
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn rejected_cases_stop_at_structural_evidence() {
        let output = std::env::temp_dir().join(format!(
            "tokimu-raster-review-rejection-{}",
            std::process::id()
        ));
        let paths = write_selected_artifacts_with_filter(
            &output,
            &RasterCaseFilter {
                format: Some("jpeg".to_owned()),
                expected: Some("candidate-rejection".to_owned()),
                feature: Some("arithmetic-sequential".to_owned()),
                ..RasterCaseFilter::default()
            },
        )
        .expect("rejected JPEG review case should record its diagnostic");

        assert_eq!(paths.len(), 1);
        assert!(output.join("jpeg-arithmetic-sequential.json").is_file());
        assert!(!output
            .join("jpeg-arithmetic-sequential.texture.json")
            .exists());
        assert!(!output.join("jpeg-arithmetic-sequential.bmp").exists());
        assert!(!output
            .join("jpeg-arithmetic-sequential.preview.txt")
            .exists());
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn filtered_runs_replace_only_stale_corpus_owned_outputs() {
        let output = std::env::temp_dir().join(format!(
            "tokimu-raster-review-filter-replacement-{}",
            std::process::id()
        ));
        write_selected_artifacts_with_filter(&output, &RasterCaseFilter::default())
            .expect("full review set should export");
        assert!(output.join("jpeg-baseline-testorig.json").is_file());

        write_selected_artifacts_with_filter(
            &output,
            &RasterCaseFilter {
                format: Some("jpeg".to_owned()),
                feature: Some("arithmetic-sequential".to_owned()),
                ..RasterCaseFilter::default()
            },
        )
        .expect("filtered review set should export");

        assert!(!output.join("jpeg-baseline-testorig.json").exists());
        assert!(output.join("jpeg-arithmetic-sequential.json").is_file());
        let _ = std::fs::remove_dir_all(output);
    }
}
