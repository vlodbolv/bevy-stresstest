// main.rs - Bevy 0.18 Ultimate Stress Test
// 
// Environment: Linux (Aurora 43) / i9-13900HK
// Features: 
// - Parallel ECS Animation (14-core optimization)
// - Dynamic Batch Stacking (Tornado effect)
// - Total Entity Counter UI
// - High-Contrast Lighting

use bevy::prelude::*;
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
    println!("  Lighting: Sun + Ambient + Point");
    println!("  Controls: SPACE to spawn 10,000 cubes");
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
        // RESOURCE: Ambient Light (Boosted Brightness)
        .insert_resource(AmbientLight {
            color: Color::srgb(0.6, 0.7, 0.8), 
            brightness: 800.0, 
        })
        .insert_resource(EnvironmentInfo { name: environment })
        // RESOURCE: Track Stats (Batch count + Total Entities)
        .insert_resource(SimulationStats { batch_count: 0, total_entities: 1 }) // Start at 1 (center cube)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (
            spawn_stress_cubes,      
            animate_cube_parallel,   
            animate_camera,          
            log_fps,                 
            update_fps_display,
            update_entity_display    // <--- NEW SYSTEM
        ))
        .run();
}

// ---------------- RESOURCES & COMPONENTS ----------------
#[derive(Resource)]
struct EnvironmentInfo { name: String }

#[derive(Resource)]
struct SimulationStats { 
    batch_count: u32,
    total_entities: u32 
}

#[derive(Component)]
struct AnimatedCube {
    rotation_speed: f32, // Allow different speeds for outer rings
}

#[derive(Component)]
struct OrbitCamera { radius: f32, speed: f32, angle: f32 }

#[derive(Component)]
struct FpsCounter { samples: VecDeque<f32>, last_update: f32, last_log: f32 }

#[derive(Component)]
struct EntityCountText; // Tag for the UI text

// ---------------- SCENE SETUP ----------------
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    env_info: Res<EnvironmentInfo>,
) {
    // 1. Spawn Center Reference Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.2, 0.2), // Red center
            metallic: 0.2,
            perceptual_roughness: 0.4,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        AnimatedCube { rotation_speed: 1.0 },
    ));

    // 2. Spawn Large Floor
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500.0, 500.0))), 
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.15), // Dark floor
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -30.0, 0.0),
    ));

    // 3. Directional Light (The Sun)
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(50.0, 80.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 4. Point Light (Fill)
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

    // 5. Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(60.0, 50.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera {
            radius: 80.0,
            speed: 0.15,
            angle: 0.0,
        },
    ));

    // 6. UI Overlay
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
                Text::new("🎮 Bevy Ultimate Stress Test"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 1.0)),
            ));

            // Subtitle
            parent.spawn((
                Text::new(format!("Running on {}", env_info.name)),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
                Node { margin: UiRect::top(Val::Px(5.0)), ..default() },
            ));

            // Stats Container
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            )).with_children(|stats| {
                // FPS
                stats.spawn((
                    Text::new("FPS: --"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::srgb(0.2, 1.0, 0.5)),
                    FpsCounter {
                        samples: VecDeque::new(),
                        last_update: 0.0,
                        last_log: 0.0,
                    },
                ));

                // Total Entities Display (NEW)
                stats.spawn((
                    Text::new("Entities: 1"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.8, 0.2)),
                    EntityCountText,
                    Node { margin: UiRect::top(Val::Px(5.0)), ..default() },
                ));
            });

            // Instructions
            parent.spawn((
                Text::new("✓ Method: Parallel Iterator\n[SPACE] Spawn 10,000 Cubes"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
                Node { margin: UiRect::top(Val::Px(20.0)), ..default() },
            ));
        });
}

