#version 410 core

noperspective in vec4 fragment_colors;
out vec4 color;

void main()
{
    color = fragment_colors;
}