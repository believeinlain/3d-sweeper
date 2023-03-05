use bevy::prelude::*;

use bevy::{
    core_pipeline::CorePipelinePlugin,
    diagnostic::DiagnosticsPlugin,
    input::InputPlugin,
    log::LogPlugin,
    pbr::PbrPlugin,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        RenderPlugin,
    },
    scene::ScenePlugin,
    sprite::SpritePlugin,
    text::TextPlugin,
    time::TimePlugin,
    ui::UiPlugin,
    window::WindowPlugin,
    winit::WinitPlugin,
};

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
        .add_startup_system(setup)
        .run();
}

#[derive(Component)]
struct Shape;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let cube = meshes.add(shape::Cube::default().into());

    commands.spawn((
        PbrBundle {
            mesh: cube,
            material: debug_material,
            ..default()
        },
        Shape,
    ));

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

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}
