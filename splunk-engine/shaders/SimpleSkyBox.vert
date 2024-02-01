#version 460 core

layout (location=0) out vec3 out_dir;

layout (binding=0) uniform CameraUniform
{
    // mat4 model;
    mat4 view;
    mat4 proj;
} camera_ubo;

layout(binding = 1) uniform ModelSpace
{
    mat4 data;
} model;

void main ()
{
    float cube_size = 100.0;
    // int idx = indices[gl_VertexIndex];

	mat4 view = camera_ubo.view;
	view[3] = vec4(0.0, 0.0, 0.0, 1.0); // remove translation data. Prevent skybox from moving

    gl_Position = camera_ubo.proj * view * vec4(cube_size * pos[idx], 1.0);

    out_dir = pos[idx].xyz;
}