# Scientific Validation Guide

## Overview

This guide covers the scientific validation framework for Gravwell,
ensuring that simulations maintain physical accuracy and energy
conservation. It includes validation against analytical solutions,
established N-body codes, and conservation law verification.

## Validation Framework Architecture

Gravwell includes a comprehensive testing suite that validates physics accuracy across multiple domains:

- **Analytical Solutions** - Two-body problems, Kepler orbits, restricted three-body problem
- **Energy Conservation** - Long-term energy drift analysis
- **Momentum Conservation** - Linear and angular momentum preservation
- **Numerical Accuracy** - Comparison with established codes (REBOUND, NBODY6)
- **Symplectic Properties** - Phase space volume preservation for symplectic integrators

## Energy Conservation Testing

Energy conservation is the primary indicator of simulation quality for gravitational N-body systems.

### Basic Energy Conservation Test

```rust
use gravwell::prelude::*;
use approx::assert_relative_eq;

#[test]
fn test_two_body_energy_conservation() {
    let mut sim = Simulation::builder()
        .integrator(VelocityVerlet::new())
        .forces(DirectGravity::new())
        .timestep(0.01)
        .build()
        .unwrap();

    // Set up Sun-Earth system
    let sun = sim.add_body(Body::new()
        .mass(Mass::SOLAR_MASS)
        .position([0.0, 0.0, 0.0])
        .velocity([0.0, 0.0, 0.0])
    ).unwrap();

    let earth = sim.add_body(Body::new()
        .mass(Mass::EARTH_MASS)
        .position([1.496e11, 0.0, 0.0])  // 1 AU
        .velocity([0.0, 29780.0, 0.0])   // Circular orbital velocity
    ).unwrap();

    let initial_energy = sim.total_energy();
    
    // Simulate for 100 orbital periods (~273 years)
    let orbital_period = 365.25 * 24.0 * 3600.0; // seconds
    let total_time = 100.0 * orbital_period;
    let steps = (total_time / sim.timestep()) as usize;
    
    for _ in 0..steps {
        sim.step();
    }
    
    let final_energy = sim.total_energy();
    let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();
    
    // Energy should be conserved to within 1e-10 for symplectic integrators
    assert!(energy_error < 1e-10, 
        "Energy drift too large: {:.3e}, initial: {:.3e}, final: {:.3e}",
        energy_error, initial_energy, final_energy
    );
}
```

### Extended Energy Conservation Analysis

```rust
use gravwell::prelude::*;
use std::collections::VecDeque;

pub struct EnergyConservationAnalyzer {
    energy_history: VecDeque<f64>,
    time_history: VecDeque<f64>,
    initial_energy: f64,
    max_history_length: usize,
}

impl EnergyConservationAnalyzer {
    pub fn new(initial_energy: f64) -> Self {
        Self {
            energy_history: VecDeque::new(),
            time_history: VecDeque::new(),
            initial_energy,
            max_history_length: 10000,
        }
    }
    
    pub fn record_energy(&mut self, time: f64, energy: f64) {
        self.energy_history.push_back(energy);
        self.time_history.push_back(time);
        
        if self.energy_history.len() > self.max_history_length {
            self.energy_history.pop_front();
            self.time_history.pop_front();
        }
    }
    
    pub fn relative_energy_drift(&self) -> f64 {
        if let Some(&latest_energy) = self.energy_history.back() {
            (latest_energy - self.initial_energy).abs() / self.initial_energy.abs()
        } else {
            0.0
        }
    }
    
    pub fn energy_drift_rate(&self) -> Option<f64> {
        if self.energy_history.len() < 2 { return None; }
        
        let n = self.energy_history.len();
        let energy_slope = linear_regression(
            &self.time_history.iter().collect::<Vec<_>>(),
            &self.energy_history.iter().collect::<Vec<_>>()
        );
        
        Some(energy_slope / self.initial_energy.abs())
    }
    
    pub fn validate_conservation(&self, threshold: f64) -> ValidationResult {
        let drift = self.relative_energy_drift();
        
        ValidationResult {
            passed: drift < threshold,
            metric_name: "Energy Conservation".to_string(),
            measured_value: drift,
            threshold,
            description: format!(
                "Relative energy drift: {:.3e} (threshold: {:.3e})",
                drift, threshold
            ),
        }
    }
}

fn linear_regression(x: &[&f64], y: &[&f64]) -> f64 {
    let n = x.len() as f64;
    let sum_x: f64 = x.iter().map(|&&v| v).sum();
    let sum_y: f64 = y.iter().map(|&&v| v).sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(&x, &y)| x * y).sum();
    let sum_x_squared: f64 = x.iter().map(|&&v| v * v).sum();
    
    (n * sum_xy - sum_x * sum_y) / (n * sum_x_squared - sum_x * sum_x)
}
```

