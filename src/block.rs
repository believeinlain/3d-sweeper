use std::f32::consts::{FRAC_PI_4, PI};

use bevy::prelude::*;

use bevy::input::mouse::MouseMotion;
use bevy::render::mesh::VertexAttributeValues;

#[derive(Component)]
struct Block;

const UVS_HIDDEN: [[f32; 2]; 4] = [[0.0, 0.0], [0.25, 0.0], [0.25, 1.0], [0.0, 1.0]];
const _UVS_1: [[f32; 2]; 4] = [[0.25, 0.0], [0.5, 0.0], [0.5, 1.0], [0.25, 1.0]];

pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn).add_system(rotate_with_mouse);
    }
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture_handle = asset_server.load("block.png");

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        ..default()
    });

    let mut cube: Mesh = shape::Cube::default().into();
    let mut uv_coordinates = Vec::new();
    for _ in 0..6 {
        uv_coordinates.extend(UVS_HIDDEN)
    }
    let uvs = VertexAttributeValues::Float32x2(uv_coordinates);
    if let Some((_id, values)) = cube.attributes_mut().nth(2) {
        *values = uvs;
    }
    let mesh = meshes.add(cube);

    let mut transform = Transform::default();
    transform.rotate_x(FRAC_PI_4);
    transform.rotate_y(FRAC_PI_4);

    commands.spawn((
        PbrBundle {
            mesh,
            material,
            transform,
            ..default()
        },
        Block,
    ));
}

fn rotate_with_mouse(
    windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&Block, &mut Transform)>,
) {
    let rotate_binding = MouseButton::Middle;

    if input_mouse.pressed(rotate_binding) {
        let mut rotation_delta = Vec2::ZERO;
        for ev in ev_motion.iter() {
            rotation_delta += ev.delta;
        }
        if rotation_delta.length_squared() > 0.0 {
            let window = get_primary_window_size(&windows);
            let delta_x = rotation_delta.x / window.x * PI * 2.0;
            let delta_y = rotation_delta.y / window.y * PI;
            for (_block, mut transform) in query.iter_mut() {
                // TODO: make rotation feel more intuitive
                transform.rotate_axis(Vec3::Y, delta_x);
                transform.rotate_axis(Vec3::Z, -delta_y);
            }
        }
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    Vec2::new(window.width(), window.height())
}
