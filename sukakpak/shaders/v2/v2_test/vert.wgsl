struct VertexOutput{
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};
[[block]]
struct Locals{
    transform: mat4x4<f32>;
};
[[group(0),binding(1)]]
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
var tex: texture_2d<f32>;
[[group(0),binding(1)]]
var sampler: sampler;
[[stage(fragment)]]
fn fs_main(in: VertexOutput)->[[location(0)]]vec4<f32>{
    //let texture = textureLoad(tex,vec2<u32>(in.tex_coord),0);
   let texture= textureSample(tex,sampler,in.tex_coord);
    let v = f32(texture.x)/255.0;
    return vec4<f32>(v,0.0,0.0,0.0);
}
