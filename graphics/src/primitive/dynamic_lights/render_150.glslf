#version 150 core

in vec2 v_RelativePosition;
in vec2 v_TexCoord;
in vec2 v_LightCoord;
in vec3 v_LightColor;
in float v_LightSourceRadius;
in float v_ShadowMapCoord;
in float v_ShadowMapSize;

out vec4 o_Light;

uniform sampler2D s_ShadowMap;
uniform sampler2D s_Normal;

const float PI = 3.14159265358979323846264338327950288;

float sample(float ratio) {
    float tex_coord = v_ShadowMapCoord + v_ShadowMapSize * ratio;
    return texture(s_ShadowMap, vec2(tex_coord, 0.0)).x;
}

void main() {
    float angle = atan(v_RelativePosition.y, v_RelativePosition.x);
    float ratio = angle / (2.0 * PI) + 0.5;
    float r = length(v_RelativePosition);

    float shadow = step(r, sample(ratio));
    float smooth_r = smoothstep(v_LightSourceRadius, 1.0, r);
    
/*
    float blur = 0.01 * smooth_r * v_LightSourceRadius;
    shadow *= 0.16;
    shadow += sample(ratio - 4.0 * blur) * 0.05;
    shadow += sample(ratio - 3.0 * blur) * 0.09;
    shadow += sample(ratio - 2.0 * blur) * 0.12;
    shadow += sample(ratio - 1.0 * blur) * 0.15;
    shadow += sample(ratio + 1.0 * blur) * 0.15;
    shadow += sample(ratio + 2.0 * blur) * 0.12;
    shadow += sample(ratio + 3.0 * blur) * 0.09;
    shadow += sample(ratio + 4.0 * blur) * 0.05;
*/

    vec3 normal = texture(s_Normal, v_TexCoord).rgb;
    normal = normalize(normal * 2.0 - vec3(1.0));
    vec3 to_light = vec3(v_LightCoord - v_TexCoord, 0.075);
    to_light = normalize(to_light);
    float reflection = max(dot(normal, to_light), 0.0);

    float attenuation = (1.0 - smooth_r);
    float intensity = shadow * reflection * attenuation;
    float source_intensity = (1.0 - smoothstep(0.0, v_LightSourceRadius, r));
    o_Light = vec4(v_LightColor * intensity, source_intensity * intensity);
}