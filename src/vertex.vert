#version 450

layout(location = 0) out vec4 o_frag_color;

void main() {

    const vec3 positions[3] = vec3[3](
        vec3( 0.0, -0.5, 1.0),
        vec3(-0.5,  0.5, 1.0),
        vec3( 0.5,  0.5, 1.0)
    );

    const vec3 colors[3] = vec3[3](
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 1.0, 0.0),
        vec3(1.0, 0.0, 0.0)
    );

    o_frag_color = vec4(colors[gl_VertexIndex], 1.0);
    gl_Position = vec4(positions[gl_VertexIndex], 1.0);

}

