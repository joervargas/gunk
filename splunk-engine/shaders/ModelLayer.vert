#version 360

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 uv;

layout(binding = 0) uniform UniformBuffer
{
    mat4 mvp;
} ubo;

struct VertexData
{
    float x, y, z;
    float u, v;
}

layout(binding = 1) readonly buffer Vertices
{
    VertexData data[];
} inVertices;

layout(binding 2) readonly buffer Indices
{
    uint data[];
} inIndices;

void main()
{
    uint idx = inIndices.data[gl_VertexIndex];
    VertexData vtx = inVertices.data[idx];

    vec3 pos = vec3(vtx.x, vtx.y, vtx.z);

    gl_Position = ubo.mvp * vec4(pos, 1.0);
    fragColor = pos;
    uv = vec2(vtx.u, vtx.z);
}