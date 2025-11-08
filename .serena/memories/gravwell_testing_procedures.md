# Gravwell Testing Procedures and Standards

## Testing Philosophy

Gravwell follows a comprehensive testing strategy that ensures both **correctness** and **performance** for physics simulations:

1. **Unit Tests**: Individual component validation
2. **Integration Tests**: Component interaction testing  
3. **Physics Validation**: Accuracy against analytical solutions
4. **Performance Tests**: Benchmark-driven development
5. **Property Tests**: Mathematical invariant verification

## Test Categories and Organization

### Unit Tests (Co-located with source)
```rust
// In src/integrators/verlet.rs
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_velocity_verlet_harmonic_oscillator() {
        // Test against analytical solution
    }
}
```

### Integration Tests (`tests/` directory)
- **`scientific_validation.rs`**: Comprehensive physics validation
- **`physics_validation.rs`**: General physics correctness
- **`energy_conservation_tests.rs`**: Long-term energy conservation
- **`momentum_conservation_tests.rs`**: Conservation law verification
- **`kepler_orbit_tests.rs`**: Orbital mechanics accuracy
- **`gpu_barnes_hut_tests.rs`**: GPU algorithm validation
- **`figure_eight_tests.rs`**: Complex system stability

### Benchmark Tests (`benches/` directory)
- **`force_calculation.rs`**: Algorithm performance comparison
- **`integration_step.rs`**: Integrator speed benchmarks
- **`full_simulation.rs`**: End-to-end performance
- **`comprehensive_performance.rs`**: Multi-algorithm analysis

## Physics Validation Standards

### Energy Conservation Testing
```rust
#[test]
fn test_two_body_energy_conservation() {
    let mut sim = setup_earth_sun_system();
    let initial_energy = sim.total_energy();
    
    // Simulate 100 orbital periods
    for _ in 0..100_000 {
        sim.step();
    }
    
    let final_energy = sim.total_energy();
    let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();
    
    // Symplectic integrators should conserve energy to within 1e-10
    assert!(energy_error < 1e-10, 
        "Energy drift too large: {:.3e}", energy_error);
}
```

### Accuracy Thresholds by Test Type
- **Energy Conservation**: < 1e-10 relative error (symplectic integrators)
- **Momentum Conservation**: < 1e-12 relative error
- **Kepler Orbit Accuracy**: < 1e-8 relative error over 100 periods
- **Force Calculation**: < 1e-6 relative error vs analytical solutions
- **Integration Accuracy**: Method-dependent (Verlet: 1e-6, RK4: 1e-12)

### Scientific Validation Commands
```bash
# Run all physics validation tests
cargo test scientific_validation -- --nocapture

# Specific validation categories
cargo test energy_conservation -- --nocapture
cargo test momentum_conservation -- --nocapture
cargo test kepler_orbit -- --nocapture
cargo test integrator_accuracy -- --nocapture

# Long-term stability tests
cargo test figure_eight -- --nocapture
cargo test three_body_system -- --nocapture
```

## Performance Testing Standards

### Benchmark Requirements
- **60 FPS Target**: 1,000 particles minimum for game mode
- **Scientific Mode**: 10,000 particles @ 1 FPS minimum
- **GPU Mode**: 50,000+ particles with WebGPU acceleration
- **Energy Efficiency**: < 1e-12 energy drift over simulation time

### Performance Validation Commands
```bash
# Run all benchmarks
cargo bench

# Specific performance categories
cargo bench force_calculation      # Algorithm comparison
cargo bench integration_step       # Integrator speed
cargo bench 60fps_target          # Real-time performance
cargo bench simd_operations       # Vectorization efficiency

# Generate performance reports
cargo bench -- --output-format html

# Profile critical paths
cargo flamegraph --bench force_calculation
```

### Performance Regression Detection
```bash
# Set baseline before changes
cargo bench --save-baseline before

# After implementation changes
cargo bench --save-baseline after

# Compare performance
cargo bench --baseline before --baseline after
```

## Property-Based Testing

### Mathematical Invariant Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_force_symmetry(
        pos1 in prop::array::uniform3(-1000.0..1000.0),
        pos2 in prop::array::uniform3(-1000.0..1000.0),
        mass1 in 1.0..100.0,
        mass2 in 1.0..100.0
    ) {
        // Force on body 1 from body 2 should equal negative force on body 2 from body 1
        let force_12 = calculate_gravitational_force(pos1, pos2, mass1, mass2);
        let force_21 = calculate_gravitational_force(pos2, pos1, mass2, mass1);
        
        prop_assert!((force_12 + force_21).norm() < 1e-10);
    }
}
```

### Property Test Categories
- **Conservation Laws**: Energy, momentum, angular momentum
- **Symmetry Properties**: Force reciprocity, time reversibility
- **Scaling Laws**: Force magnitude with distance/mass
- **Numerical Stability**: No NaN/infinity propagation

## Test Automation and CI

### Pre-commit Test Requirements
```bash
# Must pass for all commits
cargo test                    # All unit and integration tests
cargo test --release         # Performance-mode testing
cargo test --doc             # Documentation examples
cargo clippy -- -D warnings  # No linting warnings
cargo fmt --check           # Code formatting compliance
```

### Performance Monitoring in CI
```bash
# Automated performance regression detection
cargo bench --baseline main
cargo bench --baseline feature-branch
# Fail CI if performance degrades > 5%
```

### Cross-Platform Test Matrix
- **Platforms**: Linux x86_64, Windows x86_64, macOS x86_64, macOS ARM64
- **Features**: All feature flag combinations
- **Targets**: Native + WebAssembly
- **Modes**: Debug + Release builds

## Test Data Management

### Deterministic Testing
```rust
// Use fixed seeds for reproducible physics
use rand::{SeedableRng, rngs::StdRng};

#[test]
fn test_random_system_stability() {
    let mut rng = StdRng::seed_from_u64(42);  // Fixed seed
    let particles = generate_random_system(&mut rng, 100);
    
    // Test is now deterministic and reproducible
    assert_simulation_stability(particles);
}
```

### Reference Data Validation
- Comparison with REBOUND N-body integrator
- Analytical solutions for two-body problems
- Known stable configurations (figure-eight orbit)
- Published benchmark results from literature

### Test Coverage Standards
- **Core Physics**: 95% minimum coverage
- **Public API**: 90% minimum coverage  
- **Error Handling**: 85% minimum coverage
- **Mathematical Properties**: 100% coverage

## Debugging and Diagnostics

### Physics-Specific Debugging
```bash
# Debug force calculations with detailed logging
RUST_LOG=gravwell::forces=debug cargo test force_calculation

# Trace integration steps
RUST_LOG=gravwell::integrator=trace cargo test energy_conservation

# Profile memory usage
cargo test --release
valgrind --tool=massif cargo test --release
```

### GPU Testing and Validation
```bash
# Test GPU algorithm correctness
cargo test gpu_barnes_hut --features gpu -- --nocapture

# Compare GPU vs CPU results
cargo test gpu_vs_cpu_validation --features gpu

# GPU performance benchmarks
cargo bench gpu_acceleration --features gpu
```

### Error Analysis Tools
```bash
# Check for numerical instabilities
cargo test --release -- --nocapture | grep -i "nan\|inf"

# Memory leak detection
cargo test --release
valgrind --tool=memcheck --leak-check=full cargo test
```