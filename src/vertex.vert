#version 450

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_color;
layout(location = 2) in vec2 a_tex_coord;

layout(location = 0) out vec4 o_frag_color;
layout(location = 1) out vec2 o_tex_coord;

layout(push_constant) uniform Constants {
    mat4 mvp;
};

layout(binding = 0) uniform UniformData {
    vec4 color;
} u_color;

void main() {

    o_frag_color = vec4(a_color, 1.0) * u_color.color;
    o_tex_coord = a_tex_coord;
    gl_Position = mvp * vec4(a_position, 1.0);

}

