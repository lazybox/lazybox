#version 150

in vec2 v_TexCoord;

out vec4 o_Color;

uniform sampler2D s_Color;
uniform sampler2D s_Light;

void main() {
    vec4 color = texture(s_Color, v_TexCoord);
    vec4 light = texture(s_Light, v_TexCoord);
    vec3 hdr = (color.rgb * light.rgb) + (light.rgb * light.a);
    vec3 ldr = hdr / (hdr + 0.187) * 1.035;

    o_Color = vec4(ldr, color.a + light.a);
}