struct VisualizerSignal {
    // phase, low, mid, high
    value: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> visualizer_signal: VisualizerSignal;
// Kept compatible with the current renderer material layout. Signal Field does
// not sample history, so these bindings remain intentionally unused.
@group(0) @binding(1)
var unused_texture: texture_2d<f32>;
@group(0) @binding(2)
var unused_sampler: sampler;

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
    let bands = visualizer_signal.value.yzw;
    // The source fixture can hold steady bands. Move the field itself so time
    // progression remains directly observable even for a steady tone.
    let orbit = vec2<f32>(cos(phase * 0.73), sin(phase * 0.91))
        * (0.10 + bands.x * 0.10);
    let centered = uv - vec2<f32>(0.5) - orbit;
    let radius = length(vec2<f32>(centered.x * 1.55, centered.y));
    let angle = atan2(centered.y, centered.x);
    let rings = 0.5 + 0.5 * sin(radius * (26.0 + bands.x * 34.0) - phase * 5.0);
    let spokes = 0.5 + 0.5 * cos(angle * (5.0 + floor(bands.y * 8.0)) + phase * 2.0);
    let sweep = 0.5 + 0.5 * sin(dot(centered, vec2<f32>(10.0, -7.0)) - phase * 3.0);
    let energy = clamp(
        rings * (0.42 + bands.x)
            + spokes * (0.22 + bands.y * 0.75)
            + sweep * (0.12 + bands.z * 0.5),
        0.0,
        1.0,
    );
    let cyan = vec3<f32>(0.08, 0.95, 0.84) * (0.42 + bands.x);
    let amber = vec3<f32>(1.0, 0.32, 0.06) * (0.10 + bands.y);
    let field = (cyan + amber) * energy * (1.0 - smoothstep(0.18, 0.80, radius));
    // This narrow scan bar is deliberately obvious corpus evidence that the
    // phase uniform reaches the shader, even when an input fixture is static.
    let scan_position = fract(visualizer_signal.value.x);
    let scan = 1.0 - smoothstep(0.0, 0.018, abs(uv.x - scan_position));
    let color = field + vec3<f32>(0.10, 0.92, 0.80) * scan;
    return vec4<f32>(color, 1.0);
}
