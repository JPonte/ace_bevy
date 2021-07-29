use bevy::{
    input::gamepad::{Gamepad, GamepadButton, GamepadEvent, GamepadEventType},
    pbr::AmbientLight,
    prelude::*,
    render::camera::{Camera, PerspectiveProjection},
    render::mesh::Mesh,
    utils::HashSet,
};

mod terrain;
use terrain::setup_terrain;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Ace Bevy!".to_string(),
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.3, 0.56, 0.83)))
        .insert_resource(PlayerInput::default())
        .init_resource::<GamepadLobby>()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_startup_system(setup_terrain.system())
        .add_system(player_input.system())
        .add_system(player_movement.system())
        .add_system(camera_follow_player.system())
        .add_system(text_update_system.system())
        .add_system_to_stage(CoreStage::PreUpdate, connection_system.system())
        .add_system(gamepad_system.system())
        .add_system(target_ui.system())
        .add_system(fire_missle.system().label("fire_missile"))
        .add_system(missle_run.system().after("fire_missile"))
        .add_system(radar.system())
        .run();
}

#[derive(Default)]
struct Player {
    velocity: f32,
    missiles_fired: u32,
}

#[derive(Default)]
struct PlayerInput {
    axis: Vec2,
    accel: f32,
    brake: f32,
    yaw: f32,
}
struct MainCamera;

struct SpeedText;

struct Target;
struct UiTarget;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.01,
    });

    for x in -1..2 {
        for y in -1..2 {
            commands.spawn_bundle(LightBundle {
                light: Light {
                    color: Color::WHITE,
                    intensity: 20000.,
                    range: 20000.,
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(x as f32 * 500., 350.0, y as f32 * 500.)),
                ..Default::default()
            });
        }
    }

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            perspective_projection: PerspectiveProjection {
                far: 2000.,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(MainCamera);

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

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(5., 5., 5.))),
            transform: Transform::from_translation(Vec3::new(100., 50., 10.)),
            material: materials.add(StandardMaterial {
                base_color: Color::PURPLE,
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert(Target);

    let target_ui_size = 30.;

    commands
        .spawn_bundle(NodeBundle::default())
        .with_children(|parent| {
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(1.), Val::Px(target_ui_size + 1.)),
                    position_type: PositionType::Absolute,
                    position: Rect {
                        left: Val::Px(-target_ui_size / 2.),
                        bottom: Val::Px(-target_ui_size / 2.),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                material: color_materials.add(Color::rgb(0.0, 1., 0.).into()),
                ..Default::default()
            });
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(1.), Val::Px(target_ui_size + 1.)),
                    position_type: PositionType::Absolute,
                    position: Rect {
                        left: Val::Px(target_ui_size / 2.),
                        bottom: Val::Px(-target_ui_size / 2.),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                material: color_materials.add(Color::rgb(0.0, 1., 0.).into()),
                ..Default::default()
            });
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(target_ui_size + 1.), Val::Px(1.)),
                    position_type: PositionType::Absolute,
                    position: Rect {
                        left: Val::Px(-target_ui_size / 2.),
                        bottom: Val::Px(target_ui_size / 2.),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                material: color_materials.add(Color::rgb(0.0, 1., 0.).into()),
                ..Default::default()
            });
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(target_ui_size + 1.), Val::Px(1.)),
                    position_type: PositionType::Absolute,
                    position: Rect {
                        left: Val::Px(-target_ui_size / 2.),
                        bottom: Val::Px(-target_ui_size / 2.),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                material: color_materials.add(Color::rgb(0.0, 1., 0.).into()),
                ..Default::default()
            });
        })
        .insert(UiTarget);

    let plane: Handle<Scene> = asset_server.load("f35.gltf#Scene0");

    commands
        .spawn_bundle((
            Transform::from_translation(Vec3::new(0.0, 800.0, 0.0)),
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

    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexStart,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Percent(50.0),
                    left: Val::Percent(25.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 25.0,
                    color: Color::GREEN,
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Right,
                    vertical: VerticalAlign::Center,
                    ..Default::default()
                },
            ),
            ..Default::default()
        })
        .insert(SpeedText);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(200.), Val::Px(200.)),
                position_type: PositionType::Absolute,
                position: Rect {
                    right: Val::Px(10.),
                    bottom: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: color_materials.add(Color::rgb(0.0, 0.15, 0.).into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(5.), Val::Px(5.)),
                        position_type: PositionType::Relative,
                        position: Rect {
                            left: Val::Percent(50.),
                            bottom: Val::Percent(50.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    material: color_materials.add(Color::rgb(0.0, 1., 0.).into()),
                    ..Default::default()
                })
                .insert(RadarDot);
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(10.), Val::Px(10.)),
                    position_type: PositionType::Absolute,
                    position: Rect {
                        right: Val::Px(95.),
                        bottom: Val::Px(95.),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                material: color_materials.add(Color::rgb(0.0, 0.5, 1.).into()),
                ..Default::default()
            });
        })
        .insert(Radar);
}

