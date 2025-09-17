#version 410 core

layout(location=0) in vec3 position;
layout(location=1) in vec4 vertex_colors;
uniform float elapsed;
out vec4 fragment_colors;


void main()
{
    // this matrix mirrors over the x-axis and the y-axis simultaneously
    mat4 ID_MAT = mat4(
        vec4( 1.0f,  0.0f,  0.0f,  0.0f),  // column 0
        vec4( 0.0f,  elapsed,  0.0f,  0.0f),  // column 1
        vec4( 0.0f,  0.0f,  1.0f,  0.0f),  // column 2
        vec4( 0.0f,  0.0f,  0.0f,  1.0f)   // column 3
    );
    gl_Position = ID_MAT * vec4(position, 1.0f);
    fragment_colors = vertex_colors;
}
