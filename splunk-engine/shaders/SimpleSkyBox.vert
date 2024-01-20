#version 460 core

layout (location=0) out vec3 out_dir;

layout (binding=0) uniform CameraUniform
{
    mat4 model;
    mat4 view;
    mat4 proj;
} camera_ubo;

const vec3 pos[8] = vec3[8](
	vec3(-1.0,-1.0, 1.0),
	vec3( 1.0,-1.0, 1.0),
	vec3( 1.0, 1.0, 1.0),
	vec3(-1.0, 1.0, 1.0),

	vec3(-1.0,-1.0,-1.0),
	vec3( 1.0,-1.0,-1.0),
	vec3( 1.0, 1.0,-1.0),
	vec3(-1.0, 1.0,-1.0)
);

const int indices[36] = int[36](
	// front
	0, 1, 2, 2, 3, 0,
	// right
	1, 5, 6, 6, 2, 1,
	// back
	7, 6, 5, 5, 4, 7,
	// left
	4, 0, 3, 3, 7, 4,
	// bottom
	4, 5, 1, 1, 0, 4,
	// top
	3, 2, 6, 6, 7, 3
);

void main ()
{
    float cube_size = 100.0;
    int idx = indices[gl_VertexIndex];

	mat4 view = mat4(mat3(camera_ubo.view));
    gl_Position = camera_ubo.proj * view * vec4(cube_size * pos[idx], 1.0);

    out_dir = pos[idx].xyz;
}