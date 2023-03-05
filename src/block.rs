use bevy::prelude::*;

use crate::texture::uv_debug_texture;

#[derive(Component)]
struct Block;

pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

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
        Block,
    ));
}
