use std::f32::consts::{FRAC_PI_4, PI};

use bevy::prelude::*;

use bevy::{input::mouse::MouseMotion, render::mesh::VertexAttributeValues, ui::FocusPolicy};
use bevy_mod_picking::{Hover, PickableMesh, PickingEvent};

#[derive(Component)]
struct Block {
    pub mesh: Handle<Mesh>,
    pub adjacent: usize,
}

impl Block {
    pub fn new(mut meshes: ResMut<Assets<Mesh>>) -> Self {
        let mut cube: Mesh = shape::Cube::default().into();
        Self::update_face_mesh(&mut cube, 0);
        Self {
            mesh: meshes.add(cube),
            adjacent: 0,
        }
    }
    pub fn update_adjacent(&mut self, meshes: &mut ResMut<Assets<Mesh>>, adjacent: usize) {
        if let Some(cube) = meshes.get_mut(&self.mesh) {
            Self::update_face_mesh(cube, adjacent);
        }
        self.adjacent = adjacent;
    }
    fn update_face_mesh(cube: &mut Mesh, index: usize) {
        let mut uv_coordinates = Vec::new();
        for _ in 0..6 {
            uv_coordinates.extend(Self::calculate_face_uv(index))
        }
        let uvs = VertexAttributeValues::Float32x2(uv_coordinates);
        if let Some((_id, values)) = cube.attributes_mut().nth(2) {
            *values = uvs;
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
}

pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn)
            .add_system(rotate_with_mouse)
            .add_system_to_stage(CoreStage::PostUpdate, block_picking_events);
    }
}

fn spawn(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture_handle = asset_server.load("block.png");

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        ..default()
    });

    let mut transform = Transform::default();
    transform.rotate_x(FRAC_PI_4);
    transform.rotate_y(FRAC_PI_4);

    let block = Block::new(meshes);

    commands.spawn((
        PbrBundle {
            mesh: block.mesh.clone(),
            material,
            transform,
            ..default()
        },
        block,
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

fn block_picking_events(
    mut events: EventReader<PickingEvent>,
    mut query: Query<(Entity, &mut Block)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for event in events.iter() {
        if let PickingEvent::Clicked(entity) = event {
            for (e, mut block) in query.iter_mut() {
                if e == *entity {
                    let adjacent = block.adjacent + 1;
                    block.update_adjacent(&mut meshes, adjacent);
                    break;
                }
            }
        }
    }
}
