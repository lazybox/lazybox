#version 150 core

in vec4 v_Color;
in vec2 v_TexCoord;

out vec4 o_Color;
out vec4 o_Normal;
out float o_Occlusion;

layout(std140)
uniform LayerLocals {
    float u_LayerOcclusion;
};

uniform sampler2D s_Color;
uniform sampler2D s_Normal;

const float OCCLUSION_THRESHOLD = 0.75;

void main() {
    vec4 color = texture(s_Color, v_TexCoord) * v_Color;
    o_Color = color;

    // should we multiply this by the color alpha ?
    o_Normal = texture(s_Normal, v_TexCoord);

    o_Occlusion = step(OCCLUSION_THRESHOLD, color.a) * u_LayerOcclusion;
}