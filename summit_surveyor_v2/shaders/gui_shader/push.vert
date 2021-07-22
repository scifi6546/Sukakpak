#version 450
layout(location=0) in vec3 pos;
layout(location=1) in vec2 uv;
layout(location=0) out vec2 o_uv;
layout(push_constant) uniform constants{
mat4 trans;

}ub;
void main(){
	gl_Position = ub.trans*vec4(pos,1.0);
    o_uv = uv;
}
