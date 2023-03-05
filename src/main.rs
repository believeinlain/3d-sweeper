use bevy::prelude::*;

use bevy::{
    core_pipeline::CorePipelinePlugin, diagnostic::DiagnosticsPlugin, input::InputPlugin,
    log::LogPlugin, pbr::PbrPlugin, render::RenderPlugin, scene::ScenePlugin, sprite::SpritePlugin,
    text::TextPlugin, time::TimePlugin, ui::UiPlugin, window::WindowPlugin, winit::WinitPlugin,
};

mod block;
mod texture;

use block::BlockPlugin;

fn main() {
    App::new()
        .add_plugin(LogPlugin::default())
        .add_plugin(CorePlugin::default())
        .add_plugin(TimePlugin::default())
        .add_plugin(TransformPlugin::default())
        .add_plugin(DiagnosticsPlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(WindowPlugin {
            window: WindowDescriptor {
                width: 800.0,
                height: 600.0,
                title: "Sweeper 3D".to_string(),
                ..default()
            },
            ..default()
        })
        .add_plugin(AssetPlugin::default())
        .add_plugin(ScenePlugin::default())
        .add_plugin(RenderPlugin::default())
        .add_plugin(ImagePlugin::default_nearest())
        .add_plugin(CorePipelinePlugin::default())
        .add_plugin(SpritePlugin::default())
        .add_plugin(PbrPlugin::default())
        .add_plugin(UiPlugin::default())
        .add_plugin(TextPlugin::default())
        .add_plugin(WinitPlugin::default())
        .add_plugin(BlockPlugin)
        .add_startup_system(setup)
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
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
}
