#version 450
#extension GL_ARB_separate_shader_objects : enable
layout(location=0) in vec2 uv;
layout(location=1) in vec3 normal;
layout(location = 0) out vec4 outColor;
layout(set=0,binding = 0) uniform sampler2D tex;
void main(){
	vec3 SUN_DIR = normalize(vec3(1.0,1.0,0.0));
	outColor =dot(normal,SUN_DIR)* texture(tex,uv)+vec4(0.1,0.1,0.1,0.0);
	outColor = vec4(normal,1.0);
	outColor.x = min(outColor.x,1.0);
	outColor.y = min(outColor.y,1.0);
	outColor.z = min(outColor.z,1.0);
	outColor.a = 1.0;
}
