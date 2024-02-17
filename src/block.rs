use std::f32::consts::{FRAC_PI_4, PI};

use bevy::prelude::*;
use bevy::render::texture::ImageSampler;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;

use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::render::mesh::VertexAttributeValues;

#[derive(Component, Default)]
struct Block {
    /// Number of mines adjacent to this block.
    adjacent: u8,
    /// Whether this block has been marked as a mine.
    marked: bool,
    /// Whether this block has been revealed, and thus should
    /// show its number of adjacent mines.
    revealed: bool,
}
impl Block {
    pub fn new() -> Self {
        Self {
            adjacent: 0,
            marked: false,
            revealed: true,
        }
    }
}

const UVS_HIDDEN: [[f32; 2]; 4] = [[0.0, 0.0], [0.125, 0.0], [0.125, 0.5], [0.0, 0.5]];
const UVS_1: [[f32; 2]; 4] = [[0.125, 0.0], [0.25, 0.0], [0.25, 0.5], [0.125, 0.5]];
const UVS_2: [[f32; 2]; 4] = [[0.25, 0.0], [0.375, 0.0], [0.375, 0.5], [0.25, 0.5]];
const UVS_3: [[f32; 2]; 4] = [[0.375, 0.0], [0.5, 0.0], [0.5, 0.5], [0.375, 0.5]];
const UVS_FLAG: [[f32; 2]; 4] = [[0.0, 0.5], [0.125, 0.5], [0.125, 1.0], [0.0, 1.0]];

pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(
                Update,
                (
                    modify_texture,
                    rotate_with_mouse,
                    click_on_block,
                    handle_block_events,
                ),
            )
            .add_event::<BlockEvent>();
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
    set_cube_uvs(&mut cube, UVS_HIDDEN);
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
        Block::new(),
        Collider::cuboid(0.5, 0.5, 0.5),
    ));
}

/// Modify shading to use nearest neighbor sampling.
/// Texture sampling settings are in `main.rs`
fn modify_texture(
    mut asset_events: EventReader<AssetEvent<Image>>,
    mut assets: ResMut<Assets<Image>>,
) {
    for event in asset_events.read() {
        match event {
            AssetEvent::Added { id } => {
                if let Some(texture) = assets.get_mut(*id) {
                    texture.sampler = ImageSampler::nearest();
                }
            }
            _ => {}
        }
    }
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

#[derive(Event)]
pub enum BlockEvent {
    /// Uncover a block, detonating any contained mines.
    Uncover(Entity),
    /// Mark a block (or unmark if already marked) as containing a mine.
    Mark(Entity),
}

fn click_on_block(
    mut mouse_input: EventReader<MouseButtonInput>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    rapier_context: Res<RapierContext>,
    mut block_events: EventWriter<BlockEvent>,
) {
    let Some(cursor_pos) = windows.single().cursor_position() else {
        return;
    };
    let (camera, camera_trans) = cameras.single();
    for mouse_event in mouse_input.read() {
        if mouse_event.state.is_pressed() {
            debug!("Left click at {cursor_pos:?}");
            let Some(ray) = super::camera::get_cursor_ray(camera, camera_trans, cursor_pos) else {
                continue;
            };
            debug!("Cursor ray at {ray:?}");
            let max_toi = 10.0;
            let solid = true;
            let filter = QueryFilter::new();

            let Some((collider, _)) =
                rapier_context.cast_ray(ray.position, ray.direction, max_toi, solid, filter)
            else {
                continue;
            };
            debug!("Collider entity {collider:?} hit");
            match mouse_event.button {
                MouseButton::Left => block_events.send(BlockEvent::Uncover(collider)),
                MouseButton::Right => block_events.send(BlockEvent::Mark(collider)),
                _ => {}
            }
        }
    }
}

fn handle_block_events(
    mut block_events: EventReader<BlockEvent>,
    mut blocks: Query<&mut Block, &Handle<Mesh>>,
    block_meshes: Query<&Handle<Mesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for event in block_events.read() {
        match event {
            BlockEvent::Uncover(entity) => {
                debug!("Uncover block {entity:?}");
                let mut block = match blocks.get_component_mut::<Block>(*entity) {
                    Ok(block) => block,
                    Err(err) => {
                        error!("Unable to retrieve Block {entity:?}: {err}");
                        continue;
                    }
                };
                block.adjacent += 1;
                let block_mesh_handle = match block_meshes.get_component::<Handle<Mesh>>(*entity) {
                    Ok(handle) => handle,
                    Err(err) => {
                        error!("Unable to retrieve Block {entity:?} Mesh handle: {err}");
                        continue;
                    }
                };
                let Some(block_mesh) = meshes.get_mut(block_mesh_handle) else {
                    error!("Unable to retrieve Block {entity:?} Mesh");
                    continue;
                };
                update_uvs(&block, block_mesh);
            }
            BlockEvent::Mark(entity) => {
                debug!("Mark block {entity:?}");
                let mut block = match blocks.get_component_mut::<Block>(*entity) {
                    Ok(block) => block,
                    Err(err) => {
                        error!("Unable to retrieve Block {entity:?}: {err}");
                        continue;
                    }
                };
                let block_mesh_handle = match block_meshes.get_component::<Handle<Mesh>>(*entity) {
                    Ok(handle) => handle,
                    Err(err) => {
                        error!("Unable to retrieve Block {entity:?} Mesh handle: {err}");
                        continue;
                    }
                };
                let Some(block_mesh) = meshes.get_mut(block_mesh_handle) else {
                    error!("Unable to retrieve Block {entity:?} Mesh");
                    continue;
                };
                match block.marked {
                    true => {
                        debug!("Unmark block {entity:?} as mine");
                        block.marked = false;
                    }
                    false => {
                        debug!("Mark block {entity:?} as mine");
                        block.marked = true;
                    }
                }
                update_uvs(&block, block_mesh);
            }
        }
    }
}

fn update_uvs(block: &Block, block_mesh: &mut Mesh) {
    if block.marked {
        set_cube_uvs(block_mesh, UVS_FLAG);
    } else {
        if block.revealed {
            set_cube_uvs(
                block_mesh,
                match block.adjacent {
                    1 => UVS_1,
                    2 => UVS_2,
                    3 => UVS_3,
                    _ => UVS_HIDDEN,
                },
            )
        } else {
            set_cube_uvs(block_mesh, UVS_HIDDEN);
        }
    }
}

fn set_cube_uvs(cube: &mut Mesh, uvs: [[f32; 2]; 4]) {
    let mut uv_coordinates = Vec::new();
    for _ in 0..6 {
        uv_coordinates.extend(uvs)
    }
    let uvs = VertexAttributeValues::Float32x2(uv_coordinates);
    if let Some((_id, values)) = cube.attributes_mut().nth(2) {
        *values = uvs;
    }
}
