use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use particle_tools::{
    lower_particle_instances_2d, ParticleEmitter2d, ParticleSpawn2d, ParticleSystem2d,
    ParticleSystemConfig, ParticleVec2, ParticleView2d,
};
use serde_json::{json, Value};

use crate::{burst_request, spray_request, stream_request, FIXED_STEP_SECONDS, PARTICLE_SEED};

const ARTIFACT_SCHEMA: u16 = 1;

pub(crate) fn write_structural_artifacts() -> Result<Vec<PathBuf>, String> {
    let root = artifact_root();
    fs::create_dir_all(&root)
        .map_err(|error| format!("create particle artifact directory: {error}"))?;

    let cases = [
        build_burst_case()?,
        build_rate_case("stream", stream_request(), 42.0, 180)?,
        build_rate_case("spray", spray_request(), 30.0, 120)?,
    ];
    let mut paths = Vec::with_capacity(cases.len());
    for (case_id, artifact, lowering_micros) in cases {
        let path = root.join(format!("{case_id}.json"));
        let bytes = serde_json::to_vec_pretty(&artifact)
            .map_err(|error| format!("serialize particle artifact `{case_id}`: {error}"))?;
        fs::write(&path, &bytes)
            .map_err(|error| format!("write particle artifact `{}`: {error}", path.display()))?;
        eprintln!(
            "hello-particles artifact {case_id}: bytes={}, lowering_us={lowering_micros}",
            bytes.len()
        );
        paths.push(path);
    }
    Ok(paths)
}

fn build_burst_case() -> Result<(&'static str, Value, u128), String> {
    let mut system = new_system()?;
    let request = burst_request();
    let spawn = system.spawn(request).map_err(|error| error.to_string())?;
    for _ in 0..24 {
        system
            .step(FIXED_STEP_SECONDS)
            .map_err(|error| error.to_string())?;
    }
    build_artifact("burst", "burst", request, None, 24, spawn, &system)
}

fn build_rate_case(
    case_id: &'static str,
    request: ParticleSpawn2d,
    particles_per_second: f32,
    steps: usize,
) -> Result<(&'static str, Value, u128), String> {
    let mut system = new_system()?;
    let mut emitter =
        ParticleEmitter2d::new(request, particles_per_second).map_err(|error| error.to_string())?;
    let mut requested = 0;
    let mut spawned = 0;
    let mut dropped = 0;
    for _ in 0..steps {
        let report = emitter
            .emit(&mut system, FIXED_STEP_SECONDS)
            .map_err(|error| error.to_string())?;
        requested += report.requested;
        spawned += report.spawned;
        dropped += report.dropped;
        system
            .step(FIXED_STEP_SECONDS)
            .map_err(|error| error.to_string())?;
    }
    let spawn = particle_tools::ParticleSpawnReport {
        requested,
        spawned,
        dropped,
        active: system.active_count(),
    };
    build_artifact(
        case_id,
        "fixed-rate",
        request,
        Some(particles_per_second),
        steps,
        spawn,
        &system,
    )
}

fn build_artifact(
    case_id: &'static str,
    emission: &'static str,
    request: ParticleSpawn2d,
    particles_per_second: Option<f32>,
    steps: usize,
    spawn: particle_tools::ParticleSpawnReport,
    system: &ParticleSystem2d,
) -> Result<(&'static str, Value, u128), String> {
    let view = ParticleView2d::new(
        ParticleVec2::new(-1.05, -1.05),
        ParticleVec2::new(1.05, 1.05),
    )
    .map_err(|error| error.to_string())?;
    let started = Instant::now();
    let instances = lower_particle_instances_2d(system.particles(), view, system.config().capacity);
    let lowering_micros = started.elapsed().as_micros();

    let artifact = json!({
        "schema": ARTIFACT_SCHEMA,
        "case": case_id,
        "seed": PARTICLE_SEED,
        "emission": emission,
        "particles_per_second": particles_per_second,
        "fixed_step_seconds": FIXED_STEP_SECONDS,
        "steps": steps,
        "request": request,
        "spawn_report": spawn,
        "system": system.snapshot(),
        "visible_instances": instances,
        "measurement_policy": {
            "lowering_duration": "reported to stderr; excluded from deterministic artifact"
        }
    });
    Ok((case_id, artifact, lowering_micros))
}

fn new_system() -> Result<ParticleSystem2d, String> {
    ParticleSystem2d::new(
        ParticleSystemConfig {
            capacity: 512,
            maximum_burst: 96,
            maximum_lifetime: 4.0,
            maximum_step_seconds: 1.0 / 60.0,
        },
        PARTICLE_SEED,
    )
    .map_err(|error| error.to_string())
}

fn artifact_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/particle-corpus/hello-particles")
}

#[cfg(test)]
mod tests {
    use super::{build_burst_case, build_rate_case};
    use crate::{spray_request, stream_request};

    #[test]
    fn visible_cases_produce_structural_instances() {
        let (_, burst, _) = build_burst_case().unwrap();
        let (_, stream, _) = build_rate_case("stream", stream_request(), 42.0, 180).unwrap();
        let (_, spray, _) = build_rate_case("spray", spray_request(), 30.0, 120).unwrap();

        for artifact in [burst, stream, spray] {
            assert!(
                artifact["visible_instances"]["report"]["visible"]
                    .as_u64()
                    .unwrap()
                    > 0
            );
            assert_eq!(
                artifact["visible_instances"]["report"]["omitted_by_limit"],
                0
            );
        }
    }

    #[test]
    fn repeated_cases_produce_equal_structural_artifacts() {
        let (_, first_burst, _) = build_burst_case().unwrap();
        let (_, second_burst, _) = build_burst_case().unwrap();
        let (_, first_stream, _) = build_rate_case("stream", stream_request(), 42.0, 180).unwrap();
        let (_, second_stream, _) = build_rate_case("stream", stream_request(), 42.0, 180).unwrap();

        assert_eq!(first_burst, second_burst);
        assert_eq!(first_stream, second_stream);
    }
}
