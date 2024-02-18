use std::f32::consts::PI;

use bevy::prelude::*;

use bevy::input::mouse::{MouseMotion, MouseWheel};

#[derive(Component)]
pub struct MainCamera {
    zoom_speed: f32,
    zoom_limit_near: f32,
    zoom_limit_far: f32,
}
impl Default for MainCamera {
    fn default() -> Self {
        Self {
            zoom_speed: 1.0,
            zoom_limit_near: 1.0,
            zoom_limit_far: 15.0,
        }
    }
}

pub struct MainCameraPlugin;
impl Plugin for MainCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (rotate_with_mouse, zoom_with_mouse_wheel));
        #[cfg(feature = "debug-draw")]
        app.add_systems(Update, cursor_ray_gizmo);
    }
}

fn spawn(mut commands: Commands) {
    let zoom = 10.0;
    let translation = Vec3::ONE.normalize() * zoom;

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        MainCamera::default(),
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
        for (camera, mut transform) in query.iter_mut() {
            let zoom = transform.translation * scroll * camera.zoom_speed * -0.1;
            let new_translation = transform.translation + zoom;
            let zoom_dist = new_translation.distance(Vec3::ZERO);
            if zoom_dist > camera.zoom_limit_near && zoom_dist < camera.zoom_limit_far {
                *transform = transform.with_translation(new_translation);
            }
        }
    }
}

pub fn get_cursor_ray(
    camera: &Camera,
    camera_trans: &GlobalTransform,
    cursor_pos: Vec2,
) -> Option<Ray3d> {
    let view = camera_trans.compute_matrix();

    let viewport = camera.logical_viewport_rect()?;
    let screen_size = camera.logical_target_size()?;
    let invert_cursor_pos = Vec2::new(cursor_pos.x, screen_size.y - cursor_pos.y);
    let adj_cursor_pos =
        invert_cursor_pos - Vec2::new(viewport.min.x, screen_size.y - viewport.max.y);

    let projection = camera.projection_matrix();
    let far_ndc = projection.project_point3(Vec3::NEG_Z).z;
    let near_ndc = projection.project_point3(Vec3::Z).z;
    let cursor_ndc = (adj_cursor_pos / viewport.size()) * 2.0 - Vec2::ONE;
    let ndc_to_world: Mat4 = view * projection.inverse();
    let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
    let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
    let ray_direction = far - near;
    Some(Ray3d::new(near, ray_direction))
}

fn rotate_with_mouse(
    windows: Query<&Window>,
    mut motion_events: EventReader<MouseMotion>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    let rotate_binding = MouseButton::Middle;

    if input_mouse.pressed(rotate_binding) {
        let mut rotation_delta = Vec2::ZERO;
        for ev in motion_events.read() {
            rotation_delta += ev.delta;
        }
        if rotation_delta.length_squared() > 0.0 {
            let window = windows.single();
            let delta_x = rotation_delta.x / window.width() * PI * 2.0;
            let delta_y = rotation_delta.y / window.height() * PI;
            for mut transform in query.iter_mut() {
                // Rotate around local X axis and global Y axis
                let y_rot = Quat::from_axis_angle(Vec3::Y, -delta_x);
                let x_rot = Quat::from_axis_angle(*transform.local_x(), -delta_y);
                transform.rotate_around(Vec3::ZERO, x_rot);
                transform.rotate_around(Vec3::ZERO, y_rot);
            }
        }
    } else {
        motion_events.clear();
    }
}

#[cfg(feature = "debug-draw")]
fn cursor_ray_gizmo(
    mut gizmos: Gizmos,
    main_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    primary_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let Some(cursor_pos) = primary_window.single().cursor_position() else {
        return;
    };
    let (camera, camera_trans) = main_camera.single();
    let Some(ray) = get_cursor_ray(camera, camera_trans, cursor_pos) else {
        return;
    };
    gizmos.arrow(ray.origin, ray.get_point(20.0), Color::YELLOW);
}
