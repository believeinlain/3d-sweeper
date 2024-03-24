use bevy::audio::PlaybackMode;
use bevy::math::bounding::{Aabb3d, Bounded3d, RayCast3d};
use bevy::prelude::*;

use super::camera::RayEvent;
use super::minefield::{Contains, FieldEvent};
use super::{GamePiece, GameState};
use crate::{GameAssets, Settings};

pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        // Add Block systems
        app.add_systems(Startup, create_materials);
        app.add_systems(OnEnter(GameState::GameStart), setup.after(super::cleanup));
        app.add_systems(
            Update,
            handle_ray_events
                .after(super::camera::camera_controls)
                .run_if(GameState::playable()),
        );
        app.add_systems(
            Update,
            handle_block_events
                .after(super::minefield::handle_field_events)
                .run_if(GameState::in_game()),
        );
        app.add_event::<BlockEvent>();
        #[cfg(feature = "debug-draw")]
        app.add_systems(Update, block_gizmos.run_if(GameState::playable()));
    }
}

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

#[derive(Debug, Event)]
pub enum BlockEvent {
    /// Uncover a block, detonating any contained mines.
    /// Received from the Minefield enitity after checking its contents.
    Clear(Entity, Contains),
    /// Mark a block (or unmark if already marked) as containing a mine.
    Mark(Entity),
    /// Show the contents of a block after the game has ended.
    EndReveal(Entity, Contains),
}
impl BlockEvent {
    pub fn block_id(&self) -> Entity {
        match self {
            Self::Clear(e, _) | Self::Mark(e) | Self::EndReveal(e, _) => *e,
        }
    }
}

#[derive(Resource)]
pub(super) struct BlockMaterials {
    hidden: Handle<StandardMaterial>,
    marked: Handle<StandardMaterial>,
    blue: Handle<StandardMaterial>,
    green: Handle<StandardMaterial>,
    red: Handle<StandardMaterial>,
    orange: Handle<StandardMaterial>,
    purple: Handle<StandardMaterial>,
    mine: Handle<StandardMaterial>,
}

enum BlockDisplay {
    Hidden,
    Marked,
    Revealed { adjacent_mines: u8 },
    RevealedMine,
    MarkedMine,
    MissedMine,
}
impl BlockDisplay {
    fn spawn(
        &self,
        game_assets: &Res<GameAssets>,
        mat: &Res<BlockMaterials>,
        block: Entity,
        commands: &mut Commands,
    ) {
        let mut e = commands.get_or_spawn(block);
        let sweeper_objects = game_assets.sweeper_objects.unwrap();
        match self {
            Self::Hidden => e.insert((sweeper_objects.block_merged.clone(), mat.hidden.clone())),
            Self::Marked => e.insert(mat.marked.clone()),
            Self::Revealed { adjacent_mines } => {
                e.remove::<Handle<Mesh>>();
                e.remove::<Handle<StandardMaterial>>();
                let fives_place = adjacent_mines / 5;
                let ones_place = adjacent_mines % 5;
                if fives_place == 0 {
                    if let Some((child_mesh, child_mat)) = match adjacent_mines {
                        0 => None,
                        1 => Some((sweeper_objects.single1.clone(), mat.blue.clone())),
                        2 => Some((sweeper_objects.single2.clone(), mat.green.clone())),
                        3 => Some((sweeper_objects.single3.clone(), mat.red.clone())),
                        4 => Some((sweeper_objects.single4.clone(), mat.orange.clone())),
                        _ => panic!("if fives_place is 0, adjacent should be 0..5"),
                    } {
                        let child = e
                            .commands()
                            .spawn(PbrBundle {
                                mesh: child_mesh,
                                material: child_mat,
                                transform: Transform::from_scale(Vec3::splat(1.5)),
                                ..default()
                            })
                            .id();
                        e.add_child(child)
                    } else {
                        &mut e
                    }
                } else {
                    if let Some((orbit_mesh, orbit_mat)) = match ones_place {
                        0 => None,
                        1 => Some((sweeper_objects.orbit1.clone(), mat.blue.clone())),
                        2 => Some((sweeper_objects.orbit2.clone(), mat.green.clone())),
                        3 => Some((sweeper_objects.orbit3.clone(), mat.red.clone())),
                        4 => Some((sweeper_objects.orbit4.clone(), mat.orange.clone())),
                        _ => panic!("ones_place must be be 0..5"),
                    } {
                        let orbit = e
                            .commands()
                            .spawn(PbrBundle {
                                mesh: orbit_mesh,
                                material: orbit_mat,
                                ..default()
                            })
                            .id();
                        e.add_child(orbit);
                    }
                    e.insert((sweeper_objects.ring.clone(), mat.purple.clone()));
                    let (child_mesh, child_mat) = match fives_place {
                        1 => (sweeper_objects.single1.clone(), mat.blue.clone()),
                        2 => (sweeper_objects.single2.clone(), mat.green.clone()),
                        3 => (sweeper_objects.single3.clone(), mat.red.clone()),
                        4 => (sweeper_objects.single4.clone(), mat.orange.clone()),
                        _ => panic!(
                            "more than 24 adjacent mines is not supported (should not be possible)"
                        ),
                    };
                    let child = e
                        .commands()
                        .spawn(PbrBundle {
                            mesh: child_mesh,
                            material: child_mat,
                            transform: Transform::from_scale(Vec3::splat(3.0)),
                            ..default()
                        })
                        .id();
                    e.add_child(child)
                }
            }
            Self::RevealedMine => e.insert((
                game_assets.sweeper_objects.unwrap().mine_merged.clone(),
                mat.mine.clone(),
            )),
            Self::MarkedMine => e.insert((
                game_assets.sweeper_objects.unwrap().mine_merged.clone(),
                mat.green.clone(),
            )),
            Self::MissedMine => e.insert((
                game_assets.sweeper_objects.unwrap().mine_merged.clone(),
                mat.red.clone(),
            )),
        };
    }
}

