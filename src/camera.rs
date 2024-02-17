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

#[derive(Debug)]
pub struct Ray {
    pub position: Vec3,
    pub direction: Vec3,
}
impl Ray {
    fn new(position: Vec3, direction: Vec3) -> Self {
        Self {
            position,
            direction,
        }
    }
}

pub fn get_cursor_ray(
    camera: &Camera,
    camera_trans: &GlobalTransform,
    cursor_pos: Vec2,
) -> Option<Ray> {
    let view = camera_trans.compute_matrix();

    let viewport = camera.logical_viewport_rect()?;
    let screen_size = camera.logical_target_size()?;
    let adj_cursor_pos = cursor_pos - Vec2::new(viewport.min.x, screen_size.y - viewport.max.y);

    let projection = camera.projection_matrix();
    let far_ndc = projection.project_point3(Vec3::NEG_Z).z;
    let near_ndc = projection.project_point3(Vec3::Z).z;
    let cursor_ndc = (adj_cursor_pos / viewport.size()) * 2.0 - Vec2::ONE;
    let ndc_to_world: Mat4 = view * projection.inverse();
    let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
    let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
    let ray_direction = far - near;
    Some(Ray::new(near, ray_direction))
}
