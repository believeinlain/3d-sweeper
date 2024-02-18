use bevy::math::bounding::{Aabb3d, Bounded3d, RayCast3d};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use bevy::input::mouse::MouseButtonInput;

use crate::camera::MainCamera;

#[derive(Component)]
struct Block {
    /// Number of mines adjacent to this block.
    adjacent: u8,
    /// Whether this block has been marked as a mine.
    marked: bool,
    /// Whether this block has been revealed, and thus should
    /// show its number of adjacent mines.
    revealed: bool,
    /// Axis-aligned bounding box for this block
    bb: Aabb3d,
    /// Grid index of this block
    index: (u32, u32, u32),
}
impl Block {
    pub fn new(bb: Aabb3d, index: (u32, u32, u32)) -> Self {
        Self {
            adjacent: 0,
            marked: false,
            revealed: false,
            bb,
            index,
        }
    }
}

pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (click_on_block, handle_block_events))
            .add_event::<BlockEvent>();

        #[cfg(feature = "debug-draw")]
        app.add_systems(Update, block_gizmos);
    }
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut add_cube = |index, pos| {
        let material = materials.add(StandardMaterial::default());

        let transform = Transform::from_translation(pos);
        let cube = Cuboid::new(1.0, 1.0, 1.0);
        let bb = cube.aabb_3d(transform.translation, transform.rotation);
        let mesh = meshes.add(cube);

        commands
            .spawn((
                PbrBundle {
                    mesh,
                    material,
                    transform,
                    ..default()
                },
                Block::new(bb, index),
            ))
            .id()
    };

    let grid_size: (i32, i32, i32) = (3, 3, 3);
    for i in 0..grid_size.0 {
        for j in 0..grid_size.1 {
            for k in 0..grid_size.2 {
                let pos = Vec3::new(
                    (i - grid_size.0 / 2) as f32,
                    (j - grid_size.1 / 2) as f32,
                    (k - grid_size.2 / 2) as f32,
                );
                add_cube((i as u32, j as u32, k as u32), pos);
            }
        }
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
    primary_window: Query<&Window, With<PrimaryWindow>>,
    main_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    blocks: Query<(Entity, &Block)>,
    mut block_events: EventWriter<BlockEvent>,
) {
    let Some(cursor_pos) = primary_window.single().cursor_position() else {
        return;
    };
    let (camera, camera_trans) = main_camera.single();
    for mouse_event in mouse_input.read() {
        if mouse_event.state.is_pressed() {
            debug!("Click at {cursor_pos:?}");
            let Some(ray) = super::camera::get_cursor_ray(camera, camera_trans, cursor_pos) else {
                continue;
            };
            debug!("Cursor ray at {ray:?}");
            let cast = RayCast3d::from_ray(ray, 100.0);

            let mut hits: Vec<_> = blocks
                .iter()
                .filter(|(_, block)| !block.revealed)
                .filter_map(|(entity, block)| {
                    cast.aabb_intersection_at(&block.bb)
                        .map(|dist| (dist, entity, block))
                })
                .collect();
            // Consider any unresolved comparisons to be equal (i.e. NaN == NaN)
            hits.sort_unstable_by(|(a, _, _), (b, _, _)| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            });

            let Some((dist, hit, block)) = hits.first() else {
                continue;
            };
            let index = block.index;
            info!("Block {hit:?} {index:?} hit at {dist}");
            match mouse_event.button {
                MouseButton::Left => {
                    block_events.send(BlockEvent::Uncover(*hit));
                }
                MouseButton::Right => {
                    block_events.send(BlockEvent::Mark(*hit));
                }
                _ => {}
            };
        }
    }
}

fn handle_block_events(mut block_events: EventReader<BlockEvent>, mut blocks: Query<&mut Block>) {
    for event in block_events.read() {
        match event {
            BlockEvent::Uncover(entity) => {
                debug!("Uncover block {entity:?}");
                let mut block = match blocks.get_mut(*entity) {
                    Ok(block) => block,
                    Err(err) => {
                        error!("Unable to retrieve Block {entity:?}: {err}");
                        continue;
                    }
                };
                block.adjacent += 1;
            }
            BlockEvent::Mark(entity) => {
                debug!("Mark block {entity:?}");
                let mut block = match blocks.get_mut(*entity) {
                    Ok(block) => block,
                    Err(err) => {
                        error!("Unable to retrieve Block {entity:?}: {err}");
                        continue;
                    }
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
            }
        }
    }
}

#[cfg(feature = "debug-draw")]
fn block_gizmos(mut gizmos: Gizmos, blocks: Query<&Transform, With<Block>>) {
    for tf in blocks.iter() {
        gizmos.cuboid(*tf, Color::RED);
    }
}
