#version 450
layout(location = 0) in vec4 v_Position;
layout(location = 1) in vec4 v_WorldPosition;
layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform SkyMaterial_color_top {
    vec4 color_top;
};

layout(set = 2, binding = 1) uniform SkyMaterial_color_bottom {
    vec4 color_bottom;
};

layout(std140, set = 0, binding = 1) uniform CameraPosition {
    vec4 CameraPos;
};

void main() {
    float zenith_dot = dot(vec3(0,1,0), normalize(v_WorldPosition.xyz - CameraPos.xyz));
    vec3 a = color_top.rgb * zenith_dot;
    vec3 b = color_bottom.rgb * (1. - max(zenith_dot, 0.));

    o_Target = vec4(a + b, 1.);
}