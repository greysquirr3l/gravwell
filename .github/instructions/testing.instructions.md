---
applyTo: "**/test/**/*test*"
description: "Gravwell gravity simulation testing patterns and standards for comprehensive physics validation"
references:
  - rust_testing_guide: "https://doc.rust-lang.org/book/ch11-00-testing.html"
  - criterion_docs: "https://docs.rs/criterion/"
  - proptest_docs: "https://docs.rs/proptest/"
  - physics_validation: "https://rebound.readthedocs.io/en/latest/"
---

# Gravwell Physics Testing Standards and Patterns

## Core Testing Philosophy

### Test Behavior, Not Implementation

- Focus on what the code does, not how it does it
- Test public interfaces and contracts
- Avoid testing private methods directly
- Ensure tests remain valid when refactoring implementation

### Coverage Prioritization

1. **Critical Business Logic**: 100% coverage required
2. **Public APIs**: 90% coverage minimum
3. **Integration Points**: 85% coverage minimum
4. **Utility Functions**: 80% coverage minimum

## Test Organization Standards

### File Structure Conventions

```text
Rust Testing Structure:
- Unit tests: `src/module.rs` with `#[cfg(test)] mod tests`
- Integration tests: `tests/integration_test_name.rs`
- Physics validation: `tests/physics/energy_conservation.rs`
- Benchmarks: `benches/physics_performance.rs`
- Examples as tests: `examples/solar_system.rs` (with `cargo test --example`)
```

### Test Categories

- **Unit Tests**: Single component/function testing
- **Integration Tests**: Component interaction testing
- **Contract Tests**: API/Interface contract validation
- **End-to-End Tests**: Complete user workflow testing

## Language-Agnostic Test Patterns

### Arrange-Act-Assert (AAA) Pattern

```rust
#[test]
fn should_conserve_energy_during_orbit() {
    // ARRANGE: Set up test simulation with known initial conditions
    let mut sim = Simulation::builder()
        .integrator(VelocityVerlet::new())
        .gravity(DirectGravity::new())
        .timestep(0.01)
        .add_body(create_sun())
        .add_body(create_earth_at_aphelion())
        .build()
        .unwrap();
    
    let initial_energy = sim.total_energy();

    // ACT: Execute one orbital period
    let steps_per_orbit = (EARTH_ORBITAL_PERIOD / sim.timestep()) as usize;
    for _ in 0..steps_per_orbit {
        sim.step().unwrap();
    }

    // ASSERT: Verify energy conservation
    let final_energy = sim.total_energy();
    let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();
    
    assert!(energy_drift < 1e-6, "Energy drift too large: {:.2e}", energy_drift);
}
```

### Table-Driven Test Pattern

```rust
#[test]
fn test_integrator_accuracy_multiple_cases() {
    let test_cases = vec![
        ("velocity_verlet", Box::new(VelocityVerlet::new()) as Box<dyn Integrator>, 1e-6),
        ("leapfrog", Box::new(Leapfrog::new()) as Box<dyn Integrator>, 1e-6),
        ("rk4", Box::new(RungeKutta4::new()) as Box<dyn Integrator>, 1e-8),
        ("ias15", Box::new(IAS15::new()) as Box<dyn Integrator>, 1e-12),
    ];

    for (name, integrator, expected_accuracy) in test_cases {
        // Test each integrator with the same scenario
        let mut sim = create_kepler_problem_simulation(integrator);
        let initial_energy = sim.total_energy();
        
        // Simulate one orbit
        simulate_one_orbit(&mut sim);
        
        let final_energy = sim.total_energy();
        let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();
        
        assert!(
            energy_drift < expected_accuracy,
            "{} integrator: energy drift {:.2e} exceeds expected {:.2e}",
            name, energy_drift, expected_accuracy
        );
    }
}
```

### Test Data Factory Pattern

```rust
pub struct PhysicsTestFactory;

impl PhysicsTestFactory {
    /// Create a Sun-Earth system at aphelion (most distant point)
    pub fn create_sun_earth_system() -> Simulation<VelocityVerlet, DirectGravity> {
        Simulation::builder()
            .integrator(VelocityVerlet::new())
            .gravity(DirectGravity::new())
            .timestep(3600.0) // 1 hour
            .add_body(Self::create_sun())
            .add_body(Self::create_earth_at_aphelion())
            .build()
            .unwrap()
    }
    
    pub fn create_sun() -> Body {
        Body {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            mass: Mass::SOLAR_MASS,
            radius: Some(6.96e8),
            name: Some("Sun".to_string()),
        }
    }
    
