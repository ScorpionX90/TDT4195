#version 410 core

in vec4 fragment_colors;
in vec3 normal_vec;
out vec4 color;

void main()
{
    float lightVec = normal_vec.x * -0.8 + normal_vec.y * 0.5 + normal_vec.z * -0.6;
    vec3 albedo = vec3(fragment_colors) * max(0, lightVec);
    color = vec4(albedo, 1);
}