## Kepler Orbit Validation

Validate against analytical solutions for two-body orbital mechanics:

```rust
use gravwell::prelude::*;
use std::f64::consts::PI;

#[test]
fn test_circular_kepler_orbit() {
    let mut sim = setup_earth_sun_system();
    
    let analyzer = KeplerOrbitAnalyzer::new(
        sim.position(earth_handle), 
        sim.velocity(earth_handle),
        Mass::SOLAR_MASS + Mass::EARTH_MASS
    );
    
    // Simulate one orbital period
    let orbital_period = analyzer.orbital_period();
    let steps_per_period = 1000;
    let dt = orbital_period / steps_per_period as f64;
    sim.set_timestep(dt);
    
    for step in 0..steps_per_period {
        sim.step();
        
        if step % 100 == 0 {
            let current_pos = sim.position(earth_handle);
            let current_vel = sim.velocity(earth_handle);
            
            // Validate orbital elements
            let elements = analyzer.orbital_elements(current_pos, current_vel);
            
            // Semi-major axis should remain constant
            assert_relative_eq!(elements.semi_major_axis, 1.496e11, epsilon = 1e6);
            
            // Eccentricity should remain near zero for circular orbit
            assert!(elements.eccentricity < 1e-6, 
                "Orbit becoming elliptical: e = {:.3e}", elements.eccentricity);
        }
    }
    
    // After one period, position should return close to initial
    let final_position = sim.position(earth_handle);
    let initial_position = Vector3::new(1.496e11, 0.0, 0.0);
    let position_error = (final_position - initial_position).norm() / initial_position.norm();
    
    assert!(position_error < 1e-8, 
        "Position error after one orbit: {:.3e}", position_error);
}

pub struct OrbitalElements {
    pub semi_major_axis: f64,
    pub eccentricity: f64,
    pub inclination: f64,
    pub longitude_of_ascending_node: f64,
    pub argument_of_periapsis: f64,
    pub true_anomaly: f64,
}

pub struct KeplerOrbitAnalyzer {
    pub total_mass: f64,
}

impl KeplerOrbitAnalyzer {
    pub fn new(initial_position: Vector3, initial_velocity: Vector3, total_mass: f64) -> Self {
        Self { total_mass }
    }
    
    pub fn orbital_period(&self, semi_major_axis: f64) -> f64 {
        2.0 * PI * (semi_major_axis.powi(3) / (G * self.total_mass)).sqrt()
    }
    
    pub fn orbital_elements(&self, position: Vector3, velocity: Vector3) -> OrbitalElements {
        let mu = G * self.total_mass;
        let r = position.norm();
        let v_squared = velocity.norm_squared();
        
        // Specific orbital energy
        let energy = v_squared / 2.0 - mu / r;
        
        // Semi-major axis  
        let semi_major_axis = -mu / (2.0 * energy);
        
        // Angular momentum vector
        let h_vec = position.cross(&velocity);
        let h = h_vec.norm();
        
        // Eccentricity
        let eccentricity = ((1.0 + 2.0 * energy * h * h / (mu * mu)).max(0.0)).sqrt();
        
        // Inclination
        let inclination = (h_vec.z / h).acos();
        
        // Node vector
        let n_vec = Vector3::new(-h_vec.y, h_vec.x, 0.0);
        let n = n_vec.norm();
        
        // Longitude of ascending node
        let longitude_of_ascending_node = if n > 0.0 {
            (n_vec.x / n).acos() * if n_vec.y >= 0.0 { 1.0 } else { -1.0 }
        } else {
            0.0
        };
        
        // Eccentricity vector
        let e_vec = (velocity.cross(&h_vec) / mu) - (position / r);
        
        // Argument of periapsis
        let argument_of_periapsis = if n > 0.0 {
            (n_vec.dot(&e_vec) / (n * eccentricity)).acos() * 
            if e_vec.z >= 0.0 { 1.0 } else { -1.0 }
        } else {
            (e_vec.x / eccentricity).acos() * if e_vec.y >= 0.0 { 1.0 } else { -1.0 }
        };
        
        // True anomaly
        let true_anomaly = (e_vec.dot(&position) / (eccentricity * r)).acos() *
            if position.dot(&velocity) >= 0.0 { 1.0 } else { -1.0 };
        
        OrbitalElements {
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_of_ascending_node,
            argument_of_periapsis,
            true_anomaly,
        }
    }
}
```

