use bevy::{prelude::*, render::camera::*};

use super::input::*;
use super::particles::*;
use super::sky::*;

pub const ROLL_SPEED: f32 = 0.9;
pub const PITCH_SPEED: f32 = 1.8;
pub const YAW_SPEED: f32 = 0.25;
pub const MIN_SPEED: f32 = 20.;
pub const MAX_SPEED: f32 = 100.;
pub const ACCEL: f32 = 20.;
pub const BRAKE: f32 = 30.;

#[derive(Default)]
pub struct Player {
    pub velocity: f32,
    pub missiles_fired: u32,
    pub target: Option<Entity>,
}
pub struct MainCamera;
pub struct Missile {
    pub target: Option<Entity>,
    pub velocity: f32,
    pub lifetime: f32,
}
pub struct Target;

pub fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            perspective_projection: PerspectiveProjection {
                far: 2000.,
                fov: std::f32::consts::PI / 3.,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(MainCamera)
        .insert(SkyBoxCamera);

    let plane: Handle<Scene> = asset_server.load("f35.gltf#Scene0");

    commands
        .spawn_bundle((
            Transform::from_translation(Vec3::new(0.0, 400.0, 0.0)),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(plane);
        })
        .insert(Player {
            velocity: 10.,
            missiles_fired: 0,
            ..Default::default()
        });
}

pub fn camera_follow_player(
    mut query_set: QuerySet<(
        Query<&mut Transform, With<MainCamera>>,
        Query<(&Transform, &Player)>,
        Query<(&mut PerspectiveProjection, &mut Camera), With<MainCamera>>,
    )>,
    windows: Res<Windows>,
    time: Res<Time>,
) {
    let mut player_translation = Vec3::ZERO;
    let mut player_rotation = Quat::default();
    let mut player_speed = MIN_SPEED;

    if let Some((player_transform, player)) = query_set.q1().iter().next() {
        player_translation = player_transform.translation;
        player_rotation = player_transform.rotation;
        player_speed = player.velocity;
    }
    let speed_ratio = (player_speed - MIN_SPEED) / (MAX_SPEED - MIN_SPEED);

    if let Some(mut camera_transform) = query_set.q0_mut().iter_mut().next() {
        let new_transform = Transform::from_translation(
            player_translation + player_rotation * Vec3::new(-6.0, 2.0, 0.0),
        )
        .looking_at(
            player_translation + (player_rotation * Vec3::Y).normalize() * 1.5,
            (player_rotation * Vec3::Y).normalize(),
        );

        camera_transform.translation = camera_transform.translation.lerp(
            new_transform.translation,
            ((8. + (speed_ratio * 16.)) * time.delta_seconds()).clamp(0., 1.),
        );
        camera_transform.rotation = camera_transform.rotation.lerp(
            new_transform.rotation,
            (5. * time.delta_seconds()).clamp(0., 1.),
        );
    }

    if let Some(window) = windows.get_primary() {
        let (mut perspective_projection, mut camera) = query_set.q2_mut().single_mut().unwrap();

        perspective_projection.fov =
            std::f32::consts::PI / 3.0 + (speed_ratio * std::f32::consts::PI / 8.);

        perspective_projection.update(window.width(), window.height());
        camera.projection_matrix = perspective_projection.get_projection_matrix();
        camera.depth_calculation = perspective_projection.depth_calculation();
    }
}

pub fn player_input(
    mut player_input: ResMut<PlayerInput>,
    keyboard_input: Res<Input<KeyCode>>,
    lobby: Res<GamepadLobby>,
) {
    if lobby.gamepads.is_empty() {
        let mut axis = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::Left) {
            axis.x += -1.;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            axis.x += 1.;
        }
        if keyboard_input.pressed(KeyCode::Up) {
            axis.y += 1.;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            axis.y += -1.;
        }

        player_input.axis = axis;

        if keyboard_input.pressed(KeyCode::Space) {
            player_input.accel = 1.;
        } else {
            player_input.accel = 0.;
        }
        if keyboard_input.pressed(KeyCode::LShift) {
            player_input.brake = 1.;
        } else {
            player_input.brake = 0.;
        }
    }
}