fn camera_follow_player(
    mut query_set: QuerySet<(
        Query<&mut Transform, With<MainCamera>>,
        Query<&Transform, With<Player>>,
    )>,
    time: Res<Time>,
) {
    let mut player_translation = Vec3::ZERO;
    let mut player_rotation = Quat::default();

    if let Some(player_transform) = query_set.q1().iter().next() {
        player_translation = player_transform.translation;
        player_rotation = player_transform.rotation;
    }

    if let Some(mut camera_transform) = query_set.q0_mut().iter_mut().next() {
        let new_transform = Transform::from_translation(
            player_translation + player_rotation * Vec3::new(-8.0, 2.0, 0.0),
        )
        .looking_at(
            player_translation + (player_rotation * Vec3::Y).normalize() * 1.5,
            (player_rotation * Vec3::Y).normalize(),
        );

        camera_transform.translation = camera_transform.translation.lerp(
            new_transform.translation,
            (4. * time.delta_seconds()).clamp(0., 1.),
        );
        camera_transform.rotation = camera_transform.rotation.lerp(
            new_transform.rotation,
            (5. * time.delta_seconds()).clamp(0., 1.),
        );
    }
}

fn player_input(
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

const ROLL_SPEED: f32 = 1.;
const PITCH_SPEED: f32 = 2.;
const YAW_SPEED: f32 = 0.5;

const MIN_SPEED: f32 = 20.;
const MAX_SPEED: f32 = 100.;
const ACCEL: f32 = 4.;
const BRAKE: f32 = 8.;

fn player_movement(
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

fn text_update_system(
    mut query: Query<&mut Text, With<SpeedText>>,
    player_query: Query<(&Player, &Transform)>,
) {
    if let Some((player, player_transform)) = player_query.iter().next() {
        for mut text in query.iter_mut() {
            text.sections[0].value = format!(
                "{:.2} Km/H\n{} m",
                (player.velocity * 25.).round() as i32,
                (player_transform.translation.y * 5.) as i32
            );
        }
    }
}

#[derive(Default)]
struct GamepadLobby {
    gamepads: HashSet<Gamepad>,
}

fn connection_system(
    mut lobby: ResMut<GamepadLobby>,
    mut gamepad_event: EventReader<GamepadEvent>,
) {
    for event in gamepad_event.iter() {
        match &event {
            GamepadEvent(gamepad, GamepadEventType::Connected) => {
                lobby.gamepads.insert(*gamepad);
                println!("{:?} Connected", gamepad);
            }
            GamepadEvent(gamepad, GamepadEventType::Disconnected) => {
                lobby.gamepads.remove(gamepad);
                println!("{:?} Disconnected", gamepad);
            }
            _ => (),
        }
    }
}

fn gamepad_system(
    lobby: Res<GamepadLobby>,
    button_axes: Res<Axis<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut player_input: ResMut<PlayerInput>,
) {
    for gamepad in lobby.gamepads.iter().cloned() {
        let right_trigger = button_axes
            .get(GamepadButton(gamepad, GamepadButtonType::RightTrigger2))
            .unwrap();

        if right_trigger.abs() > 0.01 {
            player_input.accel = right_trigger;
        } else {
            player_input.accel = 0.0;
        }

        let left_trigger = button_axes
            .get(GamepadButton(gamepad, GamepadButtonType::LeftTrigger2))
            .unwrap();

        if left_trigger.abs() > 0.01 {
            player_input.brake = left_trigger;
        } else {
            player_input.brake = 0.0;
        }

        let left_stick_x = axes
            .get(GamepadAxis(gamepad, GamepadAxisType::LeftStickX))
            .unwrap();
        let left_stick_y = axes
            .get(GamepadAxis(gamepad, GamepadAxisType::LeftStickY))
            .unwrap();

        if left_stick_x.abs() > 0.01 || left_stick_y.abs() > 0.01 {
            player_input.axis = Vec2::new(left_stick_x, left_stick_y);
        } else {
            player_input.axis = Vec2::ZERO;
        }

        let left_shoulder = button_axes
            .get(GamepadButton(gamepad, GamepadButtonType::LeftTrigger))
            .unwrap();
        let right_shoulder = button_axes
            .get(GamepadButton(gamepad, GamepadButtonType::RightTrigger))
            .unwrap();

        player_input.yaw = left_shoulder - right_shoulder;
    }
}

fn target_ui(
    target_query: Query<&Transform, With<Target>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut ui_targets: Query<&mut Style, With<UiTarget>>,
    windows: Res<Windows>,
) {
    if let Some((camera, camera_global_transform)) = camera_query.iter().next() {
        for target_transform in target_query.iter() {
            match camera.world_to_screen(
                &windows,
                camera_global_transform,
                target_transform.translation,
            ) {
                Some(target_screen_coords) => {
                    ui_targets.iter_mut().for_each(|mut style| {
                        style.position = Rect {
                            bottom: Val::Px(target_screen_coords.y),
                            left: Val::Px(target_screen_coords.x),
                            ..Default::default()
                        };
                    });
                }
                None => {}
            }
        }
    }
}

struct Missile {
    target: Vec3,
    velocity: f32,
}

fn fire_missle(
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
                                    radius: 0.1,
                                    depth: 1.,
                                    ..Default::default()
                                })),
                                material: materials.add(StandardMaterial {
                                    base_color: Color::GOLD,
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
                                target: Vec3::new(100., 50., 10.),
                                velocity: player.velocity + 30.,
                            });
                        player.missiles_fired = player.missiles_fired + 1;
                    }
                }
                _ => {}
            }
        }
    }
}

