#version 450

layout(location = 0) out vec4 o_frag_color;
layout(location = 0) in vec4 i_frag_color;
layout(location = 1) in vec2 i_tex_coord;
layout(binding = 1) uniform sampler2D u_texture;

void main() {
    
    vec4 color = texture(u_texture, i_tex_coord);
    o_frag_color = color * i_frag_color;

}

