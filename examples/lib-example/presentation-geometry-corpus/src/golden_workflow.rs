//! Explicit blessing and comparison workflow for reviewed corpus evidence.

use crate::{
    evidence::fnv1a64, goldens, run_case, write_glyph_artifacts, write_svg_artifacts,
    write_synthetic_svg_artifacts, write_w3c_artifacts, CorpusCase, CorpusStage, StageStatus,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Returns the reviewed fixture location for a case report.
pub fn golden_snapshot_path(case_id: &str) -> PathBuf {
    golden_root(case_id).join("report.json")
}

fn golden_mesh_fingerprint_path(case_id: &str) -> PathBuf {
    golden_root(case_id).join("mesh-fingerprint.json")
}

fn golden_image_fingerprint_path(case_id: &str) -> PathBuf {
    golden_root(case_id).join("image-fingerprint.json")
}

fn golden_root(case_id: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/golden/presentation-geometry").join(golden_case_key(case_id))
}

fn golden_case_key(case_id: &str) -> String {
    format!(
        "{}--{:016x}",
        case_id.replace('/', "__"),
        fnv1a64(case_id.as_bytes(), '\0')
    )
}

/// Writes one reviewed structural snapshot. This is intentionally an explicit
/// operation; ordinary corpus runs never mutate fixtures.
pub fn bless_case(case: CorpusCase) -> Result<PathBuf, String> {
    let report = run_case(case);
    if !report.passed() {
        return Err(format!("cannot bless failed case {}", report.id));
    }
    let generated_root = generate_detailed_artifacts(case)?;
    let path = golden_snapshot_path(&report.id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create golden directory: {error}"))?;
    }
    let snapshot = goldens::snapshot(&report);
    let json = serde_json::to_string_pretty(&snapshot)
        .map_err(|error| format!("serialize golden snapshot: {error}"))?;
    fs::write(&path, format!("{json}\n"))
        .map_err(|error| format!("write golden snapshot: {error}"))?;

    if report_produced_mesh(&report.stages) {
        copy_generated_fingerprints(case, generated_root.as_deref(), &report.id)?;
    } else {
        remove_stale_mesh_fingerprint(&report.id)?;
    }
    Ok(path)
}

/// Compares one case with its reviewed structural snapshot without mutating it.
pub fn compare_case(case: CorpusCase) -> Result<(), String> {
    let report = run_case(case);
    if !report.passed() {
        return Err(format!(
            "case {} failed before golden comparison",
            report.id
        ));
    }
    compare_report_snapshot(&report.id, &report)?;

    let generated_root = generate_detailed_artifacts(case)?;
    if report_produced_mesh(&report.stages) {
        compare_mesh_fingerprint(generated_root.as_deref(), &report.id)?;
    }
    if matches!(case, CorpusCase::Glyph(_)) {
        compare_image_fingerprint(
            generated_root
                .as_deref()
                .expect("glyph cases always generate diagnostic artifacts"),
            &report.id,
        )?;
    }
    Ok(())
}

fn generate_detailed_artifacts(case: CorpusCase) -> Result<Option<PathBuf>, String> {
    match case {
        CorpusCase::Glyph(glyph) => write_glyph_artifacts(glyph).map(Some),
        CorpusCase::Svg(svg) => write_svg_artifacts(svg).map(Some),
        CorpusCase::SyntheticSvg(svg) => write_synthetic_svg_artifacts(svg).map(Some),
        CorpusCase::W3cSvg(w3c) => write_w3c_artifacts(w3c).map(Some),
        CorpusCase::Cgm(cgm) => crate::write_cgm_artifacts(cgm).map(Some),
        CorpusCase::Synthetic(_) | CorpusCase::Ui(_) => Ok(None),
    }
}

fn report_produced_mesh(stages: &[crate::StageReport]) -> bool {
    stages
        .iter()
        .any(|stage| stage.stage == CorpusStage::Mesh && stage.status == StageStatus::Ready)
}

fn copy_generated_fingerprints(
    case: CorpusCase,
    generated_root: Option<&Path>,
    case_id: &str,
) -> Result<(), String> {
    let Some(root) = generated_root.filter(|root| root.join("mesh-fingerprint.json").is_file())
    else {
        return Ok(());
    };

    fs::copy(
        root.join("mesh-fingerprint.json"),
        golden_mesh_fingerprint_path(case_id),
    )
    .map_err(|error| format!("write golden mesh fingerprint: {error}"))?;
    if matches!(case, CorpusCase::Glyph(_)) {
        fs::copy(
            root.join("image-fingerprint.json"),
            golden_image_fingerprint_path(case_id),
        )
        .map_err(|error| format!("write golden image fingerprint: {error}"))?;
    }
    Ok(())
}

fn remove_stale_mesh_fingerprint(case_id: &str) -> Result<(), String> {
    let stale_mesh = golden_mesh_fingerprint_path(case_id);
    if stale_mesh.is_file() {
        fs::remove_file(&stale_mesh)
            .map_err(|error| format!("remove stale golden mesh fingerprint: {error}"))?;
    }
    Ok(())
}

fn compare_report_snapshot(case_id: &str, report: &crate::CaseReport) -> Result<(), String> {
    let path = golden_snapshot_path(case_id);
    let expected = fs::read_to_string(&path)
        .map_err(|error| format!("read golden {}: {error}", path.display()))?;
    let actual = serde_json::to_string_pretty(&goldens::snapshot(report))
        .map_err(|error| format!("serialize golden snapshot: {error}"))?
        + "\n";
    if expected != actual {
        return Err(format!(
            "golden mismatch: {}\n{}",
            path.display(),
            goldens::first_difference(&expected, &actual)
        ));
    }
    Ok(())
}

fn compare_mesh_fingerprint(generated_root: Option<&Path>, case_id: &str) -> Result<(), String> {
    let Some(root) = generated_root.filter(|root| root.join("mesh-fingerprint.json").is_file())
    else {
        return Ok(());
    };
    let mesh_path = golden_mesh_fingerprint_path(case_id);
    let expected = fs::read_to_string(&mesh_path)
        .map_err(|error| format!("read golden {}: {error}", mesh_path.display()))?;
    let actual = fs::read_to_string(root.join("mesh-fingerprint.json"))
        .map_err(|error| format!("read generated mesh fingerprint: {error}"))?;
    if expected != actual {
        return Err(format!(
            "golden mesh mismatch: {}\n{}",
            mesh_path.display(),
            goldens::first_difference(&expected, &actual)
        ));
    }
    Ok(())
}

fn compare_image_fingerprint(generated_root: &Path, case_id: &str) -> Result<(), String> {
    let image_path = golden_image_fingerprint_path(case_id);
    let expected = fs::read_to_string(&image_path)
        .map_err(|error| format!("read golden {}: {error}", image_path.display()))?;
    let actual = fs::read_to_string(generated_root.join("image-fingerprint.json"))
        .map_err(|error| format!("read generated image fingerprint: {error}"))?;
    if expected != actual {
        return Err(format!(
            "golden image mismatch: {}\n{}",
            image_path.display(),
            goldens::first_difference(&expected, &actual)
        ));
    }
    Ok(())
}
