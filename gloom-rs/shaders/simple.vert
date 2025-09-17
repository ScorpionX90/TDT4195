#version 410 core

layout(location=0) in vec3 position;
layout(location=1) in vec4 vertex_colors;
uniform mat4 transform;
out vec4 fragment_colors;


void main()
{
    // this matrix mirrors over the x-axis and the y-axis simultaneously
    gl_Position = transform * vec4(position, 1.0f);
    fragment_colors = vertex_colors;
}