    pub fn create_earth_at_aphelion() -> Body {
        let aphelion_distance = 1.521e11; // meters
        let aphelion_velocity = 29290.0; // m/s
        
        Body {
            position: Vector3::new(aphelion_distance, 0.0, 0.0),
            velocity: Vector3::new(0.0, aphelion_velocity, 0.0),
            mass: Mass::EARTH_MASS,
            radius: Some(6.371e6),
            name: Some("Earth".to_string()),
        }
    }
    
    pub fn create_binary_system(separation: f64, eccentricity: f64) -> Simulation<VelocityVerlet, DirectGravity> {
        // Create binary star system with specified parameters
        todo!("Implement binary system factory")
    }
    
    pub fn create_n_body_cluster(n: usize, total_mass: Mass, radius: f64) -> Vec<Body> {
        // Create N-body cluster with specified properties
        todo!("Implement N-body cluster factory")
    }
}
```

## Framework-Specific Patterns

### Testing Framework Templates

#### Rust (cargo test)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    use proptest::prelude::*;

    #[test]
    fn test_create_user_success() {
        // Arrange
        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
        };

        // Act
        let result = create_user(request);

        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_create_user_async_success() {
        let request = CreateUserRequest {
            email: "async@example.com".to_string(),
            name: "Async User".to_string(),
        };

        let result = create_user_async(request).await;
        assert!(result.is_ok());
    }

    // Property-based testing with proptest
    proptest! {
        #[test]
        fn test_email_validation(email in "[a-z0-9._%+-]+@[a-z0-9.-]+\\.[a-z]{2,}") {
            let result = validate_email(&email);
            prop_assert!(result.is_ok());
        }
    }

    // Benchmark tests
    #[bench]
    fn bench_user_creation(b: &mut test::Bencher) {
        b.iter(|| {
            let request = CreateUserRequest {
                email: "bench@example.com".to_string(),
                name: "Bench User".to_string(),
            };
            create_user(request)
        });
    }
}
```

## Mocking and Test Doubles

### Mock Strategy

```text
- Mock external dependencies (databases, APIs, file systems)
- Use real objects for value objects and simple data structures
- Create test doubles for complex collaborators
- Verify behavior on mocks, state on real objects
```

### Mock Patterns

```rust
// Rust mocking with mockall crate
use mockall::{automock, predicate::*};

#[automock]
trait ForceCalculator {
    fn calculate_forces(
        &self,
        particles: &ParticleSet,
        forces: &mut [Vector3<f64>],
    ) -> Result<(), SimulationError>;
}

#[test]
fn test_simulation_with_mock_forces() {
    let mut mock_force_calc = MockForceCalculator::new();
    
    // Set up expectations
    mock_force_calc
        .expect_calculate_forces()
        .times(1)
        .withf(|particles, _forces| particles.count == 2)
        .returning(|_particles, forces| {
            // Simulate specific force values for testing
            forces[0] = Vector3::new(1e20, 0.0, 0.0);
            forces[1] = Vector3::new(-1e20, 0.0, 0.0);
            Ok(())
        });
    
    // Use mock in simulation
    let mut sim = Simulation::new(
        VelocityVerlet::new(),
        Box::new(mock_force_calc),
        create_test_particles(),
        0.01,
    );
    
    // Test that simulation uses the mocked forces
    sim.step().unwrap();
    
    // Verify specific behavior based on mocked forces
    let positions = sim.get_positions();
    assert!(positions[0].x < positions[1].x); // Particles should move apart
}
```

## Async and Concurrency Testing

### Async Test Patterns

```text
- Use proper async/await patterns for asynchronous code
- Test timeout scenarios with appropriate test timeouts
- Verify async error handling and propagation
- Test concurrent execution patterns safely
```

### Concurrency Test Strategies

```text
- Use deterministic testing approaches
- Control timing with synchronization primitives
- Test race conditions with multiple iterations
- Verify thread safety with concurrent operations
```

## Integration Test Patterns

### Database Testing

```text
- Use test databases or containers
- Implement proper test data setup and teardown
- Test transaction boundaries and rollbacks
- Verify database constraints and relationships
```

### API Testing

```text
- Test complete request/response cycles
- Verify error responses and status codes
- Test authentication and authorization
- Validate request/response schemas
```

### External Service Testing

```text
- Use service virtualization or contract testing
- Implement circuit breaker and retry logic testing
- Test failure scenarios and degraded modes
- Verify service contract compliance
```

