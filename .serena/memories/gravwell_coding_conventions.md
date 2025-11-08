# Gravwell Coding Conventions and Standards

## Rust Language Conventions

### Naming Standards
- **Types**: PascalCase (`ParticleSet`, `SimulationBuilder`, `VelocityVerlet`)
- **Functions**: snake_case (`calculate_forces`, `step_simulation`, `add_particle`)
- **Variables**: snake_case (`particle_count`, `timestep`, `gravity_constant`)
- **Constants**: SCREAMING_SNAKE_CASE (`G`, `SOLAR_MASS`, `AU`, `EARTH_MASS`)
- **Modules**: snake_case (`integrators`, `force_calculation`, `spatial`)
- **Traits**: PascalCase (`Integrator`, `ForceCalculator`, `CollisionHandler`)

### Physics-Specific Naming
- **Physical Quantities**: Include SI units in names when helpful (`velocity_ms`, `distance_m`)
- **Algorithms**: Descriptive names (`VelocityVerlet`, `BarnesHut`, `DirectGravity`)
- **Error Types**: Domain-specific (`SimulationError`, `IntegrationError`, `GpuError`)
- **Builders**: Fluent API style (`SimulationBuilder`, `BodyBuilder`)
- **Handles**: Resource management (`BodyHandle`, `SimulationHandle`)

### Code Organization Patterns

#### Module Structure
```rust
// Physics algorithm-based grouping
pub mod integrators {
    pub mod verlet;      // Velocity Verlet implementation
    pub mod leapfrog;    // Leapfrog implementation
    pub mod rk4;         // Runge-Kutta 4th order
}

pub mod forces {
    pub mod direct;      // O(N²) direct calculation
    pub mod barnes_hut;  // O(N log N) tree algorithm
    pub mod gpu;         // GPU-accelerated algorithms
}
```

#### Trait Definition Standards
```rust
// Well-documented traits with clear contracts
/// Numerical integrator for advancing particle positions and velocities
pub trait Integrator: Send + Sync + Clone {
    /// Advance the simulation by one timestep
    fn step(&mut self, dt: f64, particles: &mut ParticleSet, forces: &[Vector3]);
    
    /// Returns true if this integrator preserves symplectic structure
    fn is_symplectic(&self) -> bool;
    
    /// Human-readable name for benchmarking and debugging
    fn name(&self) -> &'static str;
}
```

## Error Handling Standards

### Structured Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum GravwellError {
    #[error("Invalid body handle: {0:?}")]
    InvalidBodyHandle(BodyHandle),
    
    #[error("Simulation configuration invalid: {reason}")]
    InvalidConfiguration { reason: String },
    
    #[error("Numerical instability detected: {details}")]
    NumericalInstability { details: String },
    
    #[error("GPU operation failed: {0}")]
    GpuError(#[from] GpuOperationError),
}
```

### Error Propagation Rules
- **Library Code**: Never panic, always return `Result<T, GravwellError>`
- **Examples**: May use `.unwrap()` for brevity in demonstration code
- **Tests**: Use `assert!` and `.unwrap()` for clear failure points
- **Validation**: Proper input validation on all public API functions
- **GPU Fallback**: Graceful degradation when GPU operations fail

## Performance Coding Standards

### SIMD Optimization Patterns
```rust
// Use portable SIMD when possible
use std::simd::{f64x4, SimdFloat};

pub fn calculate_forces_simd(
    positions: &[Vector3],
    masses: &[f64],
    forces: &mut [Vector3],
) {
    // Process particles in groups of 4 for SIMD efficiency
    for chunk in positions.chunks_exact(4) {
        let pos_x = f64x4::from_array([chunk[0].x, chunk[1].x, chunk[2].x, chunk[3].x]);
        // ... vectorized computation
    }
}
```

### Memory Layout Standards
```rust
// Structure-of-Arrays for cache efficiency
pub struct ParticleSet {
    positions: Vec<Vector3>,     // Contiguous for SIMD
    velocities: Vec<Vector3>,    // Contiguous for SIMD
    masses: Vec<f64>,           // Contiguous for SIMD
    radii: Vec<f64>,            // Contiguous for SIMD
    active: Vec<bool>,          // Active particle tracking
}

