use bevy::{prelude::*, render::camera::*};

use super::player::*;

const RADAR_RANGE: f32 = 1000.;
pub struct Radar;
pub struct RadarDot;
pub struct SpeedText;
pub struct UiTarget;

#[derive(Default)]
pub struct UiTargets {
    targets: Vec<Entity>,
    radar_dots: Vec<Entity>,
}

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
        .insert(Radar)
        .with_children(|parent| {
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
        });
}

fn spawn_target(
    commands: &mut Commands,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
) -> Entity {
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
        .insert(UiTarget)
        .id()
}

fn spawn_radar_dot(
    commands: &mut Commands,
    radar: Entity,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
) -> Entity {
    let mut child_id: Entity = Entity::new(0);
    commands.entity(radar).with_children(|parent| {
        child_id = parent
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
            .insert(RadarDot)
            .id();
    });
    child_id
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
    player_query: Query<&Player>,
    mut ui_targets: Query<&mut Style, With<UiTarget>>,
    mut ui_targets_res: ResMut<UiTargets>,
    windows: Res<Windows>,
    mut commands: Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    let player_target = player_query.single().map(|p| p.target).unwrap();
    let (camera, camera_global_transform) = camera_query.single().unwrap();

    let targets_to_draw: Vec<Vec2> = target_query
        .iter()
        .flat_map(|target_transform| {
            camera.world_to_screen(
                &windows,
                camera_global_transform,
                target_transform.translation,
            )
        })
        .filter(|target_screen_coords| {
            target_screen_coords.x < windows.get_primary().unwrap().width()
                && target_screen_coords.x > 0.
                && target_screen_coords.y < windows.get_primary().unwrap().height()
                && target_screen_coords.y > 0.
        })
        .collect();

    while ui_targets_res.targets.len() > targets_to_draw.len() {
        if let Some(target) = ui_targets_res.targets.pop() {
            commands.entity(target).despawn_recursive();
        }
    }
    while ui_targets_res.targets.len() < targets_to_draw.len() {
        ui_targets_res
            .targets
            .push(spawn_target(&mut commands, &mut color_materials));
    }

    ui_targets_res.targets.iter().zip(targets_to_draw).for_each(
        |(ui_target_entity, screen_coords)| {
            if let Ok(mut ui_target) = ui_targets.get_mut(*ui_target_entity) {
                ui_target.position = Rect {
                    bottom: Val::Px(screen_coords.y),
                    left: Val::Px(screen_coords.x),
                    ..Default::default()
                };
            }
        },
    );
}

pub fn radar(
    target_query: Query<&Transform, With<Target>>,
    player_query: Query<&Transform, With<Player>>,
    radar_query: Query<Entity, With<Radar>>,
    mut dots_query: Query<&mut Style, With<RadarDot>>,
    mut ui_targets_res: ResMut<UiTargets>,
    mut commands: Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    let radar = radar_query.single().unwrap();
    let player_transform = player_query.single().unwrap();
    let player_pos = Vec3::new(
        player_transform.translation.x,
        0.,
        player_transform.translation.z,
    );

    let mut current_vec = player_transform.rotation * Vec3::X;
    current_vec.y = 0.;
    current_vec = current_vec.normalize_or_zero();
    let rotation = Quat::from_rotation_arc(current_vec, Vec3::Z);

    let targets_to_draw: Vec<Vec3> = target_query
        .iter()
        .map(|target_transform| {
            let target_pos = Vec3::new(
                target_transform.translation.x,
                0.,
                target_transform.translation.z,
            );
            rotation * (target_pos - player_pos)
        })
        .filter(|target_vec| target_vec.x.abs() < RADAR_RANGE && target_vec.z.abs() < RADAR_RANGE)
        .collect();

    while ui_targets_res.radar_dots.len() > targets_to_draw.len() {
        if let Some(target) = ui_targets_res.radar_dots.pop() {
            commands.entity(target).despawn_recursive();
        }
    }
    while ui_targets_res.radar_dots.len() < targets_to_draw.len() {
        ui_targets_res.radar_dots.push(spawn_radar_dot(
            &mut commands,
            radar.clone(),
            &mut color_materials,
        ));
    }

    ui_targets_res
        .radar_dots
        .iter()
        .zip(targets_to_draw)
        .for_each(|(dot_entity, target_vec)| {
            if let Ok(mut dot) = dots_query.get_mut(*dot_entity) {
                dot.position = Rect {
                    left: Val::Percent(50. + (target_vec.x) / RADAR_RANGE * 50.),
                    bottom: Val::Percent(50. + (target_vec.z) / RADAR_RANGE * 50.),
                    ..Default::default()
                }
            }
        });
}
