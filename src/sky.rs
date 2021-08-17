use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::shape,
        pipeline::{FrontFace, PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
};

pub struct SkyBoxPlugin;

impl Plugin for SkyBoxPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<SkyMaterial>()
            .add_startup_system(setup.system())
            .add_system(sky_follow_camera.system());
    }
}

pub struct SkyBoxCamera;

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b25127e"]
struct SkyMaterial {
    pub color_top: Color,
    pub color_bottom: Color,
}

fn setup(
    mut commands: Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    let mut pipeline_descriptor = PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(
            ShaderStage::Vertex,
            include_str!("sky.vert"),
        )),
        fragment: Some(shaders.add(Shader::from_glsl(
            ShaderStage::Fragment,
            include_str!("sky.frag"),
        ))),
    });

    pipeline_descriptor.primitive.front_face = FrontFace::Cw;

    let pipeline_handle = pipelines.add(pipeline_descriptor);

    render_graph.add_system_node(
        "sky_material",
        AssetRenderResourcesNode::<SkyMaterial>::new(true),
    );

    render_graph
        .add_node_edge("sky_material", base::node::MAIN_PASS)
        .unwrap();

    let material = materials.add(SkyMaterial {
        color_top: Color::rgb(0.3, 0.56, 0.83),
        color_bottom: Color::rgb(0.1, 0.1, 0.2),
    });

    commands
        .spawn_bundle(MeshBundle {
            transform: Transform::from_translation(Vec3::new(0., 0.0, 0.0)),
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1000. })),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            ..Default::default()
        })
        .insert(material);
}

fn sky_follow_camera(
    mut query: QuerySet<(
        Query<&Transform, With<SkyBoxCamera>>,
        Query<&mut Transform, With<Handle<SkyMaterial>>>,
    )>,
) {
    let mut camera_transform = None;
    for camera in query.q0().iter() {
        camera_transform = Some(camera.clone());
    }
    if let Some(camera_transform) = camera_transform {
        for mut sky_transform in query.q1_mut().iter_mut() {
            sky_transform.translation = camera_transform.translation;
        }
    }
}
