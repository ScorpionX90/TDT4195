#version 410 core

in vec4 fragment_colors;
out vec4 color;

uint gridSize = 200;

void main()
{
    float col = (int(gl_FragCoord.y) / gridSize + int(gl_FragCoord.x) / gridSize) % 2;
    color = vec4(col, 0.0f, col, col);
}
