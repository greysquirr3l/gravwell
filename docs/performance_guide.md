# Gravwell Performance Guide

## Overview

This guide covers performance optimization strategies for achieving high
frame rates (30-60+ FPS) with Gravwell physics simulations. It addresses
both CPU and GPU optimizations, providing concrete techniques for different
use cases.

## Performance Targets

| Mode | Particle Count | Target FPS | Hardware Requirements |
|------|----------------|------------|---------------------|
| **Game (Basic)** | 1,000 | 30 FPS | Single-core CPU |
| **Game (Optimized)** | 1,000 | 60 FPS | Multi-core CPU + SIMD |
| **Game (Advanced)** | 5,000 | 60 FPS | Multi-core CPU + SIMD + LOD |
| **Science (Basic)** | 10,000 | 1 FPS | Single-core CPU |
| **Science (Parallel)** | 10,000 | 10 FPS | Multi-core CPU |
| **Science (GPU)** | 100,000 | 1 FPS | Modern GPU (RTX/RX series) |

## CPU Optimizations

### 1. SIMD Vectorization

Enable SIMD operations for force calculations:

```rust
use gravwell::prelude::*;

let sim = Simulation::builder()
    .forces(DirectGravity::new().with_simd(true))
    .build()?;
```

**Manual SIMD Implementation Example:**

```rust
use std::simd::f64x4;

pub fn calculate_forces_simd(
    positions: &[Vector3],
    masses: &[f64],
    forces: &mut [Vector3],
) {
    let n = positions.len();
    
    // Process 4 particles at a time
    for i in (0..n).step_by(4) {
        let chunk_size = (n - i).min(4);
        
        // Load positions into SIMD vectors
        let pos_x = f64x4::from_slice(&positions[i..i+chunk_size].iter().map(|p| p.x).collect::<Vec<_>>());
        let pos_y = f64x4::from_slice(&positions[i..i+chunk_size].iter().map(|p| p.y).collect::<Vec<_>>());
        let pos_z = f64x4::from_slice(&positions[i..i+chunk_size].iter().map(|p| p.z).collect::<Vec<_>>());
        
        // Vectorized force calculation
        for j in 0..n {
            if i <= j && j < i + chunk_size { continue; }
            
            let dx = pos_x - f64x4::splat(positions[j].x);
            let dy = pos_y - f64x4::splat(positions[j].y);
            let dz = pos_z - f64x4::splat(positions[j].z);
            
            let r_squared = dx * dx + dy * dy + dz * dz;
            let r = r_squared.sqrt();
            let force_mag = f64x4::splat(G * masses[j]) / r_squared;
            
            // Apply forces (store back to slice)
            for k in 0..chunk_size {
                let idx = i + k;
                forces[idx].x += force_mag[k] * dx[k] / r[k];
                forces[idx].y += force_mag[k] * dy[k] / r[k];
                forces[idx].z += force_mag[k] * dz[k] / r[k];
            }
        }
    }
}
```

**Performance Impact:** 2-4x speedup on modern CPUs with AVX support.

### 2. Multi-Threading with Rayon

Parallelize force calculations across CPU cores:

```rust
use gravwell::prelude::*;
use rayon::prelude::*;

let sim = Simulation::builder()
    .forces(BarnesHut::new().with_parallel(true))
    .thread_count(8) // Use 8 threads
    .build()?;
```

**Custom Parallel Implementation:**

```rust
use rayon::prelude::*;

pub fn calculate_forces_parallel(
    positions: &[Vector3],
    masses: &[f64],
    forces: &mut [Vector3],
    thread_count: usize,
) {
    let chunk_size = positions.len() / thread_count;
    
    forces.par_chunks_mut(chunk_size).enumerate().for_each(|(chunk_idx, force_chunk)| {
        let start_idx = chunk_idx * chunk_size;
        
        for (local_idx, force) in force_chunk.iter_mut().enumerate() {
            let i = start_idx + local_idx;
            *force = Vector3::zeros();
            
            for j in 0..positions.len() {
                if i == j { continue; }
                
                let r_vec = positions[j] - positions[i];
                let r_squared = r_vec.norm_squared();
                let r = r_squared.sqrt();
                
                let force_magnitude = G * masses[i] * masses[j] / r_squared;
                *force += force_magnitude * r_vec / r;
            }
        }
    });
}
```