// Avoid Array-of-Structures (cache-unfriendly)
// struct Particle { position: Vector3, velocity: Vector3, mass: f64 }
```

### Zero-Allocation Simulation Loops
```rust
impl Simulation {
    pub fn step(&mut self) {
        // Pre-allocated buffers, no allocations during simulation
        self.force_calculator.calculate_forces(&self.particles, &mut self.force_buffer);
        self.integrator.step(self.timestep, &mut self.particles, &self.force_buffer);
        self.time += self.timestep;
    }
}
```

## Documentation Standards

### API Documentation Requirements
```rust
/// Calculate gravitational forces between all particle pairs using direct summation.
/// 
/// This is an O(N²) algorithm suitable for systems with fewer than 1,000 particles.
/// For larger systems, consider using [`BarnesHut`] or [`FastMultipole`] algorithms.
/// 
/// # Performance
/// 
/// - Without SIMD: ~1,000 particles @ 30 FPS
/// - With SIMD: ~1,000 particles @ 60 FPS  
/// - Memory usage: O(N) for force storage
/// 
/// # Example
/// 
/// ```rust
/// use gravwell::prelude::*;
/// 
/// let direct_forces = DirectGravity::new()
///     .with_softening(1e-3)  // Prevent singularities
///     .with_simd(true);      // Enable vectorization
/// 
/// let mut sim = Simulation::builder()
///     .forces(direct_forces)
///     .build()?;
/// ```
/// 
/// # Scientific Accuracy
/// 
/// Direct force calculation provides exact results (within floating-point precision)
/// and is used as the reference for validating approximate algorithms.
pub struct DirectGravity {
    // ...
}
```

### Physics Algorithm Documentation
- **Complexity**: Big-O notation for time and space
- **Accuracy**: Expected numerical precision and limitations
- **Performance**: Benchmark results and scaling behavior
- **Scientific Context**: References to papers and established codes
- **Example Usage**: Complete, runnable examples in doctests

## Testing Integration Standards

### Unit Test Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_force_calculation_two_body() {
        // ARRANGE: Set up known test case
        let pos1 = Vector3::new(0.0, 0.0, 0.0);
        let pos2 = Vector3::new(1.0, 0.0, 0.0);  // 1 meter apart
        let mass1 = 1.0;  // 1 kg
        let mass2 = 1.0;  // 1 kg
        
        // ACT: Calculate force
        let force = calculate_gravitational_force(pos1, pos2, mass1, mass2);
        
        // ASSERT: Verify against analytical solution
        let expected_magnitude = G * mass1 * mass2 / 1.0; // F = GMm/r²
        assert_relative_eq!(force.norm(), expected_magnitude, epsilon = 1e-12);
    }
}
```

### Benchmark Integration
```rust
// Benchmarks co-located with performance-critical code
#[cfg(test)]
mod benches {
    use criterion::{black_box, Criterion};
    
    pub fn benchmark_direct_gravity(c: &mut Criterion) {
        c.bench_function("direct_gravity_1000_particles", |b| {
            let particles = create_test_particle_set(1000);
            let mut forces = vec![Vector3::zeros(); 1000];
            let calculator = DirectGravity::new();
            
            b.iter(|| {
                calculator.calculate_forces(black_box(&particles), black_box(&mut forces));
            });
        });
    }
}
```

## Git Commit Standards

### Commit Message Format
```
type(scope): description

feat(forces): add GPU Barnes-Hut algorithm with WebGPU compute shaders
perf(simd): vectorize direct gravity calculation with AVX-512
fix(verlet): correct energy drift in long-duration simulations
docs(api): add comprehensive examples for force calculator traits
test(validation): add Kepler orbit accuracy tests
refactor(core): simplify particle handle management
chore(deps): update nalgebra to 0.32.3 for SIMD improvements
```

### Type Categories
- **feat**: New physics algorithm or API feature
- **perf**: Performance optimization or SIMD enhancement
- **fix**: Bug fix or numerical accuracy improvement
- **docs**: Documentation, examples, or API reference updates
- **test**: Test additions, improvements, or validation enhancements
- **refactor**: Code restructuring without behavior changes
- **chore**: Maintenance, dependency updates, or tooling changes

### Scope Guidelines
- **forces**: Force calculation algorithms (direct, barnes_hut, fmm, gpu)
- **integrators**: Numerical integration methods (verlet, leapfrog, rk4, ias15)
- **core**: Core traits, types, and fundamental abstractions
- **simd**: SIMD optimizations and vectorization
- **gpu**: GPU acceleration and WebGPU features
- **validation**: Scientific validation and accuracy testing
- **api**: Public API surface and breaking changes
- **examples**: Example code and demonstrations

## Code Review Guidelines

### Physics Correctness Checklist
- [ ] Algorithm implementation matches published references
- [ ] Energy conservation verified for symplectic integrators
- [ ] Numerical stability under extreme conditions
- [ ] Proper handling of edge cases (zero distance, infinite mass)
- [ ] Unit tests with analytical solution comparisons

### Performance Review Checklist
- [ ] No unnecessary allocations in simulation loops
- [ ] SIMD-friendly data layout (Structure-of-Arrays)
- [ ] Benchmark results meet performance targets
- [ ] Memory usage analysis for large particle counts
- [ ] GPU algorithm correctness vs CPU reference

### API Design Review Checklist
- [ ] Trait-based design for extensibility
- [ ] Builder pattern for complex configuration
- [ ] Comprehensive error handling with structured types
- [ ] Zero-cost abstractions with compile-time optimization
- [ ] Backward compatibility for API changes