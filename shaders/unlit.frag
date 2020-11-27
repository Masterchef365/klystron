#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_NON : enable

layout(location = 0) in vec3 fragColor;

layout(push_constant) uniform Model {
    mat4 matrix;
    uint texture_idx;
};

layout(binding = 1) uniform Animation {
    float anim;
};

layout(binding = 2) uniform sampler2D textures[];

layout(location = 0) out vec4 outColor;

void main() {
    outColor = texture(textures[texture_idx], fragColor.xy);
}
