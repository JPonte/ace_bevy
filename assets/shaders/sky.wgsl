#import bevy_pbr::mesh_view_bind_group

struct SkyMaterial {
    color_top: vec4<f32>;
    color_bottom: vec4<f32>;
};

[[group(1), binding(0)]]
var<uniform> material: SkyMaterial;

struct FragmentInput {
    [[location(0)]] world_position: vec4<f32>;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    var zenith_dot: f32 = dot(vec3<f32>(0.0, 1.0, 0.0), normalize(in.world_position.xyz - view.world_position.xyz));
    var a: vec3<f32> = material.color_top.rgb * zenith_dot;
    var b: vec3<f32> = material.color_bottom.rgb * (1. - max(zenith_dot, 0.));

    var output_color: vec4<f32> = vec4<f32>(a + b, 1.);

    return output_color;
}