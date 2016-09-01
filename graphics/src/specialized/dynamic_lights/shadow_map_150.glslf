#version 150 core

in vec2 f_Center;
in vec2 f_Radius;
in float f_Angle;
in float f_Step;
in float f_OcclusionThreshold;

out float o_ShadowMap;

uniform sampler2D s_Occlusion;

void main() {
    vec2 ray = vec2(cos(f_Angle), sin(f_Angle)) * f_Radius;

    float r = 0.0;
    for (; r < 1.0; r += f_Step) {
        vec2 coord = f_Center + ray * r;

        if (texture(s_Occlusion, coord).x > f_OcclusionThreshold) {
            break;
        }
    }

    o_ShadowMap = r;
}