//! Repeated-sample observation for two retained non-renderer caller paths.
//!
//! The CAD path retains `hello-cad`'s cursor-to-world ray construction.  The
//! GLB path decodes the pinned Khronos Box once, then repeatedly executes the
//! retained `hello-glb` model and floor transform functions over its actual
//! positions/normals.  Asset decoding and renderer submission deliberately
//! remain outside the timing region.

use std::{path::PathBuf, time::Instant};

use gltf_corpus::decode_glb_file;
use tokimu_math_study::{
    migration_hello_cad::{camera_ray_with_a, camera_ray_with_b, camera_ray_with_c},
    migration_hello_glb::{
        floor_with_a, floor_with_b, floor_with_c, model_with_a, model_with_b, model_with_c,
    },
};

const DEFAULT_ITERATIONS: u32 = 100_000;
const DEFAULT_SAMPLES: usize = 15;

#[derive(Clone, Copy)]
enum Candidate {
    Baseline,
    ProviderBacked,
    Owned,
}

impl Candidate {
    fn cad(self, iterations: u32) -> f64 {
        let mut checksum = 0.0_f64;
        for frame in 0..iterations {
            let cursor = [
                407.5 + (frame % 97) as f32 * 0.25,
                231.25 + (frame % 61) as f32 * 0.125,
            ];
            let ray = match self {
                Self::Baseline => camera_ray_with_a([1280.0, 720.0], cursor),
                Self::ProviderBacked => camera_ray_with_b([1280.0, 720.0], cursor),
                Self::Owned => camera_ray_with_c([1280.0, 720.0], cursor),
            }
            .expect("bounded valid CAD ray");
            checksum += f64::from(core::hint::black_box(ray.origin[0] + ray.direction[2]));
        }
        checksum
    }

    fn glb(self, iterations: u32, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> f64 {
        let mut checksum = 0.0_f64;
        for frame in 0..iterations {
            let seconds = frame as f32 * 0.016;
            let (model, floor) = match self {
                Self::Baseline => (
                    model_with_a(seconds, positions, normals),
                    floor_with_a(seconds, positions, normals),
                ),
                Self::ProviderBacked => (
                    model_with_b(seconds, positions, normals),
                    floor_with_b(seconds, positions, normals),
                ),
                Self::Owned => (
                    model_with_c(seconds, positions, normals),
                    floor_with_c(seconds, positions, normals),
                ),
            };
            checksum += f64::from(core::hint::black_box(
                model.positions[0][0]
                    + model.normals[0][1]
                    + floor.positions[0][2]
                    + floor.normals[0][0],
            ));
        }
        checksum
    }
}

fn parse<T: std::str::FromStr>(value: Option<String>, default: T, label: &str) -> T {
    value
        .map(|value| {
            value
                .parse()
                .unwrap_or_else(|_| panic!("{label} must be an integer"))
        })
        .unwrap_or(default)
}

fn summary(samples: &mut [u128]) -> (u128, u128, u128) {
    samples.sort_unstable();
    (
        samples[0],
        samples[samples.len() / 2],
        samples[samples.len() - 1],
    )
}

fn measure<T>(run: impl FnOnce() -> T) -> (u128, T) {
    let started = Instant::now();
    let result = run();
    (started.elapsed().as_nanos(), result)
}

fn box_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(
        "third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb",
    )
}

fn report(label: &str, samples: &mut [u128]) {
    let (minimum, median, maximum) = summary(samples);
    println!("{label}_elapsed_ns=min:{minimum},median:{median},max:{maximum}");
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let iterations = parse(arguments.next(), DEFAULT_ITERATIONS, "iteration count");
    let samples = parse(arguments.next(), DEFAULT_SAMPLES, "sample count");
    assert!(samples > 0, "sample count must be greater than zero");
    assert!(
        arguments.next().is_none(),
        "usage: measure_caller_paths [iterations] [samples]"
    );

    let decoded = decode_glb_file(box_path()).expect("pinned Khronos Box decodes");
    let primitive = decoded.primitives.first().expect("Box has one primitive");
    assert!(!primitive.positions.is_empty(), "Box positions are present");
    assert_eq!(
        primitive.positions.len(),
        primitive.normals.len(),
        "Box normals align"
    );

    let candidates = [
        Candidate::Baseline,
        Candidate::ProviderBacked,
        Candidate::Owned,
    ];
    // The migration conformance tests compare individual outputs within their
    // retained tolerances. Repeatedly accumulating those `f32` values must not
    // become an exact cross-candidate equality test: harmless per-call
    // rounding differences are magnified by the loop. Each timing candidate
    // instead proves that it repeated its own visible work deterministically.
    let cad_expected = candidates.map(|candidate| candidate.cad(iterations));
    let glb_expected = candidates
        .map(|candidate| candidate.glb(iterations, &primitive.positions, &primitive.normals));

    let mut cad = [
        Vec::with_capacity(samples),
        Vec::with_capacity(samples),
        Vec::with_capacity(samples),
    ];
    let mut glb = [
        Vec::with_capacity(samples),
        Vec::with_capacity(samples),
        Vec::with_capacity(samples),
    ];
    for sample in 0..samples {
        for offset in 0..candidates.len() {
            let index = (sample + offset) % candidates.len();
            let candidate = candidates[index];
            let (elapsed, checksum) = measure(|| candidate.cad(iterations));
            assert_eq!(checksum, cad_expected[index]);
            cad[index].push(elapsed);
            let (elapsed, checksum) =
                measure(|| candidate.glb(iterations, &primitive.positions, &primitive.normals));
            assert_eq!(checksum, glb_expected[index]);
            glb[index].push(elapsed);
        }
    }

    println!("iterations={iterations}");
    println!("samples={samples}");
    println!("glb_primitive_vertices={}", primitive.positions.len());
    report("cad_baseline", &mut cad[0]);
    report("cad_provider_backed", &mut cad[1]);
    report("cad_owned", &mut cad[2]);
    report("glb_baseline", &mut glb[0]);
    report("glb_provider_backed", &mut glb[1]);
    report("glb_owned", &mut glb[2]);
    println!("cad_checksums={cad_expected:?}");
    println!("glb_checksums={glb_expected:?}");
}
