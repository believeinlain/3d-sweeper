// Disable console window in Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{log::LogPlugin, prelude::*, window::WindowResolution};

const SMALL_SCALE: Vec3 = Vec3::splat(0.5);
const LARGE_SCALE: Vec3 = Vec3::splat(1.0);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                // Window settings
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        visible: false,
                        resolution: WindowResolution::new(1024.0, 768.0),
                        title: "3D Sweeper".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                // Log settings
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..default()
                })
                // Texture settings
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, rotate)
        .run();
}

fn setup(
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    window.single_mut().visible = true;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1200.0,
            shadows_enabled: true,
            color: Color::rgb(1.0, 0.95, 0.90),
            ..default()
        },
        transform: Transform::from_xyz(-1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.insert_resource(AmbientLight {
        brightness: 100.0,
        color: Color::rgb(0.95, 0.95, 1.0),
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(1.0, 3.5, 8.0)
            .looking_at(Vec3::new(1.0, -2.0, 0.0), Vec3::Y),
        ..default()
    });

    let spawn_block =
        |transform, commands: &mut Commands, materials: &mut ResMut<Assets<StandardMaterial>>| {
            commands.spawn(PbrBundle {
                mesh: asset_server.load("sweeper_objects.gltf#Mesh6/Primitive0"),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(asset_server.load("concrete_02_albedo.png")),
                    metallic_roughness_texture: Some(asset_server.load("concrete_02_orm.png")),
                    perceptual_roughness: 1.0,
                    metallic: 0.0,
                    normal_map_texture: Some(asset_server.load("concrete_02_normal.png")),
                    ..default()
                }),
                transform,
                ..default()
            });
        };

    let spawn_mine =
        |transform, commands: &mut Commands, materials: &mut ResMut<Assets<StandardMaterial>>| {
            commands.spawn(PbrBundle {
                mesh: asset_server.load("sweeper_objects.gltf#Mesh4/Primitive0"),
                material: materials.add(StandardMaterial {
                    base_color: Color::DARK_GRAY,
                    ..default()
                }),
                transform,
                ..default()
            });
        };

    let spawn_1 = |transform: Transform,
                   commands: &mut Commands,
                   materials: &mut ResMut<Assets<StandardMaterial>>| {
        commands.spawn(PbrBundle {
            mesh: asset_server.load("sweeper_objects.gltf#Mesh3/Primitive0"),
            material: materials.add(StandardMaterial {
                base_color: Color::BLUE,
                ..default()
            }),
            transform,
            ..default()
        });
    };

    let spawn_2 = |transform: Transform,
                   commands: &mut Commands,
                   materials: &mut ResMut<Assets<StandardMaterial>>| {
        commands
            .spawn(PbrBundle {
                mesh: asset_server.load("sweeper_objects.gltf#Mesh2/Primitive0"),
                material: materials.add(StandardMaterial {
                    base_color: Color::GREEN,
                    ..default()
                }),
                transform,
                ..default()
            })
            .insert(Rotate(-0.01));
    };

    let spawn_3 = |transform: Transform,
                   commands: &mut Commands,
                   materials: &mut ResMut<Assets<StandardMaterial>>| {
        commands
            .spawn(PbrBundle {
                mesh: asset_server.load("sweeper_objects.gltf#Mesh0/Primitive0"),
                material: materials.add(StandardMaterial {
                    base_color: Color::RED,
                    ..default()
                }),
                transform,
                ..default()
            })
            .insert(Rotate(-0.01));
    };

    let spawn_4 = |transform: Transform,
                   commands: &mut Commands,
                   materials: &mut ResMut<Assets<StandardMaterial>>| {
        commands
            .spawn(PbrBundle {
                mesh: asset_server.load("sweeper_objects.gltf#Mesh1/Primitive0"),
                material: materials.add(StandardMaterial {
                    base_color: Color::ORANGE,
                    ..default()
                }),
                transform,
                ..default()
            })
            .insert(Rotate(-0.01));
    };

    let spawn_ring = |transform: Transform,
                      commands: &mut Commands,
                      materials: &mut ResMut<Assets<StandardMaterial>>| {
        commands.spawn(PbrBundle {
            mesh: asset_server.load("sweeper_objects.gltf#Mesh5/Primitive0"),
            material: materials.add(StandardMaterial {
                base_color: Color::PURPLE,
                ..default()
            }),
            transform,
            ..default()
        });
    };

    let spawn_5 = |transform: Transform,
                   commands: &mut Commands,
                   materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_1(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
    };

    let spawn_orbit1 = |transform: Transform,
                        commands: &mut Commands,
                        materials: &mut ResMut<Assets<StandardMaterial>>| {
        commands
            .spawn(PbrBundle {
                mesh: asset_server.load("sweeper_objects.gltf#Mesh7/Primitive0"),
                material: materials.add(StandardMaterial {
                    base_color: Color::BLUE,
                    ..default()
                }),
                transform,
                ..default()
            })
            .insert(Rotate(0.03));
    };

    let spawn_orbit2 = |transform: Transform,
                        commands: &mut Commands,
                        materials: &mut ResMut<Assets<StandardMaterial>>| {
        commands
            .spawn(PbrBundle {
                mesh: asset_server.load("sweeper_objects.gltf#Mesh8/Primitive0"),
                material: materials.add(StandardMaterial {
                    base_color: Color::GREEN,
                    ..default()
                }),
                transform,
                ..default()
            })
            .insert(Rotate(0.03));
    };

    let spawn_orbit3 = |transform: Transform,
                        commands: &mut Commands,
                        materials: &mut ResMut<Assets<StandardMaterial>>| {
        commands
            .spawn(PbrBundle {
                mesh: asset_server.load("sweeper_objects.gltf#Mesh9/Primitive0"),
                material: materials.add(StandardMaterial {
                    base_color: Color::RED,
                    ..default()
                }),
                transform,
                ..default()
            })
            .insert(Rotate(0.03));
    };

    let spawn_orbit4 = |transform: Transform,
                        commands: &mut Commands,
                        materials: &mut ResMut<Assets<StandardMaterial>>| {
        commands
            .spawn(PbrBundle {
                mesh: asset_server.load("sweeper_objects.gltf#Mesh10/Primitive0"),
                material: materials.add(StandardMaterial {
                    base_color: Color::ORANGE,
                    ..default()
                }),
                transform,
                ..default()
            })
            .insert(Rotate(0.03));
    };
    let spawn_6 = |transform: Transform,
                   commands: &mut Commands,
                   materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_1(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit1(transform, commands, materials);
    };
    let spawn_7 = |transform: Transform,
                   commands: &mut Commands,
                   materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_1(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit2(transform, commands, materials);
    };
    let spawn_8 = |transform: Transform,
                   commands: &mut Commands,
                   materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_1(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit3(transform, commands, materials);
    };
    let spawn_9 = |transform: Transform,
                   commands: &mut Commands,
                   materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_1(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit4(transform, commands, materials);
    };

    let spawn_10 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_2(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
    };
    let spawn_11 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_2(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit1(transform, commands, materials);
    };
    let spawn_12 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_2(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit2(transform, commands, materials);
    };
    let spawn_13 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_2(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit3(transform, commands, materials);
    };
    let spawn_14 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_2(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit4(transform, commands, materials);
    };

    let spawn_15 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_3(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
    };
    let spawn_16 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_3(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit1(transform, commands, materials);
    };
    let spawn_17 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_3(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit2(transform, commands, materials);
    };
    let spawn_18 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_3(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit3(transform, commands, materials);
    };
    let spawn_19 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_3(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit4(transform, commands, materials);
    };

    let spawn_20 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_4(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
    };
    let spawn_21 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_4(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit1(transform, commands, materials);
    };
    let spawn_22 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_4(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit2(transform, commands, materials);
    };
    let spawn_23 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_4(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit3(transform, commands, materials);
    };
    let spawn_24 = |transform: Transform,
                    commands: &mut Commands,
                    materials: &mut ResMut<Assets<StandardMaterial>>| {
        spawn_4(transform.with_scale(LARGE_SCALE), commands, materials);
        spawn_ring(transform, commands, materials);
        spawn_orbit4(transform, commands, materials);
    };

    spawn_block(
        Transform::from_xyz(-3.0, 0.0, 0.0),
        &mut commands,
        &mut materials,
    );
    spawn_mine(
        Transform::from_xyz(-2.0, 0.0, 0.0),
        &mut commands,
        &mut materials,
    );
    spawn_1(
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(SMALL_SCALE),
        &mut commands,
        &mut materials,
    );
    spawn_2(
        Transform::from_xyz(1.0, 0.0, 0.0).with_scale(SMALL_SCALE),
        &mut commands,
        &mut materials,
    );
    spawn_3(
        Transform::from_xyz(2.0, 0.0, 0.0).with_scale(SMALL_SCALE),
        &mut commands,
        &mut materials,
    );
    spawn_4(
        Transform::from_xyz(3.0, 0.0, 0.0).with_scale(SMALL_SCALE),
        &mut commands,
        &mut materials,
    );
    spawn_5(
        Transform::from_xyz(-1.0, 0.0, 1.0),
        &mut commands,
        &mut materials,
    );
    spawn_6(
        Transform::from_xyz(0.0, 0.0, 1.0),
        &mut commands,
        &mut materials,
    );
    spawn_7(
        Transform::from_xyz(1.0, 0.0, 1.0),
        &mut commands,
        &mut materials,
    );
    spawn_8(
        Transform::from_xyz(2.0, 0.0, 1.0),
        &mut commands,
        &mut materials,
    );
    spawn_9(
        Transform::from_xyz(3.0, 0.0, 1.0),
        &mut commands,
        &mut materials,
    );
    spawn_10(
        Transform::from_xyz(-1.0, 0.0, 2.0),
        &mut commands,
        &mut materials,
    );
    spawn_11(
        Transform::from_xyz(0.0, 0.0, 2.0),
        &mut commands,
        &mut materials,
    );
    spawn_12(
        Transform::from_xyz(1.0, 0.0, 2.0),
        &mut commands,
        &mut materials,
    );
    spawn_13(
        Transform::from_xyz(2.0, 0.0, 2.0),
        &mut commands,
        &mut materials,
    );
    spawn_14(
        Transform::from_xyz(3.0, 0.0, 2.0),
        &mut commands,
        &mut materials,
    );
    spawn_15(
        Transform::from_xyz(-1.0, 0.0, 3.0),
        &mut commands,
        &mut materials,
    );
    spawn_16(
        Transform::from_xyz(0.0, 0.0, 3.0),
        &mut commands,
        &mut materials,
    );
    spawn_17(
        Transform::from_xyz(1.0, 0.0, 3.0),
        &mut commands,
        &mut materials,
    );
    spawn_18(
        Transform::from_xyz(2.0, 0.0, 3.0),
        &mut commands,
        &mut materials,
    );
    spawn_19(
        Transform::from_xyz(3.0, 0.0, 3.0),
        &mut commands,
        &mut materials,
    );
    spawn_20(
        Transform::from_xyz(-1.0, 0.0, 4.0),
        &mut commands,
        &mut materials,
    );
    spawn_21(
        Transform::from_xyz(0.0, 0.0, 4.0),
        &mut commands,
        &mut materials,
    );
    spawn_22(
        Transform::from_xyz(1.0, 0.0, 4.0),
        &mut commands,
        &mut materials,
    );
    spawn_23(
        Transform::from_xyz(2.0, 0.0, 4.0),
        &mut commands,
        &mut materials,
    );
    spawn_24(
        Transform::from_xyz(3.0, 0.0, 4.0),
        &mut commands,
        &mut materials,
    );
}

#[derive(Component)]
struct Rotate(f32);

fn rotate(mut query: Query<(&mut Transform, &Rotate)>) {
    for (mut transform, speed) in query.iter_mut() {
        transform.rotate_axis(Vec3::Y, speed.0);
    }
}
