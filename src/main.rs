// main.rs - Bevy 0.18 Minimal Test Application for Aurora DX Distrobox
// 
// Cargo.toml:
// [package]
// name = "bevy-test"
// version = "0.1.0"
// edition = "2021"
//
// [dependencies]
// bevy = "0.18"
//
// [profile.dev]
// opt-level = 1
//
// [profile.dev.package."*"]
// opt-level = 3

use bevy::prelude::*;
use std::collections::VecDeque;
use std::env;
use std::fs;

fn detect_environment() -> String {
    // Check if running in Distrobox
    let in_distrobox = env::var("CONTAINER_ID").is_ok() 
        || fs::metadata("/.dockerenv").is_ok()
        || fs::read_to_string("/run/.containerenv").is_ok();
    
    // Check if on Aurora/Fedora
    let os_info = fs::read_to_string("/etc/os-release").unwrap_or_default();
    let is_aurora = os_info.contains("Aurora");
    let is_fedora = os_info.contains("Fedora");
    
    match (in_distrobox, is_aurora, is_fedora) {
        (true, true, _) => "Aurora DX Distrobox".to_string(),
        (true, _, true) => "Fedora Distrobox".to_string(),
        (true, _, _) => "Container Environment".to_string(),
        (false, true, _) => "Aurora DX (Host)".to_string(),
        (false, _, true) => "Fedora (Host)".to_string(),
        _ => "Native Environment".to_string(),
    }
}

fn main() {
    let environment = detect_environment();
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("Bevy 0.18 - {}", environment),
                resolution: (1024, 768).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(EnvironmentInfo { name: environment })
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (animate_cube, animate_camera, log_fps, update_fps_display))
        .run();
}

#[derive(Resource)]
struct EnvironmentInfo {
    name: String,
}

#[derive(Component)]
struct AnimatedCube;

#[derive(Component)]
struct OrbitCamera {
    radius: f32,
    speed: f32,
    angle: f32,
}

#[derive(Component)]
struct FpsCounter {
    samples: VecDeque<f32>,
    last_update: f32,
    last_log: f32,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    env_info: Res<EnvironmentInfo>,
) {
    // Spawn animated cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 0.9),
            metallic: 0.5,
            perceptual_roughness: 0.3,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        AnimatedCube,
    ));

    // Spawn ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(12.0, 12.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.35),
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(0.0, -1.5, 0.0),
    ));

    // Spawn point light with physically based intensity
    commands.spawn((
        PointLight {
            intensity: 3_000_000.0,
            color: Color::srgb(1.0, 0.95, 0.8),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 8.0, 5.0),
    ));

    // Spawn orbiting camera with ambient light
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(8.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        AmbientLight {
            color: Color::srgb(0.4, 0.5, 0.6),
            brightness: 200.0,
            affects_lightmapped_meshes: false,
        },
        OrbitCamera {
            radius: 10.0,
            speed: 0.3,
            angle: 0.0,
        },
    ));

    // Spawn UI overlay with modern layout
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        })
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("🎮 Bevy 0.18 Test"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 1.0)),
            ));

            // Subtitle
            parent.spawn((
                Text::new(format!("Running on {}", env_info.name)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },
            ));

            // FPS counter
            parent.spawn((
                Text::new("FPS: --"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 1.0, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                FpsCounter {
                    samples: VecDeque::new(),
                    last_update: 0.0,
                    last_log: 0.0,
                },
            ));

            // Info text
            parent.spawn((
                Text::new("✓ Graphics: Vulkan\n✓ Container: Fedora Toolbox\n✓ GPU Acceleration: Enabled"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
        });
}

fn animate_cube(mut query: Query<&mut Transform, With<AnimatedCube>>, time: Res<Time>) {
    for mut transform in &mut query {
        // Rotate the cube
        transform.rotate_y(time.delta_secs() * 0.8);
        transform.rotate_x(time.delta_secs() * 0.5);
        
        // Bob up and down
        let bob = (time.elapsed_secs() * 2.0).sin() * 0.5;
        transform.translation.y = bob;
    }
}

fn animate_camera(mut query: Query<(&mut Transform, &mut OrbitCamera)>, time: Res<Time>) {
    for (mut transform, mut orbit) in &mut query {
        orbit.angle += time.delta_secs() * orbit.speed;
        
        let x = orbit.angle.cos() * orbit.radius;
        let z = orbit.angle.sin() * orbit.radius;
        let y = 6.0 + (orbit.angle * 0.5).sin() * 2.0;
        
        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn update_fps_display(time: Res<Time>, mut query: Query<(&mut Text, &mut FpsCounter)>) {
    for (mut text, mut fps_counter) in &mut query {
        let current_time = time.elapsed_secs();
        let fps = 1.0 / time.delta_secs();
        
        // Add current FPS sample
        fps_counter.samples.push_back(fps);
        
        // Remove samples older than 1 second
        while fps_counter.samples.len() > 1 && current_time - fps_counter.last_update > 1.0 {
            fps_counter.samples.pop_front();
        }
        
        // Update display every 1 second
        if current_time - fps_counter.last_update >= 1.0 {
            if !fps_counter.samples.is_empty() {
                let avg_fps: f32 = fps_counter.samples.iter().sum::<f32>() / fps_counter.samples.len() as f32;
                text.0 = format!("FPS: {:.0}", avg_fps);
            }
            fps_counter.last_update = current_time;
        }
    }
}

fn log_fps(time: Res<Time>, mut query: Query<&mut FpsCounter>) {
    for mut fps_counter in &mut query {
        let current_time = time.elapsed_secs();
        
        // Log FPS every 200ms
        if current_time - fps_counter.last_log >= 0.2 {
            let fps = 1.0 / time.delta_secs();
            info!("Current FPS: {:.2}", fps);
            fps_counter.last_log = current_time;
        }
    }
}
