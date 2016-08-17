#version 150 core

in float a_Ratio;

out vec2 v_Center;
out vec2 v_Radius;
out float v_Angle;
out float v_Step;
out float v_OcclusionThreshold;

layout(std140)
uniform Camera {
    vec2 u_Translate;
    vec2 u_Scale;
};

struct Light {
    vec4 color_intensity;
    vec2 center;
    float radius;
    float source_radius;
    float occlusion_threshold;
    float shadow_map_pos;
    float shadow_map_size;
    float padding;
};

const uint LIGHT_BUFFER_SIZE = 128u;

layout(std140)
uniform Lights {
    Light u_Lights[LIGHT_BUFFER_SIZE];
};

const float PI = 3.14159265358979323846264338327950288;
const float GL_STEP = 0.001;

void main() {
    Light l = u_Lights[gl_InstanceID];
    vec2 center = (l.center + u_Translate) * u_Scale;
    vec2 radius = vec2(l.radius) * u_Scale;

    float coord = l.shadow_map_pos + l.shadow_map_size * a_Ratio;
    gl_Position = vec4(coord, 0.0, 0.0, 1.0);

    v_Center = (center + vec2(1.0)) / 2.0;
    v_Radius = radius / 2.0;
    v_Angle = ((a_Ratio * 2.0) - 1.0) * PI;
    v_Step = GL_STEP / max(radius.x, radius.y);
    v_OcclusionThreshold = l.occlusion_threshold;
}