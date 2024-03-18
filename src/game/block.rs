use bevy::audio::PlaybackMode;
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

#[derive(Resource)]
pub(super) struct BlockAssets {
    hidden_mesh: Handle<Mesh>,
    reveal_sound: Handle<AudioSource>,
    revealed_1_mesh: Handle<Mesh>,
    revealed_2_mesh: Handle<Mesh>,
    revealed_3_mesh: Handle<Mesh>,
    revealed_4_mesh: Handle<Mesh>,
    revealed_5_mesh: Handle<Mesh>,
    mine_mesh: Handle<Mesh>,
    hidden_mat: Handle<StandardMaterial>,
    marked_mat: Handle<StandardMaterial>,
    revealed_1_mat: Handle<StandardMaterial>,
    revealed_2_mat: Handle<StandardMaterial>,
    revealed_3_mat: Handle<StandardMaterial>,
    revealed_4_mat: Handle<StandardMaterial>,
    revealed_5_mat: Handle<StandardMaterial>,
    mine_mat: Handle<StandardMaterial>,
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
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(BlockAssets {
        hidden_mesh: asset_server.load("block_01.gltf#Mesh0/Primitive0"),
        reveal_sound: asset_server.load("pop2.ogg"),
        revealed_1_mesh: meshes.add(Sphere::new(0.1)),
        revealed_2_mesh: meshes.add(Sphere::new(0.15)),
        revealed_3_mesh: meshes.add(Sphere::new(0.2)),
        revealed_4_mesh: meshes.add(Sphere::new(0.25)),
        revealed_5_mesh: meshes.add(Sphere::new(0.275)),
        mine_mesh: meshes.add(Sphere::new(0.5)),
        hidden_mat: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("concrete_02_albedo.png")),
            metallic_roughness_texture: Some(asset_server.load("concrete_02_orm.png")),
            perceptual_roughness: 1.0,
            metallic: 0.0,
            normal_map_texture: Some(asset_server.load("concrete_02_normal.png")),
            ..default()
        }),
        marked_mat: materials.add(StandardMaterial {
            base_color: Color::RED,
            ..default()
        }),
        revealed_1_mat: materials.add(StandardMaterial {
            base_color: Color::BLUE,
            ..default()
        }),
        revealed_2_mat: materials.add(StandardMaterial {
            base_color: Color::GREEN,
            ..default()
        }),
        revealed_3_mat: materials.add(StandardMaterial {
            base_color: Color::RED,
            ..default()
        }),
        revealed_4_mat: materials.add(StandardMaterial {
            base_color: Color::ORANGE,
            ..default()
        }),
        revealed_5_mat: materials.add(StandardMaterial {
            base_color: Color::PURPLE,
            ..default()
        }),
        mine_mat: materials.add(StandardMaterial {
            base_color: Color::DARK_GRAY,
            ..default()
        }),
    })
}

/// Setup to be run when the game is started
pub(super) fn setup(
    settings: Res<Settings>,
    mut commands: Commands,
    block_assets: Res<BlockAssets>,
) {
    let cube = Cuboid::new(1.0, 1.0, 1.0);

    let mut add_cube = |index, pos| {
        let transform = Transform::from_translation(pos);
        let bb = cube.aabb_3d(transform.translation, transform.rotation);
        commands
            .spawn((
                PbrBundle {
                    mesh: block_assets.hidden_mesh.clone(),
                    material: block_assets.hidden_mat.clone(),
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
    block_assets: Res<BlockAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut any_blocks_cleared = false;
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
                debug!("Revealed block {entity:?}");
                block.revealed = Some(*contains);
                any_blocks_cleared = true;
                commands
                    .entity(*entity)
                    .remove::<Handle<Mesh>>()
                    .remove::<Handle<StandardMaterial>>();
                match contains {
                    Contains::Mine => {
                        commands
                            .entity(*entity)
                            .insert(block_assets.mine_mesh.clone())
                            .insert(block_assets.mine_mat.clone());
                        next_state.set(GameState::Ended);
                    }
                    Contains::Empty { adjacent_mines } => match adjacent_mines {
                        0 => {}
                        1 => {
                            commands
                                .entity(*entity)
                                .insert(block_assets.revealed_1_mesh.clone())
                                .insert(block_assets.revealed_1_mat.clone());
                        }
                        2 => {
                            commands
                                .entity(*entity)
                                .insert(block_assets.revealed_2_mesh.clone())
                                .insert(block_assets.revealed_2_mat.clone());
                        }
                        3 => {
                            commands
                                .entity(*entity)
                                .insert(block_assets.revealed_3_mesh.clone())
                                .insert(block_assets.revealed_3_mat.clone());
                        }
                        4 => {
                            commands
                                .entity(*entity)
                                .insert(block_assets.revealed_4_mesh.clone())
                                .insert(block_assets.revealed_4_mat.clone());
                        }
                        _ => {
                            commands
                                .entity(*entity)
                                .insert(block_assets.revealed_5_mesh.clone())
                                .insert(block_assets.revealed_5_mat.clone());
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
                            .insert(block_assets.hidden_mat.clone());
                    }
                    false => {
                        debug!("Mark block {entity:?} as mine");
                        block.marked = true;
                        commands
                            .entity(*entity)
                            .insert(block_assets.marked_mat.clone());
                    }
                }
            }
        }
    }
    if any_blocks_cleared {
        commands.spawn(AudioBundle {
            source: block_assets.reveal_sound.clone(),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        });
    }
}

#[cfg(feature = "debug-draw")]
fn block_gizmos(mut gizmos: Gizmos, blocks: Query<&Transform, With<Block>>) {
    for tf in blocks.iter() {
        gizmos.cuboid(*tf, Color::RED);
    }
}
