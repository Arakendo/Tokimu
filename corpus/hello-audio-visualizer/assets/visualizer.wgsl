struct VisualizerSignal {
    // Transitional carrier: phase, low, mid, high. The corpus intentionally
    // uses the renderer's existing vec4 material slot until arbitrary runtime
    // material parameters earn an execution path.
    value: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> visualizer_signal: VisualizerSignal;

struct InstanceParams {
    translation: vec2<f32>,
    scale: vec2<f32>,
    rotation: vec2<f32>,
    padding: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> instance_params: InstanceParams;

@group(2) @binding(0)
var<uniform> camera_params: mat4x4<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> VertexOutput {
    var output: VertexOutput;
    let instance_position = position.xy * instance_params.scale;
    output.position = camera_params * vec4<f32>(instance_position, position.z, 1.0);
    output.uv = position.xy;
    return output;
}

fn palette(value: f32, low: f32, mid: f32, high: f32) -> vec3<f32> {
    let base = vec3<f32>(0.03, 0.06, 0.11);
    let cyan = vec3<f32>(0.18, 0.95, 0.86) * (0.35 + low * 0.9);
    let amber = vec3<f32>(1.0, 0.48, 0.12) * (0.15 + mid * 0.85);
    let ice = vec3<f32>(0.50, 0.68, 1.0) * (0.20 + high * 0.8);
    return base + cyan * smoothstep(0.0, 0.55, value)
        + amber * smoothstep(0.42, 0.82, value)
        + ice * smoothstep(0.75, 1.0, value);
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let phase = visualizer_signal.value.x * 6.28318530718;
    let low = visualizer_signal.value.y;
    let mid = visualizer_signal.value.z;
    let high = visualizer_signal.value.w;

    let aspect_uv = vec2<f32>(uv.x * 1.55, uv.y);
    let radius = length(aspect_uv);
    let angle = atan2(aspect_uv.y, aspect_uv.x);
    let rings = 0.5 + 0.5 * sin(radius * (18.0 + low * 26.0) - phase * 2.0);
    let spokes = 0.5 + 0.5 * cos(angle * (4.0 + floor(mid * 8.0)) + phase);
    let sweep = 0.5 + 0.5 * sin((aspect_uv.x + aspect_uv.y) * 10.0 - phase * 3.0);
    let energy = clamp(rings * (0.45 + low) + spokes * mid * 0.7 + sweep * high * 0.55, 0.0, 1.0);
    let vignette = 1.0 - smoothstep(0.15, 1.35, radius);
    return vec4<f32>(palette(energy, low, mid, high) * (0.25 + vignette * 0.85), 1.0);
}
