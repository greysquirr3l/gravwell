# Gravwell Tutorial Series

A comprehensive series of tutorials guiding you from basic physics simulations to advanced optimization techniques.

## Table of Contents

### Getting Started

1. [Quick Start Guide](#1-quick-start-guide)
2. [Your First Simulation](#2-your-first-simulation)
3. [Understanding Integrators](#3-understanding-integrators)
4. [Force Calculation Basics](#4-force-calculation-basics)

### Intermediate Concepts

5. [Performance Optimization](#5-performance-optimization)
6. [Spatial Optimization](#6-spatial-optimization)
7. [Memory Management](#7-memory-management)
8. [Error Handling](#8-error-handling)

### Advanced Topics

9. [Scientific Computing](#9-scientific-computing)
10. [Game Engine Integration](#10-game-engine-integration)
11. [Custom Components](#11-custom-components)
12. [GPU Acceleration](#12-gpu-acceleration)

---

## 1. Quick Start Guide

### Installation

Add Gravwell to your `Cargo.toml`:

```toml
[dependencies]
gravwell = "0.5.0"
nalgebra = "0.32"
```

### Basic Usage

```rust
use gravwell::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simulation
    let mut sim = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(DirectGravity::new())
        .build()?;

    // Add the Sun
    let sun = sim.add_body(Body {
        mass: 1.989e30,
        position: Vector3::zeros(),
        velocity: Vector3::zeros(),
    })?;

    // Add Earth
    let earth = sim.add_body(Body {
        mass: 5.972e24,
        position: Vector3::new(1.496e11, 0.0, 0.0), // 1 AU
        velocity: Vector3::new(0.0, 29780.0, 0.0),  // Orbital velocity
    })?;

    // Run simulation
    for _ in 0..1000 {
        sim.step(86400.0)?; // 1 day timesteps
        
        // Check energy conservation
        if sim.step_count() % 365 == 0 {
            println!("Year {}: Energy = {:.3e} J", 
                sim.step_count() / 365, sim.total_energy());
        }
    }

    Ok(())
}
```

### What You'll Learn

- How to create a basic simulation
- Adding bodies to the simulation
- Running physics steps
- Monitoring energy conservation

---

## 2. Your First Simulation

### Creating a Binary Star System

Let's create a more interesting simulation with two massive stars orbiting each other:

```rust
use gravwell::prelude::*;
use std::f64::consts::PI;

fn create_binary_star_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌟 Creating Binary Star System");
    
    // High-precision simulation for orbital mechanics
    let mut sim = SimulationBuilder::new()
        .with_integrator(Leapfrog::new())  // Symplectic for energy conservation
        .with_force_calculator(DirectGravity::new())
        .with_timestep(3600.0)  // 1 hour timesteps
        .build()?;

    // System parameters
    let star1_mass = 2.0 * 1.989e30;  // 2 solar masses
    let star2_mass = 1.5 * 1.989e30;  // 1.5 solar masses
    let separation = 2.0 * 1.496e11;  // 2 AU separation
    
    // Calculate orbital velocities for circular orbit
    let total_mass = star1_mass + star2_mass;
    let orbital_velocity = (G * total_mass / separation).sqrt();
    
    // Position stars at center of mass
    let star1_distance = separation * star2_mass / total_mass;
    let star2_distance = separation * star1_mass / total_mass;

    // Add first star
    let star1 = sim.add_body(Body {
        mass: star1_mass,
        position: Vector3::new(-star1_distance, 0.0, 0.0),
        velocity: Vector3::new(0.0, -orbital_velocity * star2_mass / total_mass, 0.0),
    })?;

    // Add second star
    let star2 = sim.add_body(Body {
        mass: star2_mass,
        position: Vector3::new(star2_distance, 0.0, 0.0),
        velocity: Vector3::new(0.0, orbital_velocity * star1_mass / total_mass, 0.0),
    })?;

    // Track orbital period
    let expected_period = 2.0 * PI * (separation.powi(3) / (G * total_mass)).sqrt();
    println!("Expected orbital period: {:.2} days", expected_period / 86400.0);

    // Run simulation for several orbits
    let steps_per_orbit = (expected_period / 3600.0) as usize;
    
    for step in 0..(steps_per_orbit * 3) {  // 3 orbital periods
        sim.step()?;
        
        // Track positions every 10% of orbit
        if step % (steps_per_orbit / 10) == 0 {
            let pos1 = sim.position(star1);
            let pos2 = sim.position(star2);
            let distance = (pos1 - pos2).norm();
            
            println!("Step {}: Separation = {:.3e} m ({:.2} AU)", 
                step, distance, distance / 1.496e11);
        }
    }

    // Final energy check
    let final_energy = sim.total_energy();
    println!("Final energy: {:.6e} J", final_energy);
    
    Ok(())
}
```

### Adding Visualization Data

```rust
use std::fs::File;
use std::io::Write;

fn save_trajectory_data(sim: &Simulation, handles: &[BodyHandle], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(filename)?;
    writeln!(file, "time,star1_x,star1_y,star1_z,star2_x,star2_y,star2_z")?;
    
    let time = sim.current_time();
    let pos1 = sim.position(handles[0]);
    let pos2 = sim.position(handles[1]);
    
    writeln!(file, "{},{},{},{},{},{},{}", 
        time, pos1.x, pos1.y, pos1.z, pos2.x, pos2.y, pos2.z)?;
    
    Ok(())
}
```

### Key Concepts

- **Symplectic Integration**: Use Leapfrog for energy conservation
- **Center of Mass**: Position bodies correctly for stable orbits
- **Timestep Selection**: Balance accuracy vs. performance
- **Data Export**: Save trajectories for visualization

---

## 3. Understanding Integrators

Different integrators offer trade-offs between speed, accuracy, and stability.

### Integrator Comparison

```rust
use gravwell::prelude::*;

fn compare_integrators() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Integrator Comparison");
    
    let integrators: Vec<(&str, Box<dyn Integrator>)> = vec![
        ("Semi-Implicit Euler", Box::new(SemiImplicitEuler::new())),
        ("Velocity Verlet", Box::new(VelocityVerlet::new())),
        ("Leapfrog", Box::new(Leapfrog::new())),
        ("RK4", Box::new(RK4::new())),
    ];
    
    let timesteps = vec![0.1, 0.01, 0.001];
    
    for &dt in &timesteps {
        println!("\nTimestep: {:.3} seconds", dt);
        println!("Integrator           | Energy Error | Position Error | Time");
        println!("--------------------|--------------|----------------|------");
        
        for (name, integrator) in &integrators {
            let result = test_integrator_accuracy(integrator.as_ref(), dt)?;
            println!("{:19} | {:.3e}   | {:.3e}      | {:.1}ms",
                name, result.energy_error, result.position_error, result.computation_time_ms);
        }
    }
    
    Ok(())
}

struct IntegratorTestResult {
    energy_error: f64,
    position_error: f64,
    computation_time_ms: f64,
}

fn test_integrator_accuracy(integrator: &dyn Integrator, dt: f64) -> Result<IntegratorTestResult, Box<dyn std::error::Error>> {
    // Create test simulation with known analytical solution
    let mut sim = SimulationBuilder::new()
        .with_integrator(integrator.clone())
        .with_force_calculator(DirectGravity::new())
        .with_timestep(dt)
        .build()?;
    
    // Simple two-body system
    setup_earth_sun_system(&mut sim)?;
    
    let initial_energy = sim.total_energy();
    let initial_position = sim.position(earth_handle);
    
    // Time the integration
    let start = std::time::Instant::now();
    
    // Simulate one orbital period
    let orbital_period = 365.25 * 86400.0; // Earth year in seconds
    let steps = (orbital_period / dt) as usize;
    
    for _ in 0..steps {
        sim.step()?;
    }
    
    let computation_time = start.elapsed();
    
    // Calculate errors
    let final_energy = sim.total_energy();
    let final_position = sim.position(earth_handle);
    
    let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();
    let position_error = (final_position - initial_position).norm() / 1.496e11; // Error in AU
    
    Ok(IntegratorTestResult {
        energy_error,
        position_error,
        computation_time_ms: computation_time.as_secs_f64() * 1000.0,
    })
}
```

### When to Use Each Integrator

#### Semi-Implicit Euler

```rust
let integrator = SemiImplicitEuler::new();
```

**Best for:**

- Real-time games (60+ FPS)
- Interactive simulations
- Large timesteps without instability

**Characteristics:**

- Fast: ~50ns per particle per step
- Stable: Handles large timesteps well
- Energy drift: Moderate, acceptable for games

#### Velocity Verlet

```rust
let integrator = VelocityVerlet::new();
```

**Best for:**

- Balanced accuracy/performance
- General-purpose simulations
- Educational demonstrations

**Characteristics:**

- Performance: ~80ns per particle per step
- Accuracy: Second-order, good energy conservation
- Stability: Symplectic, time-reversible

#### Leapfrog

```rust
let integrator = Leapfrog::new();
```

**Best for:**

- Scientific simulations
- Long-term orbital evolution
- Maximum energy conservation

**Characteristics:**

- Performance: ~75ns per particle per step
- Accuracy: Excellent energy conservation
- Stability: Symplectic, ideal for Hamiltonian systems

#### Runge-Kutta 4

```rust
let integrator = RK4::new();
```

**Best for:**

- High-precision requirements
- Short-term accurate integration
- Non-Hamiltonian systems

**Characteristics:**

- Performance: ~200ns per particle per step
- Accuracy: Fourth-order truncation error
- Stability: Not symplectic, energy drift over time

### Adaptive Integration

```rust
use gravwell::integrators::IAS15;

fn adaptive_precision_example() -> Result<(), Box<dyn std::error::Error>> {
    let integrator = IAS15::new()
        .with_tolerance(1e-12)     // Precision requirement
        .with_min_timestep(1.0)    // Minimum timestep (seconds)
        .with_max_timestep(86400.0); // Maximum timestep (1 day)
    
    let mut sim = SimulationBuilder::new()
        .with_integrator(integrator)
        .with_force_calculator(DirectGravity::new())
        .build()?;
    
    // The integrator automatically adjusts timestep
    // based on local truncation error estimates
    
    for _ in 0..1000 {
        sim.step_adaptive()?; // Variable timestep
        
        if sim.step_count() % 100 == 0 {
            println!("Step {}: dt = {:.1}s, time = {:.2} days",
                sim.step_count(),
                sim.current_timestep(),
                sim.current_time() / 86400.0);
        }
    }
    
    Ok(())
}
```

---

## 4. Force Calculation Basics

Understanding force calculation algorithms and their performance characteristics.

### Direct Gravity (O(N²))

The simplest and most accurate method:

```rust
use gravwell::prelude::*;

fn direct_gravity_example() -> Result<(), Box<dyn std::error::Error>> {
    let force_calc = DirectGravity::new()
        .with_softening(0.0)       // No softening for exact forces
        .with_simd(true)           // Enable SIMD acceleration
        .with_parallel(true);      // Use all CPU cores
    
    let mut sim = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(force_calc)
        .build()?;
    
    // Add particles
    for i in 0..5000 {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / 5000.0;
        let radius = 1000.0 + 500.0 * (i % 10) as f64;
        
        sim.add_body(Body {
            mass: 1e20,
            position: Vector3::new(
                radius * angle.cos(),
                (i as f64 - 2500.0) * 10.0,
                radius * angle.sin(),
            ),
            velocity: Vector3::new(
                -50.0 * angle.sin(),
                0.0,
                50.0 * angle.cos(),
            ),
        })?;
    }
    
    // Benchmark force calculation
    let start = std::time::Instant::now();
    for _ in 0..10 {
        sim.step(1.0)?;
    }
    let elapsed = start.elapsed();
    
    println!("Direct gravity (5000 particles): {:.2}ms per step", 
        elapsed.as_secs_f64() * 100.0); // 10 steps -> per step
    
    Ok(())
}
```

### Barnes-Hut Tree (O(N log N))

Approximate algorithm for larger systems:

```rust
use gravwell::forces::BarnesHut;

fn barnes_hut_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌳 Barnes-Hut Algorithm Demo");
    
    // Test different theta values
    let theta_values = vec![0.3, 0.5, 0.7, 1.0];
    
    for theta in theta_values {
        let force_calc = BarnesHut::new()
            .theta(theta)              // Accuracy parameter
            .max_depth(12)             // Tree depth
            .leaf_capacity(16)         // Particles per leaf
            .parallel(true);
        
        let mut sim = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(force_calc)
            .build()?;
        
        // Create galaxy-like distribution
        for i in 0..20000 {
            let (position, velocity) = generate_galaxy_particle(i);
            sim.add_body(Body {
                mass: random_stellar_mass(),
                position,
                velocity,
            })?;
        }
        
        // Benchmark
        let start = std::time::Instant::now();
        for _ in 0..5 {
            sim.step(1000.0)?; // 1000 second timesteps
        }
        let elapsed = start.elapsed();
        
        println!("θ = {:.1}: {:.2}ms per step, Energy = {:.3e} J",
            theta, elapsed.as_secs_f64() * 200.0, sim.total_energy());
    }
    
    Ok(())
}

fn generate_galaxy_particle(index: usize) -> (Vector3, Vector3) {
    let angle = 4.0 * std::f64::consts::PI * index as f64 / 20000.0;
    let radius = (index as f64 / 20000.0).sqrt() * 5000.0 + 100.0;
    let height = ((index % 1000) as f64 - 500.0) * 50.0;
    
    let position = Vector3::new(
        radius * angle.cos(),
        height,
        radius * angle.sin(),
    );
    
    // Orbital velocity for galaxy rotation
    let orbital_speed = (50000.0 / radius).sqrt() * 20.0;
    let velocity = Vector3::new(
        -orbital_speed * angle.sin(),
        0.0,
        orbital_speed * angle.cos(),
    );
    
    (position, velocity)
}

fn random_stellar_mass() -> f64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let log_mass = rng.gen_range(-1.0..2.0); // Log10 of solar masses
    10.0_f64.powf(log_mass) * 1.989e30
}
```

### Performance Comparison

```rust
fn force_algorithm_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Force Algorithm Performance Comparison");
    
    let particle_counts = vec![1000, 5000, 10000, 20000];
    
    for &n in &particle_counts {
        println!("\n{} Particles:", n);
        println!("Algorithm       | Time per Step | Memory Usage | Accuracy");
        println!("----------------|---------------|--------------|----------");
        
        // Direct Gravity
        let direct_time = benchmark_force_algorithm(&DirectGravity::new(), n)?;
        println!("Direct Gravity  | {:8.2} ms   | {:7.1} MB   | Exact",
            direct_time, estimate_memory_usage_direct(n));
        
        // Barnes-Hut
        let bh_time = benchmark_force_algorithm(&BarnesHut::new().theta(0.5), n)?;
        println!("Barnes-Hut 0.5  | {:8.2} ms   | {:7.1} MB   | ~99.5%",
            bh_time, estimate_memory_usage_barnes_hut(n));
        
        // Performance crossover point
        if n == 10000 {
            println!("\n💡 Recommendation: Use Barnes-Hut for N > 5,000 particles");
        }
    }
    
    Ok(())
}

fn benchmark_force_algorithm(force_calc: &dyn ForceCalculator, n: usize) -> Result<f64, Box<dyn std::error::Error>> {
    let mut sim = create_benchmark_simulation(force_calc.clone(), n)?;
    
    let start = std::time::Instant::now();
    for _ in 0..10 {
        sim.step(1.0)?;
    }
    let elapsed = start.elapsed();
    
    Ok(elapsed.as_secs_f64() * 100.0) // ms per step
}

fn estimate_memory_usage_direct(n: usize) -> f64 {
    // Particle data + temporary force arrays
    (n * (3 * 8 + 3 * 8 + 8) + n * 3 * 8) as f64 / 1_000_000.0 // MB
}

fn estimate_memory_usage_barnes_hut(n: usize) -> f64 {
    // Particle data + tree nodes
    let particle_memory = n * (3 * 8 + 3 * 8 + 8); // positions + velocities + mass
    let tree_memory = n * 8 * 8; // Approximate tree overhead
    (particle_memory + tree_memory) as f64 / 1_000_000.0 // MB
}
```

---

## 5. Performance Optimization

Learn how to achieve 60+ FPS with thousands of particles.

### Profile-Guided Optimization

```rust
use gravwell::profiling::*;

fn optimize_for_60fps() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Optimizing for 60 FPS Performance");
    
    let mut sim = create_performance_simulation(10000)?; // 10k particles
    let mut profiler = Profiler::new();
    
    let target_frame_time = 16.67; // 60 FPS = 16.67ms per frame
    
    for frame in 0..300 { // 5 seconds at 60 FPS
        profiler.begin_frame();
        
        let frame_start = std::time::Instant::now();
        
        // Physics step
        {
            let _timer = profiler.time_section("physics");
            sim.step(0.016)?; // 60 FPS timestep
        }
        
        // Spatial updates (simulated)
        {
            let _timer = profiler.time_section("spatial");
            std::thread::sleep(std::time::Duration::from_micros(500)); // Simulate spatial work
        }
        
        let frame_time = frame_start.elapsed().as_secs_f64() * 1000.0;
        profiler.end_frame();
        
        // Adaptive quality adjustment
        if frame_time > target_frame_time && frame % 60 == 0 {
            println!("Frame {}: {:.2}ms (target: {:.2}ms) - reducing quality",
                frame, frame_time, target_frame_time);
            // In real implementation: reduce particle count, LOD, etc.
        }
        
        // Report every second
        if frame % 60 == 0 {
            let report = profiler.generate_report();
            println!("Second {}: Physics={:.2}ms, Spatial={:.2}ms, Total={:.2}ms",
                frame / 60, report.physics_avg_ms, report.spatial_avg_ms, report.total_avg_ms);
        }
    }
    
    Ok(())
}
```

### SIMD Optimization

```rust
use gravwell::optimization::simd::*;

fn enable_simd_acceleration() -> Result<(), Box<dyn std::error::Error>> {
    // Check SIMD support
    println!("🚀 SIMD Acceleration Status:");
    println!("AVX2 support: {}", SimdCapabilities::has_avx2());
    println!("AVX-512 support: {}", SimdCapabilities::has_avx512());
    
    // Create SIMD-optimized simulation
    let force_calc = DirectGravity::new()
        .with_simd(true)
        .with_simd_instruction_set(InstructionSet::Avx2); // Force specific instruction set
    
    let mut sim = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(force_calc)
        .build()?;
    
    // Benchmark SIMD vs scalar
    let (simd_time, scalar_time) = benchmark_simd_performance(&mut sim, 5000)?;
    
    println!("Performance comparison (5000 particles):");
    println!("SIMD enabled:  {:.2}ms per step", simd_time);
    println!("Scalar only:   {:.2}ms per step", scalar_time);
    println!("Speedup:       {:.1}x", scalar_time / simd_time);
    
    Ok(())
}
```

### Multi-Threading

```rust
use gravwell::parallel::*;

fn parallel_performance_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔀 Multi-Threading Performance Demo");
    
    let thread_counts = vec![1, 2, 4, 8];
    let particle_count = 15000;
    
    for &threads in &thread_counts {
        let force_calc = BarnesHut::new()
            .theta(0.5)
            .parallel(threads > 1)
            .thread_count(threads);
        
        let mut sim = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(force_calc)
            .build()?;
        
        // Populate simulation
        populate_galaxy_simulation(&mut sim, particle_count)?;
        
        // Benchmark
        let start = std::time::Instant::now();
        for _ in 0..20 {
            sim.step(100.0)?;
        }
        let elapsed = start.elapsed().as_secs_f64() * 50.0; // ms per step
        
        let efficiency = if threads == 1 { 
            100.0 
        } else { 
            (thread_counts[0] as f64 * elapsed) / (threads as f64 * elapsed) * 100.0 
        };
        
        println!("{} threads: {:.2}ms per step ({:.1}% efficiency)",
            threads, elapsed, efficiency);
    }
    
    Ok(())
}
```

---

## 6. Spatial Optimization

Implementing spatial optimization for massive particle counts.

### Spatial Hash Grid

```rust
use gravwell::spatial::SpatialHashGrid;

fn spatial_optimization_tutorial() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗺️  Spatial Optimization Tutorial");
    
    // Create simulation with spatial optimization
    let mut sim = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(BarnesHut::new().theta(0.6))
        .with_spatial_optimization(true)
        .build()?;
    
    // Create spatial hash grid
    let mut spatial_grid = SpatialHashGrid::new(100.0); // 100m cells
    
    // Populate with clustered particles
    for cluster in 0..5 {
        let cluster_center = Vector3::new(
            (cluster as f64 - 2.0) * 2000.0,
            0.0,
            0.0,
        );
        
        for i in 0..4000 {
            let offset = random_sphere_point() * 500.0; // 500m radius clusters
            let position = cluster_center + offset;
            
            let handle = sim.add_body(Body {
                mass: 1e22,
                position,
                velocity: random_velocity(),
            })?;
            
            spatial_grid.insert(handle, position);
        }
    }
    
    println!("Created 5 clusters with 4000 particles each (20,000 total)");
    
    // Demonstrate spatial queries
    let query_position = Vector3::new(0.0, 0.0, 0.0);
    let query_radius = 1000.0;
    
    let nearby_particles = spatial_grid.find_neighbors(query_position, query_radius);
    println!("Found {} particles within {}m of origin", 
        nearby_particles.len(), query_radius);
    
    // Performance comparison: brute force vs spatial
    let brute_force_time = benchmark_brute_force_neighbors(&sim, query_position, query_radius)?;
    let spatial_time = benchmark_spatial_neighbors(&spatial_grid, query_position, query_radius)?;
    
    println!("Neighbor search performance:");
    println!("Brute force: {:.3}ms", brute_force_time);
    println!("Spatial grid: {:.3}ms", spatial_time);
    println!("Speedup: {:.1}x", brute_force_time / spatial_time);
    
    Ok(())
}

fn random_sphere_point() -> Vector3 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Uniform distribution on unit sphere
    let theta = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
    let phi = (1.0 - 2.0 * rng.gen::<f64>()).acos();
    let r = rng.gen::<f64>().cbrt(); // Uniform volume distribution
    
    Vector3::new(
        r * phi.sin() * theta.cos(),
        r * phi.sin() * theta.sin(),
        r * phi.cos(),
    )
}

fn random_velocity() -> Vector3 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    Vector3::new(
        rng.gen_range(-50.0..50.0),
        rng.gen_range(-50.0..50.0),
        rng.gen_range(-50.0..50.0),
    )
}
```

### Frustum Culling

```rust
use gravwell::spatial::{Frustum, Camera};

fn frustum_culling_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("📷 Frustum Culling Demo");
    
    // Set up camera
    let camera = Camera {
        position: Vector3::new(0.0, 1000.0, 2000.0),
        direction: Vector3::new(0.0, -0.3, -0.9).normalize(),
        up: Vector3::new(0.0, 1.0, 0.0),
        fov_radians: 60.0_f64.to_radians(),
        aspect_ratio: 16.0 / 9.0,
        near_distance: 10.0,
        far_distance: 10000.0,
    };
    
    let frustum = Frustum::from_camera(&camera);
    
    // Create simulation
    let mut sim = create_large_galaxy_simulation(50000)?;
    
    // Test frustum culling
    let mut visible_count = 0;
    let mut culled_count = 0;
    
    for handle in sim.active_bodies() {
        let position = sim.position(handle);
        
        if frustum.contains_point(position) {
            visible_count += 1;
        } else {
            culled_count += 1;
        }
    }
    
    let culling_efficiency = culled_count as f64 / (visible_count + culled_count) as f64;
    
    println!("Frustum culling results:");
    println!("Visible particles: {}", visible_count);
    println!("Culled particles: {}", culled_count);
    println!("Culling efficiency: {:.1}%", culling_efficiency * 100.0);
    
    // Simulate moving camera
    println!("\nSimulating camera movement...");
    for frame in 0..180 { // 3 seconds at 60 FPS
        let angle = frame as f64 * 2.0;
        let new_camera = Camera {
            position: Vector3::new(
                3000.0 * (angle * 0.01).cos(),
                1000.0 + 500.0 * (angle * 0.02).sin(),
                3000.0 * (angle * 0.01).sin(),
            ),
            ..camera
        };
        
        let moving_frustum = Frustum::from_camera(&new_camera);
        
        let frame_visible = sim.active_bodies()
            .filter(|&handle| moving_frustum.contains_point(sim.position(handle)))
            .count();
        
        if frame % 30 == 0 {
            println!("Frame {}: {} visible particles", frame, frame_visible);
        }
    }
    
    Ok(())
}
```

---

## 7. Memory Management

Optimizing memory usage for large-scale simulations.

### Memory Pools

```rust
use gravwell::memory::{MemoryPool, PooledVector};

fn memory_pool_tutorial() -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 Memory Pool Tutorial");
    
    // Create memory pool for 50k particles
    let pool = MemoryPool::new()
        .with_vector_capacity(50000)
        .with_scalar_capacity(50000)
        .with_temporary_storage(10); // 10 temporary arrays
    
    // Create simulation with memory pool
    let mut sim = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(BarnesHut::new())
        .with_memory_pool(pool.clone())
        .build()?;
    
    // Populate simulation
    populate_performance_test_simulation(&mut sim, 25000)?;
    
    // Benchmark with and without memory pool
    println!("Running memory allocation benchmark...");
    
    let pooled_time = benchmark_simulation_with_pool(&mut sim, &pool)?;
    let standard_time = benchmark_simulation_standard(&mut sim)?;
    
    println!("Performance comparison (1000 steps):");
    println!("With memory pool:    {:.2}ms total ({:.3}ms per step)", 
        pooled_time, pooled_time / 1000.0);
    println!("Standard allocation: {:.2}ms total ({:.3}ms per step)", 
        standard_time, standard_time / 1000.0);
    println!("Memory pool speedup: {:.1}x", standard_time / pooled_time);
    
    // Memory usage analysis
    let pool_stats = pool.statistics();
    println!("\nMemory pool statistics:");
    println!("Total allocated: {:.1} MB", pool_stats.total_allocated_mb);
    println!("Currently in use: {:.1} MB", pool_stats.currently_used_mb);
    println!("Peak usage: {:.1} MB", pool_stats.peak_usage_mb);
    println!("Allocation count: {}", pool_stats.allocation_count);
    println!("Efficiency: {:.1}%", pool_stats.efficiency_percent);
    
    Ok(())
}

fn demonstrate_custom_allocator() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Custom Allocator Demo");
    
    // Create allocator optimized for physics simulation
    let allocator = PhysicsAllocator::new()
        .with_particle_block_size(1000)    // Allocate particles in blocks
        .with_alignment(32)                // SIMD alignment
        .with_preallocation_factor(1.5);   // 50% extra capacity
    
    let mut sim = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(DirectGravity::new())
        .with_custom_allocator(allocator)
        .build()?;
    
    // Test dynamic allocation patterns
    println!("Testing dynamic particle management...");
    
    for wave in 0..5 {
        // Add particles in waves
        let wave_size = 5000;
        let mut wave_handles = Vec::new();
        
        for i in 0..wave_size {
            let handle = sim.add_body(create_random_particle(i))?;
            wave_handles.push(handle);
        }
        
        // Simulate for a while
        for _ in 0..100 {
            sim.step(0.01)?;
        }
        
        // Remove some particles
        for (i, handle) in wave_handles.iter().enumerate() {
            if i % 3 == 0 { // Remove every third particle
                sim.remove_body(*handle)?;
            }
        }
        
        println!("Wave {}: Added {}, removed {}, active: {}", 
            wave, wave_size, wave_size / 3, sim.active_particle_count());
    }
    
    Ok(())
}
```

### Structure-of-Arrays Optimization

```rust
fn soa_vs_aos_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Structure-of-Arrays vs Array-of-Structures Comparison");
    
    let particle_count = 20000;
    
    // Test SoA layout (Gravwell's default)
    println!("Testing Structure-of-Arrays (SoA) layout...");
    let soa_time = benchmark_soa_layout(particle_count)?;
    
    // Test AoS layout (traditional approach)
    println!("Testing Array-of-Structures (AoS) layout...");
    let aos_time = benchmark_aos_layout(particle_count)?;
    
    println!("\nResults for {} particles:", particle_count);
    println!("SoA layout: {:.2}ms per force calculation", soa_time);
    println!("AoS layout: {:.2}ms per force calculation", aos_time);
    println!("SoA speedup: {:.1}x", aos_time / soa_time);
    
    // Explain the performance difference
    println!("\nWhy SoA is faster:");
    println!("✅ Better cache locality for vectorized operations");
    println!("✅ SIMD instructions can process multiple particles simultaneously");
    println!("✅ Reduced memory bandwidth requirements");
    println!("✅ Compiler auto-vectorization works better");
    
    Ok(())
}

// Traditional AoS approach for comparison
#[derive(Clone)]
struct ParticleAoS {
    position: Vector3,
    velocity: Vector3,
    mass: f64,
}

fn benchmark_aos_layout(n: usize) -> Result<f64, Box<dyn std::error::Error>> {
    let mut particles = Vec::with_capacity(n);
    
    // Initialize particles
    for i in 0..n {
        particles.push(ParticleAoS {
            position: random_position(),
            velocity: random_velocity(),
            mass: random_mass(),
        });
    }
    
    let mut forces = vec![Vector3::zeros(); n];
    
    // Benchmark force calculation
    let start = std::time::Instant::now();
    
    for _ in 0..10 {
        calculate_forces_aos(&particles, &mut forces);
    }
    
    let elapsed = start.elapsed().as_secs_f64() * 100.0; // ms per iteration
    Ok(elapsed)
}

fn calculate_forces_aos(particles: &[ParticleAoS], forces: &mut [Vector3]) {
    for i in 0..particles.len() {
        forces[i] = Vector3::zeros();
        
        for j in 0..particles.len() {
            if i == j { continue; }
            
            let r_vec = particles[j].position - particles[i].position;
            let r_squared = r_vec.norm_squared();
            let r = r_squared.sqrt();
            
            let force_magnitude = G * particles[i].mass * particles[j].mass / r_squared;
            forces[i] += force_magnitude * r_vec / r;
        }
    }
}
```

---

This tutorial series provides comprehensive guidance from basic physics simulations to
advanced optimization techniques. Each section builds upon previous concepts while
introducing new features and best practices for achieving optimal performance with Gravwell.

Continue with the remaining sections (8-12) to cover error handling, scientific computing,
game engine integration, custom components, and GPU acceleration.
