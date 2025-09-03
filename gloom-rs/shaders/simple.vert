#version 410 core

layout(location=0) in vec3 position;
layout(location=1) in vec4 vertex_colors;
out vec4 fragment_colors;

// this matrix mirrors over the x-axis and the y-axis simultaneously
const mat4 flip = mat4(
    vec4(-1.0f,  0.0f,  0.0f,  0.0f),  // column 0
    vec4( 0.0f, -1.0f,  0.0f,  0.0f),  // column 1
    vec4( 0.0f,  0.0f,  1.0f,  0.0f),  // column 2
    vec4( 0.0f,  0.0f,  0.0f,  1.0f)   // column 3
);

void main()
{
    gl_Position = flip * vec4(position, 1.0f);
    fragment_colors = vertex_colors;
}
