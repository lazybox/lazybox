#version 150 core

in vec2 a_Position; // FIXME: unused
in vec2 a_TranslateInf;
in vec2 a_TranslateSup;
in vec2 a_TexCoordInf;
in vec2 a_TexCoordSup;
in uint a_Color;

out vec2 v_TexCoord;
out vec4 v_Color;

layout(std140)
uniform Camera {
    vec2 u_Translate;
    vec2 u_Scale;
};

vec4 unpack_color(in uint);

void main() {
    vec2[4] positions = vec2[4](
    	vec2(a_TranslateSup.x, a_TranslateInf.y), // bottom right
        vec2(a_TranslateSup.x, a_TranslateSup.y), // top right
        vec2(a_TranslateInf.x, a_TranslateSup.y), // top left
        vec2(a_TranslateInf.x, a_TranslateInf.y) // bottom left
    );
    gl_Position = vec4((positions[gl_VertexID] + u_Translate) * u_Scale, 0.0, 1.0);

    vec2[4] tex_coords = vec2[4](
    	vec2(a_TexCoordSup.x, a_TexCoordSup.y), // bottom right
        vec2(a_TexCoordSup.x, a_TexCoordInf.y), // top right
        vec2(a_TexCoordInf.x, a_TexCoordInf.y), // top left
        vec2(a_TexCoordInf.x, a_TexCoordSup.y) // bottom left
    );
	v_TexCoord = tex_coords[gl_VertexID];
    v_Color = unpack_color(a_Color);
}

vec4 unpack_color(in uint color) {
    const uint u8mask = 0x000000FFu;

    return vec4(float( color >> 24),
                float((color >> 16) & u8mask),
                float((color >>  8) & u8mask),
                float( color        & u8mask)) / 255.0;
}