**Performance Impact:** Near-linear scaling with CPU core count (6-8x on 8-core CPUs).

### 3. Algorithm Optimization

#### Barnes-Hut Tree (O(N log N))

For systems with > 1,000 particles, use Barnes-Hut instead of direct calculation:

```rust
use gravwell::prelude::*;

// O(N²) - Good for N < 1,000
let direct_forces = DirectGravity::new();

// O(N log N) - Good for N = 1,000 - 100,000  
let barnes_hut = BarnesHut::new()
    .theta(0.5)     // Accuracy parameter (0.3 = high accuracy, 0.7 = fast)
    .parallel(true) // Enable multi-threading
    .simd(true);    // Enable SIMD

let sim = Simulation::builder()
    .forces(barnes_hut)
    .build()?;
```

**Theta Parameter Tuning:**

- θ = 0.3: High accuracy, slower (good for science mode)
- θ = 0.5: Balanced accuracy/performance (recommended for games)  
- θ = 0.7: Lower accuracy, fastest (suitable for distant objects)

### 4. Memory Optimization

#### Structure-of-Arrays Layout

Gravwell automatically uses SoA layout for cache efficiency:

```rust
// Automatic SoA layout - cache-friendly for SIMD
pub struct ParticleSet {
    positions: Vec<Vector3>,  // Contiguous memory
    velocities: Vec<Vector3>, // Contiguous memory
    masses: Vec<f64>,         // Contiguous memory
    radii: Vec<f64>,          // Contiguous memory
}

// Versus AoS layout - cache-unfriendly
pub struct Particle {
    position: Vector3,
    velocity: Vector3,
    mass: f64,
    radius: f64,
}
pub struct ParticleSetAoS {
    particles: Vec<Particle>,  // Scattered memory access
}
```

#### Memory Pool Allocation

Pre-allocate temporary vectors to avoid allocations in simulation loop:

```rust
use gravwell::prelude::*;

let mut sim = Simulation::builder()
    .integrator(VelocityVerlet::new())
    .forces(BarnesHut::new())
    .reserve_capacity(10000)  // Pre-allocate for 10k particles
    .build()?;

// No allocations during simulation loop
for _ in 0..1000 {
    sim.step();  // Uses pre-allocated memory
}
```

## Level of Detail (LOD) System

For large particle counts, implement LOD to reduce computational load:

```rust
use gravwell::prelude::*;

let sim = Simulation::builder()
    .forces(BarnesHut::new())
    .lod_system(LODSettings {
        distance_thresholds: vec![1000.0, 5000.0, 20000.0],
        detail_levels: vec![
            DetailLevel::Full,      // < 1000 units: full physics
            DetailLevel::Reduced,   // < 5000 units: reduced timestep  
            DetailLevel::Minimal,   // < 20000 units: approximate forces
            DetailLevel::Culled,    // > 20000 units: no physics
        ],
    })
    .build()?;
```

**LOD Implementation Example:**

```rust
pub enum DetailLevel {
    Full,      // Full physics calculation
    Reduced,   // Larger timesteps, less frequent updates
    Minimal,   // Approximate forces, simple integration
    Culled,    // No physics updates
}

impl LODSystem {
    pub fn update_detail_levels(&mut self, camera_position: Vector3) {
        for (i, position) in self.positions.iter().enumerate() {
            let distance = (position - camera_position).norm();
            
            self.detail_levels[i] = match distance {
                d if d < 1000.0 => DetailLevel::Full,
                d if d < 5000.0 => DetailLevel::Reduced,
                d if d < 20000.0 => DetailLevel::Minimal,
                _ => DetailLevel::Culled,
            };
        }
    }
}
```

