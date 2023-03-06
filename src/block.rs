use std::f32::consts::{FRAC_PI_4, PI};

use bevy::prelude::*;

use bevy::{input::mouse::MouseMotion, render::mesh::VertexAttributeValues, ui::FocusPolicy};
use bevy_mod_picking::{Hover, PickableMesh, PickingEvent};

#[derive(Component)]
struct Block;

pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn)
            .add_system(rotate_with_mouse)
            .add_system_to_stage(CoreStage::PostUpdate, block_picking_events);
    }
}

fn calculate_face_uv(sprite_index: usize) -> [[f32; 2]; 4] {
    let cols = 8;
    let rows = 8;
    let sprite_width = 16.0;
    let sprite_height = 16.0;
    let texture_width = sprite_width * cols as f32;
    let texture_height = sprite_height * rows as f32;
    let col_index = sprite_index % cols;
    let row_index = sprite_index / rows;
    let fractional_width = sprite_width / texture_width;
    let fractional_height = sprite_height / texture_height;
    let left = fractional_width * col_index as f32;
    let top = fractional_width * row_index as f32;
    let right = left + fractional_width;
    let bottom = top + fractional_height;
    [[left, top], [right, top], [right, bottom], [left, bottom]]
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
        uv_coordinates.extend(calculate_face_uv(0))
    }
    let uvs = VertexAttributeValues::Float32x2(uv_coordinates);
    if let Some((_id, values)) = cube.attributes_mut().nth(2) {
        *values = uvs;
    }
    let mesh_handle = meshes.add(cube);

    let mut transform = Transform::default();
    transform.rotate_x(FRAC_PI_4);
    transform.rotate_y(FRAC_PI_4);

    commands.spawn((
        PbrBundle {
            mesh: mesh_handle.clone(),
            material,
            transform,
            ..default()
        },
        Block,
        (
            PickableMesh::default(),
            Interaction::default(),
            FocusPolicy::default(),
            Hover::default(),
        ),
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

fn block_picking_events(mut events: EventReader<PickingEvent>) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(e) => info!("A selection event happened: {:?}", e),
            PickingEvent::Hover(e) => info!("Egads! A hover event!? {:?}", e),
            PickingEvent::Clicked(e) => info!("Gee Willikers, it's a click! {:?}", e),
        }
    }
}