## Momentum Conservation Testing

```rust
use gravwell::prelude::*;

#[test]
fn test_momentum_conservation() {
    let mut sim = setup_solar_system();
    
    let initial_momentum = sim.total_momentum();
    let initial_angular_momentum = sim.total_angular_momentum();
    
    // Simulate for extended period
    for _ in 0..10000 {
        sim.step();
    }
    
    let final_momentum = sim.total_momentum();
    let final_angular_momentum = sim.total_angular_momentum();
    
    // Linear momentum should be conserved (center of mass at rest)
    let momentum_error = (final_momentum - initial_momentum).norm() / 
        (initial_momentum.norm() + 1e-100); // Avoid division by zero
    
    assert!(momentum_error < 1e-12, 
        "Linear momentum not conserved: error = {:.3e}", momentum_error);
    
    // Angular momentum should be conserved
    let angular_momentum_error = (final_angular_momentum - initial_angular_momentum).norm() / 
        initial_angular_momentum.norm();
    
    assert!(angular_momentum_error < 1e-12,
        "Angular momentum not conserved: error = {:.3e}", angular_momentum_error);
}

impl Simulation {
    pub fn total_momentum(&self) -> Vector3 {
        let mut total_momentum = Vector3::zeros();
        
        for handle in self.active_bodies() {
            let velocity = self.velocity(handle);
            let mass = self.mass(handle);
            total_momentum += mass.value() * velocity;
        }
        
        total_momentum
    }
    
    pub fn total_angular_momentum(&self) -> Vector3 {
        let center_of_mass = self.center_of_mass();
        let mut total_angular_momentum = Vector3::zeros();
        
        for handle in self.active_bodies() {
            let position = self.position(handle) - center_of_mass;
            let velocity = self.velocity(handle);
            let mass = self.mass(handle);
            
            total_angular_momentum += mass.value() * position.cross(&velocity);
        }
        
        total_angular_momentum
    }
}
```

## Integrator Accuracy Testing

Compare different integrators against analytical solutions:

```rust
use gravwell::prelude::*;
use std::collections::HashMap;

#[test]
fn compare_integrator_accuracy() {
    let integrators: Vec<(&str, Box<dyn Integrator>)> = vec![
        ("Semi-Implicit Euler", Box::new(SemiImplicitEuler::new())),
        ("Velocity Verlet", Box::new(VelocityVerlet::new())),
        ("Leapfrog", Box::new(Leapfrog::new())),
        ("RK4", Box::new(RK4::new())),
    ];
    
    let timesteps = vec![0.1, 0.01, 0.001, 0.0001];
    
    for &dt in &timesteps {
        println!("Timestep: {}", dt);
        
        for (name, integrator) in &integrators {
            let accuracy = test_integrator_accuracy(integrator.as_ref(), dt);
            println!("  {}: Energy error = {:.3e}, Position error = {:.3e}", 
                name, accuracy.energy_error, accuracy.position_error);
        }
        println!();
    }
}

struct AccuracyMetrics {
    energy_error: f64,
    position_error: f64,
}

fn test_integrator_accuracy(integrator: &dyn Integrator, dt: f64) -> AccuracyMetrics {
    let mut sim = Simulation::builder()
        .integrator(integrator.clone())
        .forces(DirectGravity::new())
        .timestep(dt)
        .build()
        .unwrap();
    
    // Set up circular orbit
    setup_circular_orbit(&mut sim);
    
    let initial_energy = sim.total_energy();
    let initial_position = sim.position(earth_handle);
    
    // Simulate one orbital period
    let orbital_period = 2.0 * PI * (AU.powi(3) / (G * SOLAR_MASS)).sqrt();
    let steps = (orbital_period / dt) as usize;
    
    for _ in 0..steps {
        sim.step();
    }
    
    let final_energy = sim.total_energy();
    let final_position = sim.position(earth_handle);
    
    let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();
    let position_error = (final_position - initial_position).norm() / AU;
    
    AccuracyMetrics { energy_error, position_error }
}
```

