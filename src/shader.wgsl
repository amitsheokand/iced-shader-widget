struct Uniforms {
	resolution: vec2f,
	center: vec2f,
	scale: vec2f,
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexIn {
	@builtin(vertex_index) vertex_index: u32,
}

struct VertexOut {
	@builtin(position) position: vec4f,
	@location(0) uv: vec2f,
}

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
	let uv = vec2f(vec2u((in.vertex_index << 1) & 2, in.vertex_index & 2));
	let position = vec4f(uv * 2. - 1., 0., 1.);
	return VertexOut(position, uv);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
	let uv = in.uv;
	
	// Debug: Show UV coordinates as colors
	if (uv.x < 0.5 && uv.y < 0.5) {
		return vec4f(1.0, 0.0, 0.0, 1.0);  // Red
	} else if (uv.x >= 0.5 && uv.y < 0.5) {
		return vec4f(0.0, 1.0, 0.0, 1.0);  // Green
	} else if (uv.x < 0.5 && uv.y >= 0.5) {
		return vec4f(0.0, 0.0, 1.0, 1.0);  // Blue
	} else {
		return vec4f(1.0, 1.0, 0.0, 1.0);  // Yellow
	}
	
	// Original texture sampling code (commented out for now)
	/*
	let tex_coord = vec2f(
		uv.x * uniforms.scale.x + uniforms.center.x,
		uv.y * uniforms.scale.x + uniforms.center.y
	);
	
	if (tex_coord.x < 0.0 || tex_coord.x > 1.0 || tex_coord.y < 0.0 || tex_coord.y > 1.0) {
		return vec4f(0.0, 0.0, 1.0, 1.0);
	}
	
	let color = textureSample(t_diffuse, s_diffuse, tex_coord);
	return vec4f(color.rgb, 1.0);
	*/
}
