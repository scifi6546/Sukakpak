#version 450
layout(push_constant) uniform constants{
    mat4 proj;
} ubo;
layout(location=0) in vec3 pos;
layout(location=1) in vec2 uv;
layout(location=0) out vec2 o_uv;
void main(){
	gl_Position = ubo.proj*vec4(pos,1.0);
    o_uv = uv;
}