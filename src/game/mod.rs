use bevy::prelude::*;

mod block;
mod camera;
mod minefield;

use block::BlockEvent;
use camera::RayEvent;
use minefield::FieldEvent;

use crate::GlobalState;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    /// Game has started, but no cell has been clicked yet.
    Start,
    /// Game transitions to this state once the first cell is clicked.
    /// This is when the field actually initializes and determines mine placement.
    /// At this point the position of all mines is known.
    Playing,
    /// Game has ended, either by clicking on a mine or by clearing all non-mines.
    #[default]
    Ended,
}

/// Marker component indicating an entity to be removed when the game is reset.
#[derive(Component)]
struct GameComponent;

/// [camera::camera_controls] consumes [crate::InputEvent] and produces [RayEvent].  
/// [block::handle_ray_events] consumes [RayEvent] and produces [FieldEvent] and/or [BlockEvent].  
/// [minefield::handle_field_events] consumes [FieldEvent] and produces [BlockEvent].  
/// [block::handle_block_events] consumes [BlockEvent] and potentially changes [GameState].  
pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let playing_game = || in_state(GlobalState::Game).and_then(not(in_state(GameState::Ended)));
        let on_game_setup = || OnEnter(GameState::Start);
        app.init_state::<GameState>();
        // Run game cleanup to remove old entities before starting a new game
        app.add_systems(OnEnter(GameState::Start), cleanup_game);
        // Add Block systems
        app.add_systems(Startup, block::initialize)
            .add_systems(on_game_setup(), block::setup.after(cleanup_game))
            .add_systems(
                Update,
                (
                    block::handle_ray_events.after(camera::camera_controls),
                    block::handle_block_events.after(minefield::handle_field_events),
                )
                    .run_if(playing_game()),
            )
            .add_event::<BlockEvent>();
        #[cfg(feature = "debug-draw")]
        app.add_systems(Update, block_gizmos.run_if(playing_game.clone()));
        // Add Minefield systems
        app.add_systems(on_game_setup(), minefield::spawn.after(cleanup_game))
            .add_systems(
                Update,
                minefield::handle_field_events
                    .after(block::handle_ray_events)
                    .run_if(playing_game()),
            )
            .add_event::<FieldEvent>();
        // Add Camera systems
        app.add_systems(on_game_setup(), camera::spawn.after(cleanup_game))
            .add_systems(
                Update,
                camera::camera_controls.run_if(in_state(GlobalState::Game)),
            )
            .add_event::<RayEvent>();
        #[cfg(feature = "debug-draw")]
        app.add_systems(Update, cursor_ray_gizmo.run_if(playing_game.clone()));
    }
}

fn cleanup_game(to_despawn: Query<Entity, With<GameComponent>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
