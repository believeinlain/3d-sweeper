use bevy::{
    app::AppExit, input::keyboard::KeyboardInput, log::LogPlugin, prelude::*,
    window::WindowResolution,
};

mod block;
mod camera;

use block::BlockPlugin;
use camera::MainCameraPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                // Window settings
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(800.0, 600.0),
                        title: "3D Sweeper".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                // Log settings
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..default()
                })
                // Texture settings
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, handle_key_events)
        .add_plugins((BlockPlugin, MainCameraPlugin))
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1000.0,
            shadows_enabled: true,
            color: Color::rgb(1.0, 0.9, 0.85),
            ..default()
        },
        transform: Transform::from_xyz(-1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.insert_resource(AmbientLight {
        brightness: 80.0,
        color: Color::rgb(0.9, 0.9, 1.0),
    });
}

fn handle_key_events(
    mut key_events: EventReader<KeyboardInput>,
    mut exit_events: EventWriter<AppExit>,
) {
    for event in key_events.read() {
        match event {
            KeyboardInput {
                key_code, state, ..
            } if matches!(key_code, KeyCode::Escape) && state.is_pressed() => {
                info!("Pressed ESC key, exiting...");
                exit_events.send(AppExit);
            }
            _ => {}
        }
    }
}
