//! Corpus-local native WGPU control for ordered point/frustum classification.
//!
//! This binary has no engine contract: its WGSL, buffer layout, provider
//! lifecycle, and readback are intentionally local to Slice 9.

use std::{sync::mpsc, time::Instant};

use tokimu_math_study::bulk_reference::{
    candidate_count, classify_aabbs, classify_points, generated_aabbs, generated_points,
    unit_cube_planes, Classification,
};

const WGSL: &str = r#"
struct Points { values: array<vec4<f32>>, };
struct Results { values: array<u32>, };

@group(0) @binding(0) var<storage, read> points: Points;
@group(0) @binding(1) var<storage, read_write> results: Results;

fn inside(point: vec3<f32>) -> bool {
    return all(point >= vec3<f32>(-1.0)) && all(point <= vec3<f32>(1.0));
}

@compute @workgroup_size(64)
fn classify(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if (index >= arrayLength(&points.values)) { return; }
    results.values[index] = select(0u, 1u, inside(points.values[index].xyz));
}
"#;
const AABB_WGSL: &str = r#"
struct Values { values: array<vec4<f32>>, };
struct Results { values: array<u32>, };
@group(0) @binding(0) var<storage, read> bounds: Values;
@group(0) @binding(1) var<storage, read_write> results: Results;
@compute @workgroup_size(64)
fn classify(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if (index >= arrayLength(&results.values)) { return; }
    let minimum = bounds.values[index * 2u].xyz;
    let maximum = bounds.values[index * 2u + 1u].xyz;
    let candidate = all(maximum >= vec3<f32>(-1.0)) && all(minimum <= vec3<f32>(1.0));
    results.values[index] = select(0u, 1u, candidate);
}
"#;
const WARM_SAMPLES: usize = 3;

fn encode_points(points: &[[f32; 4]]) -> Vec<u8> {
    points
        .iter()
        .flat_map(|point| point.iter().flat_map(|component| component.to_ne_bytes()))
        .collect()
}

fn decode_flags(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(std::mem::size_of::<u32>())
        .map(|chunk| u32::from_ne_bytes(chunk.try_into().expect("u32 chunks")))
        .collect()
}