## Performance and Load Testing

### Performance Test Types

```text
- Load Testing: Normal expected load
- Stress Testing: Beyond normal capacity
- Spike Testing: Sudden load increases
- Volume Testing: Large amounts of data
- Endurance Testing: Extended periods
```

### Performance Assertions

```text
- Response time thresholds
- Throughput requirements
- Resource utilization limits
- Memory leak detection
- Database query performance
```

## Security Testing Patterns

### Security Test Categories

```text
- Input validation and sanitization
- Authentication and authorization
- Session management
- Data encryption and protection
- SQL injection and XSS prevention
```

### Security Assertions

```text
- Verify access control enforcement
- Test input boundary conditions
- Validate error message information leakage
- Check for sensitive data exposure
- Test rate limiting and DOS protection
```

## Test Maintenance and Quality

### Test Code Quality

```text
- Apply same coding standards as production code
- Use descriptive test names that explain the scenario
- Keep tests independent and isolated
- Minimize test setup complexity
- Refactor tests when production code changes
```

### Test Documentation

```text
- Document complex test scenarios
- Explain non-obvious test data choices
- Maintain test suite documentation
- Document test environment requirements
- Keep test coverage reports updated
```

## Continuous Integration Integration

### CI/CD Test Strategy

```text
- Unit tests: Run on every commit
- Integration tests: Run on pull requests
- E2E tests: Run on staging deployments
- Performance tests: Run on release candidates
- Security tests: Run on security-sensitive changes
```

### Test Reporting

```text
- Generate test coverage reports
- Track test execution trends
- Report test failure patterns
- Monitor test suite execution time
- Maintain test quality metrics
```

---

## Testing Framework Customization

### Quick Start Commands

```bash
# Rust Physics Testing Commands
# Core Testing
cargo test                  # Run all tests
cargo test --release        # Run tests in release mode (for performance)
cargo test -- --nocapture   # Run tests with output
cargo bench                 # Run criterion benchmarks

# Physics Validation
cargo test energy_conservation      # Test energy conservation
cargo test integrator_accuracy      # Test numerical integrators
cargo test force_calculation        # Test force algorithms
cargo test collision_detection      # Test collision systems

# Performance Testing
cargo bench -- --baseline new       # Set new performance baseline
cargo flamegraph --bench force_calculation  # Profile benchmarks
RUST_LOG=gravwell::forces=debug cargo test  # Debug force calculations


```

### IDE Integration

```text
VS Code Rust Library Development Testing Setup:
- Configure rust-analyzer for test discovery and inline test results
- Set up Code Coverage visualization with tarpaulin extension
- Configure test debugging with LLDB for complex physics simulations
- Set up criterion benchmark visualization and trend analysis
- Configure tracing integration for physics debugging and performance analysis
- Set up property testing with proptest for mathematical validation
- Configure documentation testing for API examples
```

---

## 🔬 Gravwell Physics Testing Patterns

### Physics Simulation Testing