pub fn player_movement(
    player_input: Res<PlayerInput>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    timer: Res<Time>,
) {
    if let Some((mut player_transform, mut player)) = player_query.iter_mut().next() {
        let pitch_delta = player_input.axis.x * timer.delta_seconds() * PITCH_SPEED;
        let roll_delta = -player_input.axis.y * timer.delta_seconds() * ROLL_SPEED;
        let yaw_delta = player_input.yaw * timer.delta_seconds() * YAW_SPEED;

        let ypr_rotation = Quat::from_rotation_ypr(yaw_delta, pitch_delta, roll_delta);

        player.velocity = (player.velocity
            + (player_input.accel * ACCEL - player_input.brake * BRAKE) * timer.delta_seconds())
        .clamp(MIN_SPEED, MAX_SPEED);

        player_transform.rotation = player_transform.rotation * ypr_rotation;

        player_transform.translation = player_transform.translation
            + (player_transform.rotation * Vec3::X * player.velocity * timer.delta_seconds());
    }
}

pub fn fire_missle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gamepad_event: EventReader<GamepadEvent>,
    mut player_query: Query<(&Transform, &mut Player)>,
) {
    if let Some((player_transform, mut player)) = player_query.iter_mut().next() {
        for event in gamepad_event.iter() {
            match &event {
                GamepadEvent(
                    _,
                    GamepadEventType::ButtonChanged(GamepadButtonType::East, value),
                ) => {
                    if *value > 0. {
                        commands
                            .spawn_bundle(PbrBundle {
                                mesh: meshes.add(Mesh::from(shape::Capsule {
                                    radius: 0.05,
                                    depth: 1.5,
                                    ..Default::default()
                                })),
                                material: materials.add(StandardMaterial {
                                    base_color: Color::GRAY,
                                    ..Default::default()
                                }),
                                transform: Transform {
                                    translation: player_transform.translation
                                        - (player_transform.rotation * Vec3::Y * 0.5)
                                        + player_transform.rotation
                                            * Vec3::Z
                                            * if player.missiles_fired % 2 == 0 {
                                                1.
                                            } else {
                                                -1.
                                            },
                                    rotation: player_transform.rotation
                                        * Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(Missile {
                                target: player.target,
                                velocity: player.velocity,
                                lifetime: 5.,
                            })
                            .insert(Emitter {
                                direction: Vec3::X,
                                spread: 0.,
                                speed: 0.5,
                                lifetime: 3.,
                                last_emitted: None,
                            });
                        player.missiles_fired = player.missiles_fired + 1;
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn missle_run(
    mut commands: Commands,
    mut missile_query: Query<(&mut Missile, Entity)>,
    mut transforms_query: QuerySet<(
        Query<&mut Transform, With<Missile>>,
        Query<&Transform, With<Target>>,
    )>,
    time: Res<Time>,
) {
    for (mut missile, missile_entity) in missile_query.iter_mut() {
        let target_translation = missile.target.map(|target| {
            let target_transform = transforms_query.q1().get(target).unwrap();
            target_transform.translation
        });

        let mut missile_transform = transforms_query.q0_mut().get_mut(missile_entity).unwrap();
        let current_dir = (missile_transform.rotation * Vec3::Y).normalize_or_zero();

        let target_dir = match target_translation {
            Some(dir) => (dir - missile_transform.translation).normalize_or_zero(),
            None => current_dir,
        };

        let velocity = if current_dir.angle_between(target_dir).abs() < std::f32::consts::FRAC_PI_2
        {
            current_dir
                .lerp(target_dir, time.delta_seconds() * 1.5)
                .normalize_or_zero()
                * missile.velocity
        } else {
            current_dir * missile.velocity
        };

        missile.velocity = (missile.velocity + time.delta_seconds() * 50.).clamp(0., 400.);

        missile_transform.translation =
            missile_transform.translation + velocity * time.delta_seconds();
        missile_transform.rotation = Quat::from_rotation_arc(Vec3::Y, velocity.normalize_or_zero());

        let distance_to_target = target_translation
            .map(|t| (t - missile_transform.translation).length())
            .unwrap_or(f32::INFINITY);
        missile.lifetime -= time.delta_seconds();
        if distance_to_target < 2. || missile.lifetime < 0. {
            commands.entity(missile_entity).despawn_recursive();
        }
    }
}
