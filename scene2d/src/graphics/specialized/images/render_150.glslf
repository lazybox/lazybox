#version 150 core

in vec2 v_TexCoord;
in vec4 v_Color;

out vec4 Target0;

uniform sampler2D s_Image;

void main() {
    Target0 = v_Color * texture(s_Image, v_TexCoord);
}