#### Energy Conservation Testing
```rust
#[cfg(test)]
mod energy_conservation_tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_two_body_circular_orbit_energy_conservation() {
        let mut sim = Simulation::builder()
            .integrator(VelocityVerlet::new())
            .gravity(DirectGravity::new())
            .timestep(0.01)
            .add_body(Body {
                position: Vector3::new(0.0, 0.0, 0.0),
                velocity: Vector3::new(0.0, 0.0, 0.0),
                mass: Mass::SOLAR_MASS,
                radius: Some(6.96e8),
                name: Some("Sun".to_string()),
            })
            .add_body(Body {
                position: Vector3::new(1.496e11, 0.0, 0.0), // 1 AU
                velocity: Vector3::new(0.0, 29780.0, 0.0),  // Orbital velocity
                mass: Mass::EARTH_MASS,
                radius: Some(6.371e6),
                name: Some("Earth".to_string()),
            })
            .build()
            .unwrap();

        let initial_energy = sim.total_energy();
        
        // Simulate for one orbital period (365.25 days * 24 hours * 3600 seconds)
        let orbital_period_seconds = 365.25 * 24.0 * 3600.0;
        let steps = (orbital_period_seconds / sim.timestep) as usize;
        
        for _ in 0..steps {
            sim.step().unwrap();
        }
        
        let final_energy = sim.total_energy();
        let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();
        
        // Energy should be conserved to within 1e-6 for symplectic integrators
        assert!(energy_drift < 1e-6, 
            "Energy drift too large: {:.3e}, initial: {:.3e}, final: {:.3e}",
            energy_drift, initial_energy, final_energy);
    }

    #[test]
    fn test_three_body_figure_eight_stability() {
        // Famous three-body figure-eight solution (Moore, 1993)
        let mut sim = Simulation::builder()
            .integrator(VelocityVerlet::new())
            .gravity(DirectGravity::new())
            .timestep(0.001) // Smaller timestep for stability
            .add_body(Body {
                position: Vector3::new(-1.0, 0.0, 0.0),
                velocity: Vector3::new(0.347111, 0.532728, 0.0),
                mass: Mass::new(1.0).unwrap(),
                radius: None,
                name: Some("Body1".to_string()),
            })
            .add_body(Body {
                position: Vector3::new(1.0, 0.0, 0.0),
                velocity: Vector3::new(0.347111, 0.532728, 0.0),
                mass: Mass::new(1.0).unwrap(),
                radius: None,
                name: Some("Body2".to_string()),
            })
            .add_body(Body {
                position: Vector3::new(0.0, 0.0, 0.0),
                velocity: Vector3::new(-0.694222, -1.065456, 0.0),
                mass: Mass::new(1.0).unwrap(),
                radius: None,
                name: Some("Body3".to_string()),
            })
            .build()
            .unwrap();

        let initial_energy = sim.total_energy();
        
        // Simulate for one period (T ≈ 6.32591398)
        let period = 6.32591398;
        let steps = (period / 0.001) as usize;
        
        for _ in 0..steps {
            sim.step().unwrap();
        }
        
        let final_energy = sim.total_energy();
        let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();
        
        // Three-body system should maintain energy conservation
        assert!(energy_drift < 1e-4, "Energy drift: {:.3e}", energy_drift);
    }
}
```

