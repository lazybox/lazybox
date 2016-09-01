#version 150 core

in float a_Ratio;

out vs_out {
  vec2 center;
  vec2 radius;
  float angle;
  float step;
  float occlusion_threshold;
  uint map_index;
} vs;

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
    uint shadow_map_index;
    vec2 padding;
};

const uint LIGHT_BUFFER_SIZE = 256u;

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

    gl_Position = vec4(a_Ratio, 0.0, 0.0, 1.0);

    vs.center = (center + vec2(1.0)) / 2.0;
    vs.radius = radius / 2.0;
    vs.angle = ((a_Ratio * 2.0) - 1.0) * PI;
    vs.step = GL_STEP / max(radius.x, radius.y);
    vs.occlusion_threshold = l.occlusion_threshold;
    vs.map_index = l.shadow_map_index;
}
