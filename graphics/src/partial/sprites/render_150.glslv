#version 150 core

in vec2 a_Position;
in vec2 a_Translate;
in float a_Rotate;
in vec2 a_Scale;
in uint a_Color;
in vec2 a_TexCoordInf;
in vec2 a_TexCoordSup;

out vec4 v_Color;
out vec2 v_TexCoord;
out float v_Occlusion;

layout(std140)
uniform Camera {
    vec2 u_Translate;
    vec2 u_Scale;
};

mat2 rotation_mat(in float);
vec4 unpack_color(in uint);

void main() {
    mat2 rotate = rotation_mat(a_Rotate);
    vec2 world_position = (rotate * a_Position * a_Scale) + a_Translate;
    gl_Position = vec4((world_position + u_Translate) * u_Scale, 0.0, 1.0);
    
    vec2[4] tex_coords = vec2[4](
    	vec2(a_TexCoordSup.x, a_TexCoordSup.y), // bottom right
        vec2(a_TexCoordSup.x, a_TexCoordInf.y), // top right
        vec2(a_TexCoordInf.x, a_TexCoordInf.y), // top left
        vec2(a_TexCoordInf.x, a_TexCoordSup.y) // bottom left
    );
	v_TexCoord = tex_coords[gl_VertexID];
    v_Color = unpack_color(a_Color);
}

mat2 rotation_mat(in float rotate) {
    float c = cos(rotate);
    float s = sin(rotate);

    return mat2(c, s, -s, c);
}

vec4 unpack_color(in uint color) {
    const uint u8mask = 0x000000FFu;

    return vec4(float( color >> 24),
                float((color >> 16) & u8mask),
                float((color >>  8) & u8mask),
                float( color        & u8mask)) / 255.0;
}
