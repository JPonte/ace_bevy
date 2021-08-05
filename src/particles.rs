use bevy::prelude::*;
use bevy::render::camera::*;
use bevy::{
    reflect::TypeUuid,
    render::{
        pipeline::*,
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
        texture::TextureFormat,
        mesh::Indices
    },
};


pub struct Emitter {
    pub direction: Vec3,
    pub spread: f32,
    pub speed: f32,
    pub lifetime: f32,
    pub last_emitted: Option<Entity>,
}

pub struct Particle {
    vel: Vec3,
    lifetime: f32,
    life: f32,
    drag: f32,
    previous_particle: Option<Entity>,
}

#[derive(Default)]
pub struct SmokeTextures {
    pub text1: Handle<Texture>,
    pub text2: Handle<Texture>,
    pub pipeline: Handle<PipelineDescriptor>,
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b25127e"]
pub struct ParticleMaterial {
    pub previous_pos: Mat4,
    pub alpha: f32
}

pub fn setup_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut smoke_textures_res: ResMut<SmokeTextures>,

    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    smoke_textures_res.text1 = asset_server.load("puff.png");
    smoke_textures_res.text2 = asset_server.load("puff2.png");

    smoke_textures_res.pipeline = pipelines.add(PipelineDescriptor {
        name: None,
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::None,
            polygon_mode: PolygonMode::Fill,
        },
        layout: None,
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState {
                front: StencilFaceState::IGNORE,
                back: StencilFaceState::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
            bias: DepthBiasState {
                constant: 0,
                slope_scale: 0.0,
                clamp: 0.0,
            },
            clamp_depth: false,
        }),
        color_target_states: vec![ColorTargetState {
            format: TextureFormat::default(),
            color_blend: BlendState {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: BlendState {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            write_mask: ColorWrite::ALL,
        }],
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        shader_stages: ShaderStages {
            vertex: shaders.add(Shader::from_glsl(
                ShaderStage::Vertex,
                include_str!("background.vert"),
            )),
            fragment: Some(shaders.add(Shader::from_glsl(
                ShaderStage::Fragment,
                include_str!("background.frag"),
            ))),
        },
    });

    render_graph.add_system_node(
        "particle_material",
        AssetRenderResourcesNode::<ParticleMaterial>::new(true),
    );
    render_graph
        .add_node_edge("particle_material", base::node::MAIN_PASS)
        .unwrap();

    // commands
    //     .spawn_bundle(PbrBundle {
    //         transform: Transform::from_translation(Vec3::new(0., 0., 5.)),
    //         mesh: meshes.add(Mesh::from(shape::Icosphere {
    //             radius: 0.1,
    //             ..Default::default()
    //         })),
    //         material: standard_materials.add(StandardMaterial {
    //             base_color: Color::WHITE,
    //             metallic: 0.0,
    //             roughness: 1.0,
    //             ..Default::default()
    //         }),
    //         ..Default::default()
    //     })
    //     .insert(Emitter {
    //         direction: Vec3::Y,
    //         spread: 0.001,
    //         speed: 0.5,
    //         lifetime: 2.,
    //         last_emitted: None,
    //     });
}

pub fn move_emitter(mut query: Query<&mut Transform, With<Emitter>>, time: Res<Time>) {
    for mut emitter in query.iter_mut() {
        let dir = emitter.rotation * Vec3::X;
        emitter.translation += dir.normalize_or_zero() * 20. * time.delta_seconds();
        emitter.rotation *=
            Quat::from_rotation_ypr(rand::random::<f32>() * 10. * time.delta_seconds(), 0., 0.)
    }
}

