use bevy::{core::FixedTimestep, pbr::AmbientLight, prelude::*};
use bevy_rapier3d::prelude::*;

mod input;
mod particles;
mod player;
mod sky;
mod terrain;
mod ui;

use input::*;
use particles::*;
use player::*;
use sky::*;
use terrain::*;
use ui::*;

const PLAYER_MOVEMENT_LABEL: &str = "player_movement";
const FIRE_MISSILE_LABEL: &str = "fire_missile";

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
        .init_resource::<UiTargets>()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierRenderPlugin)
        .add_plugin(SkyBoxPlugin)
        .add_asset::<ParticleMaterial>()
        .insert_resource(SmokeTextures::default())
        .add_startup_system(setup_particles.system())
        .add_startup_system(setup.system())
        .add_startup_system(setup_terrain.system())
        .add_startup_system(setup_ui.system())
        .add_startup_system(setup_player.system())
        .add_system(player_input.system())
        .add_system(player_movement.system().label(PLAYER_MOVEMENT_LABEL))
        .add_system_to_stage(
            bevy_rapier3d::physics::PhysicsStages::SyncTransforms,
            camera_follow_player
                .system()
                .after(bevy_rapier3d::physics::PhysicsSystems::SyncTransforms),
        )
        .add_system(text_update_system.system())
        .add_system_to_stage(CoreStage::PreUpdate, connection_system.system())
        .add_system(gamepad_system.system())
        .add_system(target_ui.system())
        .add_system(fire_missle.system().label(FIRE_MISSILE_LABEL))
        .add_system(missle_run.system().after(FIRE_MISSILE_LABEL))
        .add_system(radar.system())
        .add_system(drone_movement.system())
        .add_system(
            run_emitter
                .system()
                .with_run_criteria(FixedTimestep::step(0.05)),
        )
        .add_system(run_particles.system())
        .run();
}

pub struct Drone;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                transform: Transform::from_translation(Vec3::new(
                    x as f32 * 500.,
                    350.0,
                    y as f32 * 500.,
                )),
                ..Default::default()
            });
        }
    }

    commands
        .spawn_bundle((
            Transform::from_translation(Vec3::new(50.0, 300.0, 0.0)),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("f35.gltf#Scene0"));
        })
        .insert(Target)
        .insert(Drone);

    commands
        .spawn_bundle((
            Transform::from_translation(Vec3::new(0.0, 350.0, -50.0)),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("f35.gltf#Scene0"));
        })
        .insert(Target)
        .insert(Drone);
}

fn drone_movement(mut drone_query: Query<(&mut Transform, &Drone)>, timer: Res<Time>) {
    for (mut drone_transform, _) in drone_query.iter_mut() {
        let pitch_delta = 0. * timer.delta_seconds() * PITCH_SPEED;
        let roll_delta = 0. * timer.delta_seconds() * ROLL_SPEED;
        let yaw_delta = 0.5 * timer.delta_seconds() * YAW_SPEED;

        let ypr_rotation = Quat::from_rotation_ypr(yaw_delta, pitch_delta, roll_delta);

        let velocity = 90.;

        drone_transform.rotation = drone_transform.rotation * ypr_rotation;

        drone_transform.translation = drone_transform.translation
            + (drone_transform.rotation * Vec3::X * velocity * timer.delta_seconds());
    }
}
