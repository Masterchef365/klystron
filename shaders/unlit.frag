#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform RealtimeUBO {
    mat4 matrix;
    float time;
} realtime;

layout(location = 0) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(fragColor + vec3(cos(realtime.time)), 1.0);
}
