#version 450
layout(binding = 0) uniform UniformBufferObject {
    mat4 proj;
} ubo;
layout(location=0) in vec3 pos;
void main(){
	gl_Position = ubo.proj*vec4(pos,1.0);

}
