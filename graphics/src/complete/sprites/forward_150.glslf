#version 150

in vec2 v_TexCoord;

out vec4 o_Color;

uniform sampler2D s_SpriteColor;
uniform sampler2D s_Light;
uniform sampler2D s_ConrodColor;

void main() {
    vec4 color = texture(s_SpriteColor, v_TexCoord);
    vec4 light = texture(s_Light, v_TexCoord);
    vec3 hdr = (color.rgb * light.rgb) + (light.rgb * light.a);
    vec3 ldr = hdr / (hdr + 0.187) * 1.035;

    color = vec4(ldr, color.a + light.a);
    vec4 conrod = texture(s_ConrodColor, v_TexCoord);

    o_Color = vec4(
        vec3(conrod.rgb * conrod.a + color.rgb * (1. - conrod.a)),
        conrod.a + color.a
    ); 
}