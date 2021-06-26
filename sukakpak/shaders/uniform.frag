#version 450
#extension GL_ARB_separate_shader_objects : enable
layout(location=0) in vec2 uv;
layout(location = 0) out vec4 outColor;
layout(set=0,binding = 0) uniform sampler2D tex;
layout(set=1,binding=1) uniform C{vec3 color;}c;
void main(){
	outColor = texture(tex,uv)*vec4(c.color,1.0);
}