## Validation Against REBOUND

Compare Gravwell results with the established REBOUND N-body code:

```python
# Python script to generate REBOUND reference data
import rebound
import numpy as np
import json

def generate_rebound_reference():
    sim = rebound.Simulation()
    sim.units = ('AU', 'Msun', 'yr2pi')  # Astronomical units
    
    # Add Sun
    sim.add(m=1.0)  # Solar masses
    
    # Add Earth
    sim.add(m=3.003e-6, a=1.0, e=0.0, inc=0.0, Omega=0.0, omega=0.0, f=0.0)
    
    # Move to center of mass frame
    sim.move_to_com()
    
    # Integration settings
    sim.integrator = "whfast"  # Symplectic integrator
    sim.dt = 0.01  # Years
    
    # Record data
    times = []
    energies = []
    positions = []
    velocities = []
    
    # Simulate for 100 years
    for i in range(10000):
        sim.integrate(i * 0.01)
        
        times.append(sim.t)
        energies.append(sim.calculate_energy())
        
        # Store Earth's position and velocity
        earth = sim.particles[1]
        positions.append([earth.x, earth.y, earth.z])
        velocities.append([earth.vx, earth.vy, earth.vz])
    
    # Save reference data
    reference_data = {
        'times': times,
        'energies': energies,
        'positions': positions,
        'velocities': velocities
    }
    
    with open('rebound_reference.json', 'w') as f:
        json.dump(reference_data, f)

if __name__ == "__main__":
    generate_rebound_reference()
```

```rust
// Rust code to compare with REBOUND
use gravwell::prelude::*;
use serde_json;
use std::fs::File;

#[derive(serde::Deserialize)]
struct ReboundReference {
    times: Vec<f64>,
    energies: Vec<f64>,
    positions: Vec<[f64; 3]>,
    velocities: Vec<[f64; 3]>,
}

#[test]
fn compare_with_rebound() {
    // Load REBOUND reference data
    let file = File::open("tests/data/rebound_reference.json").unwrap();
    let reference: ReboundReference = serde_json::from_reader(file).unwrap();
    
    // Set up identical system in Gravwell
    let mut sim = Simulation::builder()
        .integrator(Leapfrog::new())  // Closest to REBOUND's WHFast
        .forces(DirectGravity::new())
        .timestep(0.01 * YEAR_IN_SECONDS)  // Convert years to seconds
        .build()
        .unwrap();
    
    // Add Sun (1 solar mass at origin)
    let sun = sim.add_body(Body::new()
        .mass(Mass::SOLAR_MASS)
        .position([0.0, 0.0, 0.0])
        .velocity([0.0, 0.0, 0.0])
    ).unwrap();
    
    // Add Earth (at 1 AU)
    let earth = sim.add_body(Body::new()
        .mass(Mass::EARTH_MASS)
        .position([AU, 0.0, 0.0])
        .velocity([0.0, 29780.0, 0.0])  // Circular orbital velocity
    ).unwrap();
    
    // Move to center of mass frame
    sim.move_to_center_of_mass();
    
    let mut max_position_error = 0.0;
    let mut max_energy_error = 0.0;
    
    for (i, &ref_time) in reference.times.iter().enumerate() {
        sim.integrate_to(ref_time * YEAR_IN_SECONDS);
        
        // Compare positions (convert to AU for comparison)
        let gravwell_pos = sim.position(earth) / AU;
        let rebound_pos = Vector3::from_row_slice(&reference.positions[i]);
        
        let position_error = (gravwell_pos - rebound_pos).norm();
        max_position_error = max_position_error.max(position_error);
        
        // Compare energies
        let gravwell_energy = sim.total_energy();
        let rebound_energy = reference.energies[i] * ENERGY_CONVERSION_FACTOR;
        
        let energy_error = (gravwell_energy - rebound_energy).abs() / rebound_energy.abs();
        max_energy_error = max_energy_error.max(energy_error);
    }
    
    // Errors should be small (within numerical precision)
    assert!(max_position_error < 1e-10, 
        "Maximum position error vs REBOUND: {:.3e} AU", max_position_error);
    assert!(max_energy_error < 1e-12,
        "Maximum energy error vs REBOUND: {:.3e}", max_energy_error);
    
    println!("REBOUND comparison successful:");
    println!("  Max position error: {:.3e} AU", max_position_error);
    println!("  Max energy error: {:.3e}", max_energy_error);
}
```

