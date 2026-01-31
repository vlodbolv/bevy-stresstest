# Bevy ECS Stress Test & Benchmark

A high-performance ECS (Entity Component System) stress test application built with **Bevy 0.18**.

This project benchmarks parallel system execution and rendering performance on Linux systems—specifically **Aurora DX** and **Fedora**—running inside **Distrobox** containers with GPU passthrough.

---

## 🛠️ Environment Setup (CRITICAL)

This project relies on specific system dependencies (ALSA, Udev, Vulkan headers) and GPU drivers often missing from standard containers.

**Before cloning or running this repository**, configure your environment using the **Bevy Scaffold** scripts to ensure your Distrobox container is correctly initialized:

👉 **[https://github.com/vlodbolv/bevy-scaffold](https://github.com/vlodbolv/bevy-scaffold)**

---

## 🚀 Features

* **Parallel ECS Iteration:** Leverages Bevy's internal task pool to spread component updates across all CPU cores (Optimized for high-core count architectures like the i9-13900HK).
* **Dynamic Stress Testing:** Spawns **10,000 entities** per batch at runtime without stalling the main render loop.
* **Tornado Stacking:** Dynamically calculates spatial offsets to create a spiraling effect, preventing mesh overlap as entity counts reach 100k+.
* **Environment Awareness:** Auto-detects if running inside `Distrobox`, `Docker`, or Native hosts to adjust resource allocation.
* **Modern Rendering:** Utilizes Bevy 0.18's `Mesh3d` and `MeshMaterial3d` with directional sun, ambient skylight, and shadow mapping.

---

## 🎮 Controls

| Input | Action |
| --- | --- |
| **SPACE** | Spawn **10,000** new animated cubes. |
| **Mouse** | The camera orbits automatically (Cinematic Mode). |

---

## ⚡ Performance Optimization

To achieve maximum framerates (100+ FPS with 50k+ entities), the project utilizes aggressive `Cargo.toml` profiles:

1. **Dependency Optimization:** `[profile.dev.package."*"] opt-level = 3` ensures the Bevy engine itself is fully optimized even during debug builds.
2. **LTO (Link Time Optimization):** Enabled for release builds to reduce binary size and improve cross-crate ECS iteration speed.
3. **Parallel execution:** The `animate_cube_parallel` system handles transform updates on the CPU using Bevy's multi-threaded query iteration.

---

## 🏃‍♀️ How to Run

### Development Mode

Fast compilation, standard performance:

```bash
cargo run

```

### Benchmark Mode (Recommended)

Maximum performance (enables LTO and full engine optimizations):

```bash
cargo run --release

```

---

## 📝 Code Overview

The logic is contained within `main.rs`, organized into the following core functions:

* **`detect_environment()`**: Identifies the container/OS context to output telemetry.
* **`animate_cube_parallel()`**: The core CPU stress test system using `par_iter_mut()`.
* **`spawn_stress_cubes()`**: Handles batch generation, color cycling, and procedural placement.

---

## 📂 Project structure

```text
bevy-benchmark/
├─ hello/
│  ├─ Cargo.toml         # Optimized profiles & Bevy 0.18 config
│  └─ src/
│     └─ main.rs         # Parallel systems & stress-test logic
├─ init_distrobox.sh      # Environment creation script
├─ setup_inside_distrobox.sh
└─ README.md

```
