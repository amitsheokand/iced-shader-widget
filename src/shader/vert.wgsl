struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // Define vertices for two triangles in counter-clockwise order
    var pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),  // Triangle 1
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),   // Triangle 2
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );

    // Define corresponding texture coordinates
    var tex_coords = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),    // Triangle 1
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),    // Triangle 2
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 0.0)
    );

    return VertexOutput(
        vec4<f32>(pos[in_vertex_index], 0.0, 1.0),
        tex_coords[in_vertex_index]
    );
}
