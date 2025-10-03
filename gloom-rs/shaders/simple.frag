#version 410 core

in vec4 fragment_colors;
in vec3 normal_vec;
in vec3 viewing_vec;
out vec4 color;

void main()
{
    // Constants
    vec3 i_ambient = vec3(0.7);
    vec3 i_diffuse = vec3(0.6);
    vec3 i_spec = vec3(0.8);

    float k_ambient = 0.4;
    float k_diffuse = 0.7;
    float k_spec = 0.5;
    float alpha = 16;

    // Light vector
    vec3 L = normalize(vec3(-0.8, 0.5, -0.6));

    // Reflected light vector
    vec3 R = 2 * dot(normalize(normal_vec), L) * normalize(normal_vec) - L;

    // Component vectors
    vec3 ambient = k_ambient * i_ambient;
    vec3 diff = k_diffuse * max(dot(normalize(normal_vec), L), 0.0) * i_diffuse;
    vec3 spec = k_spec * pow(max(dot(R, normalize(viewing_vec)), 0.0), alpha) * i_spec;

    // Final color vector
    vec3 intensity = ambient + diff + spec;
    vec3 albedo = fragment_colors.rgb * intensity;
    color = vec4(albedo, 1);
}