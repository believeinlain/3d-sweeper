use std::f32::consts::{FRAC_PI_4, PI};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;

use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::render::mesh::VertexAttributeValues;

#[derive(Component)]
struct Block;

const UVS_HIDDEN: [[f32; 2]; 4] = [[0.0, 0.0], [0.25, 0.0], [0.25, 1.0], [0.0, 1.0]];
const _UVS_1: [[f32; 2]; 4] = [[0.25, 0.0], [0.5, 0.0], [0.5, 1.0], [0.25, 1.0]];

#[derive(Debug)]
struct Ray {
    position: Vec3,
    direction: Vec3,
}
impl Ray {
    fn new(position: Vec3, direction: Vec3) -> Self {
        Self {
            position,
            direction,
        }
    }
}

pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (rotate_with_mouse, click_on_block));
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
            transform: transform.clone(),
            ..default()
        },
        Block,
        Collider::cuboid(0.5, 0.5, 0.5),
    ));
}

fn rotate_with_mouse(
    windows: Query<&Window>,
    mut ev_motion: EventReader<MouseMotion>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&Block, &mut Transform)>,
) {
    let rotate_binding = MouseButton::Middle;

    if input_mouse.pressed(rotate_binding) {
        let mut rotation_delta = Vec2::ZERO;
        for ev in ev_motion.read() {
            rotation_delta += ev.delta;
        }
        if rotation_delta.length_squared() > 0.0 {
            let window = windows.single();
            let delta_x = rotation_delta.x / window.width() * PI * 2.0;
            let delta_y = rotation_delta.y / window.height() * PI;
            for (_block, mut transform) in query.iter_mut() {
                // TODO: make rotation feel more intuitive
                transform.rotate_axis(Vec3::Y, delta_x);
                transform.rotate_axis(Vec3::Z, -delta_y);
            }
        }
    } else {
        ev_motion.clear();
    }
}

fn click_on_block(
    mut mouse_input: EventReader<MouseButtonInput>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    rapier_context: Res<RapierContext>,
) {
    let Some(cursor_pos) = windows.single().cursor_position() else {
        return;
    };
    let (camera, camera_trans) = cameras.single();
    for mouse_event in mouse_input.read() {
        if mouse_event.state.is_pressed() {
            trace!("Left click at {cursor_pos:?}");
            let Some(ray) = get_cursor_ray(camera, camera_trans, cursor_pos) else {
                continue;
            };
            trace!("Cursor ray at {ray:?}");
            let max_toi = 10.0;
            let solid = true;
            let filter = QueryFilter::new();

            let Some(collider) =
                rapier_context.cast_ray(ray.position, ray.direction, max_toi, solid, filter)
            else {
                continue;
            };
            info!("Collider entity {collider:?} hit");
        }
    }
}

fn get_cursor_ray(
    camera: &Camera,
    camera_trans: &GlobalTransform,
    cursor_pos: Vec2,
) -> Option<Ray> {
    let view = camera_trans.compute_matrix();

    let viewport = camera.logical_viewport_rect()?;
    let screen_size = camera.logical_target_size()?;
    let adj_cursor_pos = cursor_pos - Vec2::new(viewport.min.x, screen_size.y - viewport.max.y);

    let projection = camera.projection_matrix();
    let far_ndc = projection.project_point3(Vec3::NEG_Z).z;
    let near_ndc = projection.project_point3(Vec3::Z).z;
    let cursor_ndc = (adj_cursor_pos / viewport.size()) * 2.0 - Vec2::ONE;
    let ndc_to_world: Mat4 = view * projection.inverse();
    let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
    let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
    let ray_direction = far - near;
    Some(Ray::new(near, ray_direction))
}