## Physics-Render Decoupling

Run physics at 30 Hz while rendering at 60 Hz for smooth visuals:

```rust
use gravwell::prelude::*;
use std::time::{Duration, Instant};

struct GameLoop {
    simulation: Simulation,
    physics_timestep: Duration,
    render_timestep: Duration,
    last_physics_update: Instant,
    last_render_update: Instant,
}

impl GameLoop {
    pub fn new() -> Result<Self> {
        Ok(Self {
            simulation: Simulation::builder()
                .integrator(SemiImplicitEuler::new())
                .forces(BarnesHut::new().theta(0.6))
                .build()?,
            physics_timestep: Duration::from_millis(33), // 30 Hz physics
            render_timestep: Duration::from_millis(16),  // 60 Hz rendering
            last_physics_update: Instant::now(),
            last_render_update: Instant::now(),
        })
    }
    
    pub fn update(&mut self) {
        let now = Instant::now();
        
        // Physics update (30 Hz)
        if now.duration_since(self.last_physics_update) >= self.physics_timestep {
            self.simulation.step();
            self.last_physics_update = now;
        }
        
        // Render update (60 Hz) with interpolation
        if now.duration_since(self.last_render_update) >= self.render_timestep {
            let interpolation_factor = now.duration_since(self.last_physics_update).as_secs_f64() 
                / self.physics_timestep.as_secs_f64();
            
            self.render_interpolated(interpolation_factor);
            self.last_render_update = now;
        }
    }
    
    fn render_interpolated(&self, t: f64) {
        // Interpolate positions between physics updates for smooth visuals
        for handle in self.simulation.active_bodies() {
            let current_pos = self.simulation.position(handle);
            let velocity = self.simulation.velocity(handle);
            let interpolated_pos = current_pos + velocity * t;
            
            // Render at interpolated position
            render_particle(interpolated_pos);
        }
    }
}
```

## GPU Acceleration

For very large systems (10,000+ particles), use GPU compute shaders:

```rust
use gravwell::prelude::*;

// Enable GPU acceleration
let sim = Simulation::builder()
    .forces(GpuBarnesHut::new())
    .gpu_device(GpuDevice::best_available()?)
    .build()?;
```

**GPU Compute Shader Example (WGSL):**

```wgsl
struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>, 
    mass: f32,
    _padding: f32,
}

struct ForceBuffer {
    forces: array<vec3<f32>>,
}

@group(0) @binding(0) var<storage, read> particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> forces: ForceBuffer;
@group(0) @binding(2) var<uniform> params: SimulationParams;

@compute @workgroup_size(64, 1, 1)
fn calculate_forces(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if i >= arrayLength(&particles) { return; }
    
    var total_force = vec3<f32>(0.0);
    let particle_i = particles[i];
    
    for (var j = 0u; j < arrayLength(&particles); j++) {
        if i == j { continue; }
        
        let particle_j = particles[j];
        let r_vec = particle_j.position - particle_i.position;
        let r_squared = dot(r_vec, r_vec);
        let r = sqrt(r_squared);
        
        let force_magnitude = params.G * particle_i.mass * particle_j.mass / r_squared;
        total_force += force_magnitude * r_vec / r;
    }
    
    forces.forces[i] = total_force;
}
```

**Performance Impact:** 10-100x speedup for large systems (> 10,000 particles).

## Spatial Culling

Remove off-screen particles from physics calculations:

```rust
use gravwell::prelude::*;

struct SpatialCuller {
    view_frustum: Frustum,
    spatial_index: SpatialHashGrid,
}

impl SpatialCuller {
    pub fn cull_particles(&self, simulation: &Simulation) -> Vec<BodyHandle> {
        let mut visible_particles = Vec::new();
        
        for handle in simulation.active_bodies() {
            let position = simulation.position(handle);
            
            if self.view_frustum.contains_point(position) {
                visible_particles.push(handle);
            }
        }
        
        visible_particles
    }
}

// Usage in game loop
let visible_particles = culler.cull_particles(&simulation);
simulation.set_active_particles(visible_particles);
```