#### Integrator Accuracy Testing
```rust
#[cfg(test)]
mod integrator_tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_velocity_verlet_harmonic_oscillator() {
        // Test integrator accuracy with analytical solution
        // x(t) = A*cos(ωt + φ), ω = sqrt(k/m)
        
        let omega = 2.0; // Angular frequency
        let amplitude = 1.0;
        let phase = 0.0;
        let dt = 0.01;
        let t_final = 2.0 * std::f64::consts::PI / omega; // One period
        
        let mut integrator = VelocityVerlet::new();
        let mut position = Vector3::new(amplitude, 0.0, 0.0);
        let mut velocity = Vector3::zeros();
        let mass = Mass::new(1.0).unwrap();
        
        let steps = (t_final / dt) as usize;
        for step in 0..steps {
            let t = step as f64 * dt;
            
            // Simple harmonic force: F = -k*x = -m*ω²*x
            let force = -mass.value() * omega * omega * position;
            let acceleration = force / mass.value();
            
            // Manual Velocity Verlet step
            let new_position = position + velocity * dt + 0.5 * acceleration * dt * dt;
            let new_acceleration = -mass.value() * omega * omega * new_position / mass.value();
            let new_velocity = velocity + 0.5 * (acceleration + new_acceleration) * dt;
            
            position = new_position;
            velocity = new_velocity;
            
            // Check against analytical solution every 10 steps
            if step % 10 == 0 {
                let t_current = (step + 1) as f64 * dt;
                let analytical_x = amplitude * (omega * t_current + phase).cos();
                let analytical_v = -amplitude * omega * (omega * t_current + phase).sin();
                
                assert_relative_eq!(position.x, analytical_x, epsilon = 1e-3);
                assert_relative_eq!(velocity.x, analytical_v, epsilon = 1e-2);
            }
        }
    }

    #[test]
    fn test_leapfrog_vs_velocity_verlet_accuracy() {
        // Compare integrator accuracy for Kepler problem
        let central_mass = Mass::SOLAR_MASS;
        let orbiting_mass = Mass::EARTH_MASS;
        let dt = 3600.0; // 1 hour timestep
        
        // Initial conditions for circular orbit
        let radius = 1.496e11; // 1 AU
        let orbital_velocity = (G * central_mass.value() / radius).sqrt();
        
        let initial_pos = Vector3::new(radius, 0.0, 0.0);
        let initial_vel = Vector3::new(0.0, orbital_velocity, 0.0);
        
        // Set up two identical simulations with different integrators
        let mut sim_vv = Simulation::builder()
            .integrator(VelocityVerlet::new())
            .gravity(DirectGravity::new())
            .timestep(dt)
            .add_body(Body {
                position: Vector3::zeros(),
                velocity: Vector3::zeros(),
                mass: central_mass,
                radius: None,
                name: Some("Sun".to_string()),
            })
            .add_body(Body {
                position: initial_pos,
                velocity: initial_vel,
                mass: orbiting_mass,
                radius: None,
                name: Some("Earth".to_string()),
            })
            .build()
            .unwrap();
        
        let mut sim_lf = Simulation::builder()
            .integrator(Leapfrog::new())
            .gravity(DirectGravity::new())
            .timestep(dt)
            .add_body(Body {
                position: Vector3::zeros(),
                velocity: Vector3::zeros(),
                mass: central_mass,
                radius: None,
                name: Some("Sun".to_string()),
            })
            .add_body(Body {
                position: initial_pos,
                velocity: initial_vel,
                mass: orbiting_mass,
                radius: None,
                name: Some("Earth".to_string()),
            })
            .build()
            .unwrap();
        
        let initial_energy_vv = sim_vv.total_energy();
        let initial_energy_lf = sim_lf.total_energy();
        
        // Simulate for 10 orbits
        let steps_per_orbit = (365.25 * 24.0 * 3600.0 / dt) as usize;
        let total_steps = 10 * steps_per_orbit;
        
        for _ in 0..total_steps {
            sim_vv.step().unwrap();
            sim_lf.step().unwrap();
        }
        
        let final_energy_vv = sim_vv.total_energy();
        let final_energy_lf = sim_lf.total_energy();
        
        let energy_drift_vv = (final_energy_vv - initial_energy_vv).abs() / initial_energy_vv.abs();
        let energy_drift_lf = (final_energy_lf - initial_energy_lf).abs() / initial_energy_lf.abs();
        
        // Both integrators should conserve energy well (symplectic property)
        assert!(energy_drift_vv < 1e-6, "Velocity Verlet energy drift: {:.3e}", energy_drift_vv);
        assert!(energy_drift_lf < 1e-6, "Leapfrog energy drift: {:.3e}", energy_drift_lf);
    }

    #[test]
    fn test_adaptive_timestep_accuracy() {
        // Test that adaptive timestep maintains accuracy
        let mut adaptive_timestep = AdaptiveTimestep::new(1e-6, 1.0, 1e-8);
        
        // Create highly eccentric orbit (e = 0.9)
        let mut sim = create_eccentric_orbit_simulation(0.9);
        let initial_energy = sim.total_energy();
        
        // Simulate with adaptive timestep
        let mut time = 0.0;
        let target_time = 365.25 * 24.0 * 3600.0; // One year
        
        while time < target_time {
            let dt = adaptive_timestep.calculate_timestep(&sim.particles);
            sim.timestep = dt;
            sim.step().unwrap();
            time += dt;
        }
        
        let final_energy = sim.total_energy();
        let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();
        
        // Adaptive timestep should maintain better accuracy for eccentric orbits
        assert!(energy_drift < 1e-8, "Adaptive timestep energy drift: {:.3e}", energy_drift);
    }
}

fn create_eccentric_orbit_simulation(eccentricity: f64) -> Simulation<VelocityVerlet, DirectGravity> {
    let central_mass = Mass::SOLAR_MASS;
    let orbiting_mass = Mass::EARTH_MASS;
    
    // Semi-major axis
    let a = 1.496e11; // 1 AU
    
    // Initial position at aphelion
    let r_apo = a * (1.0 + eccentricity);
    let position = Vector3::new(r_apo, 0.0, 0.0);
    
    // Velocity at aphelion
    let v_apo = (G * central_mass.value() * (1.0 - eccentricity) / (a * (1.0 + eccentricity))).sqrt();
    let velocity = Vector3::new(0.0, v_apo, 0.0);
    
    Simulation::builder()
        .integrator(VelocityVerlet::new())
        .gravity(DirectGravity::new())
        .timestep(3600.0) // Will be overridden by adaptive controller
        .add_body(Body {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            mass: central_mass,
            radius: None,
            name: Some("Sun".to_string()),
        })
        .add_body(Body {
            position,
            velocity,
            mass: orbiting_mass,
            radius: None,
            name: Some("Planet".to_string()),
        })
        .build()
        .unwrap()
}
```

