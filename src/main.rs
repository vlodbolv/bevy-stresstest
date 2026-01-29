// main.rs - Bevy 0.18 Ultimate Stress Test (Icosahedron Edition - Optimized)

// Optimizations:
// 1. Added parallel FPS averaging over 3-second window
// 2. Terminal logging every 5 seconds with entity count
// 3. Optimized queries for better parallel performance
// 4. Simplified and improved iterator usage
// 5. Better resource management and idiomatic Rust patterns

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::VecDeque;
use std::env;
use std::fs;

// ---------------- ENVIRONMENT DETECTION ----------------
fn detect_environment() -> String {
    let in_distrobox = env::var("CONTAINER_ID").is_ok() 
        || fs::metadata("/.dockerenv").is_ok()
        || fs::read_to_string("/run/.containerenv").is_ok();
    
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

// ---------------- MAIN APP ENTRY ----------------
fn main() {
    let environment = detect_environment();
    
    println!("------------------------------------------------");
    println!("  Bevy Ultimate Performance Test");
    println!("  Environment: {}", environment);
    println!("  Shapes: Icosahedrons (20-sided Platonic Solid)");
    println!("  Controls: SPACE to spawn 10,000 shapes");
    println!("------------------------------------------------");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("Bevy Optimization - {}", environment),
                resolution: (1024.0, 768.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(AmbientLight {
            color: Color::srgb(0.6, 0.7, 0.8), 
            brightness: 800.0, 
        })
        .insert_resource(EnvironmentInfo { name: environment })
        .insert_resource(SimulationStats { 
            batch_count: 0, 
            total_entities: 1,
            last_5s_log: 0.0,
        })
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (
            spawn_stress_shapes,      
            animate_shapes_parallel,   
            animate_camera,          
            log_fps_periodic,
            update_fps_display,
            update_entity_display,
        ))
        .run();
}

// ---------------- RESOURCES & COMPONENTS ----------------
#[derive(Resource)]
struct EnvironmentInfo { 
    name: String 
}

#[derive(Resource)]
struct SimulationStats { 
    batch_count: u32,
    total_entities: u32,
    last_5s_log: f32,
}

#[derive(Component)]
struct AnimatedShape {
    rotation_speed: f32,
}

#[derive(Component)]
struct OrbitCamera { 
    radius: f32, 
    speed: f32, 
    angle: f32 
}

#[derive(Component)]
struct FpsCounter { 
    samples: VecDeque<f32>,
    last_update: f32,
    #[allow(dead_code)] // Used for potential future logging
    sample_start: f32,
    rolling_sum: f32,
    sample_count: u32,
}

#[derive(Component)]
struct EntityCountText;

// ---------------- CUSTOM MESH GENERATOR (ICOSAHEDRON) ----------------
fn create_icosahedron_mesh(radius: f32) -> Mesh {
    let phi = (1.0 + 5.0f32.sqrt()) / 2.0;

    let positions = [
        Vec3::new(-1.0,  phi, 0.0).normalize() * radius,
        Vec3::new( 1.0,  phi, 0.0).normalize() * radius,
        Vec3::new(-1.0, -phi, 0.0).normalize() * radius,
        Vec3::new( 1.0, -phi, 0.0).normalize() * radius,

        Vec3::new( 0.0, -1.0,  phi).normalize() * radius,
        Vec3::new( 0.0,  1.0,  phi).normalize() * radius,
        Vec3::new( 0.0, -1.0, -phi).normalize() * radius,
        Vec3::new( 0.0,  1.0, -phi).normalize() * radius,

        Vec3::new( phi, 0.0, -1.0).normalize() * radius,
        Vec3::new( phi, 0.0,  1.0).normalize() * radius,
        Vec3::new(-phi, 0.0, -1.0).normalize() * radius,
        Vec3::new(-phi, 0.0,  1.0).normalize() * radius,
    ];

    let indices = [
        0, 11, 5,   0, 5, 1,   0, 1, 7,   0, 7, 10,  0, 10, 11,
        1, 5, 9,    5, 11, 4,  11, 10, 2, 10, 7, 6,  7, 1, 8,
        3, 9, 4,    3, 4, 2,   3, 2, 6,   3, 6, 8,   3, 8, 9,
        4, 9, 5,    2, 4, 11,  6, 2, 10,  8, 6, 7,   9, 8, 1,
    ];

    // FLAT SHADING: 20 faces × 3 vertices = 60 unique vertices
    let mut final_positions = Vec::with_capacity(60);
    let mut final_normals = Vec::with_capacity(60);
    let mut final_indices = Vec::with_capacity(60);

    for face in indices.chunks_exact(3) {
        let [idx0, idx1, idx2] = [face[0], face[1], face[2]];
        
        let p0 = positions[idx0];
        let p1 = positions[idx1];
        let p2 = positions[idx2];

        let normal = (p1 - p0).cross(p2 - p0).normalize();

        final_positions.extend([p0, p1, p2]);
        final_normals.extend([normal; 3]);

        let start_idx = final_indices.len() as u32;
        final_indices.extend(start_idx..start_idx + 3);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, final_positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, final_normals);
    mesh.insert_indices(Indices::U32(final_indices));
    mesh
}

// ---------------- SCENE SETUP ----------------
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    env_info: Res<EnvironmentInfo>,
) {
    // Center Reference Shape
    commands.spawn((
        Mesh3d(meshes.add(create_icosahedron_mesh(1.5))), 
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.2, 0.2), 
            metallic: 0.2,
            perceptual_roughness: 0.4,
            double_sided: true, 
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        AnimatedShape { rotation_speed: 1.0 },
    ));

    // Large Floor
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500.0, 500.0))), 
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.15),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -30.0, 0.0),
    ));

    // Sun
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(50.0, 80.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Fill Light
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            color: Color::srgb(1.0, 0.8, 0.6),
            shadows_enabled: true,
            range: 200.0,
            ..default()
        },
        Transform::from_xyz(-50.0, 30.0, -50.0),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(60.0, 50.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera {
            radius: 80.0,
            speed: 0.15,
            angle: 0.0,
        },
    ));

    // UI
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(20.0)),
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            Text::new("🎮 Bevy Icosahedron Test"),
            TextFont { font_size: 32.0, ..default() },
            TextColor(Color::srgb(0.9, 0.9, 1.0)),
        ));

        parent.spawn((
            Text::new(format!("Running on {}", env_info.name)),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.8)),
            Node { margin: UiRect::top(Val::Px(5.0)), ..default() },
        ));

        parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        }).with_children(|stats| {
            stats.spawn((
                Text::new("FPS: --"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(0.2, 1.0, 0.5)),
                FpsCounter {
                    samples: VecDeque::with_capacity(150), // 3 seconds at 50 FPS
                    last_update: 0.0,
                    sample_start: 0.0,
                    rolling_sum: 0.0,
                    sample_count: 0,
                },
            ));

            stats.spawn((
                Text::new("Entities: 1"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(1.0, 0.8, 0.2)),
                EntityCountText,
                Node { margin: UiRect::top(Val::Px(5.0)), ..default() },
            ));
        });

        parent.spawn((
            Text::new("✓ Method: Parallel Iterator\n[SPACE] Spawn 10,000 Shapes"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.7)),
            Node { margin: UiRect::top(Val::Px(20.0)), ..default() },
        ));
    });
}

