struct MilkDropClassicControls {
    // phase, combined audio energy, preset decay, preset zoom
    value: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> controls: MilkDropClassicControls;
@group(0) @binding(1)
var previous_frame: texture_2d<f32>;
@group(0) @binding(2)
var previous_frame_sampler: sampler;

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
    output.uv = vec2<f32>(position.x + 0.5, 0.5 - position.y);
    return output;
}

fn rotate(point: vec2<f32>, angle: f32) -> vec2<f32> {
    let sine = sin(angle);
    let cosine = cos(angle);
    return mat2x2<f32>(cosine, -sine, sine, cosine) * point;
}

fn injection(uv: vec2<f32>, phase: f32, energy: f32) -> vec3<f32> {
    let centered = uv - vec2<f32>(0.5);
    let angle = atan2(centered.y, centered.x);
    let radius = length(centered);
    let moving_ring = abs(radius - (0.18 + 0.08 * sin(phase * 2.3)));
    let ring = 1.0 - smoothstep(0.008, 0.035, moving_ring);
    let spokes = 0.5 + 0.5 * cos(angle * 7.0 - phase * 4.0);
    let orbit_center = vec2<f32>(cos(phase * 1.7), sin(phase * 1.3)) * 0.22;
    let orbit = 1.0 - smoothstep(0.025, 0.12, length(centered - orbit_center));
    let pulse = 0.18 + energy * 1.8;
    let cyan = vec3<f32>(0.04, 0.92, 0.82) * ring * (0.3 + spokes * 0.7);
    let amber = vec3<f32>(1.0, 0.28, 0.04) * orbit;
    return (cyan + amber) * pulse;
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let phase = controls.value.x * 6.28318530718;
    let energy = controls.value.y;
    let decay = controls.value.z;
    let zoom = controls.value.w;

    let centered = uv - vec2<f32>(0.5);
    let warped = rotate(centered / max(zoom, 0.001), 0.003 + energy * 0.012);
    let drift = vec2<f32>(cos(phase * 0.31), sin(phase * 0.27)) * (0.002 + energy * 0.004);
    let previous_uv = vec2<f32>(0.5) + warped + drift;
    let prior = textureSample(previous_frame, previous_frame_sampler, previous_uv).rgb;
    let color = prior * decay + injection(uv, phase, energy);
    return vec4<f32>(color, 1.0);
}
