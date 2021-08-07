use bevy::prelude::*;

use image::{self, ImageBuffer, Luma};

struct TerrainMeshOptions {
    pub width: u32,
    pub length: u32,
    pub height: u32,
}

fn get_normal(v1: &[f32; 3], v2: &[f32; 3], v3: &[f32; 3]) -> [f32; 3] {
    let a = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
    let b = [v3[0] - v1[0], v3[1] - v1[1], v3[2] - v1[2]];
    cross(&a, &b)
}

fn cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    let nx = a[1] * b[2] - a[2] * b[1];
    let ny = a[2] * b[0] - a[0] * b[2];
    let nz = a[0] * b[1] - a[1] * b[0];
    [nx, ny, nz]
}

fn sample_heightmap(x: u32, y: u32, heightmap: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (f32, [f32; 3]) {
    let height = heightmap.get_pixel(x, y).0[0] as f32;

    let target = [0., height, 0.];
    let right = [1., heightmap.get_pixel(x + 1, y).0[0] as f32, 0.];
    let left = [-1., heightmap.get_pixel(x - 1, y).0[0] as f32, 0.];
    let top = [0., heightmap.get_pixel(x, y + 1).0[0] as f32, 1.];
    let bottom = [0., heightmap.get_pixel(x, y - 1).0[0] as f32, -1.];

    let normal_1 = get_normal(&target, &top, &right);
    let normal_2 = get_normal(&target, &bottom, &left);
    let new_normal = [
        (normal_1[0] + normal_2[0]) / 2.,
        (normal_1[1] + normal_2[1]) / 2.,
        (normal_1[2] + normal_2[2]) / 2.,
    ];

    (height / 255., new_normal)
}

fn mesh_from_heightmap(
    filename: &str,
    mesh_options: TerrainMeshOptions,
) -> Vec<([f32; 3], [f32; 3], [f32; 2])> {
    if let Ok(terrain_bitmap) = image::open(filename) {
        let grayscaled = terrain_bitmap.grayscale();

        let heightmap = grayscaled.as_luma8().unwrap();

        let offset = 1;
        let step_w = (heightmap.width() - offset * 2) / mesh_options.width;
        let step_h = (heightmap.height() - offset * 2) / mesh_options.length;

        let mut vertices_vec = Vec::new();
        for h in 0..mesh_options.length {
            for w in 0..mesh_options.width {
                let (x, y) = (w * step_w + offset, h * step_h + offset);
                let (height, normal) = sample_heightmap(x, y, heightmap);
                let vertex = [w as f32, mesh_options.height as f32 * height, h as f32];
                let uv = [
                    x as f32 / heightmap.width() as f32,
                    y as f32 / heightmap.height() as f32,
                ];
                vertices_vec.push((vertex, normal, uv));
            }
        }
        vertices_vec
    } else {
        println!("Failed to load {}", filename);
        Vec::new()
    }
}

const WIDTH: u32 = 1000;
const LENGTH: u32 = 1000;

pub fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let vertices_vec = mesh_from_heightmap(
        "assets/heightmap.png",
        TerrainMeshOptions {
            width: LENGTH,
            length: WIDTH,
            height: 150,
        },
    );

    let mut indices_vec = Vec::new();
    for y in 0..(LENGTH - 1) {
        for x in 0..(WIDTH - 1) {
            indices_vec.push(x + y * LENGTH);
            indices_vec.push(x + (y + 1) * LENGTH);
            indices_vec.push(x + 1 + y * LENGTH);

            indices_vec.push(x + 1 + y * LENGTH);
            indices_vec.push(x + (y + 1) * LENGTH);
            indices_vec.push(x + 1 + (y + 1) * LENGTH);
        }
    }

    let indices = bevy::render::mesh::Indices::U32(indices_vec);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for (position, normal, uv) in vertices_vec.iter() {
        positions.push(*position);
        normals.push(*normal);
        uvs.push(*uv);
    }

    let mut mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    let scale_factor = 2.;
    commands.spawn_bundle(PbrBundle {
        transform: Transform {
            translation: Vec3::new(
                -(WIDTH as f32 * scale_factor / 2.),
                0.,
                -(LENGTH as f32 * scale_factor / 2.),
            ),
            scale: Vec3::new(scale_factor, scale_factor, scale_factor),
            ..Default::default()
        },
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.93, 0.79, 0.69),
            metallic: 0.0,
            roughness: 1.0,
            ..Default::default()
        }),
        ..Default::default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 2000. })),
        transform: Transform::from_translation(Vec3::new(0., 10., 0.)),
        material: materials.add(StandardMaterial {
            base_color: Color::MIDNIGHT_BLUE,
            roughness: 0.7,
            metallic: 0.3,
            ..Default::default()
        }),
        ..Default::default()
    });
}
