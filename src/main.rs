use bevy::{log::LogPlugin, prelude::*, window::WindowResolution};
use bevy_rapier3d::prelude::*;

mod block;
mod camera;

use block::BlockPlugin;
use camera::MainCameraPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(800.0, 600.0),
                        title: "3D Sweeper".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
        )
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_plugins((BlockPlugin, MainCameraPlugin))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(16.0, 8.0, 8.0),
        ..default()
    });
}
