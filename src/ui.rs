use bevy::{prelude::*, render::camera::*};

use super::player::*;

const RADAR_RANGE: f32 = 250.;
pub struct Radar;
pub struct RadarDot;
pub struct SpeedText;
pub struct UiTarget;

pub fn setup_ui(
    mut commands: Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
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
}

pub fn text_update_system(
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

pub fn target_ui(
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

pub fn radar(
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
