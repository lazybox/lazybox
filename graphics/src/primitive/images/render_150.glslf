#version 150

in vec2 v_TexCoord;
in vec4 v_Color;

out vec4 o_Color;

uniform sampler2D s_Image;

void main() {
    o_Color = v_Color * texture(s_Image, v_TexCoord);
}