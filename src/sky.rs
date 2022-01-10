use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::{MaterialPipeline, SpecializedMaterial},
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{
            std140::{AsStd140, Std140},
            *,
        },
        renderer::RenderDevice,
    },
};

pub struct SkyBoxPlugin;

impl Plugin for SkyBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<SkyMaterial>::default())
            .add_startup_system(setup.system())
            .add_system(sky_follow_camera.system());
    }
}

#[derive(Component)]
pub struct SkyBoxCamera;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "4ee9c363-1124-4113-890e-199d81b00281"]
struct SkyMaterial {
    color_top: Color,
    color_bottom: Color,
}

#[derive(Clone, Default, AsStd140)]
struct SkyMaterialUniformData {
    pub color_top: Vec4,
    pub color_bottom: Vec4,
}

#[derive(Clone)]
pub struct GpuCustomMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    commands.spawn_bundle(MaterialMeshBundle {
        transform: Transform::from_translation(Vec3::new(0., 0.0, 0.0)),
        mesh: meshes.add(Mesh::from(shape::Cube { size: 2500. })),
        material: materials.add(SkyMaterial {
            color_top: Color::rgb(0.3, 0.56, 0.83),
            color_bottom: Color::rgb(0.7, 0.7, 1.0),
        }),
        ..Default::default()
    });
}

fn sky_follow_camera(
    mut query: QuerySet<(
        QueryState<&Transform, With<SkyBoxCamera>>,
        QueryState<&mut Transform, With<Handle<SkyMaterial>>>,
    )>,
) {
    let mut camera_transform = None;
    for camera in query.q0().iter() {
        camera_transform = Some(camera.clone());
    }
    if let Some(camera_transform) = camera_transform {
        for mut sky_transform in query.q1().iter_mut() {
            sky_transform.translation = camera_transform.translation;
        }
    }
}

impl RenderAsset for SkyMaterial {
    type ExtractedAsset = SkyMaterial;
    type PreparedAsset = GpuCustomMaterial;
    type Param = (SRes<RenderDevice>, SRes<MaterialPipeline<Self>>);
    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let data = SkyMaterialUniformData {
            color_top: extracted_asset.color_top.as_linear_rgba_f32().into(),
            color_bottom: extracted_asset.color_bottom.as_linear_rgba_f32().into(),
        };

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: data.as_std140().as_bytes(),
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
            layout: &material_pipeline.material_layout,
        });

        Ok(GpuCustomMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}

impl SpecializedMaterial for SkyMaterial {
    type Key = ();

    fn key(_: &<SkyMaterial as RenderAsset>::PreparedAsset) -> Self::Key {}

    fn specialize(_: Self::Key, descriptor: &mut RenderPipelineDescriptor) {
        descriptor.primitive.front_face = FrontFace::Cw;
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/sky.wgsl"))
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(
                        SkyMaterialUniformData::std140_size_static() as u64
                    ),
                },
                count: None,
            }],
            label: None,
        })
    }
}
