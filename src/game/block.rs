use bevy::math::bounding::{Aabb3d, Bounded3d, RayCast3d};
use bevy::prelude::*;

use super::minefield::{Contains, FieldEvent};
use super::RayEvent;
use super::{GameComponent, GameState};
use crate::Settings;

#[derive(Component)]
pub struct Block {
    /// Whether this block has been marked as a mine.
    marked: bool,
    /// Whether this block has been revealed, and thus should
    /// show its number of adjacent mines.
    revealed: Option<Contains>,
    /// Axis-aligned bounding box for this block
    bb: Aabb3d,
    /// Field index of this block
    index: [usize; 3],
}
impl Block {
    pub fn new(bb: Aabb3d, index: [usize; 3]) -> Self {
        Self {
            marked: false,
            revealed: None,
            bb,
            index,
        }
    }
    pub fn index(&self) -> [usize; 3] {
        self.index
    }
}

#[derive(Component)]
pub(super) struct BlockMeshes {
    hidden: Handle<Mesh>,
    revealed_1: Handle<Mesh>,
    revealed_2: Handle<Mesh>,
    revealed_3: Handle<Mesh>,
    revealed_4: Handle<Mesh>,
    revealed_5: Handle<Mesh>,
    mine: Handle<Mesh>,
}

#[derive(Component)]
pub(super) struct BlockMaterials {
    hidden: Handle<StandardMaterial>,
    marked: Handle<StandardMaterial>,
    revealed_1: Handle<StandardMaterial>,
    revealed_2: Handle<StandardMaterial>,
    revealed_3: Handle<StandardMaterial>,
    revealed_4: Handle<StandardMaterial>,
    revealed_5: Handle<StandardMaterial>,
    mine: Handle<StandardMaterial>,
}

fn calculate_position(index: [usize; 3], dim: [usize; 3]) -> Vec3 {
    Vec3::new(
        (index[0] as isize - dim[0] as isize / 2) as f32,
        (index[1] as isize - dim[1] as isize / 2) as f32,
        (index[2] as isize - dim[2] as isize / 2) as f32,
    )
}

/// Initialize common meshes and materials that are re-used between games
pub(super) fn initialize(
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
        revealed_1: materials.add(StandardMaterial {
            base_color: Color::BLUE,
            ..default()
        }),
        revealed_2: materials.add(StandardMaterial {
            base_color: Color::GREEN,
            ..default()
        }),
        revealed_3: materials.add(StandardMaterial {
            base_color: Color::RED,
            ..default()
        }),
        revealed_4: materials.add(StandardMaterial {
            base_color: Color::ORANGE,
            ..default()
        }),
        revealed_5: materials.add(StandardMaterial {
            base_color: Color::PURPLE,
            ..default()
        }),
        mine: materials.add(StandardMaterial {
            base_color: Color::DARK_GRAY,
            ..default()
        }),
    };

    let cube = Cuboid::new(1.0, 1.0, 1.0);

    let block_meshes = BlockMeshes {
        hidden: meshes.add(cube),
        revealed_1: meshes.add(Sphere::new(0.1)),
        revealed_2: meshes.add(Sphere::new(0.15)),
        revealed_3: meshes.add(Sphere::new(0.2)),
        revealed_4: meshes.add(Sphere::new(0.25)),
        revealed_5: meshes.add(Sphere::new(0.275)),
        mine: meshes.add(Sphere::new(0.5)),
    };

    // Keep the different possible meshes and materials for each block on a hidden entity
    commands.spawn((block_meshes, block_materials, Visibility::Hidden));
}

/// Setup to be run when the game is started
pub(super) fn setup(
    settings: Res<Settings>,
    mut commands: Commands,
    block_meshes: Query<&BlockMeshes>,
    block_materials: Query<&BlockMaterials>,
) {
    let block_meshes = block_meshes.single();
    let block_materials = block_materials.single();

    let cube = Cuboid::new(1.0, 1.0, 1.0);

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
                GameComponent,
            ))
            .id()
    };

    let field_size = settings.field_size;
    for i in 0..field_size[0] {
        for j in 0..field_size[1] {
            for k in 0..field_size[2] {
                let pos = calculate_position([i, j, k], field_size);
                add_cube([i, j, k], pos);
            }
        }
    }
}

