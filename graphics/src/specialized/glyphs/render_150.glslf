#version 150 core

in vec2 v_TexCoord;
in vec4 v_Color;

out vec4 Target0;

uniform sampler2D s_Glyph;

void main() {
    Target0 = v_Color * vec4(vec3(1.0), texture(s_Glyph, v_TexCoord).x);
}