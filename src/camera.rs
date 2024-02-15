use bevy::prelude::*;

use bevy::input::mouse::MouseWheel;

#[derive(Component)]
struct MainCamera {
    pub zoom: f32,
}

pub struct MainCameraPlugin;
impl Plugin for MainCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, zoom_with_mouse_wheel);
    }
}

fn spawn(mut commands: Commands) {
    let zoom = 10.0;
    let translation = Vec3::new(zoom, 0.0, 0.0);

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        MainCamera { zoom },
    ));
}

fn zoom_with_mouse_wheel(
    mut ev_scroll: EventReader<MouseWheel>,
    mut query: Query<(&mut MainCamera, &mut Transform)>,
) {
    let mut scroll = 0.0;
    for ev in ev_scroll.read() {
        scroll += ev.y;
    }
    if scroll.abs() > 0.0 {
        for (mut camera, mut transform) in query.iter_mut() {
            camera.zoom -= scroll * camera.zoom * 0.2;
            camera.zoom = f32::max(camera.zoom, 0.05);

            let translation = Vec3::new(camera.zoom, 0.0, 0.0);
            *transform = Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y);
        }
    }
}
