#version 410 core

in vec4 fragment_colors;
out vec4 color;

uint gridSize = 200;

void main()
{
    color = fragment_colors;
}
