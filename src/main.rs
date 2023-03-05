use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::window::WindowPlugin;
use bevy::winit::WinitPlugin;

fn main() {
    App::new()
        .add_plugin(CorePlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(WindowPlugin {
            window: WindowDescriptor { 
                width: 640.0, 
                height: 480.0,
                title: "Sweeper 3D".to_string(),
                ..default()
            },
            ..default()
        })
        .add_plugin(WinitPlugin::default())
        .add_startup_system(setup)
        .run();
}

fn setup() {

}