pub fn run_particles(
    mut commands: Commands,
    mut query_set: QuerySet<(
        Query<&Transform, With<Camera>>,
        Query<(
            &mut Transform,
            &mut Particle,
            Entity,
        )>,
        Query<(&Transform, &Particle, &Handle<ParticleMaterial>)>,
    )>,
    time: Res<Time>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
) {
    let mut camera_transform = None;
    for x in query_set.q0().iter() {
        camera_transform = Some(x.clone());
    }

    if let Some(camera_transform) = camera_transform {
        // let mut pre_mat = Mat4::default();

        for (mut particle_transform, mut particle, particle_entity) in
            query_set.q1_mut().iter_mut()
        {
            particle_transform.translation += particle.vel * time.delta_seconds();
            particle.life += time.delta_seconds();
            particle.vel = particle.vel
                + (particle.vel.normalize_or_zero() * particle.drag * time.delta_seconds());

            particle_transform.look_at(
                camera_transform.translation,
                camera_transform.rotation * Vec3::Y,
            );
            particle_transform.rotate(Quat::from_axis_angle(Vec3::X, -std::f32::consts::FRAC_PI_2));

            if particle.life > particle.lifetime {
                commands.entity(particle_entity).despawn_recursive();
            }
        }

        for (particle_transform, particle, material) in query_set.q2().iter() {
            if let Some(mat) = materials.get_mut(material.id) {
                if let Some(prev_particle) = particle.previous_particle {
                    if let Ok(prev_particle) = query_set.q2().get(prev_particle) {

                        mat.previous_pos = prev_particle.0.compute_matrix();
                    } else {
                        mat.previous_pos = particle_transform.compute_matrix();
                    }
                } else {
                    mat.previous_pos = particle_transform.compute_matrix();
                }
                mat.alpha = 1. - particle.life / particle.lifetime;
            }
        }
    }
}

fn trail_mesh() -> Mesh
    {
        let extent_z = 0.3;
        let extent_x = 0.;


        let vertices = [
            ([extent_x, 0.0, -extent_z], [0.0, 1.0, 0.0], [1.0, 1.0]),
            ([extent_x, 0.0, extent_z], [0.0, 1.0, 0.0], [1.0, 0.0]),
            ([-extent_x, 0.0, extent_z], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([-extent_x, 0.0, -extent_z], [0.0, 1.0, 0.0], [0.0, 1.0]),
        ];

        let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2]);

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        for (position, normal, uv) in vertices.iter() {
            positions.push(*position);
            normals.push(*normal);
            uvs.push(*uv);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }

pub fn run_emitter(
    mut emitter_query: Query<(&mut Emitter, &Transform)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    smoke_textures_res: Res<SmokeTextures>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
) {
    for (mut emitter, emitter_transform) in emitter_query.iter_mut() {
        let random_vec = rand_sphere_vector(emitter.spread);
        let vel = Quat::from_rotation_arc(Vec3::Y, emitter.direction) * random_vec * emitter.speed;
        let transform = Transform::from_translation(emitter_transform.translation.clone());
        let particle = commands
            .spawn_bundle(MeshBundle {
                transform: transform,
                mesh: meshes.add(trail_mesh()),
                render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                    smoke_textures_res.pipeline.clone(),
                )]),
                visible: Visible {
                    is_transparent: true,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Particle {
                vel: vel,
                lifetime: emitter.lifetime,
                life: 0.,
                drag: 0.0,
                previous_particle: emitter.last_emitted,
            })
            .insert(materials.add(ParticleMaterial {
                previous_pos: transform.compute_matrix(), // texture: smoke_textures_res.text1.clone(),
                alpha: 1.
            }))
            .id();

        emitter.last_emitted = Some(particle);
    }
}

fn rand_sphere_vector(spread: f32) -> Vec3 {
    let y = (1. - spread) + (spread * rand::random::<f32>());
    let t = rand::random::<f32>() * std::f32::consts::PI * 2.;

    let x = f32::sqrt(1.0 - y * y) * f32::cos(t);
    let z = f32::sqrt(1.0 - y * y) * f32::sin(t);

    Vec3::new(x, y, z)
}
