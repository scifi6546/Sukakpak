struct VertexOutput{
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};
[[block]]
struct Locals{
    transform: mat4x4<f32>;
};
[[group(0),binding(3)]]
var<uniform> locals: Locals;
[[stage(vertex)]]
fn vs_main(
	[[location(0)]] position: vec3<f32>,
	[[location(1)]] tex_coord: vec2<f32>,
	[[location(2)]] normal: vec3<f32>,
)->VertexOutput{
    var out: VertexOutput;
    out.tex_coord=tex_coord;
    out.position = locals.transform*vec4<f32>(position,1.0);
    return out;
}
[[group(0),binding(0)]]
var mesh_texture: texture_2d<f32>;
[[group(0),binding(1)]]
var sampler: sampler;
[[stage(fragment)]]
fn fs_main(in: VertexOutput)->[[location(0)]]vec4<f32>{
    let texture= textureSample(mesh_texture,sampler,in.tex_coord);
    return vec4<f32>(texture.x+in.tex_coord.x,texture.y+in.tex_coord.y,1.0,1.0);
}
