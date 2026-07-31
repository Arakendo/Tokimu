struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_fixture(@location(0) position: vec3<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 1.0);
    return output;
}

@fragment
fn fs_fixture() -> @location(0) vec4<f32> {
    // Intentionally unresolved. This fixture must reach backend compilation so
    // the corpus can verify diagnostic provenance without ever submitting it.
    return unresolved_fixture_symbol;
}
