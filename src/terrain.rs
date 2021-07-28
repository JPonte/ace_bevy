use bevy::prelude::*;
use noise::{Fbm, NoiseFn};

const WIDTH: u32 = 1000;
const LENGTH: u32 = 1000;

fn get_vert(x: i32, y: i32, noise_fn: &Fbm) -> [f32; 3] {
    let height = noise_fn.get([x as f64 / 100., y as f64 / 100.]);
    [x as f32, (height as f32) * 50., y as f32]
}

fn get_normal(v1: &[f32; 3], v2: &[f32; 3], v3: &[f32; 3]) -> [f32; 3] {
    let a = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
    let b = [v3[0] - v1[0], v3[1] - v1[1], v3[2] - v1[2]];
    let nx = a[1] * b[2] - a[2] * b[1];
    let ny = a[2] * b[0] - a[0] * b[2];
    let nz = a[0] * b[1] - a[1] * b[0];
    [nx, ny, nz]
}

pub fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let noise_fn = Fbm::new();
    let mut vertices_vec = Vec::new();
    for y in 0..LENGTH {
        for x in 0..WIDTH {
            let vert = get_vert(x as i32, y as i32, &noise_fn);
            let top = get_vert(x as i32, y as i32 + 1, &noise_fn);
            let left = get_vert(x as i32 - 1, y as i32, &noise_fn);
            let right = get_vert(x as i32 + 1, y as i32, &noise_fn);
            let bottom = get_vert(x as i32, y as i32 - 1, &noise_fn);

            let normal_1 = get_normal(&vert, &top, &right);
            let normal_2 = get_normal(&vert, &bottom, &left);
            let new_normal = [
                (normal_1[0] + normal_2[0]) / 2.,
                (normal_1[1] + normal_2[1]) / 2.,
                (normal_1[2] + normal_2[2]) / 2.,
            ];

            let uv = [1.0, 1.0];
            vertices_vec.push((vert, new_normal, uv));
        }
    }

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

    commands.spawn_bundle(PbrBundle {
        transform: Transform {
            translation: Vec3::new(-(WIDTH as f32 * 3. / 2.), 0., -(LENGTH as f32 * 3. / 2.)),
            scale: Vec3::new(3.0, 3.0, 3.0),
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
}
