#version 410 core

layout(location=0) in vec3 position;
layout(location=1) in vec4 vertex_colors;
layout(location=2) in vec3 normal;
uniform mat4 mvp_transform;
uniform mat4 model_transform;
out vec4 fragment_colors;
out vec3 normal_vec;
out vec3 viewing_vec;

void main()
{
    gl_Position = mvp_transform * vec4(position, 1.0f);
    fragment_colors = vertex_colors;
    normal_vec = normalize(mat3(model_transform) * normal);
    viewing_vec = vec3(-gl_Position);
}