fn missle_run(
    mut commands: Commands,
    mut missile_query: Query<(&mut Transform, &Missile, Entity)>,
    time: Res<Time>,
) {
    for (mut missile_transform, missile, entity) in missile_query.iter_mut() {
        let target_dir = (missile.target - missile_transform.translation).normalize_or_zero();

        let current_dir = (missile_transform.rotation * Vec3::Y).normalize_or_zero();

        let velocity = if current_dir.angle_between(target_dir).abs() < std::f32::consts::FRAC_PI_2
        {
            current_dir
                .lerp(target_dir, time.delta_seconds() * 1.5)
                .normalize_or_zero()
                * missile.velocity
        } else {
            current_dir * missile.velocity
        };

        missile_transform.translation =
            missile_transform.translation + velocity * time.delta_seconds();
        missile_transform.rotation = Quat::from_rotation_arc(Vec3::Y, velocity.normalize_or_zero());

        if (missile.target - missile_transform.translation).length() < 2. {
            commands.entity(entity).despawn_recursive();
        }
    }
}

const RADAR_RANGE: f32 = 250.;
struct Radar;
struct RadarDot;

fn radar(
    target_query: Query<&Transform, With<Target>>,
    player_query: Query<&Transform, With<Player>>,
    mut dots_query: Query<&mut Style, With<RadarDot>>,
) {
    if let Some(player_transform) = player_query.iter().next() {
        let player_pos = Vec3::new(
            player_transform.translation.x,
            0.,
            player_transform.translation.z,
        );

        for target_transform in target_query.iter() {
            let target_pos = Vec3::new(
                target_transform.translation.x,
                0.,
                target_transform.translation.z,
            );

            let mut current_vec = player_transform.rotation * Vec3::X;
            current_vec.y = 0.;
            current_vec = current_vec.normalize_or_zero();
            let rotation = Quat::from_rotation_arc(current_vec, Vec3::Z);

            let target_vec = rotation * (target_pos - player_pos);

            if let Some(mut dot) = dots_query.iter_mut().next() {
                if target_vec.x.abs() < RADAR_RANGE && target_vec.z.abs() < RADAR_RANGE {
                    dot.position = Rect {
                        left: Val::Percent(50. + (target_vec.x) / RADAR_RANGE * 50.),
                        bottom: Val::Percent(50. + (target_vec.z) / RADAR_RANGE * 50.),
                        ..Default::default()
                    }
                } else {
                    dot.position = Rect {
                        left: Val::Percent(-100.),
                        bottom: Val::Percent(-100.),
                        ..Default::default()
                    }
                }
            }
        }
    }
}
