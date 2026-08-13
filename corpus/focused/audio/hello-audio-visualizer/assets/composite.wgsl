struct VisualizerSignal {
    // phase, low, mid, high
    value: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> visualizer_signal: VisualizerSignal;
@group(0) @binding(1)
var signal_texture: texture_2d<f32>;
@group(0) @binding(2)
var signal_sampler: sampler;

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

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let phase = visualizer_signal.value.x * 6.28318530718;
    let high = visualizer_signal.value.w;
    // A deliberately visible target-to-target sample offset makes the third
    // pass distinguishable from both direct signal rendering and feedback.
    let offset = vec2<f32>(sin(phase * 0.83), cos(phase * 1.17))
        * (0.014 + high * 0.032);
    let cyan = textureSample(signal_texture, signal_sampler, uv + offset).rgb;
    let amber = textureSample(signal_texture, signal_sampler, uv - offset).rgb;
    let vignette = 1.0 - smoothstep(0.32, 0.76, length(uv - vec2<f32>(0.5)));
    let mix_amount = 0.25 + visualizer_signal.value.z * 0.35 + sin(phase * 0.6) * 0.15;
    let color = mix(cyan, amber.bgr, clamp(mix_amount, 0.0, 1.0));
    let scan = 1.0 - smoothstep(0.0, 0.016, abs(uv.y - visualizer_signal.value.x));
    return vec4(
        color * (0.45 + vignette * 0.75) + vec3<f32>(0.96, 0.38, 0.10) * scan,
        1.0,
    );
}
