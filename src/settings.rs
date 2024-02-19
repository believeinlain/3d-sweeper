use bevy::prelude::*;

#[derive(Debug, Resource)]
pub struct GameSettings {
    pub field_size: [usize; 3],
    /// Average density of mines (number of mines/number of cells)
    pub mine_density: f32,
    /// Minefield generation constraints after first click
    pub safety: FirstClickSafety,
}
impl Default for GameSettings {
    fn default() -> Self {
        Self {
            field_size: [5, 5, 5],
            mine_density: 0.1,
            safety: FirstClickSafety::default(),
        }
    }
}

/// Define conditions imposed on the mine generation after the
/// first click.
#[derive(Debug, Default)]
pub enum FirstClickSafety {
    /// The first click is guaranteed to be safe, but not necessarily convenient.
    #[default]
    GuaranteedEmpty,
    /// The first click is guranteed to not be adjacent to a mine.
    GuaranteedNoAdjacent,
    /// No guarantees - the first click could lose the game.
    Random,
}

pub struct GameSettingsPlugin;
impl Plugin for GameSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameSettings::default());
    }
}
