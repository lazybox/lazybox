#version 150

in vec2 v_TexCoord;

out vec4 o_Color;

uniform sampler2D s_Color;

void main() {
    o_Color = texture(s_Color, v_TexCoord);
}