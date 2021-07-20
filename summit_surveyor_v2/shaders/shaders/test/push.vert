#version 450
layout(push_constant) uniform constants{
    mat4 transform;
} ubo;
layout(location=0) in vec3 pos;
layout(location=1) in vec2 uv;
layout(location=2) in vec3 normal;
layout(location=0) out vec2 o_uv;
void main(){
	gl_Position = ubo.transform*vec4(pos,1.0);
    o_uv = uv;
}
