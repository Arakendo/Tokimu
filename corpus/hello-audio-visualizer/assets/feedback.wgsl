struct VisualizerSignal {
    // phase, low, mid, high
    value: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> visualizer_signal: VisualizerSignal;
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

fn signal(uv: vec2<f32>, phase: f32, low: f32, mid: f32, high: f32) -> vec3<f32> {
    let orbit = vec2<f32>(cos(phase * 0.73), sin(phase * 0.91)) * (0.10 + low * 0.10);
    let centered = uv - vec2<f32>(0.5, 0.5) - orbit;
    let radius = length(vec2<f32>(centered.x * 1.55, centered.y));
    let angle = atan2(centered.y, centered.x);
    let rings = 0.5 + 0.5 * sin(radius * (26.0 + low * 34.0) - phase * 5.0);
    let spokes = 0.5 + 0.5 * cos(angle * (5.0 + floor(mid * 8.0)) + phase * 2.0);
    let sweep = 0.5 + 0.5 * sin(dot(centered, vec2<f32>(10.0, -7.0)) - phase * 3.0);
    let energy = clamp(
        rings * (0.42 + low) + spokes * (0.22 + mid * 0.75) + sweep * (0.12 + high * 0.5),
        0.0,
        1.0,
    );
    let cyan = vec3<f32>(0.08, 0.95, 0.84) * (0.42 + low);
    let amber = vec3<f32>(1.0, 0.32, 0.06) * (0.10 + mid);
    return (cyan + amber) * energy * (1.0 - smoothstep(0.18, 0.80, radius));
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let phase = visualizer_signal.value.x * 6.28318530718;
    let centered = uv - vec2<f32>(0.5);
    let drift = vec2<f32>(cos(phase * 0.41), sin(phase * 0.37)) * 0.006;
    let prior_uv = vec2<f32>(0.5) + centered * 0.992 + drift;
    let prior = textureSample(previous_frame, previous_frame_sampler, prior_uv).rgb;
    let injection = signal(
        uv,
        phase,
        visualizer_signal.value.y,
        visualizer_signal.value.z,
        visualizer_signal.value.w,
    );
    let scan = 1.0 - smoothstep(0.0, 0.014, abs(uv.x - visualizer_signal.value.x));
    return vec4<f32>(prior * 0.94 + injection * 0.42 + vec3<f32>(0.08, 0.72, 0.64) * scan, 1.0);
}
