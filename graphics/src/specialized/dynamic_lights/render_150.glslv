#version 150 core

in vec2 a_Position;

out vec2 v_RelativePosition;
out vec2 v_TexCoord;
out vec2 v_LightCoord;
out vec3 v_LightColor;
out float v_LightSourceRadius;
out float v_MapLevel;

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

void main() {
    Light l = u_Lights[gl_InstanceID];

    vec2 world_position = a_Position * l.radius + l.center;
    vec2 position = (world_position + u_Translate) * u_Scale;
    vec2 light_position = (l.center + u_Translate) * u_Scale;

    gl_Position = vec4(position, 0.0, 1.0);
    v_RelativePosition = a_Position;
    v_TexCoord = (position + 1.0) / 2.0;
    v_LightCoord = (light_position + 1.0) / 2.0;
    v_LightColor = l.color_intensity.rgb * l.color_intensity.a;
    v_LightSourceRadius = l.source_radius / l.radius;
    v_MapLevel = float(l.shadow_map_index);
}