# Gravwell Project Architecture and Code Organization

## Module Structure

### Core Library (`src/`)

```plaintext
src/
├── lib.rs              # Public API surface and re-exports
├── prelude.rs          # Convenient imports module  
├── error.rs            # Error types with thiserror
├── types.rs            # Core type definitions
├── builder.rs          # SimulationBuilder pattern
├── core/               # Core abstractions (no_std compatible)
│   ├── integrator.rs   # Integrator trait and implementations
│   ├── forces.rs       # ForceCalculator trait
│   ├── particle.rs     # ParticleSet and Body types
│   └── math.rs         # Vector math utilities
├── integrators/        # Numerical integration methods
│   ├── verlet.rs       # Velocity Verlet (symplectic)
│   ├── leapfrog.rs     # Leapfrog (symplectic)
│   ├── rk4.rs          # Runge-Kutta 4th order
│   └── ias15.rs        # IAS15 adaptive 15th order
├── forces/             # Force calculation algorithms
│   ├── direct.rs       # Direct O(N²) calculation
│   ├── barnes_hut.rs   # Barnes-Hut O(N log N)
│   ├── fmm.rs          # Fast Multipole Method O(N)
│   └── gpu_barnes_hut.rs # GPU Barnes-Hut with WebGPU
├── simd/               # SIMD optimizations
│   ├── avx.rs          # AVX/AVX2 implementations
│   └── portable.rs     # Portable SIMD
├── spatial/            # Spatial data structures
│   ├── octree.rs       # Octree for Barnes-Hut
│   └── morton.rs       # Morton coding for GPU
├── collision/          # Collision detection (optional)
│   ├── spatial_hash.rs # Spatial hash grid
│   └── aabb.rs         # AABB tree
└── utils/              # Utility functions
    ├── constants.rs    # Physical constants
    └── validation.rs   # Analytical solution validation
```

## Core Design Patterns

### Trait-Based Architecture

```rust
// Core physics traits for extensibility
pub trait Integrator {
    fn step(&mut self, dt: f64, particles: &mut ParticleSet, forces: &[Vector3]);
    fn is_symplectic(&self) -> bool;
}

pub trait ForceCalculator {
    fn calculate_forces(&self, particles: &ParticleSet, forces: &mut [Vector3]);
    fn complexity(&self) -> Complexity;
}
```

### Builder Pattern

```rust
// Type-safe simulation construction
let sim = Simulation::builder()
    .integrator(VelocityVerlet::new())
    .forces(BarnesHut::new().theta(0.5))
    .timestep(0.01)
    .gravity_constant(G)
    .build()?;
```

### Data Layout (Structure-of-Arrays)

```rust
// SIMD-friendly memory layout
pub struct ParticleSet {
    positions: Vec<Vector3>,    // Contiguous for vectorization
    velocities: Vec<Vector3>,   // Contiguous for vectorization
    masses: Vec<f64>,           // Contiguous for vectorization
    active: Vec<bool>,          // Active particle tracking
}
```

## Testing Architecture

### Test Organization

```
tests/
├── scientific_validation.rs      # Energy conservation, momentum
├── physics_validation.rs         # General physics correctness
├── kepler_validation.rs          # Orbital mechanics accuracy
├── energy_conservation_tests.rs  # Long-term stability
├── momentum_conservation_tests.rs # Conservation laws
├── gpu_barnes_hut_tests.rs       # GPU algorithm validation
├── kepler_orbit_tests.rs         # Analytical solutions
├── figure_eight_tests.rs         # Complex system validation
└── validation/                   # Additional validation tests
```

### Benchmark Organization

```
benches/
├── full_simulation.rs            # End-to-end performance
├── comprehensive_performance.rs  # Multi-algorithm comparison
├── force_calculation.rs          # Force algorithm benchmarks
├── integration_step.rs           # Integrator performance
└── spatial_culling_performance.rs # Optimization benchmarks
```

### Example Organization

```
examples/
├── basic_usage.rs                # Getting started
├── solar_system.rs               # Realistic simulation
├── binary_orbit.rs               # Two-body problem
├── performance_test.rs           # Performance validation
├── gpu_acceleration.rs           # GPU usage example
├── simd_optimization.rs          # SIMD demonstration
└── scientific_computing.rs       # Research use case
```

## Memory Management Strategy

### Particle Handle System

```rust
// Stable handles for particle references
pub struct BodyHandle {
    index: usize,
    generation: u32,  // Prevents use-after-free
}
```

### Memory Pool Allocation

- Pre-allocated capacity for particle arrays
- Reuse of temporary computation buffers
- Zero-allocation simulation loops (after initialization)

### GPU Memory Management

- Efficient buffer uploads/downloads
- Staging buffers for large datasets
- Memory-mapped GPU buffers when available

## Performance Optimization Layers

### SIMD Vectorization

- Auto-vectorized force calculations
- Manual SIMD for critical paths
- Fallback to scalar operations

### Parallel Processing

- Data parallelism with rayon
- Thread-local storage for temporary buffers
- NUMA-aware memory allocation

### GPU Acceleration

- WebGPU compute shaders
- Asynchronous GPU execution
- CPU-GPU overlap for maximum throughput

## Error Handling Strategy

### Structured Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum GravwellError {
    #[error("Invalid body handle: {0:?}")]
    InvalidBodyHandle(BodyHandle),
    
    #[error("Numerical instability detected")]
    NumericalInstability,
    
    #[error("GPU error: {0}")]
    GpuError(String),
}
```

### Result Propagation

- All fallible operations return `Result<T, GravwellError>`
- No panics in library code (only in examples/tests)
- Graceful degradation for GPU failures