fn median(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn main() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let invalid_shader = arguments
        .iter()
        .any(|argument| argument == "--invalid-shader");
    let aabb = arguments.iter().any(|argument| argument == "--aabb");
    let cpu_fallback = arguments
        .iter()
        .any(|argument| argument == "--cpu-fallback");
    let numeric: Vec<_> = arguments
        .iter()
        .filter(|argument| {
            argument.as_str() != "--invalid-shader"
                && argument.as_str() != "--aabb"
                && argument.as_str() != "--cpu-fallback"
        })
        .collect();
    let count = numeric
        .first()
        .map(|value| value.parse::<usize>().map_err(|_| "count must be usize"))
        .transpose()?
        .unwrap_or(100_000);
    if numeric.len() > 1 || count == 0 || count > 1_000_000 {
        return Err(
            "usage: measure_bulk_wgpu_native [1..=1000000] [--aabb] [--invalid-shader] [--cpu-fallback]".into(),
        );
    }

    let (cpu_records, point_bytes, workload) = if aabb {
        let input = generated_aabbs(count);
        let records = classify_aabbs(&unit_cube_planes(), &input);
        let values: Vec<_> = input
            .iter()
            .flat_map(|item| {
                [
                    [
                        item.bounds.minimum[0],
                        item.bounds.minimum[1],
                        item.bounds.minimum[2],
                        0.0,
                    ],
                    [
                        item.bounds.maximum[0],
                        item.bounds.maximum[1],
                        item.bounds.maximum[2],
                        0.0,
                    ],
                ]
            })
            .collect();
        (records, encode_points(&values), "ordered_aabb")
    } else {
        let input = generated_points(count);
        let records = classify_points(&unit_cube_planes(), &input);
        let values: Vec<_> = input
            .iter()
            .map(|point| [point.position[0], point.position[1], point.position[2], 0.0])
            .collect();
        (records, encode_points(&values), "ordered_point")
    };
    let expected_flags: Vec<u32> = cpu_records
        .iter()
        .map(|record| u32::from(record.result == Classification::Candidate))
        .collect();

    // This explicitly models caller-owned provider selection. It is not a
    // renderer fallback policy: the CPU result is already computed above and
    // this corpus driver emits one bounded terminal observation instead of
    // attempting a WGPU lifecycle.
    if cpu_fallback {
        println!(
            "status=cpu-fallback; workload={workload}; count={count}; candidates={}; reason=caller-selected-provider-bypass; observations=1",
            candidate_count(&cpu_records),
        );
        return Ok(());
    }

    let total_started = Instant::now();
    let instance = wgpu::Instance::default();
    let adapter_started = Instant::now();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .ok_or_else(|| {
                "provider-unavailable: WGPU adapter request returned none".to_string()
            })?;
    let adapter_elapsed = adapter_started.elapsed();
    let info = adapter.get_info();
    let device_started = Instant::now();
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
            .map_err(|error| {
                format!("provider-unavailable: WGPU device request failed: {error}")
            })?;
    let device_elapsed = device_started.elapsed();

    let setup_started = Instant::now();
    let input = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("option-c-slice9-points"),
        size: u64::try_from(point_bytes.len()).map_err(|_| "input too large")?,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let output_size =
        u64::try_from(count * std::mem::size_of::<u32>()).map_err(|_| "output too large")?;
    let output = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("option-c-slice9-flags"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("option-c-slice9-readback"),
        size: output_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("option-c-slice9-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("option-c-slice9-bind-group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output.as_entire_binding(),
            },
        ],
    });
    // WGPU otherwise reports an unscoped validation fault through its default
    // uncaptured-error path, which can abort a corpus executable. This local
    // scope turns setup validation into the same bounded `Result` path used for
    // unavailable adapters/devices. It is not a shared diagnostic owner.
    device.push_error_scope(wgpu::ErrorFilter::Validation);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("option-c-slice9-point-classification"),
        source: wgpu::ShaderSource::Wgsl(
            if invalid_shader {
                "this is deliberately invalid WGSL"
            } else if aabb {
                AABB_WGSL
            } else {
                WGSL
            }
            .into(),
        ),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("option-c-slice9-pipeline"),
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("option-c-slice9-pipeline-layout"),
                bind_group_layouts: &[&layout],
                push_constant_ranges: &[],
            }),
        ),
        module: &shader,
        entry_point: Some("classify"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    if let Some(error) = pollster::block_on(device.pop_error_scope()) {
        return Err(format!("provider-validation-rejected: {error}"));
    }
    let setup_elapsed = setup_started.elapsed();

    let mut uploads = Vec::with_capacity(WARM_SAMPLES);
    let mut dispatches = Vec::with_capacity(WARM_SAMPLES);
    let mut readbacks = Vec::with_capacity(WARM_SAMPLES);
    for sample in 0..WARM_SAMPLES {
        let upload_started = Instant::now();
        queue.write_buffer(&input, 0, &point_bytes);
        uploads.push(upload_started.elapsed().as_nanos());
        let dispatch_started = Instant::now();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("option-c-slice9-encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("option-c-slice9-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                u32::try_from(count.div_ceil(64)).map_err(|_| "too many workgroups")?,
                1,
                1,
            );
        }
        encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, output_size);
        queue.submit(Some(encoder.finish()));
        dispatches.push(dispatch_started.elapsed().as_nanos());
        let readback_started = Instant::now();
        let slice = readback.slice(..);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        let _ = device.poll(wgpu::Maintain::Wait);
        receiver
            .recv()
            .map_err(|_| "readback channel closed")?
            .map_err(|error| format!("readback failed: {error:?}"))?;
        let flags = decode_flags(&slice.get_mapped_range());
        readback.unmap();
        readbacks.push(readback_started.elapsed().as_nanos());
        if flags != expected_flags {
            return Err(format!(
                "semantic-mismatch: CPU and WGPU point flags differ on sample {sample}"
            ));
        }
    }
    println!("status=completed; workload={workload}; count={count}; candidates={}; samples={WARM_SAMPLES}; backend={:?}; device={:?}; adapter={}; adapter_ns={}; device_ns={}; setup_ns={}; warm_upload_ns={}; warm_dispatch_ns={}; warm_readback_ns={}; total_ns={}", candidate_count(&cpu_records), info.backend, info.device_type, info.name, adapter_elapsed.as_nanos(), device_elapsed.as_nanos(), setup_elapsed.as_nanos(), median(&mut uploads), median(&mut dispatches), median(&mut readbacks), total_started.elapsed().as_nanos());
    Ok(())
}
