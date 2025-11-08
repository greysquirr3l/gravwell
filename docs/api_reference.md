# Gravwell API Reference

This document provides comprehensive API documentation for Gravwell's physics engine, including performance characteristics, usage patterns, and integration guidance.

## Table of Contents

1. [Core API Overview](#core-api-overview)
2. [Simulation Builder](#simulation-builder)
3. [Integrator Systems](#integrator-systems)
4. [Force Calculation](#force-calculation)
5. [Spatial Optimization](#spatial-optimization)
6. [Memory Management](#memory-management)
7. [Performance Monitoring](#performance-monitoring)
8. [Error Handling](#error-handling)
9. [Advanced Features](#advanced-features)

## Core API Overview

### Primary Types

#### `Simulation<I, F>`

The main simulation container managing particle state and physics integration.

```rust
pub struct Simulation<I: Integrator, F: ForceCalculator> {
    // Implementation details...
}
```

**Performance Characteristics:**

- Memory: O(N) where N is particle count
- Physics step: O(complexity of F)
- State access: O(1)

**Key Methods:**

##### `step(dt: f64) -> Result<()>`

Advances simulation by one timestep.

```rust
sim.step(0.016)?; // 60 FPS timestep
```

**Performance:** Depends on integrator and force calculator complexity
**Thread Safety:** Not thread-safe (use separate instances)
**Error Conditions:**

- Numerical instability (NaN/infinity)
- Invalid timestep (≤ 0 or too large)

##### `add_body(body: Body) -> Result<BodyHandle>`

Adds a new particle to the simulation.

```rust
let handle = sim.add_body(Body {
    mass: 1.989e30,
    position: Vector3::zeros(),
    velocity: Vector3::zeros(),
})?;
```

**Performance:** O(1) amortized, O(N) worst case during reallocation
**Memory:** Grows particle arrays by growth factor (typically 1.5x)

##### `remove_body(handle: BodyHandle) -> Result<()>`

Removes a particle from active simulation.

```rust
sim.remove_body(handle)?;
```

**Performance:** O(1) - marks inactive, no array compaction
**Note:** Bodies are not immediately removed to maintain handle validity

#### `Body`

Represents a single particle/body in the simulation.

```rust
pub struct Body {
    pub mass: f64,
    pub position: Vector3,
    pub velocity: Vector3,
}
```

**Usage Patterns:**

```rust
// Solar mass at origin
let sun = Body {
    mass: 1.989e30,
    position: Vector3::zeros(),
    velocity: Vector3::zeros(),
};

// Earth at 1 AU with orbital velocity
let earth = Body {
    mass: 5.972e24,
    position: Vector3::new(1.496e11, 0.0, 0.0),
    velocity: Vector3::new(0.0, 29780.0, 0.0),
};
```

#### `BodyHandle`

Type-safe handle for referencing bodies in the simulation.

```rust
pub struct BodyHandle {
    index: usize,
    generation: u32,
}
```

**Features:**

- Generation counter prevents use-after-free
- Implements `Copy` for efficient passing
- Invalid handles return errors gracefully

**Validation:**

```rust
if sim.is_valid_handle(handle) {
    let position = sim.position(handle);
}
```

## Simulation Builder

### `SimulationBuilder`

Type-safe builder pattern for creating configured simulations.

```rust
let sim = SimulationBuilder::new()
    .with_integrator(VelocityVerlet::new())
    .with_force_calculator(BarnesHut::new().theta(0.5))
    .with_timestep(0.001)
    .with_gravity_constant(6.67430e-11)
    .build()?;
```

#### Builder Methods

##### `with_integrator<T: Integrator>(integrator: T) -> SimulationBuilder<T, F>`

Sets the numerical integrator.

**Available Integrators:**

- `SemiImplicitEuler`: Fast, stable for games
- `VelocityVerlet`: Balanced accuracy/performance
- `Leapfrog`: Symplectic, energy-conserving
- `RK4`: High accuracy, not symplectic
- `IAS15`: Adaptive precision for scientific use

##### `with_force_calculator<T: ForceCalculator>(calc: T) -> SimulationBuilder<I, T>`

Sets the force calculation method.

**Available Calculators:**

- `DirectGravity`: O(N²), exact forces
- `BarnesHut`: O(N log N), configurable accuracy
- `FastMultipole`: O(N), high particle counts

##### Configuration Methods

```rust
.with_timestep(dt: f64)              // Default: 0.01
.with_gravity_constant(G: f64)       // Default: 6.67430e-11
.with_initial_capacity(capacity: usize) // Pre-allocate particle arrays
.with_spatial_optimization(enabled: bool) // Enable spatial culling
```

## Integrator Systems

All integrators implement the `Integrator` trait with consistent API.

### Semi-Implicit Euler

Fast, first-order integrator ideal for real-time applications.

```rust
let integrator = SemiImplicitEuler::new();
```

**Performance:** ~50ns per particle per step
**Accuracy:** First-order, moderate energy drift
**Stability:** Excellent for large timesteps
**Use Cases:** Games, real-time visualization

### Velocity Verlet

Second-order symplectic integrator with good accuracy/performance balance.

```rust
let integrator = VelocityVerlet::new();
```

**Performance:** ~80ns per particle per step
**Accuracy:** Second-order, low energy drift
**Stability:** Good, symplectic
**Use Cases:** Balanced simulations, orbital mechanics

### Leapfrog

Symplectic integrator with excellent long-term energy conservation.

```rust
let integrator = Leapfrog::new();
```

**Performance:** ~75ns per particle per step
**Accuracy:** Second-order, excellent energy conservation
**Stability:** Excellent, symplectic
**Use Cases:** Scientific computing, long-term evolution

### Runge-Kutta 4 (RK4)

Fourth-order accuracy for high-precision applications.

```rust
let integrator = RK4::new();
```

**Performance:** ~200ns per particle per step (4x force evaluations)
**Accuracy:** Fourth-order, very low truncation error
**Stability:** Good but not symplectic
**Use Cases:** High-precision scientific simulations

### IAS15 Adaptive

Adaptive 15th-order integrator for extreme precision.

```rust
let integrator = IAS15::new()
    .with_tolerance(1e-12)
    .with_max_iterations(12);
```

**Performance:** Variable, typically 500-2000ns per particle per step
**Accuracy:** 15th-order adaptive, machine precision
**Stability:** Excellent with automatic step size control
**Use Cases:** Research simulations, astrodynamics

## Force Calculation

### Direct Gravity

Exact O(N²) gravitational force calculation.

```rust
let forces = DirectGravity::new()
    .with_softening(0.01)     // Softening parameter
    .with_simd(true)          // Enable SIMD acceleration
    .with_parallel(true);     // Enable multi-threading
```

**Complexity:** O(N²)
**Accuracy:** Exact (within floating-point precision)
**Memory:** O(1) additional storage
**Recommended:** N < 10,000 particles

**Performance Scaling:**

- 1,000 particles: ~0.5ms per step
- 5,000 particles: ~12ms per step  
- 10,000 particles: ~50ms per step

### Barnes-Hut Tree

Approximate O(N log N) algorithm using spatial tree.

```rust
let forces = BarnesHut::new()
    .theta(0.5)               // Accuracy parameter
    .max_depth(10)            // Tree depth limit
    .leaf_capacity(16)        // Particles per leaf
    .parallel(true);          // Enable parallelization
```

**Complexity:** O(N log N)
**Accuracy:** Controlled by theta parameter
**Memory:** O(N) for tree storage
**Recommended:** N = 1,000 - 100,000 particles

**Theta Parameter Effects:**

- θ = 0.3: High accuracy, slower
- θ = 0.5: Balanced (recommended)
- θ = 0.7: Lower accuracy, faster

**Performance Scaling:**

- 10,000 particles: ~5ms per step (θ=0.5)
- 50,000 particles: ~30ms per step
- 100,000 particles: ~70ms per step

### Fast Multipole Method

Linear O(N) algorithm for massive particle counts.

```rust
let forces = FastMultipole::new()
    .expansion_order(8)       // Multipole expansion terms
    .max_level(6)             // Tree levels
    .parallel(true);
```

**Complexity:** O(N)
**Accuracy:** Controlled by expansion order
**Memory:** O(N) for multipole coefficients
**Recommended:** N > 50,000 particles

## Spatial Optimization

### Spatial Hash Grid

Efficient O(1) spatial partitioning for proximity queries.

```rust
use gravwell::spatial::SpatialHashGrid;

let mut grid = SpatialHashGrid::new(50.0); // 50m cell size

// Insert particles
for handle in sim.active_bodies() {
    let position = sim.position(handle);
    grid.insert(handle, position);
}

// Find neighbors within radius
let neighbors = grid.find_neighbors(position, 100.0);
```

**Performance:**

- Insertion: O(1) average, O(N) worst case
- Query: O(1) average for sparse grids
- Memory: O(N + C) where C is cell count

**Configuration:**

```rust
let grid = SpatialHashGrid::new(cell_size)
    .with_expected_particles(10000)  // Pre-allocate
    .with_load_factor(0.75);         // Hash table efficiency
```

### Frustum Culling

Remove particles outside camera view for rendering optimization.

```rust
use gravwell::spatial::{Frustum, AdvancedFrustumCuller};

// Create frustum from camera parameters
let frustum = Frustum::from_camera(
    camera_position,
    camera_direction,
    fov_radians,
    aspect_ratio,
    near_distance,
    far_distance,
);

// Test particle visibility
let visible = frustum.contains_point(particle_position);
let intersects = frustum.intersects_sphere(position, radius);
```

**Performance:**

- Point test: ~10ns per particle
- Sphere test: ~20ns per particle
- Typical culling: 50-80% particles removed

### Activation Manager

Dynamic particle activation based on importance and distance.

```rust
use gravwell::spatial::ActivationManager;

let mut manager = ActivationManager::new()
    .with_distance_threshold(1000.0)  // Activation distance
    .with_particle_budget(5000)       // Max active particles
    .with_hysteresis_factor(0.1);     // Prevent flickering

// Update activation states
manager.update_activation(
    &mut sim,
    camera_position,
    &importance_metrics,
)?;
```

**Budget Management:**

- Guarantees maximum active particle count
- Importance-based prioritization
- Smooth transitions with hysteresis

## Memory Management

### Memory Pools

Pre-allocated memory for temporary vectors and arrays.

```rust
use gravwell::memory::MemoryPool;

let pool = MemoryPool::new()
    .with_vector_capacity(10000)      // Vector3 arrays
    .with_scalar_capacity(10000)      // f64 arrays
    .with_handle_capacity(10000);     // BodyHandle arrays

// Use pooled memory
let temp_forces = pool.get_vector_array(particle_count);
// ... compute forces ...
pool.return_vector_array(temp_forces);
```

**Benefits:**

- Eliminates allocation overhead in physics loop
- Reduces garbage collection pressure
- Improves cache locality

**Performance Impact:**

- 10-30% speedup in force calculations
- Reduced memory fragmentation
- Consistent frame times

### Particle Storage

Structure-of-Arrays (SoA) layout for SIMD efficiency.

```rust
// Internal storage layout (conceptual)
struct ParticleSet {
    positions: Vec<Vector3>,    // Contiguous x,y,z data
    velocities: Vec<Vector3>,   // Enables SIMD operations
    masses: Vec<f64>,          // Cache-friendly access
    active: Vec<bool>,         // Activation state
}
```

**SIMD Benefits:**

- 2-4x speedup on modern CPUs
- Automatic vectorization by compiler
- Optimal memory bandwidth usage

## Performance Monitoring

### Built-in Profiling

Real-time performance metrics and bottleneck identification.

```rust
use gravwell::profiling::{Profiler, ProfileConfig};

let mut profiler = Profiler::new(ProfileConfig {
    enable_detailed_timing: true,
    sample_frequency: 60,        // Samples per second
    history_length: 300,         // 5 seconds at 60 FPS
});

// Instrument simulation loop
profiler.begin_frame();
{
    let _timer = profiler.time_section("physics_step");
    sim.step(dt)?;
}
{
    let _timer = profiler.time_section("spatial_update");
    spatial_culler.update(&sim);
}
profiler.end_frame();

// Get performance report
let report = profiler.generate_report();
println!("Physics: {:.2}ms, Spatial: {:.2}ms", 
    report.physics_time_ms, report.spatial_time_ms);
```

### Benchmarking Integration

Built-in integration with Criterion benchmarking.

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use gravwell::benchmarks::*;

fn bench_force_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("force_calculation");
    
    for particle_count in [1000, 5000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("direct", particle_count),
            &particle_count,
            |b, &n| {
                let sim = create_benchmark_sim(n);
                b.iter(|| sim.calculate_forces());
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_force_calculation);
criterion_main!(benches);
```

## Error Handling

### Error Types

Comprehensive error handling with context information.

```rust
use gravwell::error::{GravwellError, Result};

#[derive(thiserror::Error, Debug)]
pub enum GravwellError {
    #[error("Invalid body handle: {handle:?}")]
    InvalidBodyHandle { handle: BodyHandle },
    
    #[error("Numerical instability detected: {description}")]
    NumericalInstability { description: String },
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("Memory allocation failed")]
    OutOfMemory,
    
    #[error("Validation failed: {metric} = {value:.3e}, threshold = {threshold:.3e}")]
    ValidationFailed {
        metric: String,
        value: f64,
        threshold: f64,
    },
}
```

### Error Recovery

Graceful error handling with recovery strategies.

```rust
match sim.step(dt) {
    Ok(()) => { /* Continue simulation */ },
    
    Err(GravwellError::NumericalInstability { description }) => {
        eprintln!("Reducing timestep due to instability: {}", description);
        dt *= 0.5; // Reduce timestep
        continue;
    },
    
    Err(GravwellError::InvalidBodyHandle { handle }) => {
        eprintln!("Removing invalid handle: {:?}", handle);
        sim.cleanup_invalid_handles();
    },
    
    Err(e) => return Err(e), // Propagate unrecoverable errors
}
```

## Advanced Features

### Custom Integrators

Implement custom integration schemes.

```rust
use gravwell::core::Integrator;

pub struct CustomIntegrator {
    // State storage
    accelerations: Vec<Vector3>,
}

impl Integrator for CustomIntegrator {
    fn step(
        &mut self,
        positions: &mut [Vector3],
        velocities: &mut [Vector3],
        forces: &[Vector3],
        masses: &[f64],
        dt: f64,
    ) -> Result<()> {
        // Custom integration logic
        for i in 0..positions.len() {
            let acceleration = forces[i] / masses[i];
            velocities[i] += acceleration * dt;
            positions[i] += velocities[i] * dt;
        }
        Ok(())
    }
    
    fn name(&self) -> &str { "Custom" }
    fn order(&self) -> u8 { 1 }
    fn is_symplectic(&self) -> bool { false }
}
```

### GPU Acceleration

Offload computations to GPU using compute shaders.

```rust
use gravwell::gpu::{GpuDevice, GpuForceCalculator};

let gpu_device = GpuDevice::best_available()?;
let gpu_forces = GpuForceCalculator::new(gpu_device)
    .with_workgroup_size(64)
    .with_shared_memory_kb(48);

let sim = SimulationBuilder::new()
    .with_force_calculator(gpu_forces)
    .build()?;
```

**Performance:**

- 10-100x speedup for large systems (>10k particles)
- Requires GPU with compute shader support
- Automatic fallback to CPU if GPU unavailable

### Serialization Support

Save and restore simulation state.

```rust
use gravwell::serialization::{save_simulation, load_simulation};

// Save simulation state
save_simulation(&sim, "simulation_state.bin")?;

// Load simulation state
let restored_sim = load_simulation("simulation_state.bin")?;

// Verify state preservation
assert_eq!(sim.total_energy(), restored_sim.total_energy());
```

**Features:**

- Binary format for efficiency
- Cross-platform compatibility
- Deterministic restoration
- Optional compression

### Integration Callbacks

Hook into simulation events for custom processing.

```rust
use gravwell::callbacks::{SimulationCallbacks, StepCallback};

struct DataLogger {
    file: File,
}

impl StepCallback for DataLogger {
    fn on_step(&mut self, sim: &Simulation, step: usize, time: f64) {
        if step % 100 == 0 {
            writeln!(self.file, "{},{:.6e}", time, sim.total_energy()).unwrap();
        }
    }
}

let callbacks = SimulationCallbacks::new()
    .with_step_callback(Box::new(DataLogger { file }));

sim.set_callbacks(callbacks);
```

---

This API reference provides comprehensive documentation for all major Gravwell components,
including performance characteristics, usage patterns, and integration guidance. For specific
implementation details, refer to the source code documentation generated by `cargo doc`.
