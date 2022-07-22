#version 450

layout(location = 0) out vec4 o_frag_color;
layout(location = 0) in vec4 i_frag_color;

void main() {
    
    o_frag_color = i_frag_color;

}