// ---------------- SYSTEM: STRESS SPAWNER ----------------
fn spawn_stress_shapes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    input: Res<ButtonInput<KeyCode>>,
    mut stats: ResMut<SimulationStats>,
) {
    if input.just_pressed(KeyCode::Space) {
        const COUNT: u32 = 10_000;
        
        stats.batch_count += 1;
        stats.total_entities += COUNT;

        let hue = (stats.batch_count as f32 * 0.5).sin() * 0.5 + 0.5;
        let mat_handle = materials.add(StandardMaterial {
            base_color: Color::hsl(hue * 360.0, 0.8, 0.5),
            metallic: 0.5,
            perceptual_roughness: 0.4,
            double_sided: true,
            ..default()
        });

        let mesh_handle = meshes.add(create_icosahedron_mesh(0.5));
        
        let radius_offset = stats.batch_count as f32 * 10.0; 
        let y_offset = stats.batch_count as f32 * 5.0;

        info!("💥 Spawning Batch {}: Total Entities {}", stats.batch_count, stats.total_entities);

        // Use iterators for better performance
        (0..COUNT).for_each(|i| {
            let i_f = i as f32;
            
            let angle = i_f * 0.1;
            let radius = 15.0 + radius_offset + (i_f * 0.01);
            let height = (i_f % 100.0) * 0.5 + y_offset - 10.0;

            let x = angle.cos() * radius;
            let z = angle.sin() * radius;

            commands.spawn((
                Mesh3d(mesh_handle.clone()), 
                MeshMaterial3d(mat_handle.clone()),
                Transform::from_xyz(x, height, z),
                AnimatedShape { 
                    rotation_speed: 1.0 - (stats.batch_count as f32 * 0.05).clamp(0.0, 0.8) 
                }, 
            ));
        });
    }
}

