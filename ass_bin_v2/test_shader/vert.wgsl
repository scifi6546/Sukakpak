struct VertexOutput{
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};
[[block]]
struct Locals{
    transform: mat4x4<f32>;
};
[[group(0),binding(0)]]
var<uniform> locals: Locals;
[[stage(vertex)]]
fn vs_main(
	[[location(0)]] position: vec4<f32>,
	[[location(1)]] tex_coord: vec2<f32>,
)->VertexOutput{
    var out: VertexOutput;
    out.tex_coord=tex_coord;
    out.position = locals.transform*position;
    return out;
}
