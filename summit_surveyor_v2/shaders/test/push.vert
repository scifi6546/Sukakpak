#version 450
layout(location=0) in vec3 pos;
layout(location=1) in vec2 uv;
layout(push_constant) uniform constants{
mat4 proj;

}ub;
layout(location=0) out vec2 o_uv;
void main(){
	gl_Position = ub.proj*vec4(pos,1.0);
    o_uv = uv;
}