#### Force Calculation Validation
```rust
#[cfg(test)]
mod force_calculation_tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_direct_gravity_two_body_force() {
        // Test Newton's law of gravitation: F = G*m1*m2/r²
        let force_calc = DirectGravity::new();
        
        let mut particles = ParticleSet::with_capacity(2);
        particles.add_body(Body {
            position: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::zeros(),
            mass: Mass::new(1e12).unwrap(), // 1e12 kg
            radius: None,
            name: None,
        });
        particles.add_body(Body {
            position: Vector3::new(1e6, 0.0, 0.0), // 1 million meters
            velocity: Vector3::zeros(),
            mass: Mass::new(1e10).unwrap(), // 1e10 kg
            radius: None,
            name: None,
        });
        
        let mut forces = vec![Vector3::zeros(); 2];
        force_calc.calculate_forces(&particles, &mut forces).unwrap();
        
        // Expected force magnitude: G * m1 * m2 / r²
        let expected_force_mag = G * 1e12 * 1e10 / (1e6 * 1e6);
        
        // Force on first body should point toward second body (+x direction)
        assert_relative_eq!(forces[0].x, expected_force_mag, epsilon = 1e-10);
        assert_relative_eq!(forces[0].y, 0.0, epsilon = 1e-15);
        assert_relative_eq!(forces[0].z, 0.0, epsilon = 1e-15);
        
        // Force on second body should point toward first body (-x direction)
        assert_relative_eq!(forces[1].x, -expected_force_mag, epsilon = 1e-10);
        assert_relative_eq!(forces[1].y, 0.0, epsilon = 1e-15);
        assert_relative_eq!(forces[1].z, 0.0, epsilon = 1e-15);
        
        // Newton's third law: forces should be equal and opposite
        assert_relative_eq!(forces[0].magnitude(), forces[1].magnitude(), epsilon = 1e-15);
    }

    #[test]
    fn test_barnes_hut_vs_direct_accuracy() {
        // Compare Barnes-Hut approximation with direct calculation
        let direct_calc = DirectGravity::new();
        let barnes_hut_calc = BarnesHut::new().theta(0.5);
        
        // Create a cluster of particles
        let mut particles = ParticleSet::with_capacity(10);
        for i in 0..10 {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / 10.0;
            let radius = 1e8 + (i as f64 * 1e7); // Varying radii
            
            particles.add_body(Body {
                position: Vector3::new(
                    radius * angle.cos(),
                    radius * angle.sin(),
                    0.0,
                ),
                velocity: Vector3::zeros(),
                mass: Mass::new(1e20 + i as f64 * 1e19).unwrap(), // Varying masses
                radius: None,
                name: None,
            });
        }
        
        let mut forces_direct = vec![Vector3::zeros(); 10];
        let mut forces_bh = vec![Vector3::zeros(); 10];
        
        direct_calc.calculate_forces(&particles, &mut forces_direct).unwrap();
        barnes_hut_calc.calculate_forces(&particles, &mut forces_bh).unwrap();
        
        // Barnes-Hut should approximate direct calculation within tolerance
        for i in 0..10 {
            let relative_error = (forces_bh[i] - forces_direct[i]).magnitude() 
                               / forces_direct[i].magnitude();
            
            // For θ = 0.5, error should be < 1%
            assert!(relative_error < 0.01, 
                "Particle {}: Barnes-Hut error {:.4}% too large", i, relative_error * 100.0);
        }
    }

    #[test]
    fn test_softening_parameter_effect() {
        // Test that softening prevents divergence at close approach
        let softening = 1e5; // 100 km softening
        let force_calc_soft = DirectGravity::new().with_softening(softening);
        let force_calc_hard = DirectGravity::new();
        
        let mut particles = ParticleSet::with_capacity(2);
        particles.add_body(Body {
            position: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::zeros(),
            mass: Mass::EARTH_MASS,
            radius: None,
            name: None,
        });
        particles.add_body(Body {
            position: Vector3::new(1e3, 0.0, 0.0), // Very close: 1 km
            velocity: Vector3::zeros(),
            mass: Mass::EARTH_MASS,
            radius: None,
            name: None,
        });
        
        let mut forces_soft = vec![Vector3::zeros(); 2];
        let mut forces_hard = vec![Vector3::zeros(); 2];
        
        force_calc_soft.calculate_forces(&particles, &mut forces_soft).unwrap();
        force_calc_hard.calculate_forces(&particles, &mut forces_hard).unwrap();
        
        // Softened force should be much smaller than hard force at close approach
        let force_ratio = forces_soft[0].magnitude() / forces_hard[0].magnitude();
        
        assert!(force_ratio < 0.1, "Softening should reduce force significantly: ratio = {:.3}", force_ratio);
        assert!(forces_soft[0].magnitude().is_finite(), "Softened force should be finite");
    }

    #[test]
    fn test_force_symmetry_conservation() {
        // Test that forces conserve momentum (sum to zero)
        let force_calc = DirectGravity::new();
        
        let mut particles = ParticleSet::with_capacity(5);
        
        // Add particles at random positions
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        
        for _ in 0..5 {
            particles.add_body(Body {
                position: Vector3::new(
                    rng.gen_range(-1e8..1e8),
                    rng.gen_range(-1e8..1e8),
                    rng.gen_range(-1e8..1e8),
                ),
                velocity: Vector3::zeros(),
                mass: Mass::new(rng.gen_range(1e18..1e22)).unwrap(),
                radius: None,
                name: None,
            });
        }
        
        let mut forces = vec![Vector3::zeros(); 5];
        force_calc.calculate_forces(&particles, &mut forces).unwrap();
        
        // Sum of all forces should be zero (momentum conservation)
        let total_force: Vector3<f64> = forces.iter().sum();
        
        assert!(total_force.magnitude() < 1e-6, 
            "Total force should be ~zero: |F_total| = {:.3e}", total_force.magnitude());
    }
}
```

