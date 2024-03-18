// Disable console window in Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{log::LogPlugin, prelude::*, window::WindowResolution};

mod game;
mod input;
mod menu;
mod settings;

use game::GamePlugin;
use input::InputPlugin;
use menu::MenuPlugin;
use settings::SettingsPlugin;

pub use input::InputEvent;
pub use settings::Settings;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GlobalState {
    #[default]
    Menu,
    Game,
}

fn main() {
    App::new()
        .init_state::<GlobalState>()
        .add_plugins(
            DefaultPlugins
                // Window settings
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        visible: false,
                        resolution: WindowResolution::new(1024.0, 768.0),
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
        .add_plugins((MenuPlugin, SettingsPlugin, GamePlugin, InputPlugin))
        .run();
}

fn setup(mut commands: Commands, mut window: Query<&mut Window>) {
    window.single_mut().visible = true;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1200.0,
            shadows_enabled: true,
            color: Color::rgb(1.0, 0.95, 0.90),
            ..default()
        },
        transform: Transform::from_xyz(-1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.insert_resource(AmbientLight {
        brightness: 100.0,
        color: Color::rgb(0.95, 0.95, 1.0),
    });
}
