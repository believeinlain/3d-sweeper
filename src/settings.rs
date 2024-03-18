use bevy::prelude::*;

pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::default());
    }
}

#[derive(Debug, Resource, PartialEq)]
pub struct Settings {
    pub field_size: [usize; 3],
    /// Average density of mines (number of mines/number of cells)
    pub mine_density: f32,
    /// Minefield generation constraints after first click
    pub safety: Safety,
}
impl Settings {
    pub fn small() -> Self {
        Self {
            field_size: [3, 3, 3],
            ..default()
        }
    }
    pub fn medium() -> Self {
        Self {
            field_size: [5, 5, 5],
            ..default()
        }
    }
    pub fn large() -> Self {
        Self {
            field_size: [10, 10, 10],
            ..default()
        }
    }
    /// Split this struct into mutable fields that can be passed to UI elements
    pub fn fields_mut(&mut self) -> (&mut [usize], &mut f32, &mut Safety) {
        (
            self.field_size.as_mut_slice(),
            &mut self.mine_density,
            &mut self.safety,
        )
    }
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            field_size: [5, 5, 5],
            mine_density: 0.1,
            safety: Safety::default(),
        }
    }
}

/// Define conditions imposed on the mine generation after the
/// first click.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum Safety {
    /// The first click is guaranteed to be safe, but not necessarily convenient.
    #[default]
    Safe,
    /// The first click is guranteed to not be adjacent to a mine.
    Clear,
    /// No guarantees - the first click could lose the game.
    Random,
}
