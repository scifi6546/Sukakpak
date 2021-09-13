#version 450
#extension GL_ARB_separate_shader_objects : enable
layout(location=0) in vec2 uv;
layout(location=1) in vec3 normal;
layout(location = 0) out vec4 outColor;
layout(set=0,binding = 0) uniform sampler2D tex;
void main(){
	outColor =dot(normal,vec3(1.0,0.0,0.0))* texture(tex,uv);
}