## Symplectic Property Validation

For symplectic integrators, verify phase space volume conservation:

```rust
use gravwell::prelude::*;
use nalgebra::DMatrix;

#[test]
fn test_symplectic_property() {
    let integrators_symplectic = vec![
        ("Velocity Verlet", VelocityVerlet::new(), true),
        ("Leapfrog", Leapfrog::new(), true),
        ("Semi-Implicit Euler", SemiImplicitEuler::new(), true),
        ("RK4", RK4::new(), false),  // Not symplectic
    ];
    
    for (name, integrator, expected_symplectic) in integrators_symplectic {
        let is_symplectic = test_phase_space_volume_conservation(&integrator);
        
        if expected_symplectic {
            assert!(is_symplectic, "{} should preserve phase space volume", name);
        }
        
        println!("{}: Phase space volume preserved = {}", name, is_symplectic);
    }
}

fn test_phase_space_volume_conservation(integrator: &dyn Integrator) -> bool {
    const N_PARTICLES: usize = 3;  // Small system for computational efficiency
    const N_STEPS: usize = 1000;
    
    // Create a small cluster of particles
    let mut sim = Simulation::builder()
        .integrator(integrator.clone())
        .forces(DirectGravity::new())
        .timestep(0.001)
        .build()
        .unwrap();
    
    // Add particles in a small region of phase space
    for i in 0..N_PARTICLES {
        let angle = 2.0 * PI * i as f64 / N_PARTICLES as f64;
        sim.add_body(Body::new()
            .mass(Mass::SOLAR_MASS)
            .position([angle.cos(), angle.sin(), 0.0])
            .velocity([-angle.sin(), angle.cos(), 0.0])
        ).unwrap();
    }
    
    // Sample phase space volume using nearby trajectories
    let initial_volume = calculate_phase_space_volume(&sim);
    
    for _ in 0..N_STEPS {
        sim.step();
    }
    
    let final_volume = calculate_phase_space_volume(&sim);
    let volume_change = (final_volume - initial_volume).abs() / initial_volume;
    
    // Symplectic integrators should preserve volume to within numerical precision
    volume_change < 1e-10
}

fn calculate_phase_space_volume(sim: &Simulation) -> f64 {
    // For simplicity, use the determinant of the covariance matrix
    // as a proxy for phase space volume
    
    let n = sim.particle_count();
    let mut phase_space_matrix = DMatrix::zeros(6 * n, 1);
    
    for (i, handle) in sim.active_bodies().enumerate() {
        let pos = sim.position(handle);
        let vel = sim.velocity(handle);
        
        phase_space_matrix[(6 * i + 0, 0)] = pos.x;
        phase_space_matrix[(6 * i + 1, 0)] = pos.y;
        phase_space_matrix[(6 * i + 2, 0)] = pos.z;
        phase_space_matrix[(6 * i + 3, 0)] = vel.x;
        phase_space_matrix[(6 * i + 4, 0)] = vel.y;
        phase_space_matrix[(6 * i + 5, 0)] = vel.z;
    }
    
    // Calculate covariance matrix
    let mean = phase_space_matrix.row_mean();
    let centered = &phase_space_matrix - &mean;
    let covariance = &centered * centered.transpose() / (n - 1) as f64;
    
    // Volume proxy: square root of determinant
    covariance.determinant().abs().sqrt()
}
```

