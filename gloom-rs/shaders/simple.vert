#version 410 core

in vec3 position;

// this matrix mirrors over the x-axis and the y-axis simultaneously
uniform mat4 flip = mat4(
    vec4(-1.0f,  0.0f,  0.0f,  0.0f),  // column 0
    vec4( 0.0f, -1.0f,  0.0f,  0.0f),  // column 1
    vec4( 0.0f,  0.0f,  1.0f,  0.0f),  // column 2
    vec4( 0.0f,  0.0f,  0.0f,  1.0f)   // column 3
);

void main()
{
    gl_Position = flip * vec4(position, 1.0f);
}
