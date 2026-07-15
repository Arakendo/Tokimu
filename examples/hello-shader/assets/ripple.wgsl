struct MaterialColor {
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> material_color: MaterialColor;

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
    @location(0) local_position: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOutput {
    let scaled_position = position.xy * instance_params.scale;
    let rotated_position = vec2<f32>(
        (scaled_position.x * instance_params.rotation.y) - (scaled_position.y * instance_params.rotation.x),
        (scaled_position.x * instance_params.rotation.x) + (scaled_position.y * instance_params.rotation.y)
    );
    let instance_position = rotated_position + instance_params.translation;
    var output: VertexOutput;
    output.position = camera_params * vec4<f32>(instance_position.x, instance_position.y, position.z, 1.0);
    output.local_position = position;
    return output;
}

@fragment
fn fs_main(@location(0) local_position: vec3<f32>) -> @location(0) vec4<f32> {
    let radius = length(local_position.xy);
    let rings = 0.5 + 0.5 * sin(radius * 24.0 - local_position.x * 6.0);
    let wave = 0.5 + 0.5 * cos(local_position.x * 14.0 + local_position.y * 11.0);
    let tint = vec3<f32>(0.32 + rings * 0.68, 0.40 + wave * 0.50, 0.95 - rings * 0.25);
    return vec4<f32>(material_color.color.rgb * tint, material_color.color.a);
}