## Comprehensive Validation Suite

```rust
use gravwell::prelude::*;

pub struct ValidationSuite {
    pub tests: Vec<ValidationTest>,
}

impl ValidationSuite {
    pub fn new() -> Self {
        Self {
            tests: vec![
                ValidationTest::energy_conservation(),
                ValidationTest::momentum_conservation(),
                ValidationTest::kepler_orbits(),
                ValidationTest::three_body_figure_eight(),
                ValidationTest::restricted_three_body(),
                ValidationTest::symplectic_property(),
                ValidationTest::rebound_comparison(),
            ],
        }
    }
    
    pub fn run_all(&self, integrator: &dyn Integrator) -> ValidationReport {
        let mut report = ValidationReport::new();
        
        for test in &self.tests {
            let result = test.run(integrator);
            report.add_result(result);
        }
        
        report
    }
}

pub struct ValidationReport {
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub results: Vec<ValidationResult>,
    pub overall_score: f64,
}

impl ValidationReport {
    pub fn print_summary(&self) {
        println!("Validation Summary");
        println!("=================");
        println!("Passed: {}", self.passed_tests);
        println!("Failed: {}", self.failed_tests);
        println!("Overall Score: {:.1}%", self.overall_score * 100.0);
        println!();
        
        for result in &self.results {
            let status = if result.passed { "✓" } else { "✗" };
            println!("{} {}: {:.3e} (threshold: {:.3e})",
                status, result.metric_name, result.measured_value, result.threshold);
        }
    }
}

// Run comprehensive validation
#[test]
fn comprehensive_validation() {
    let suite = ValidationSuite::new();
    
    let integrators: Vec<Box<dyn Integrator>> = vec![
        Box::new(VelocityVerlet::new()),
        Box::new(Leapfrog::new()),
        Box::new(RK4::new()),
    ];
    
    for integrator in integrators {
        println!("Testing integrator: {}", integrator.name());
        let report = suite.run_all(integrator.as_ref());
        report.print_summary();
        
        // Require minimum 80% pass rate for scientific accuracy
        assert!(report.overall_score >= 0.8,
            "Integrator {} failed validation with score {:.1}%",
            integrator.name(), report.overall_score * 100.0);
    }
}
```

## Benchmarking Scientific Accuracy vs Performance

```rust
use gravwell::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn accuracy_vs_performance_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("accuracy_vs_performance");
    
    let configurations = vec![
        ("Game Mode: Fast", SemiImplicitEuler::new(), 0.01, 1e-3),
        ("Balanced: Verlet", VelocityVerlet::new(), 0.001, 1e-6), 
        ("Science: Leapfrog", Leapfrog::new(), 0.0001, 1e-9),
        ("High Precision: RK4", RK4::new(), 0.0001, 1e-12),
    ];
    
    for (name, integrator, timestep, accuracy_threshold) in configurations {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut sim = setup_test_system(integrator.clone(), timestep);
                
                // Run simulation and measure accuracy
                let initial_energy = sim.total_energy();
                
                for _ in 0..1000 {
                    black_box(sim.step());
                }
                
                let final_energy = sim.total_energy();
                let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();
                
                // Verify accuracy meets threshold
                assert!(energy_error < accuracy_threshold,
                    "Accuracy threshold not met: {:.3e} > {:.3e}",
                    energy_error, accuracy_threshold);
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, accuracy_vs_performance_benchmark);
criterion_main!(benches);
```

This comprehensive validation framework ensures that Gravwell maintains
scientific accuracy across different use cases while providing clear
performance trade-offs. The validation suite can be integrated into CI/CD
pipelines to catch regressions in physical accuracy.