fn calculate_position(index: [usize; 3], dim: [usize; 3]) -> Vec3 {
    Vec3::new(
        (index[0] as isize - dim[0] as isize / 2) as f32,
        (index[1] as isize - dim[1] as isize / 2) as f32,
        (index[2] as isize - dim[2] as isize / 2) as f32,
    )
}

/// Initialize materials that are re-used between games
pub(super) fn create_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(BlockMaterials {
        hidden: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("concrete_02_albedo.png")),
            metallic_roughness_texture: Some(asset_server.load("concrete_02_orm.png")),
            perceptual_roughness: 1.0,
            metallic: 0.0,
            normal_map_texture: Some(asset_server.load("concrete_02_normal.png")),
            ..default()
        }),
        marked: materials.add(Color::RED),
        blue: materials.add(Color::BLUE),
        green: materials.add(Color::GREEN),
        red: materials.add(Color::RED),
        orange: materials.add(Color::ORANGE),
        purple: materials.add(Color::PURPLE),
        mine: materials.add(Color::DARK_GRAY),
    })
}

/// Setup to be run when the game is started
pub(super) fn setup(
    settings: Res<Settings>,
    mut commands: Commands,
    block_mat: Res<BlockMaterials>,
    game_assets: Res<GameAssets>,
    mut field_events: EventWriter<FieldEvent>,
) {
    let mut add_cube = |index, pos| {
        let transform = Transform::from_translation(pos);
        let bb = Cuboid::new(1.0, 1.0, 1.0).aabb_3d(transform.translation, transform.rotation);
        let block = commands
            .spawn((
                PbrBundle {
                    transform,
                    ..default()
                },
                Block::new(bb, index),
                GamePiece,
            ))
            .id();
        BlockDisplay::Hidden.spawn(&game_assets, &block_mat, block, &mut commands);
        debug!("Send FieldEvent::SpawnBlock");
        field_events.send(FieldEvent::SpawnBlock(block, index));
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

pub(super) fn handle_ray_events(
    mut ray_events: EventReader<RayEvent>,
    blocks: Query<(Entity, &Block)>,
    mut block_events: EventWriter<BlockEvent>,
    mut field_events: EventWriter<FieldEvent>,
) {
    for ray_event in ray_events.read() {
        match ray_event {
            RayEvent::ClearBlock(ray) => {
                if let Some((block, _entity, index)) = raycast_blocks(*ray, &blocks) {
                    if !block.marked {
                        debug!("Send FieldEvent::ClearBlock");
                        field_events.send(FieldEvent::ClearBlock(index));
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
    block_mat: Res<BlockMaterials>,
    game_assets: Res<GameAssets>,
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
                match *contains {
                    Contains::Mine => {
                        BlockDisplay::RevealedMine.spawn(
                            &game_assets,
                            &block_mat,
                            *entity,
                            &mut commands,
                        );
                        next_state.set(GameState::GameOver);
                    }
                    Contains::Empty { adjacent_mines } => BlockDisplay::Revealed { adjacent_mines }
                        .spawn(&game_assets, &block_mat, *entity, &mut commands),
                }
            }
            BlockEvent::EndReveal(entity, contains) => {
                debug!("Revealed block {entity:?} at end of game");
                match *contains {
                    Contains::Mine if block.revealed.is_some() => {
                        BlockDisplay::MissedMine.spawn(
                            &game_assets,
                            &block_mat,
                            *entity,
                            &mut commands,
                        );
                    }
                    Contains::Mine if block.marked => {
                        BlockDisplay::MarkedMine.spawn(
                            &game_assets,
                            &block_mat,
                            *entity,
                            &mut commands,
                        );
                    }
                    Contains::Mine => {
                        BlockDisplay::RevealedMine.spawn(
                            &game_assets,
                            &block_mat,
                            *entity,
                            &mut commands,
                        );
                    }
                    Contains::Empty { adjacent_mines } => BlockDisplay::Revealed { adjacent_mines }
                        .spawn(&game_assets, &block_mat, *entity, &mut commands),
                }
                block.revealed = Some(*contains);
            }
            BlockEvent::Mark(entity) => match block.marked {
                true => {
                    debug!("Unmark block {entity:?}");
                    block.marked = false;
                    BlockDisplay::Hidden.spawn(&game_assets, &block_mat, *entity, &mut commands);
                }
                false => {
                    debug!("Mark block {entity:?}");
                    block.marked = true;
                    BlockDisplay::Marked.spawn(&game_assets, &block_mat, *entity, &mut commands);
                }
            },
        }
    }
    if any_blocks_cleared {
        commands.spawn(AudioBundle {
            source: game_assets.pop2.clone(),
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
