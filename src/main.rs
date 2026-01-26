// main.rs - Bevy 0.18 Ultimate Stress Test (Icosahedron Edition - Final)
// 
// Environment: Linux (Aurora 43) / i9-13900HK
// Fixes: Removed unused 'mut' from positions vector. 
// Result: Solid 20-sided geometry, flat shading, zero warnings.

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
        .insert_resource(SimulationStats { batch_count: 0, total_entities: 1 })
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (
            spawn_stress_shapes,      
            animate_shapes_parallel,   
            animate_camera,          
            log_fps,                 
            update_fps_display,
            update_entity_display
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
struct AnimatedShape {
    rotation_speed: f32,
}

#[derive(Component)]
struct OrbitCamera { radius: f32, speed: f32, angle: f32 }

#[derive(Component)]
struct FpsCounter { samples: VecDeque<f32>, last_update: f32, last_log: f32 }

#[derive(Component)]
struct EntityCountText;

// ---------------- CUSTOM MESH GENERATOR (ICOSAHEDRON) ----------------
// 
// Generates a 20-sided Icosahedron. 
// Uses 3 orthogonal Golden Rectangles.
fn create_icosahedron_mesh(radius: f32) -> Mesh {
    let t = (1.0 + 5.0f32.sqrt()) / 2.0;

    // The 12 vertices of an icosahedron (removed 'mut' here)
    let positions = vec![
        Vec3::new(-1.0,  t, 0.0).normalize() * radius,
        Vec3::new( 1.0,  t, 0.0).normalize() * radius,
        Vec3::new(-1.0, -t, 0.0).normalize() * radius,
        Vec3::new( 1.0, -t, 0.0).normalize() * radius,

        Vec3::new( 0.0, -1.0,  t).normalize() * radius,
        Vec3::new( 0.0,  1.0,  t).normalize() * radius,
        Vec3::new( 0.0, -1.0, -t).normalize() * radius,
        Vec3::new( 0.0,  1.0, -t).normalize() * radius,

        Vec3::new( t, 0.0, -1.0).normalize() * radius,
        Vec3::new( t, 0.0,  1.0).normalize() * radius,
        Vec3::new(-t, 0.0, -1.0).normalize() * radius,
        Vec3::new(-t, 0.0,  1.0).normalize() * radius,
    ];

    // The 20 triangular faces (indices into positions)
    let indices = vec![
        0, 11, 5,   0, 5, 1,   0, 1, 7,   0, 7, 10,  0, 10, 11,
        1, 5, 9,    5, 11, 4,  11, 10, 2, 10, 7, 6,  7, 1, 8,
        3, 9, 4,    3, 4, 2,   3, 2, 6,   3, 6, 8,   3, 8, 9,
        4, 9, 5,    2, 4, 11,  6, 2, 10,  8, 6, 7,   9, 8, 1,
    ];

    // FLAT SHADING LOGIC:
    // We must duplicate vertices for every face so each face gets its own normal.
    // 20 faces * 3 vertices = 60 unique vertices.
    let mut final_positions = Vec::new();
    let mut final_normals = Vec::new();
    let mut final_indices = Vec::new();

    for i in (0..indices.len()).step_by(3) {
        let idx0 = indices[i];
        let idx1 = indices[i+1];
        let idx2 = indices[i+2];

        let p0 = positions[idx0];
        let p1 = positions[idx1];
        let p2 = positions[idx2];

        // Calculate face normal (Cross product)
        let normal = (p1 - p0).cross(p2 - p0).normalize();

        // Push new unique vertices
        final_positions.push(p0);
        final_positions.push(p1);
        final_positions.push(p2);

        final_normals.push(normal);
        final_normals.push(normal);
        final_normals.push(normal);

        // Indices are now just linear: 0, 1, 2,  3, 4, 5...
        let start_idx = i as u32;
        final_indices.push(start_idx);
        final_indices.push(start_idx + 1);
        final_indices.push(start_idx + 2);
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
    // 1. Center Reference Shape
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

    // 2. Large Floor
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500.0, 500.0))), 
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.15),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -30.0, 0.0),
    ));

    // 3. Sun
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(50.0, 80.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 4. Fill Light
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

    // 6. UI
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        })
        .with_children(|parent| {
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

            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            )).with_children(|stats| {
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
        let count = 10_000;
        
        stats.batch_count += 1;
        stats.total_entities += count;

        let hue = (stats.batch_count as f32 * 0.5).sin() * 0.5 + 0.5;
        let mat_handle = materials.add(StandardMaterial {
            base_color: Color::hsl(hue * 360.0, 0.8, 0.5),
            metallic: 0.5,
            perceptual_roughness: 0.4,
            double_sided: true,
            ..default()
        });

        // Use Icosahedron (20-sided)
        let mesh_handle = meshes.add(create_icosahedron_mesh(0.5));
        
        let radius_offset = stats.batch_count as f32 * 10.0; 
        let y_offset = stats.batch_count as f32 * 5.0;

        info!("💥 Spawning Batch {}: Total Entities {}", stats.batch_count, stats.total_entities);

        for i in 0..count {
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
        }
    }
}

// ---------------- SYSTEM: UI UPDATER ----------------
fn update_entity_display(
    stats: Res<SimulationStats>, 
    mut query: Query<&mut Text, With<EntityCountText>>
) {
    if stats.is_changed() {
        for mut text in &mut query {
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
    
    query.par_iter_mut().for_each(|(mut transform, shape)| {
        transform.rotate_y(delta_seconds * 0.8 * shape.rotation_speed);
        transform.rotate_x(delta_seconds * 0.5 * shape.rotation_speed);
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
