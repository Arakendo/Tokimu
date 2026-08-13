//! Headless MilkDrop parser and scalar-evaluator corpus consumer.
//!
//! This executable records selected scalar and literal custom-wave evidence.
//! It does not execute per-pixel or custom-wave equations, open an audio
//! device, compile shaders, or render a frame.

use std::{error::Error, fs, path::PathBuf};

use milkdrop_tools::{
    evaluate_selected_equations, inspect_shader_entries, resolve_selected_custom_shapes,
    resolve_selected_custom_waves, resolve_selected_parameters, MilkDropEvaluationPhase,
    MilkDropEvaluationState, MilkDropPresetDocument,
};
use serde_json::json;

type AppResult<T> = Result<T, Box<dyn Error>>;

const FIXTURE_SOURCE: &str = include_str!("../assets/tokimu-selected-fixture.milk");
const EQUATION_MATRIX_SOURCE: &str = include_str!("../assets/tokimu-equation-matrix.milk");
const CONSTRUCT_MATRIX_SOURCE: &str = include_str!("../assets/tokimu-construct-matrix.milk");

struct Fixture<'a> {
    identity: &'a str,
    source: &'a str,
}

fn main() -> AppResult<()> {
    let fixtures = [
        Fixture {
            identity: "tokimu-selected-fixture",
            source: FIXTURE_SOURCE,
        },
        Fixture {
            identity: "tokimu-equation-matrix",
            source: EQUATION_MATRIX_SOURCE,
        },
        Fixture {
            identity: "tokimu-construct-matrix",
            source: CONSTRUCT_MATRIX_SOURCE,
        },
    ];

    let artifacts = fixtures
        .iter()
        .map(build_artifact)
        .collect::<Result<Vec<_>, _>>()?;

    if std::env::args().any(|argument| argument == "--write-artifacts") {
        let output = PathBuf::from("target/hello-milkdrop");
        fs::create_dir_all(&output)?;
        for artifact in &artifacts {
            let identity = artifact["fixture"]["identity"]
                .as_str()
                .expect("fixture identities are static");
            fs::write(
                output.join(format!("{identity}.inspection.json")),
                format!("{}\n", serde_json::to_string_pretty(artifact)?),
            )?;
        }
        println!(
            "wrote {} MilkDrop inspection artifact(s) under {}",
            artifacts.len(),
            output.display()
        );
    }

    for artifact in artifacts {
        let fixture = artifact["fixture"]["identity"]
            .as_str()
            .expect("fixture identities are static");
        let inspection = &artifact["inspection"];
        let execution = &artifact["execution"];
        println!(
            "hello-milkdrop {fixture}: sections={}, entries={}, deferred={}, unsupported={}, initialization_equations={}, per_frame_equations={}, renderer=false",
            inspection["sections"].as_array().map_or(0, Vec::len),
            inspection["sections"]
                .as_array()
                .map(|sections| sections.iter().map(|section| section["entries"].as_array().map_or(0, Vec::len)).sum::<usize>())
                .unwrap_or_default(),
            inspection["deferred_entries"].as_u64().unwrap_or_default(),
            inspection["unsupported_entries"].as_u64().unwrap_or_default(),
            execution["initialization_equations"].as_u64().unwrap_or_default(),
            execution["per_frame_equations"].as_u64().unwrap_or_default(),
        );
    }
    Ok(())
}