#### Performance Testing with Criterion
```rust
// In benches/physics_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use gravwell::{Simulation, Body, Mass, Vector3, DirectGravity, BarnesHut, VelocityVerlet};
use std::time::Duration;

fn benchmark_force_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("force_calculations");
    
    // Test different particle counts
    for &n_particles in &[10, 50, 100, 500, 1000] {
        // Direct O(N²) calculation
        group.bench_with_input(
            BenchmarkId::new("direct_gravity", n_particles),
            &n_particles,
            |b, &n| {
                let particles = create_test_particle_set(n);
                let force_calc = DirectGravity::new();
                let mut forces = vec![Vector3::zeros(); n];
                
                b.iter(|| {
                    force_calc.calculate_forces(
                        black_box(&particles), 
                        black_box(&mut forces)
                    ).unwrap();
                });
            },
        );
        
        // Barnes-Hut O(N log N) calculation (for n >= 50)
        if n_particles >= 50 {
            group.bench_with_input(
                BenchmarkId::new("barnes_hut", n_particles),
                &n_particles,
                |b, &n| {
                    let particles = create_test_particle_set(n);
                    let force_calc = BarnesHut::new().theta(0.5);
                    let mut forces = vec![Vector3::zeros(); n];
                    
                    b.iter(|| {
                        force_calc.calculate_forces(
                            black_box(&particles), 
                            black_box(&mut forces)
                        ).unwrap();
                    });
                },
            );
        }
    }
    
    group.finish();
}

fn benchmark_integration_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("integration_step");
    
    // Test different integrators
    for &n_particles in &[100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("velocity_verlet", n_particles),
            &n_particles,
            |b, &n| {
                let mut sim = create_test_simulation_vv(n);
                b.iter(|| {
                    black_box(sim.step()).unwrap();
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("leapfrog", n_particles),
            &n_particles,
            |b, &n| {
                let mut sim = create_test_simulation_lf(n);
                b.iter(|| {
                    black_box(sim.step()).unwrap();
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_simd_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_operations");
    group.measurement_time(Duration::from_secs(10));
    
    // Compare SIMD vs scalar force calculation
    for &n_particles in &[128, 256, 512, 1024] { // Powers of 2 for SIMD alignment
        group.bench_with_input(
            BenchmarkId::new("scalar_forces", n_particles),
            &n_particles,
            |b, &n| {
                let particles = create_test_particle_set(n);
                let force_calc = DirectGravity::new().without_simd();
                let mut forces = vec![Vector3::zeros(); n];
                
                b.iter(|| {
                    force_calc.calculate_forces(
                        black_box(&particles), 
                        black_box(&mut forces)
                    ).unwrap();
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("simd_forces", n_particles),
            &n_particles,
            |b, &n| {
                let particles = create_test_particle_set(n);
                let force_calc = SIMDGravity::new();
                let mut forces = vec![Vector3::zeros(); n];
                
                b.iter(|| {
                    force_calc.calculate_forces_simd(
                        black_box(&particles), 
                        black_box(&mut forces)
                    ).unwrap();
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_60fps_target(c: &mut Criterion) {
    let mut group = c.benchmark_group("60fps_target");
    group.measurement_time(Duration::from_secs(5));
    
    // Target: 16.67ms per frame for 60 FPS
    let target_frame_time = Duration::from_millis(16);
    
    for &n_particles in &[100, 500, 1000, 2000] {
        group.bench_with_input(
            BenchmarkId::new("full_step", n_particles),
            &n_particles,
            |b, &n| {
                let mut sim = create_optimized_simulation(n);
                
                b.iter_custom(|iters| {
                    let start = std::time::Instant::now();
                    for _ in 0..iters {
                        black_box(sim.step()).unwrap();
                    }
                    start.elapsed()
                });
            },
        );
    }
    
    group.finish();
}

// Helper functions
fn create_test_particle_set(n: usize) -> ParticleSet {
    let mut particles = ParticleSet::with_capacity(n);
    
    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
        let radius = 1e11 * (1.0 + 0.1 * (i as f64 / 10.0));
        
        particles.add_body(Body {
            position: Vector3::new(
                radius * angle.cos(),
                radius * angle.sin(),
                0.0,
            ),
            velocity: Vector3::new(
                -3e4 * angle.sin(),
                3e4 * angle.cos(),
                0.0,
            ),
            mass: Mass::new(1e24 + (i as f64 * 1e22)).unwrap(),
            radius: None,
            name: None,
        });
    }
    
    particles
}

fn create_optimized_simulation(n: usize) -> Simulation<VelocityVerlet, BarnesHut> {
    let mut builder = Simulation::builder()
        .integrator(VelocityVerlet::new())
        .gravity(BarnesHut::new().theta(0.7)) // Faster, less accurate for real-time
        .timestep(0.01);
    
    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
        let radius = 1e11 * (1.0 + 0.1 * (i as f64 / 10.0));
        
        builder = builder.add_body(Body {
            position: Vector3::new(
                radius * angle.cos(),
                radius * angle.sin(),
                0.0,
            ),
            velocity: Vector3::new(
                -3e4 * angle.sin(),
                3e4 * angle.cos(),
                0.0,
            ),
            mass: Mass::new(1e24 + (i as f64 * 1e22)).unwrap(),
            radius: None,
            name: None,
        });
    }
    
    builder.build().unwrap()
}

criterion_group!(benches, 
    benchmark_force_calculations, 
    benchmark_integration_step, 
    benchmark_simd_operations,
    benchmark_60fps_target
);
criterion_main!(benches);
```