## Benchmarking and Profiling

### Built-in Benchmarks

Run performance benchmarks to identify bottlenecks:

```bash
# Run all benchmarks
cargo bench

# Specific benchmarks
cargo bench force_calculation
cargo bench integration_step
cargo bench full_simulation

# Generate flame graphs
cargo flamegraph --bench force_calculation
```

### Custom Profiling

```rust
use gravwell::prelude::*;
use std::time::Instant;

fn profile_simulation() {
    let mut sim = Simulation::builder()
        .forces(BarnesHut::new().theta(0.5))
        .build()
        .unwrap();
    
    // Add 1000 particles
    for _ in 0..1000 {
        sim.add_random_particle();
    }
    
    // Profile physics step
    let start = Instant::now();
    for _ in 0..100 {
        sim.step();
    }
    let duration = start.elapsed();
    
    println!("100 steps took: {:?}", duration);
    println!("Average per step: {:?}", duration / 100);
    println!("Estimated FPS: {:.1}", 1.0 / (duration.as_secs_f64() / 100.0));
}
```

## Performance Tuning Checklist

### For 30 FPS (Basic Performance)

- [ ] Use appropriate algorithm (Direct for N < 1000, Barnes-Hut for larger)
- [ ] Enable compiler optimizations (`cargo build --release`)
- [ ] Choose stable integrator (Semi-implicit Euler or Velocity Verlet)

### For 60 FPS (High Performance)

- [ ] Enable SIMD optimizations
- [ ] Use multi-threading (Rayon)
- [ ] Implement physics-render decoupling
- [ ] Tune Barnes-Hut theta parameter (try 0.6-0.7)
- [ ] Pre-allocate memory pools

### For 100+ FPS (Maximum Performance)

- [ ] Implement LOD system
- [ ] Add spatial culling
- [ ] Use GPU acceleration for large systems
- [ ] Optimize memory layout further
- [ ] Consider async physics thread

### For Large Systems (10,000+ particles)

- [ ] Use GPU compute shaders
- [ ] Implement hierarchical LOD
- [ ] Add aggressive culling strategies
- [ ] Consider distributed computing

## Platform-Specific Optimizations

### Windows

```toml
[profile.release]
target-cpu = "native"  # Use CPU-specific optimizations
lto = "fat"           # Link-time optimization
codegen-units = 1     # Single codegen unit for better optimization
```

### Linux  

```bash
# Use performance governor for benchmarking
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Set CPU affinity for physics thread
taskset -c 0-7 cargo run --release
```

### WebAssembly

```rust
// Enable SIMD for WASM
#[cfg(target_arch = "wasm32")]
use std::arch::wasm32::*;

// Use smaller particle counts for browser targets
#[cfg(target_arch = "wasm32")]
const MAX_PARTICLES: usize = 1000;

#[cfg(not(target_arch = "wasm32"))]
const MAX_PARTICLES: usize = 10000;
```

## Common Performance Issues

### Issue: Low FPS with Direct Gravity

**Symptom:** Frame rate drops significantly with > 500 particles
**Solution:** Switch to Barnes-Hut algorithm

### Issue: High Memory Usage

**Symptom:** Memory usage grows during simulation
**Solution:** Pre-allocate particle capacity, reuse temporary vectors

### Issue: Inconsistent Frame Times

**Symptom:** Frame rate varies widely, stuttering
**Solution:** Implement physics-render decoupling, use fixed timesteps

### Issue: Poor Multi-threading Performance  

**Symptom:** Adding threads doesn't improve performance
**Solution:** Check for false sharing, increase workload per thread

### Issue: SIMD Not Working

**Symptom:** No performance improvement with SIMD enabled
**Solution:** Verify CPU support, check alignment requirements, profile assembly output

This performance guide provides concrete strategies for optimizing Gravwell
across different hardware configurations and use cases, from basic 30 FPS
gameplay to high-performance scientific computing.
