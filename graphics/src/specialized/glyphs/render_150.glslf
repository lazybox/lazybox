#version 150

in vec2 v_TexCoord;
in vec4 v_Color;

out vec4 o_Color;

uniform sampler2D s_Glyph;

void main() {
    o_Color = v_Color * vec4(vec3(1.0), texture(s_Glyph, v_TexCoord).x);
}