use bevy::prelude::*;
use ndarray::prelude::*;
use rand::prelude::*;

use super::{
    block::{Block, BlockEvent},
    GamePiece, GameState,
};
use crate::Settings;

pub struct FieldPlugin;
impl Plugin for FieldPlugin {
    fn build(&self, app: &mut App) {
        // Add Minefield systems
        app.add_systems(OnEnter(GameState::GameStart), spawn.after(super::cleanup));
        app.add_systems(
            Update,
            handle_field_events
                .after(super::block::handle_ray_events)
                .run_if(GameState::playable()),
        );
        app.add_systems(OnEnter(GameState::GameOver), reveal_all);
        app.add_event::<FieldEvent>();
    }
}

#[derive(Event)]
pub enum FieldEvent {
    SpawnBlock(Entity, [usize; 3]),
    ClearBlock([usize; 3]),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Cell {
    contains: Contains,
    revealed: bool,
    block: Option<Entity>,
}

#[derive(Debug, Clone, Copy)]
pub enum Contains {
    Mine,
    Empty { adjacent_mines: u8 },
}
impl Default for Contains {
    fn default() -> Self {
        Self::Empty { adjacent_mines: 0 }
    }
}

#[derive(Component)]
pub struct Minefield {
    cells: Array3<Cell>,
    density: f64,
}
impl Minefield {
    /// Initialize the [Minefield], placing mines randomly according to [Minefield::density].
    fn initialize(&mut self, blocks: &Query<(Entity, &Block)>) {
        debug!("Creating minefield");
        let mut rng = rand::thread_rng();
        // Save Block ids
        for (entity, block) in blocks {
            self.cells[block.index()].block = Some(entity)
        }
        // Place mines
        for (index, cell) in self.cells.indexed_iter_mut() {
            if cell.revealed {
                // TODO: implement FirstClickSafety setting
                debug!("Cell at {index:?} already revealed, will not place mine there");
            } else if rng.gen_bool(self.density) {
                cell.contains = Contains::Mine;
            }
        }
        // Determine adjacent value in each cell
        let mines: Vec<_> = self
            .cells
            .indexed_iter()
            .filter_map(|(i, c)| match c.contains {
                Contains::Mine => Some(i),
                _ => None,
            })
            .collect();
        for index in mines {
            debug!("Placed mine at {index:?}");
            let (i, j, k) = index;
            let mut increment_adjacent = |i_off, j_off, k_off| {
                let adj_index = (
                    i.wrapping_add_signed(i_off),
                    j.wrapping_add_signed(j_off),
                    k.wrapping_add_signed(k_off),
                );
                if let Some(adj) = self.cells.get_mut(adj_index) {
                    if let Contains::Empty {
                        ref mut adjacent_mines,
                    } = adj.contains
                    {
                        debug!("Increment adjacent at {adj_index:?}");
                        *adjacent_mines += 1;
                    }
                }
            };
            for i_off in -1..=1 {
                for j_off in -1..=1 {
                    for k_off in -1..=1 {
                        // The block at index is not adjacent to itself
                        if i_off == 0 && j_off == 0 && k_off == 0 {
                            continue;
                        }
                        increment_adjacent(i_off, j_off, k_off);
                    }
                }
            }
        }
    }
    fn reveal_adjacent(
        &mut self,
        index: (usize, usize, usize),
        block_events: &mut EventWriter<BlockEvent>,
    ) {
        let (i, j, k) = index;
        for i_off in -1..=1 {
            for j_off in -1..=1 {
                for k_off in -1..=1 {
                    // The block at index is not adjacent to itself
                    if i_off == 0 && j_off == 0 && k_off == 0 {
                        continue;
                    }
                    // Get a block adjacent to index
                    let adj_index = (
                        i.wrapping_add_signed(i_off),
                        j.wrapping_add_signed(j_off),
                        k.wrapping_add_signed(k_off),
                    );
                    // Make sure we have a valid adj_index
                    let Some(adj) = self.cells.get_mut(adj_index) else {
                        continue;
                    };
                    // If the adjacent block is already revealed, don't bother
                    // TODO: maybe not good idea?
                    if adj.revealed {
                        continue;
                    }
                    let contains = adj.contains;
                    // Don't reveal mines
                    let Contains::Empty { adjacent_mines } = contains else {
                        continue;
                    };
                    // Get the entity to send with the message
                    let Some(adj_id) = adj.block else {
                        continue;
                    };
                    adj.revealed = true;
                    // Send a message to reveal this block
                    let event = BlockEvent::Clear(adj_id, contains);
                    debug!("Send {event:?}");
                    block_events.send(event);
                    // Recurse only if this block was not adjacent to any mines
                    if adjacent_mines == 0 {
                        self.reveal_adjacent(adj_index, block_events);
                    }
                }
            }
        }
    }
    /// Return true iff the Minefield has been fully revealed (victory condition)
    fn fully_revealed(&self) -> bool {
        for cell in &self.cells {
            if !cell.revealed && !matches!(cell.contains, Contains::Mine) {
                return false;
            }
        }
        true
    }
}

fn spawn(settings: Res<Settings>, mut commands: Commands) {
    let field = Minefield {
        cells: Array3::default(settings.field_size),
        density: settings.mine_density.into(),
    };
    commands.spawn((field, GamePiece));
}

pub(super) fn handle_field_events(
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    blocks: Query<(Entity, &Block)>,
    mut field: Query<&mut Minefield>,
    mut field_events: EventReader<FieldEvent>,
    mut block_events: EventWriter<BlockEvent>,
) {
    for event in field_events.read() {
        match event {
            FieldEvent::SpawnBlock(entity, index) => {
                let mut field = field.single_mut();
                let Some(cell) = field.cells.get_mut(*index) else {
                    continue;
                };
                cell.block = Some(*entity);
            }
            FieldEvent::ClearBlock(index) => {
                let mut field = field.single_mut();
                let Some(cell) = field.cells.get_mut(*index) else {
                    continue;
                };
                cell.revealed = true;
                if matches!(game_state.get(), GameState::GameStart) {
                    debug!("Transition to GameState::Playing");
                    next_state.set(GameState::GamePlaying);
                    field.initialize(&blocks);
                }
                // Get the updated field
                let Some(cell) = field.cells.get_mut(*index) else {
                    continue;
                };
                let contains = cell.contains;
                let event = BlockEvent::Clear(cell.block.unwrap(), contains);
                debug!("Send {event:?}");
                block_events.send(event);
                if matches!(contains, Contains::Empty { adjacent_mines } if adjacent_mines == 0) {
                    field.reveal_adjacent((index[0], index[1], index[2]), &mut block_events);
                }
                if field.fully_revealed() {
                    info!("Victory!");
                    debug!("Transition to GameState::Ended");
                    next_state.set(GameState::GameOver);
                }
            }
        }
    }
}

fn reveal_all(mut field: Query<&mut Minefield>, mut block_events: EventWriter<BlockEvent>) {
    for cell in field.single_mut().cells.iter_mut() {
        cell.revealed = true;
        block_events.send(BlockEvent::EndReveal(cell.block.unwrap(), cell.contains));
    }
}
