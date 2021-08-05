#version 450

layout(location = 0) in vec4 v_Position;
layout(location = 1) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 1) uniform ParticleMaterial_alpha {
    float alpha;
};

float hash(vec3 p)
{
    p  = 50.0*fract( p*0.3183099 + vec3(0.71,0.113,0.419));
    return -1.0+2.0*fract( p.x*p.y*p.z*(p.x+p.y+p.z) );
}

float random (in vec2 st) {
    return fract(sin(dot(st.xy,
                         vec2(12.9898,78.233)))
                 * 43758.5453123);
}

// 2D Noise based on Morgan McGuire @morgan3d
// https://www.shadertoy.com/view/4dS3Wd
float noise (in vec2 st) {
    vec2 i = floor(st);
    vec2 f = fract(st);

    // Four corners in 2D of a tile
    float a = random(i);
    float b = random(i + vec2(1.0, 0.0));
    float c = random(i + vec2(0.0, 1.0));
    float d = random(i + vec2(1.0, 1.0));

    // Smooth Interpolation

    // Cubic Hermine Curve.  Same as SmoothStep()
    vec2 u = f*f*(3.0-2.0*f);
    // u = smoothstep(0.,1.,f);

    // Mix 4 coorners percentages
    return mix(a, b, u.x) +
            (c - a)* u.y * (1.0 - u.x) +
            (d - b) * u.x * u.y;
}


void main() {
    // float b2 = hash(vec3(v_Uv.x * 0.01, v_Uv.y * 0.01, 0.));
    float b2 = noise(vec2(v_Position.x, v_Position.y) * 2.);
    vec2 center_dist = vec2(v_Uv.x - 0.5, v_Uv.y - 0.5);
    float b = clamp(0.5 - sqrt(center_dist.y * center_dist.y), 0., 1.);
    // float b3 = b * b2;

    o_Target = vec4(1, 1, 1, b * b2 * b * alpha);
}