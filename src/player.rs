use bevy::{prelude::*, render::camera::*};
use bevy_rapier3d::na::Quaternion;
use bevy_rapier3d::na::Unit;
use bevy_rapier3d::na::Vector3;
use bevy_rapier3d::prelude::*;

use super::input::*;
// use super::particles::*;
use super::sky::*;
use super::Drone;

pub const ROLL_SPEED: f32 = 20.;
pub const PITCH_SPEED: f32 = 8.;
pub const YAW_SPEED: f32 = 2.5;
pub const MIN_SPEED: f32 = 0.;
pub const MAX_SPEED: f32 = 750.;
pub const ACCEL: f32 = 100.;
pub const BRAKE: f32 = 0.75;

#[derive(Default, Component)]
pub struct Player {
    pub missiles_fired: u32,
    pub target: Option<Entity>,
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Missile {
    pub target: Option<Entity>,
    pub velocity: f32,
    pub lifetime: f32,
}

#[derive(Component)]
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

    let mut start_transform = Transform::from_translation(Vec3::new(-700., 50., -210.));
    start_transform.look_at(Vec3::new(-600., 50., -700.), Vec3::Y);

    let target = commands
        .spawn_bundle((
            Transform::from_translation(Vec3::new(0.0, 325.0, 0.0)),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("f35.gltf#Scene0"));
        })
        .insert(Target)
        .insert(Drone)
        .id();

    let rigid_body = RigidBodyBundle {
        position: start_transform.translation.into(),
        forces: RigidBodyForcesComponent(RigidBodyForces {
            gravity_scale: 1.,
            ..Default::default()
        }),
        damping: RigidBodyDampingComponent(RigidBodyDamping {
            linear_damping: 0.1,
            angular_damping: 4.0,
        }),
        ccd: RigidBodyCcdComponent(RigidBodyCcd {
            ccd_enabled: true,
            ..Default::default()
        }),
        activation: RigidBodyActivationComponent(RigidBodyActivation::cannot_sleep()),
        mass_properties: RigidBodyMassPropsComponent(RigidBodyMassProps {
            ..Default::default()
        }),
        ..Default::default()
    };

    let collider = ColliderBundle {
        shape: ColliderShapeComponent(ColliderShape::ball(1.)),
        material: ColliderMaterialComponent(ColliderMaterial::default()),
        ..Default::default()
    };

    commands
        .spawn()
        .insert(Transform::default())
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("f35.gltf#Scene0"));
        })
        .insert(Player {
            target: Some(target),
            missiles_fired: 0,
            ..Default::default()
        })
        .insert_bundle(rigid_body)
        .insert_bundle(collider)
        .insert(RigidBodyPositionSync::Discrete)
        .insert(ColliderDebugRender::with_id(1));
}

