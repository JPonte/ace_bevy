#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec2 Vertex_Uv;

// layout(set = 2, binding = 0) uniform texture2D ParticleTexture_texture;

layout(set = 2, binding = 0) uniform ParticleMaterial_previous_pos {
    mat4 previous_pos;
};

layout(location = 0) out vec4 v_Position;
layout(location = 1) out vec2 v_Uv;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    v_Uv = Vertex_Uv;
    if (v_Uv.x < 1.) {
        v_Position = ViewProj * previous_pos * vec4(Vertex_Position, 1.0);
    } else {
        v_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
    }
    gl_Position = v_Position;
}