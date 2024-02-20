use bevy::prelude::*;

pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::default());
    }
}

#[derive(Debug, Resource)]
pub struct Settings {
    pub field_size: [usize; 3],
    /// Average density of mines (number of mines/number of cells)
    pub mine_density: f32,
    /// Minefield generation constraints after first click
    pub safety: FirstClickSafety,
}
impl Default for Settings {
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