### Quick Test Commands for Rust Library Development

```bash
# Unit Testing
cargo test                          # Run all tests
cargo test energy_conservation      # Run energy conservation tests
cargo test integrator              # Run integrator accuracy tests
cargo test -- --nocapture         # Show println! output
cargo test --release              # Test optimized builds

# Physics-Specific Testing
cargo test force_calculation       # Test force calculation accuracy
cargo test integrator_tests        # Test numerical integrator accuracy
cargo test energy_conservation_tests # Test symplectic property

# Property-Based Testing
cargo install proptest             # Install property testing
cargo test prop_                   # Run property tests

# Performance Testing
cargo bench                        # Run all benchmarks
cargo bench --bench physics_performance # Run physics benchmarks
cargo bench force_calculations     # Benchmark force calculation methods
cargo bench 60fps_target          # Test 60 FPS performance targets

# Coverage Testing
cargo install cargo-tarpaulin      # Install coverage tool
cargo tarpaulin --out Html         # Generate HTML coverage report
cargo tarpaulin --out Lcov         # Generate LCOV format for CI

# Integration Testing
cargo test --test integration      # Run integration tests
cargo test --test validation       # Test against analytical solutions

# Scientific Validation
cargo test analytical_solutions    # Test against known analytical solutions
cargo test kepler_orbits           # Validate Kepler orbit accuracy
cargo test nbody_benchmarks        # Compare with established N-body codes

# Continuous Testing
cargo install cargo-watch          # Install file watcher
cargo watch -x test               # Auto-run tests on file changes
cargo watch -x "test force"       # Watch force calculation tests
cargo watch -x "bench --bench physics" # Continuous performance monitoring

# Documentation Testing
cargo test --doc                  # Test documentation examples
cargo doc --open                  # Generate and view documentation

# Cross-Platform Testing
cargo test --target wasm32-unknown-unknown  # Test WASM compatibility
cargo test --target x86_64-pc-windows-msvc  # Test Windows compatibility
```

This provides comprehensive Rust library development testing patterns specifically tailored for Gravwell, focusing on physics validation, energy conservation, integrator accuracy, performance benchmarking, and scientific reproducibility.
