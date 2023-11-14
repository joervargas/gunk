#version 460

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inTexCoords;

layout(binding = 0) uniform CameraUniform {
    mat4 model;
    mat4 view;
    mat4 proj;
} camera_ubo;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 outTexCoords;

void main()
{
    gl_Position = camera_ubo.proj * camera_ubo.view * camera_ubo.model * vec4(inPos, 1.0);

    fragColor = inColor;
    outTexCoords = inTexCoords;
}