fn build_artifact(fixture: &Fixture<'_>) -> AppResult<serde_json::Value> {
    let document = MilkDropPresetDocument::parse(fixture.source)?;
    let parameters = resolve_selected_parameters(&document)?;
    let custom_waves = resolve_selected_custom_waves(&document)?;
    let custom_shapes = resolve_selected_custom_shapes(&document)?;
    let shader_inspection = inspect_shader_entries(&document);
    let mut evaluation = MilkDropEvaluationState::default();
    let initialization_equations = evaluate_selected_equations(
        &document,
        MilkDropEvaluationPhase::Initialization,
        &mut evaluation,
    )?;
    let per_frame_equations = evaluate_selected_equations(
        &document,
        MilkDropEvaluationPhase::PerFrame,
        &mut evaluation,
    )?;
    verify_fixture_expectations(fixture.identity, &evaluation)?;
    verify_fixture_classifications(fixture.identity, &document)?;
    let artifact = json!({
        "schema": "tokimu-milkdrop-parser-corpus-v1",
        "producer": "hello-milkdrop",
        "fixture": {
            "identity": fixture.identity,
            "origin": "Tokimu-authored",
            "external_preset": false,
            "source_bytes": fixture.source.len(),
        },
        "inspection": serde_json::from_str::<serde_json::Value>(&document.to_structural_json()?)?,
        "execution": {
            "selected_parameters": parameters,
            "selected_custom_waves": custom_waves,
            "selected_custom_shapes": custom_shapes,
            "custom_wave_equations_executed": false,
            "custom_shape_equations_executed": false,
            "custom_wave_renderer_required": false,
            "custom_shape_renderer_required": false,
            "equations_evaluated": true,
            "initialization_equations": initialization_equations,
            "per_frame_equations": per_frame_equations,
            "per_pixel_equations_deferred": document.sections.iter().flat_map(|section| &section.entries)
                .filter(|entry| entry.construct == milkdrop_tools::MilkDropConstruct::PerPixelEquation)
                .count(),
            "shader_entries": shader_inspection,
            "scalar_state": evaluation,
            "audio_device_required": false,
            "renderer_required": false,
            "shader_translation_performed": false,
            "shader_compilation_performed": false,
        },
    });
    Ok(artifact)
}

fn verify_fixture_expectations(fixture: &str, state: &MilkDropEvaluationState) -> AppResult<()> {
    let expected: &[(&str, f64)] = match fixture {
        "tokimu-selected-fixture" => &[("q1", 1.0)],
        "tokimu-equation-matrix" => &[
            ("q1", 16.0),
            ("q2", 3.0),
            ("q3", 2.5),
            ("q4", 2.5),
            ("q5", 19.0),
            ("q6", 16.5),
        ],
        _ => return Ok(()),
    };
    for (name, value) in expected {
        let actual = state.value(name).ok_or_else(|| {
            format!("fixture `{fixture}` did not define expected variable `{name}`")
        })?;
        if (actual - value).abs() > f64::EPSILON {
            return Err(format!(
                "fixture `{fixture}` evaluated `{name}` as {actual}, expected {value}"
            )
            .into());
        }
    }
    Ok(())
}

fn verify_fixture_classifications(
    fixture: &str,
    document: &MilkDropPresetDocument,
) -> AppResult<()> {
    if fixture != "tokimu-construct-matrix" {
        return Ok(());
    }

    use milkdrop_tools::MilkDropConstruct;
    let expected = [
        (MilkDropConstruct::ScalarParameter, 11_usize),
        (MilkDropConstruct::InitEquation, 1),
        (MilkDropConstruct::PerFrameEquation, 1),
        (MilkDropConstruct::PerPixelEquation, 1),
        (MilkDropConstruct::UnsupportedCustomWave, 2),
        (MilkDropConstruct::UnsupportedCustomShape, 2),
        (MilkDropConstruct::UnsupportedWarpShader, 1),
        (MilkDropConstruct::UnsupportedCompositeShader, 1),
        (MilkDropConstruct::UnsupportedUnknown, 1),
    ];

    for (construct, expected_count) in expected {
        let actual = document
            .sections
            .iter()
            .flat_map(|section| &section.entries)
            .filter(|entry| entry.construct == construct)
            .count();
        if actual != expected_count {
            return Err(format!(
                "fixture `{fixture}` classified {actual} entries as {construct:?}, expected {expected_count}"
            )
            .into());
        }
    }
    if document.deferred_entries != 1 || document.unsupported_entries != 7 {
        return Err(format!(
            "fixture `{fixture}` recorded deferred={} and unsupported={}, expected 1 and 7",
            document.deferred_entries, document.unsupported_entries
        )
        .into());
    }
    Ok(())
}
