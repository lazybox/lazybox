#version 150 core

in vec2 a_Position;
in uint a_Color;

layout(std140)
uniform Camera {
    vec2 u_Translate;
    vec2 u_Scale;
};

out vec4 v_Color;

vec4 unpack_color(in uint);

void main() {
    gl_Position = vec4((a_Position + u_Translate) * u_Scale, 0.0, 1.0);
    v_Color = unpack_color(a_Color);
}

vec4 unpack_color(in uint color) {
    const uint u8mask = 0x000000FFu;
    
    return vec4(float( a_Color >> 24),
                float((a_Color >> 16) & u8mask),
                float((a_Color >>  8) & u8mask),
                float( a_Color        & u8mask)) / 255.0;
}