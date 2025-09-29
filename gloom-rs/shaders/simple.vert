#version 410 core

layout(location=0) in vec3 position;
layout(location=1) in vec4 vertex_colors;
layout(location=2) in vec3 normal;
uniform mat4 transform;
out vec4 fragment_colors;
out vec3 normal_vec;


void main()
{
    gl_Position = transform * vec4(position, 1.0f);
    fragment_colors = vertex_colors;
    normal_vec = normal;
}
