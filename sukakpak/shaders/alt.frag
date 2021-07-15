#version 450
#extension GL_ARB_separate_shader_objects : enable
layout(location=0) in vec2 uv;
layout(location=1) in vec3 norm_in;
layout(location = 0) out vec4 outColor;
layout(set=0,binding = 0) uniform sampler2D tex;
/*
float dot(vec3 a,vec3 b){
	return a.x*b.x+a.y*b.y+a.z*b.z;
}
vec3 norm(vec3 inp){
	//todo
	return inp;

}
*/
void main(){
	
	outColor = texture(tex,uv)*dot(vec3(0.5,0.5,0.5),norm_in);
}
