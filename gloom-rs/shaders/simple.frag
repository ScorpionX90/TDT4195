#version 410 core

in vec4 fragment_colors;
out vec4 color;

uint grid_size = 200;

void main()
{
    float col = (1 - (int(gl_FragCoord.y) / grid_size % 2) - (int(gl_FragCoord.x)) / grid_size % 2) * 1.0f;
    color = vec4(col, 0.0f, col, col);
}
