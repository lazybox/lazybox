#version 150 core

in vec2 v_TexCoord;

out vec4 Target0;

uniform sampler2D s_Color;

void main() {
    Target0 = texture(s_Color, v_TexCoord);
}