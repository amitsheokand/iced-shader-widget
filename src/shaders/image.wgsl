struct Vertex {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct Output {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var<uniform> color: vec4<f32>;
@group(0) @binding(2) var tex_sampler: sampler;
@group(0) @binding(3) var tex: texture_2d<f32>;

@vertex
fn vs_main(vertex: Vertex) -> Output {
    var out: Output;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(in: Output) -> @location(0) vec4<f32> {
    let tex_color = textureSample(tex, tex_sampler, in.uv);
    return tex_color * color;
} 