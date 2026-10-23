// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct Uniforms {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> u: Uniforms;

struct Model {
    model: mat4x4<f32>,
};
@group(1) @binding(0) var<uniform> m: Model;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = u.view_proj * m.model * vec4<f32>(model.position, 0.0, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

// Fragment shader

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

struct Outline {
    color: vec4<f32>,
    thickness: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};
@group(3) @binding(0) var<uniform> outline: Outline;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Distance from each edge, in texels
    let px_per_uv = 1.0 / fwidth(in.tex_coords);
    let edge_dist = min(in.tex_coords, vec2<f32>(1.0) - in.tex_coords) * px_per_uv;
    let d = min(edge_dist.x, edge_dist.y);

    let t = select(0.0, 1.0, d <= outline.thickness);

    // Border color over the sprite
    return mix(base, vec4<f32>(outline.color.rgb, 1.0), t * outline.color.a);
}