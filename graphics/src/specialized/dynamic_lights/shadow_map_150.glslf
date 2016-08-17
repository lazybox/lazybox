#version 150 core

in vec2 v_Center;
in vec2 v_Radius;
in float v_Angle;
in float v_Step;
in float v_OcclusionThreshold;

out float o_ShadowMap;

uniform sampler2D s_Occlusion;

void main() {
    vec2 ray = vec2(cos(v_Angle), sin(v_Angle)) * v_Radius;

    float r = 0.0;
    for (; r < 1.0; r += v_Step) {
        vec2 coord = v_Center + ray * r;

        if (texture(s_Occlusion, coord).x > v_OcclusionThreshold) {
            break;
        }
    }

    o_ShadowMap = r;
}