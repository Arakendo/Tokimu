use std::{error::Error, fs, path::PathBuf};

use serde_json::{json, Value};
use visualizer_tools::{
    decode_pcm16_wav, encode_pcm16_wav_fixture, observe_pcm_analysis_timing,
    observe_pcm_analysis_working_set, PcmAnalysisConfig, PcmAnalyzer, PcmFixture,
};

type AppResult<T> = Result<T, Box<dyn Error>>;

const ARTIFACT_SCHEMA: &str = "tokimu-audio-analysis-inspection-v1";
const TIMING_ITERATIONS: usize = 32;

fn main() -> AppResult<()> {
    let artifacts = inspect_all_fixtures()?;
    if std::env::args().any(|argument| argument == "--write-artifacts") {
        write_artifacts(&artifacts)?;
    }

    println!(
        "hello-audio-analysis inspected {} generated PCM16 WAVE fixtures; renderer=false, audio_device=false, playback=false",
        artifacts.len()
    );
    for artifact in &artifacts {
        println!(
            "  {}: {} Hz, {} channel(s), {} frames",
            artifact["fixture"].as_str().unwrap_or("unknown"),
            artifact["decoded_window"]["sample_rate_hz"]
                .as_u64()
                .unwrap_or_default(),
            artifact["decoded_window"]["channels"]
                .as_u64()
                .unwrap_or_default(),
            artifact["decoded_window"]["frames"]
                .as_u64()
                .unwrap_or_default(),
        );
    }
    Ok(())
}

fn inspect_all_fixtures() -> AppResult<Vec<Value>> {
    PcmFixture::ALL.into_iter().map(inspect_fixture).collect()
}

fn inspect_fixture(fixture: PcmFixture) -> AppResult<Value> {
    let source_bytes = encode_pcm16_wav_fixture(fixture);
    let window = decode_pcm16_wav(&source_bytes)?;
    let config = PcmAnalysisConfig::default();
    let analysis = PcmAnalyzer::analyze(&window, config)?;
    let timing = observe_pcm_analysis_timing(&window, config, TIMING_ITERATIONS)?;
    let working_set = observe_pcm_analysis_working_set(&window, config)?;

    Ok(json!({
        "schema": ARTIFACT_SCHEMA,
        "producer": "hello-audio-analysis",
        "fixture": fixture.label(),
        "source": {
            "kind": "generated-riff-wave-pcm16-little-endian",
            "bytes": source_bytes.len(),
            "fingerprint": format!("fnv1a64:{:016x}", fnv1a64(&source_bytes)),
            "provider_scope": "corpus-byte-source-adapter-not-playback-or-capture",
        },
        "decoded_window": {
            "sample_rate_hz": window.sample_rate_hz,
            "channels": window.channels,
            "frames": window.frame_count(),
            "normalized_pcm": true,
        },
        "analysis": serde_json::from_str::<Value>(&analysis.to_structural_json()?)?,
        "timing": serde_json::from_str::<Value>(&timing.to_observation_json()?)?,
        "working_set": serde_json::from_str::<Value>(&working_set.to_observation_json()?)?,
        "renderer_required": false,
        "audio_device_required": false,
        "playback_performed": false,
    }))
}

fn write_artifacts(artifacts: &[Value]) -> AppResult<()> {
    let output = PathBuf::from("target/hello-audio-analysis");
    fs::create_dir_all(&output)?;
    for artifact in artifacts {
        let fixture = artifact["fixture"]
            .as_str()
            .ok_or("fixture label is missing")?;
        fs::write(
            output.join(format!("{fixture}.inspection.json")),
            format!("{}\n", serde_json::to_string_pretty(artifact)?),
        )?;
    }
    fs::write(
        output.join("index.json"),
        format!("{}\n", serde_json::to_string_pretty(artifacts)?),
    )?;
    println!(
        "wrote {} artifacts under {}",
        artifacts.len(),
        output.display()
    );
    Ok(())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_generated_wave_fixture_produces_headless_structural_evidence() {
        let artifacts = inspect_all_fixtures().unwrap();
        assert_eq!(artifacts.len(), PcmFixture::ALL.len());
        for artifact in artifacts {
            assert_eq!(artifact["schema"], ARTIFACT_SCHEMA);
            assert_eq!(
                artifact["source"]["kind"],
                "generated-riff-wave-pcm16-little-endian"
            );
            assert_eq!(artifact["renderer_required"], false);
            assert_eq!(artifact["audio_device_required"], false);
            assert_eq!(artifact["playback_performed"], false);
            assert!(artifact["analysis"]["spectrum"].as_array().is_some());
        }
    }
}
