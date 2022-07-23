#version 450

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_color;

layout(location = 0) out vec4 o_frag_color;

void main() {

    o_frag_color = vec4(a_color, 1.0);
    gl_Position = vec4(a_position, 1.0);

}