pub fn camera_follow_player(
    mut query_set: QuerySet<(
        QueryState<&mut Transform, With<MainCamera>>,
        QueryState<(&Transform, &RigidBodyVelocityComponent), With<Player>>,
        QueryState<(&mut PerspectiveProjection, &mut Camera), With<MainCamera>>,
    )>,
    player_input: Res<PlayerInput>,
    windows: Res<Windows>,
    time: Res<Time>,
) {
    let mut player_translation = Vec3::ZERO;
    let mut player_rotation = Quat::default();
    let mut player_speed = MIN_SPEED;

    if let Some((player_transform, rb_vel)) = query_set.q1().iter().next() {
        player_translation = player_transform.translation;
        player_rotation = player_transform.rotation;
        player_speed = rb_vel.linvel.magnitude();
    }
    let speed_ratio = (player_speed - MIN_SPEED) / (MAX_SPEED - MIN_SPEED);

    if let Some(mut camera_transform) = query_set.q0().iter_mut().next() {
        let camera_x = -player_input.camera_axis.x * std::f32::consts::PI;
        let camera_y = player_input.camera_axis.y * std::f32::consts::FRAC_2_PI;

        let axis_rot = Quat::from_rotation_x(camera_x) * Quat::from_rotation_z(camera_y);

        let mut new_transform = Transform::from_translation(
            player_translation + player_rotation * axis_rot * Vec3::new(-15.0, 2.5, 0.0),
        );
        new_transform = new_transform.looking_at(
            player_translation + (player_rotation * Vec3::Y).normalize() * 1.5,
            (player_rotation * Vec3::Y).normalize(),
        );

        camera_transform.translation = camera_transform
            .translation
            .lerp(new_transform.translation, 0.4);
        camera_transform.rotation = camera_transform.rotation.lerp(new_transform.rotation, 0.4);

        // camera_transform.translation = new_transform.translation;
        // camera_transform.rotation = new_transform.rotation;

        // let new_transform =
        //     Transform::from_translation(player_translation + Vec3::new(-15.0, 6.0, 0.0))
        //         .looking_at(player_translation, Vec3::Y);
        // camera_transform.translation = new_transform.translation;
        // camera_transform.rotation = new_transform.rotation;
    }

    if let Some(window) = windows.get_primary() {
        for (mut perspective_projection, mut camera) in query_set.q2().iter_mut() {
            perspective_projection.fov =
                std::f32::consts::PI / 3.0 + (speed_ratio * std::f32::consts::PI / 8.);

            perspective_projection.update(window.width(), window.height());
            camera.projection_matrix = perspective_projection.get_projection_matrix();
            camera.depth_calculation = perspective_projection.depth_calculation();
        }
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

fn calculate_lift(
    vel: Vector3<f32>,
    rotation: Unit<Quaternion<f32>>,
    surface_normal: Vector3<f32>,
    constant: f32,
) -> Vector3<f32> {
    let angle_of_attack = if vel.magnitude_squared() == 0. {
        0.
    } else {
        -vel.normalize().dot(&(rotation * surface_normal))
    };

    let lift_magintude = vel.magnitude() * constant * angle_of_attack;

    rotation * (surface_normal * lift_magintude)
}

pub fn player_movement(
    player_input: Res<PlayerInput>,
    mut player_query: Query<
        (
            &mut RigidBodyForcesComponent,
            &RigidBodyVelocityComponent,
            &RigidBodyPositionComponent,
            &RigidBodyMassPropsComponent,
            &mut RigidBodyDampingComponent,
        ),
        With<Player>,
    >,
) {
    if let Some((mut rb_forces, rb_vel, rb_pos, rb_mprops, mut rb_damp)) =
        player_query.iter_mut().next()
    {
        let pitch_axis = -player_input.axis.y;
        let roll_axis = player_input.axis.x;
        let yaw_axis = player_input.yaw;

        let lift_up = calculate_lift(
            rb_vel.linvel,
            rb_pos.position.rotation,
            Vector3::from(Vec3::Y),
            10.,
        );
        let lift_side = calculate_lift(
            rb_vel.linvel,
            rb_pos.position.rotation,
            Vector3::from(Vec3::Z),
            15.,
        );

        let thrust_raw: Vector3<f32> =
            Vec3::new((player_input.accel * ACCEL) * rb_mprops.mass(), 0., 0.).into();
        let thrust: Vector3<f32> = rb_pos.position.rotation * thrust_raw;

        let ypr_vec: Vector3<f32> = Vec3::new(
            roll_axis * ROLL_SPEED,
            yaw_axis * YAW_SPEED,
            pitch_axis * PITCH_SPEED,
        )
        .into();
        rb_forces.torque = rb_pos.position.rotation * ypr_vec;
        rb_forces.force = thrust + lift_up + lift_side;

        rb_damp.linear_damping = player_input.brake * BRAKE + 0.1;
    }
}

pub fn fire_missle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gamepad_event: EventReader<GamepadEvent>,
    mut player_query: Query<(&Transform, &mut Player, &RigidBodyVelocityComponent)>,
) {
    if let Some((player_transform, mut player, rb_vel)) = player_query.iter_mut().next() {
        for event in gamepad_event.iter() {
            match &event {
                GamepadEvent(
                    _,
                    GamepadEventType::ButtonChanged(GamepadButtonType::East, value),
                ) => {
                    if *value > 0. {
                        commands
                            .spawn_bundle(PbrBundle {
                                mesh: meshes.add(Mesh::from(bevy::render::mesh::shape::Capsule {
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
                                velocity: rb_vel.linvel.magnitude(),
                                lifetime: 5.,
                            })
                            // .insert(Emitter {
                            //     direction: Vec3::X,
                            //     spread: 0.,
                            //     speed: 0.5,
                            //     lifetime: 3.,
                            //     last_emitted: None,
                            // })
                            ;
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
        QueryState<&mut Transform, With<Missile>>,
        QueryState<&Transform, With<Target>>,
    )>,
    time: Res<Time>,
) {
    for (mut missile, missile_entity) in missile_query.iter_mut() {
        let target_translation = missile.target.map(|target| {
            let target_transform = transforms_query.q1().get(target).unwrap();
            target_transform.translation
        });

        for mut missile_transform in transforms_query.q0().get_mut(missile_entity) {
            let current_dir = (missile_transform.rotation * Vec3::Y).normalize_or_zero();

            let target_dir = match target_translation {
                Some(dir) => (dir - missile_transform.translation).normalize_or_zero(),
                None => current_dir,
            };

            let velocity =
                if current_dir.angle_between(target_dir).abs() < std::f32::consts::FRAC_PI_2 {
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
            missile_transform.rotation =
                Quat::from_rotation_arc(Vec3::Y, velocity.normalize_or_zero());

            let distance_to_target = target_translation
                .map(|t| (t - missile_transform.translation).length())
                .unwrap_or(f32::INFINITY);
            missile.lifetime -= time.delta_seconds();
            if distance_to_target < 2. || missile.lifetime < 0. {
                commands.entity(missile_entity).despawn_recursive();
            }
        }
    }
}