// ---------------- SYSTEM: STRESS SPAWNER ----------------
fn spawn_stress_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut stats: ResMut<SimulationStats>,
) {
    if input.just_pressed(KeyCode::Space) {
        let count = 10_000;
        
        // Update stats
        stats.batch_count += 1;
        stats.total_entities += count;

        // Visuals: Cycle colors based on batch number
        let hue = (stats.batch_count as f32 * 0.5).sin() * 0.5 + 0.5;
        let mat_handle = materials.add(StandardMaterial {
            base_color: Color::hsl(hue * 360.0, 0.8, 0.5),
            metallic: 0.5,
            perceptual_roughness: 0.4,
            ..default()
        });

        let mesh_handle = meshes.add(Cuboid::new(0.5, 0.5, 0.5));
        
        // DISPLACEMENT LOGIC:
        // Each batch spawns further out (radius_offset) and higher up (y_offset)
        let radius_offset = stats.batch_count as f32 * 10.0; 
        let y_offset = stats.batch_count as f32 * 5.0;

        info!("💥 Spawning Batch {}: Total Entities {}", stats.batch_count, stats.total_entities);

        for i in 0..count {
            let i_f = i as f32;
            
            // Formulas for "Tornado" distribution
            let angle = i_f * 0.1;
            let radius = 15.0 + radius_offset + (i_f * 0.01);
            let height = (i_f % 100.0) * 0.5 + y_offset - 10.0;

            let x = angle.cos() * radius;
            let z = angle.sin() * radius;

            commands.spawn((
                Mesh3d(mesh_handle.clone()), 
                MeshMaterial3d(mat_handle.clone()),
                Transform::from_xyz(x, height, z),
                // Outer rings rotate slightly slower to look cool
                AnimatedCube { 
                    rotation_speed: 1.0 - (stats.batch_count as f32 * 0.05).clamp(0.0, 0.8) 
                }, 
            ));
        }
    }
}

// ---------------- SYSTEM: UI UPDATER (NEW) ----------------
fn update_entity_display(
    stats: Res<SimulationStats>, 
    mut query: Query<&mut Text, With<EntityCountText>>
) {
    if stats.is_changed() {
        for mut text in &mut query {
            // Format number with commas (simple hack)
            text.0 = format!("Entities: {}", stats.total_entities);
        }
    }
}

// ---------------- SYSTEM: OPTIMIZED PARALLEL ANIMATION ----------------
fn animate_cube_parallel(
    mut query: Query<(&mut Transform, &AnimatedCube)>, 
    time: Res<Time>
) {
    let delta_seconds = time.delta_secs();
    
    // We cannot pre-calculate rotation perfectly because each cube now 
    // has a unique 'rotation_speed', so we do the math inside the parallel loop.
    // This is still extremely fast on 14 cores.
    
    query.par_iter_mut().for_each(|(mut transform, cube)| {
        transform.rotate_y(delta_seconds * 0.8 * cube.rotation_speed);
        transform.rotate_x(delta_seconds * 0.5 * cube.rotation_speed);
    });
}

// ---------------- SYSTEM: CAMERA & UTILS ----------------
fn animate_camera(mut query: Query<(&mut Transform, &mut OrbitCamera)>, time: Res<Time>) {
    for (mut transform, mut orbit) in &mut query {
        orbit.angle += time.delta_secs() * orbit.speed;
        
        let x = orbit.angle.cos() * orbit.radius;
        let z = orbit.angle.sin() * orbit.radius;
        let y = 40.0 + (orbit.angle * 0.5).sin() * 10.0; 
        
        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn update_fps_display(time: Res<Time>, mut query: Query<(&mut Text, &mut FpsCounter)>) {
    for (mut text, mut fps_counter) in &mut query {
        let current_time = time.elapsed_secs();
        let fps = 1.0 / time.delta_secs();
        
        fps_counter.samples.push_back(fps);
        
        while fps_counter.samples.len() > 1 && current_time - fps_counter.last_update > 1.0 {
            fps_counter.samples.pop_front();
        }
        
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
        
        if current_time - fps_counter.last_log >= 0.2 {
            let fps = 1.0 / time.delta_secs();
            info!("Current FPS: {:.2}", fps);
            fps_counter.last_log = current_time;
        }
    }
}