#[derive(Debug, Event)]
pub enum BlockEvent {
    /// Uncover a block, detonating any contained mines.
    /// Received from the Minefield enitity after checking its contents.
    Clear(Entity, Contains),
    /// Mark a block (or unmark if already marked) as containing a mine.
    Mark(Entity),
}
impl BlockEvent {
    pub fn block_id(&self) -> Entity {
        match self {
            Self::Clear(e, _) | Self::Mark(e) => *e,
        }
    }
}

pub(super) fn handle_ray_events(
    mut ray_events: EventReader<RayEvent>,
    blocks: Query<(Entity, &Block)>,
    mut block_events: EventWriter<BlockEvent>,
    mut field_events: EventWriter<FieldEvent>,
) {
    for ray_event in ray_events.read() {
        match ray_event {
            RayEvent::ClearBlock(ray) => {
                if let Some((block, entity, index)) = raycast_blocks(*ray, &blocks) {
                    if !block.marked {
                        debug!("Send FieldEvent::ClearBlock");
                        field_events.send(FieldEvent::ClearBlock(entity, index));
                    }
                }
            }
            RayEvent::MarkBlock(ray) => {
                if let Some((_block, entity, _index)) = raycast_blocks(*ray, &blocks) {
                    debug!("Send BlockEvent::Mark");
                    block_events.send(BlockEvent::Mark(entity));
                }
            }
        }
    }
}

fn raycast_blocks<'a>(
    ray: Ray3d,
    blocks: &'a Query<(Entity, &Block)>,
) -> Option<(&'a Block, Entity, [usize; 3])> {
    let cast = RayCast3d::from_ray(ray, 100.0);

    let mut hits: Vec<_> = blocks
        .iter()
        .filter(|(_, block)| block.revealed.is_none())
        .filter_map(|(entity, block)| {
            cast.aabb_intersection_at(&block.bb)
                .map(|dist| (dist, entity, block))
        })
        .collect();

    // Sort hits by distance
    // Consider any unresolved comparisons to be equal (i.e. NaN == NaN)
    hits.sort_unstable_by(|(a, _, _), (b, _, _)| {
        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
    });

    let (dist, hit, block) = hits.first()?;
    let index = block.index;
    debug!("Block {hit:?} {index:?} hit at {dist}");
    Some((block, *hit, index))
}

pub(super) fn handle_block_events(
    mut commands: Commands,
    mut block_events: EventReader<BlockEvent>,
    mut blocks: Query<&mut Block>,
    block_meshes: Query<&BlockMeshes>,
    block_materials: Query<&BlockMaterials>,
    mut next_state: ResMut<NextState<GameState>>,
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
            BlockEvent::Clear(entity, contains) => {
                info!("Revealed block {entity:?}");
                block.revealed = Some(*contains);
                commands.entity(*entity).remove::<Handle<Mesh>>();
                commands
                    .entity(*entity)
                    .remove::<Handle<StandardMaterial>>();
                match contains {
                    Contains::Mine => {
                        commands
                            .entity(*entity)
                            .insert(block_meshes.mine.clone())
                            .insert(block_materials.mine.clone());
                        next_state.set(GameState::Ended);
                    }
                    Contains::Empty { adjacent_mines } => match adjacent_mines {
                        0 => {}
                        1 => {
                            commands
                                .entity(*entity)
                                .insert(block_meshes.revealed_1.clone())
                                .insert(block_materials.revealed_1.clone());
                        }
                        2 => {
                            commands
                                .entity(*entity)
                                .insert(block_meshes.revealed_2.clone())
                                .insert(block_materials.revealed_2.clone());
                        }
                        3 => {
                            commands
                                .entity(*entity)
                                .insert(block_meshes.revealed_3.clone())
                                .insert(block_materials.revealed_3.clone());
                        }
                        4 => {
                            commands
                                .entity(*entity)
                                .insert(block_meshes.revealed_4.clone())
                                .insert(block_materials.revealed_4.clone());
                        }
                        _ => {
                            commands
                                .entity(*entity)
                                .insert(block_meshes.revealed_5.clone())
                                .insert(block_materials.revealed_5.clone());
                        }
                    },
                }
            }
            BlockEvent::Mark(entity) => {
                debug!("Mark block {entity:?}");
                commands
                    .entity(*entity)
                    .remove::<Handle<StandardMaterial>>();
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
