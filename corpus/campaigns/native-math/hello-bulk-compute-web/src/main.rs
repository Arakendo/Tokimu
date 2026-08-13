//! Browser/WASM Slice 9 control for one bounded ordered-point workload.
//!
//! This is deliberately a corpus fixture, not a Tokimu compute API. The local
//! WGSL and buffers validate browser execution against the Slice 8 CPU result.

#[cfg(target_arch = "wasm32")]
use js_sys::{Date, Function, Promise};
#[cfg(target_arch = "wasm32")]
use std::borrow::Cow;
#[cfg(target_arch = "wasm32")]
use tokimu_math_study::bulk_reference::{
    candidate_count, classify_points, generated_points, unit_cube_planes, Classification,
};
#[cfg(target_arch = "wasm32")]
use tokimu_math_study::chart_junction::{
    trace_fingerprint, trace_with_a, trace_with_b, trace_with_c,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("hello-bulk-compute-web is a browser/WASM Slice 9 corpus fixture");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
fn encode_points(points: &[[f32; 4]]) -> Vec<u8> {
    points
        .iter()
        .flat_map(|point| point.iter().flat_map(|component| component.to_ne_bytes()))
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn decode_flags(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(std::mem::size_of::<u32>())
        .map(|chunk| u32::from_ne_bytes(chunk.try_into().expect("u32 chunks")))
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn validate_count(count: usize) -> Result<(), JsValue> {
    if !(1..=1_000_000).contains(&count) {
        return Err(JsValue::from_str("count must be between 1 and 1,000,000"));
    }
    Ok(())
}

/// Browser/WASM execution control for Slice 11's ordinary chart mechanics.
///
/// It intentionally creates no GPU provider or renderer resource. Its only
/// claim is that the fixed A/C chart control executed in this browser target.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run_chart_control() -> Result<String, JsValue> {
    console_error_panic_hook::set_once();
    let baseline = trace_fingerprint(trace_with_a());
    let provider_backed = trace_fingerprint(trace_with_b());
    let owned = trace_fingerprint(trace_with_c());
    if baseline != provider_backed || baseline != owned {
        return Err(JsValue::from_str(
            "AR-0026 chart control A/C fingerprint mismatch",
        ));
    }
    Ok(format!(
        "status=completed; workload=ar-0026-chart-control; alternatives=A,B,C0; fingerprint={baseline:08x}; host=DOM; provider=none"
    ))
}

#[cfg(target_arch = "wasm32")]
async fn map_readback(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Result<Vec<u8>, JsValue> {
    let promise = Promise::new(&mut |resolve: Function, reject: Function| {
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| match result {
                Ok(()) => {
                    let _ = resolve.call0(&JsValue::NULL);
                }
                Err(error) => {
                    let _ = reject.call1(&JsValue::NULL, &JsValue::from_str(&error.to_string()));
                }
            });
    });
    let _ = device.poll(wgpu::Maintain::Poll);
    JsFuture::from(promise).await?;
    let bytes = buffer.slice(..).get_mapped_range().to_vec();
    buffer.unmap();
    Ok(bytes)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn run_point_control(count: usize) -> Result<String, JsValue> {
    console_error_panic_hook::set_once();
    validate_count(count)?;

    let input = generated_points(count);
    let cpu = classify_points(&unit_cube_planes(), &input);
    let expected: Vec<u32> = cpu
        .iter()
        .map(|record| u32::from(record.result == Classification::Candidate))
        .collect();
    let values: Vec<_> = input
        .iter()
        .map(|point| [point.position[0], point.position[1], point.position[2], 0.0])
        .collect();
    let input_bytes = encode_points(&values);
    let output_size = (count * std::mem::size_of::<u32>()) as u64;
    let total_started = Date::now();

    let adapter_started = Date::now();
    let instance =
        wgpu::util::new_instance_with_webgpu_detection(wgpu::InstanceDescriptor::default()).await;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .ok_or_else(|| JsValue::from_str("browser WebGPU did not provide an adapter"))?;
    let adapter_ms = Date::now() - adapter_started;
    let info = adapter.get_info();
    let device_started = Date::now();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .map_err(|error| JsValue::from_str(&format!("browser device request failed: {error}")))?;
    let device_ms = Date::now() - device_started;

    let setup_started = Date::now();
    device.push_error_scope(wgpu::ErrorFilter::Validation);
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("slice-9-browser-point-layout"),
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
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("slice-9-browser-point-pipeline-layout"),
        bind_group_layouts: &[&layout],
        push_constant_ranges: &[],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("slice-9-browser-point-wgsl"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(WGSL)),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("slice-9-browser-point-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("classify"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    if let Some(error) = device.pop_error_scope().await {
        return Err(JsValue::from_str(&format!(
            "provider-validation-rejected: {error}"
        )));
    }

    let input_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("slice-9-browser-point-input"),
        size: input_bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("slice-9-browser-point-output"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("slice-9-browser-point-readback"),
        size: output_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("slice-9-browser-point-bind-group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buffer.as_entire_binding(),
            },
        ],
    });
    let setup_allocation_ms = Date::now() - setup_started;
    const WARM_SAMPLES: usize = 3;
    let mut uploads = Vec::with_capacity(WARM_SAMPLES);
    let mut dispatches = Vec::with_capacity(WARM_SAMPLES);
    let mut readbacks = Vec::with_capacity(WARM_SAMPLES);
    for sample in 0..WARM_SAMPLES {
        let upload_started = Date::now();
        queue.write_buffer(&input_buffer, 0, &input_bytes);
        uploads.push(Date::now() - upload_started);

        let dispatch_started = Date::now();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("slice-9-browser-point-encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("slice-9-browser-point-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((count as u32).div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&output_buffer, 0, &readback_buffer, 0, output_size);
        queue.submit(Some(encoder.finish()));
        dispatches.push(Date::now() - dispatch_started);

        let readback_started = Date::now();
        let actual = decode_flags(&map_readback(&device, &readback_buffer).await?);
        readbacks.push(Date::now() - readback_started);
        if actual != expected {
            return Err(JsValue::from_str(&format!(
                "browser GPU ordered flags disagreed with CPU reference on sample {sample}"
            )));
        }
    }
    uploads.sort_by(f64::total_cmp);
    dispatches.sort_by(f64::total_cmp);
    readbacks.sort_by(f64::total_cmp);
    let middle = WARM_SAMPLES / 2;
    Ok(format!(
        "status=completed; workload=ordered_point; count={count}; candidates={}; samples={WARM_SAMPLES}; backend={:?}; adapter={}; adapter_ms={adapter_ms:.3}; device_ms={device_ms:.3}; setup_allocation_ms={setup_allocation_ms:.3}; warm_upload_ms={:.3}; warm_dispatch_ms={:.3}; warm_readback_ms={:.3}; total_ms={:.3}; build=debug; host=DOM",
        candidate_count(&cpu),
        info.backend,
        info.name,
        uploads[middle],
        dispatches[middle],
        readbacks[middle],
        Date::now() - total_started,
    ))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run_cpu_fallback(count: usize) -> Result<String, JsValue> {
    validate_count(count)?;
    let input = generated_points(count);
    let cpu = classify_points(&unit_cube_planes(), &input);
    Ok(format!(
        "status=cpu-fallback; workload=ordered_point; count={count}; candidates={}; reason=caller-selected-provider-bypass; observations=1; host=DOM",
        candidate_count(&cpu),
    ))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run_invalid_input_control() -> Result<String, JsValue> {
    match validate_count(0) {
        Ok(()) => Err(JsValue::from_str(
            "zero count unexpectedly passed validation",
        )),
        Err(error) => Ok(format!(
            "status=input-rejected; input=count=0; diagnostic={}; host=DOM",
            error
                .as_string()
                .unwrap_or_else(|| "non-string validation error".into())
        )),
    }
}

/// Exercises an explicit, idle resource release. This does not simulate device
/// loss or cancellation of submitted work; those remain separate provider
/// lifecycle questions.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn run_disposal_control() -> Result<String, JsValue> {
    let instance =
        wgpu::util::new_instance_with_webgpu_detection(wgpu::InstanceDescriptor::default()).await;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .ok_or_else(|| JsValue::from_str("browser WebGPU did not provide an adapter"))?;
    let (device, _) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .map_err(|error| JsValue::from_str(&format!("browser device request failed: {error}")))?;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("slice-9-browser-disposal-control"),
        size: 16,
        usage: wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    buffer.destroy();
    Ok("status=disposed; phase=idle-buffer-destroy; observations=1; host=DOM".into())
}

/// Exercises a bounded browser-provider validation failure without allowing the
/// fixture to panic or invent a fallback. This is evidence about provider
/// failure containment only; it does not add a Tokimu shader API.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn run_invalid_shader_control() -> Result<String, JsValue> {
    console_error_panic_hook::set_once();
    let instance =
        wgpu::util::new_instance_with_webgpu_detection(wgpu::InstanceDescriptor::default()).await;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .ok_or_else(|| JsValue::from_str("browser WebGPU did not provide an adapter"))?;
    let (device, _) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .map_err(|error| JsValue::from_str(&format!("browser device request failed: {error}")))?;
    device.push_error_scope(wgpu::ErrorFilter::Validation);
    let _invalid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("slice-9-browser-intentionally-invalid-wgsl"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed("this is intentionally not WGSL")),
    });
    match device.pop_error_scope().await {
        Some(error) => Ok(format!(
            "status=provider-validation-rejected; phase=shader-creation; diagnostic={error}; host=DOM"
        )),
        None => Err(JsValue::from_str(
            "invalid shader did not produce a bounded provider validation diagnostic",
        )),
    }
}
