#version 460 core

layout (location=0) in vec3 in_dir;

layout (location=0) out vec4 out_color;

layout (binding=2) uniform samplerCube sky_texture;

void main()
{
    out_color = texture(sky_texture, in_dir);
}