// ---------------- SYSTEM: UI UPDATER ----------------
fn update_entity_display(
    stats: Res<SimulationStats>, 
    mut query: Query<&mut Text, With<EntityCountText>>
) {
    if stats.is_changed() {
        for mut text in query.iter_mut() {
            text.0 = format!("Entities: {}", stats.total_entities);
        }
    }
}

// ---------------- SYSTEM: OPTIMIZED PARALLEL ANIMATION ----------------
fn animate_shapes_parallel(
    mut query: Query<(&mut Transform, &AnimatedShape)>, 
    time: Res<Time>
) {
    let delta_seconds = time.delta_secs();
    
    // Parallel iteration for maximum CPU utilization
    query.par_iter_mut().for_each(|(mut transform, shape)| {
        let speed = shape.rotation_speed;
        transform.rotate_y(delta_seconds * 0.8 * speed);
        transform.rotate_x(delta_seconds * 0.5 * speed);
    });
}

// ---------------- SYSTEM: CAMERA & UTILS ----------------
fn animate_camera(mut query: Query<(&mut Transform, &mut OrbitCamera)>, time: Res<Time>) {
    let delta = time.delta_secs();
    
    for (mut transform, mut orbit) in query.iter_mut() {
        orbit.angle += delta * orbit.speed;
        
        let x = orbit.angle.cos() * orbit.radius;
        let z = orbit.angle.sin() * orbit.radius;
        let y = 40.0 + (orbit.angle * 0.5).sin() * 10.0; 
        
        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn update_fps_display(time: Res<Time>, mut query: Query<(&mut Text, &mut FpsCounter)>) {
    let current_time = time.elapsed_secs();
    
    for (mut text, mut fps_counter) in query.iter_mut() {
        let fps = 1.0 / time.delta_secs();
        
        // Update rolling average over 3 seconds
        if current_time - fps_counter.sample_start >= 3.0 {
            fps_counter.rolling_sum = 0.0;
            fps_counter.sample_count = 0;
            fps_counter.sample_start = current_time;
        }
        
        fps_counter.rolling_sum += fps;
        fps_counter.sample_count += 1;
        
        // Update display every second
        if current_time - fps_counter.last_update >= 1.0 {
            if fps_counter.sample_count > 0 {
                let avg_fps = fps_counter.rolling_sum / fps_counter.sample_count as f32;
                text.0 = format!("FPS: {:.0}", avg_fps);
            }
            fps_counter.last_update = current_time;
        }
        
        fps_counter.samples.push_back(fps);
        if fps_counter.samples.len() > 150 { // Keep ~3 seconds of history
            fps_counter.samples.pop_front();
        }
    }
}

fn log_fps_periodic(time: Res<Time>, mut stats: ResMut<SimulationStats>, query: Query<&FpsCounter>) {
    let current_time = time.elapsed_secs();
    
    // Log to terminal every 5 seconds
    if current_time - stats.last_5s_log >= 5.0 {
        if let Ok(fps_counter) = query.get_single() {
            let total_entities = stats.total_entities;
            
            // Calculate 3-second average
            let three_sec_avg = if fps_counter.sample_count > 0 {
                fps_counter.rolling_sum / fps_counter.sample_count as f32
            } else if !fps_counter.samples.is_empty() {
                fps_counter.samples.iter().sum::<f32>() / fps_counter.samples.len() as f32
            } else {
                0.0
            };
            
            println!(
                "[{:.1}s] Entities: {}, 3-sec Avg FPS: {:.1}",
                current_time,
                total_entities,
                three_sec_avg
            );
        }
        
        stats.last_5s_log = current_time;
    }
}
