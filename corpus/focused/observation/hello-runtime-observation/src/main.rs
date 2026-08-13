use hello_runtime_observation::{
    build_session, load_hole_punch_catalog, sample_hole_punch_translations,
    scripted_command_requests, serialize_observation, CommandTrace, ObservationLimits,
    PlaybackCommand, PlaybackPolicy, PlaybackState, PresentationCommand, ScenarioPresentation,
};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = build_session(8);
    let summary = session.observe(0, None, ObservationLimits::default());
    let detail = session.observe(1, Some(session.arm_id()), ObservationLimits::default());
    for request in scripted_command_requests(&session) {
        let result = session.enqueue(request);
        assert_eq!(
            result.disposition,
            hello_runtime_observation::CommandDisposition::Queued
        );
    }
    let command_trace = session.apply_pending_at_tick(1);
    let final_detail = session.observe(2, Some(session.arm_id()), ObservationLimits::default());
    let summary_path = output_directory().join("world-summary.json");
    let detail_path = output_directory().join("world-selected-detail.json");
    let trace_path = output_directory().join("command-trace.json");
    let final_detail_path = output_directory().join("world-final-selected-detail.json");
    write_observation(&summary_path, &summary)?;
    write_observation(&detail_path, &detail)?;
    write_trace(&trace_path, &command_trace)?;
    write_observation(&final_detail_path, &final_detail)?;

    let catalog = load_hole_punch_catalog()?;
    let mut playback = PlaybackState::initial(PlaybackPolicy {
        hold_completed_steps: true,
    });
    let play = playback.apply_command(&catalog, PlaybackCommand::Play { clip: 1 });
    playback.advance_fixed_step(&catalog);
    let pause = playback.apply_command(&catalog, PlaybackCommand::Pause);
    let translations = sample_hole_punch_translations(&playback)?;
    let catalog_path = output_directory().join("animation-catalog.json");
    let playback_path = output_directory().join("playback-evidence.json");
    write_json(&catalog_path, &catalog)?;
    write_json(
        &playback_path,
        &serde_json::json!({
            "owner": "application",
            "fixed_step_seconds": 1.0_f32 / 60.0_f32,
            "results": [play, pause],
            "state": playback,
            "sampled_translations": translations,
        }),
    )?;

    let mut presentation = ScenarioPresentation::for_hole_punch(session.arm_id().0);
    let arm_mapping = presentation
        .mapping_for_entity(session.arm_id().0)
        .expect("the scenario arm must have one explicit presentation mapping")
        .clone();
    let select = presentation.apply(PresentationCommand::Select {
        target: arm_mapping.presentation_target.clone(),
    });
    let hotspot = presentation.apply(PresentationCommand::SetHotspot {
        target: arm_mapping.presentation_target.clone(),
    });
    let clear_hotspot = presentation.apply(PresentationCommand::ClearHotspot {
        target: arm_mapping.presentation_target,
    });
    let presentation_path = output_directory().join("presentation-mapping.json");
    write_json(
        &presentation_path,
        &serde_json::json!({
            "owner": "application_presentation_adapter",
            "commands": [select, hotspot, clear_hotspot],
            "observation": presentation.observe(),
        }),
    )?;

    println!(
        "entities={} component_types={} resource_types={} relationship_types={} selected={}",
        summary.payload.entity_count,
        summary.payload.component_types.len(),
        summary.payload.resource_types.len(),
        summary.payload.relationship_types.len(),
        detail
            .payload
            .selected
            .as_ref()
            .map(|selected| selected.entity.to_string())
            .unwrap_or_else(|| "none".to_owned()),
    );
    println!("artifact={}", summary_path.display());
    println!("artifact={}", detail_path.display());
    println!("artifact={}", trace_path.display());
    println!("artifact={}", final_detail_path.display());
    println!("artifact={}", catalog_path.display());
    println!("artifact={}", playback_path.display());
    println!("artifact={}", presentation_path.display());
    Ok(())
}

fn write_trace(
    output: &std::path::Path,
    trace: &CommandTrace,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, serde_json::to_vec_pretty(trace)?)?;
    Ok(())
}

fn write_json<T: Serialize>(
    output: &std::path::Path,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn write_observation(
    output: &std::path::Path,
    observation: &hello_runtime_observation::ObservationEnvelope,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = serialize_observation(observation)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, &bytes)?;
    Ok(())
}

fn output_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .join("target/runtime-observation-command/hello-runtime-observation")
}
