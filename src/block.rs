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

#[derive(Component)]
struct BlockMeshes {
    hidden: Handle<Mesh>,
    revealed: Handle<Mesh>,
}

#[derive(Component)]
struct BlockMaterials {
    hidden: Handle<StandardMaterial>,
    marked: Handle<StandardMaterial>,
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let block_materials = BlockMaterials {
        hidden: materials.add(StandardMaterial::default()),
        marked: materials.add(StandardMaterial {
            base_color: Color::RED,
            ..default()
        }),
    };

    let cube = Cuboid::new(1.0, 1.0, 1.0);

    let block_meshes = BlockMeshes {
        hidden: meshes.add(cube),
        revealed: meshes.add(Sphere::new(0.25)),
    };

    let mut add_cube = |index, pos| {
        let transform = Transform::from_translation(pos);
        let bb = cube.aabb_3d(transform.translation, transform.rotation);
        commands
            .spawn((
                PbrBundle {
                    mesh: block_meshes.hidden.clone(),
                    material: block_materials.hidden.clone(),
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

    // Keep the different possible meshes and materials for each block on a hidden entity
    commands.spawn((block_meshes, block_materials, Visibility::Hidden));
}

#[derive(Event)]
pub enum BlockEvent {
    /// Uncover a block, detonating any contained mines.
    Reveal(Entity),
    /// Mark a block (or unmark if already marked) as containing a mine.
    Mark(Entity),
}
impl BlockEvent {
    pub fn block_id(&self) -> Entity {
        match self {
            Self::Reveal(e) | Self::Mark(e) => *e,
        }
    }
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
            debug!("Block {hit:?} {index:?} hit at {dist}");
            match mouse_event.button {
                MouseButton::Left => {
                    block_events.send(BlockEvent::Reveal(*hit));
                }
                MouseButton::Right => {
                    block_events.send(BlockEvent::Mark(*hit));
                }
                _ => {}
            };
        }
    }
}

fn handle_block_events(
    mut commands: Commands,
    mut block_events: EventReader<BlockEvent>,
    mut blocks: Query<&mut Block>,
    block_meshes: Query<&BlockMeshes>,
    block_materials: Query<&BlockMaterials>,
) {
    let block_meshes = block_meshes.single();
    let block_materials = block_materials.single();
    for event in block_events.read() {
        let id = event.block_id();
        let mut block = match blocks.get_mut(id) {
            Ok(block) => block,
            Err(err) => {
                error!("Unable to retrieve Block {id:?}: {err}");
                continue;
            }
        };
        match event {
            BlockEvent::Reveal(entity) => {
                debug!("Revealed block {entity:?}");
                block.revealed = true;
                commands.entity(*entity).remove::<Handle<Mesh>>();
                match block.adjacent {
                    0 => {}
                    _ => {
                        commands
                            .entity(*entity)
                            .insert(block_meshes.revealed.clone());
                    }
                }
            }
            BlockEvent::Mark(entity) => {
                debug!("Mark block {entity:?}");
                commands.entity(*entity).remove::<Handle<StandardMaterial>>();
                match block.marked {
                    true => {
                        debug!("Unmark block {entity:?} as mine");
                        block.marked = false;
                        commands
                            .entity(*entity)
                            .insert(block_materials.hidden.clone());
                    }
                    false => {
                        debug!("Mark block {entity:?} as mine");
                        block.marked = true;
                        commands
                            .entity(*entity)
                            .insert(block_materials.marked.clone());
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
