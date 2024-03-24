use bevy::prelude::*;

pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameSettings::default());
        app.insert_resource(FieldSettings::default());
    }
}

#[derive(Debug, Default, Resource)]
pub struct GameSettings {
    /// Minefield generation constraints after first click
    pub safety: Safety,
}

#[derive(Debug, Resource, PartialEq)]
pub struct FieldSettings {
    /// Minefield dimensions
    pub field_size: [usize; 3],
    /// Average density of mines (number of mines/number of cells)
    pub mine_density: f32,
}
impl FieldSettings {
    pub fn small() -> Self {
        Self {
            field_size: [3, 3, 3],
            mine_density: 0.2,
        }
    }
    pub fn medium() -> Self {
        Self {
            field_size: [5, 5, 5],
            mine_density: 0.1,
        }
    }
    pub fn large() -> Self {
        Self {
            field_size: [10, 10, 10],
            mine_density: 0.1,
        }
    }
    /// Split this struct into mutable fields that can be passed to UI elements
    pub fn fields_mut(&mut self) -> (&mut [usize], &mut f32) {
        (self.field_size.as_mut_slice(), &mut self.mine_density)
    }
}
impl Default for FieldSettings {
    fn default() -> Self {
        Self::medium()
    }
}

/// Define conditions imposed on the mine generation after the
/// first click.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Safety {
    /// The first click is guranteed to not be adjacent to a mine. Easiest.
    #[default]
    Clear,
    /// The first click is guaranteed to be safe, but not necessarily convenient. More difficult.
    Safe,
    /// No guarantees - the first click could lose the game.
    Random,
}
