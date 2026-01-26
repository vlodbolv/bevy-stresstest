# Bevy ECS Stress Test & Benchmark

A high-performance ECS (Entity Component System) stress test application built with [Bevy](https://bevyengine.org/). 

This project is designed to benchmark parallel system execution and rendering performance on Linux systems (specifically **Aurora DX** and **Fedora**) running inside **Distrobox** containers with GPU passthrough.

![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)
![Bevy](https://img.shields.io/badge/Bevy-0.15-white.svg)
![Platform](https://img.shields.io/badge/Platform-Linux%20(Aurora%2FBluefin)-blue.svg)

## 🛠️ Environment Setup (CRITICAL)

**Before cloning or running this repository**, you must set up your development environment correctly. This project relies on specific system dependencies (ALSA, Udev, Vulkan headers) often missing from standard containers.

Please configure your environment using the **Bevy Scaffold** scripts:

👉 **[https://github.com/vlodbolv/bevy-scaffold](https://github.com/vlodbolv/bevy-scaffold)**

Do not attempt to compile this project until you have run the setup from the repo above.

## 🚀 Features

* **Parallel ECS Iteration:** Utilizes `par_iter_mut()` to spread component updates across all available CPU cores (Optimized for 14-core i9-13900HK).
* **Dynamic Stress Testing:** Spawns **10,000 entities** per batch at runtime without stalling the render loop.
* **Smart Stacking Logic:** Dynamically calculates offsets to create a "Tornado" stacking effect, preventing mesh overlap as entity counts grow into the hundreds of thousands.
* **Environment Awareness:** Auto-detects if running inside `Distrobox`, `Docker`, or Native Fedora/Aurora hosts.
* **High-Fidelity Lighting:** Features a directional sun, ambient skylight, and point light fill with shadow mapping enabled.
* **Real-time UI:** Monitors FPS, Frame Time, and Total Entity Count.

## 🎮 Controls

| Input | Action |
| :--- | :--- |
| **SPACE** | Spawn **10,000** new animated cubes. |
| **Mouse** | The camera orbits automatically (visual cinematic mode). |

## ⚡ Performance Optimization

To achieve maximum framerates (100+ FPS with 50k+ entities), this project uses specific `Cargo.toml` profiles:

1.  **Dependency Optimization:** `[profile.dev.package."*"] opt-level = 3` ensures the Bevy engine is fully optimized even during development.
2.  **LTO (Link Time Optimization):** Enabled for release builds to reduce binary size and improve ECS iteration speed.
3.  **Parallel execution:** The `animate_cube_parallel` system handles transform updates on the CPU using multi-threading.

## 🏃‍♀️ How to Run

### Development Mode
Fast compilation, decent performance:

cargo run

Benchmark Mode (Recommended)
Slow compilation, maximum performance (enables LTO and full optimizations):

cargo run --release

📝 Code Overview
main.rs: Contains all logic.

detect_environment(): Identifies the container/OS context.

animate_cube_parallel(): The core CPU stress test system.

spawn_stress_cubes(): Handles batch generation and color cycling.


