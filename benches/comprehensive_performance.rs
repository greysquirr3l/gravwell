//! Comprehensive Performance Benchmarking Suite
//!
//! This module provides extensive performance benchmarks for all core algorithms,
//! integrators, and force calculators. Designed to track performance regressions
//! and guide optimization decisions.

use gravwell::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

/// Particle count configurations for benchmarking
const PARTICLE_COUNTS: &[usize] = &[100, 500, 1000, 2000, 5000];

/// Timestep configurations for integration benchmarks
const TIMESTEPS: &[f64] = &[0.01, 0.001, 0.0001];

/// Number of simulation steps for longer benchmarks
const LONG_BENCHMARK_STEPS: usize = 1000;

/// Setup helper functions for different simulation types
mod setup {
    use super::*;
    
    pub fn create_random_system(particle_count: usize) -> Vec<Body> {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;
        
        let mut rng = ChaCha8Rng::seed_from_u64(42); // Deterministic for benchmarking
        let mut bodies = Vec::with_capacity(particle_count);
        
        for i in 0..particle_count {
            let mass = if i == 0 {
                SOLAR_MASS // Central star
            } else {
                EARTH_MASS * rng.gen_range(0.1..10.0) // Planets/asteroids
            };
            
            let distance = if i == 0 {
                0.0
            } else {
                AU * rng.gen_range(0.5..50.0) // Orbital distances
            };
            
            let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
            let position = [distance * angle.cos(), distance * angle.sin(), 0.0];
            
            // Approximate circular velocity for stability
            let orbital_velocity = if distance > 0.0 {
                (G * SOLAR_MASS / distance).sqrt()
            } else {
                0.0
            };
            
            let velocity = [
                -orbital_velocity * angle.sin(),
                orbital_velocity * angle.cos(), 
                0.0
            ];
            
            bodies.push(Body::new()
                .with_mass(mass)
                .with_position(position)
                .with_velocity(velocity));
        }
        
        bodies
    }
    
    pub fn create_solar_system() -> Vec<Body> {
        vec![
            // Sun
            Body::new()
                .with_mass(SOLAR_MASS)
                .with_position([0.0, 0.0, 0.0])
                .with_velocity([0.0, 0.0, 0.0]),
            // Mercury
            Body::new()
                .with_mass(3.3011e23)
                .with_position([0.387 * AU, 0.0, 0.0])
                .with_velocity([0.0, 47870.0, 0.0]),
            // Venus 
            Body::new()
                .with_mass(4.8675e24)
                .with_position([0.723 * AU, 0.0, 0.0])
                .with_velocity([0.0, 35020.0, 0.0]),
            // Earth
            Body::new()
                .with_mass(EARTH_MASS)
                .with_position([AU, 0.0, 0.0])
                .with_velocity([0.0, 29785.0, 0.0]),
            // Mars
            Body::new()
                .with_mass(6.4171e23)
                .with_position([1.524 * AU, 0.0, 0.0])
                .with_velocity([0.0, 24077.0, 0.0]),
            // Jupiter
            Body::new()
                .with_mass(JUPITER_MASS)
                .with_position([5.204 * AU, 0.0, 0.0])
                .with_velocity([0.0, 13070.0, 0.0]),
        ]
    }
}

/// Force calculation benchmarks
mod force_benchmarks {
    use super::*;
    
    pub fn bench_direct_gravity(c: &mut Criterion) {
        let mut group = c.benchmark_group("force_calculation/direct_gravity");
        
        for &particle_count in PARTICLE_COUNTS {
            group.throughput(Throughput::Elements(particle_count as u64));
            
            group.bench_with_input(
                BenchmarkId::new("particles", particle_count),
                &particle_count,
                |b, &count| {
                    let bodies = setup::create_random_system(count);
                    let mut sim = SimulationBuilder::new()
                        .with_integrator(VelocityVerlet::new())
                        .with_force_calculator(DirectGravity::new());
                    
                    for body in bodies {
                        sim = sim.add_body(body).unwrap();
                    }
                    
                    let mut sim = sim.build().unwrap();
                    
                    b.iter(|| {
                        // Benchmark single force calculation step
                        black_box(sim.step(0.001).unwrap());
                    });
                },
            );
        }
        
        group.finish();
    }
    
    pub fn bench_barnes_hut(c: &mut Criterion) {
        let mut group = c.benchmark_group("force_calculation/barnes_hut");
        
        // Test different theta values for accuracy vs performance
        let theta_values = vec![0.3, 0.5, 0.7, 1.0];
        
        for &particle_count in &[500, 1000, 2000, 5000] {
            for &theta in &theta_values {
                group.bench_with_input(
                    BenchmarkId::new(format!("particles_{}_theta_{}", particle_count, theta), particle_count),
                    &(particle_count, theta),
                    |b, &(count, theta_val)| {
                        let bodies = setup::create_random_system(count);
                        let mut sim = SimulationBuilder::new()
                            .with_integrator(VelocityVerlet::new())
                            .with_force_calculator(BarnesHut::new().theta(theta_val));
                        
                        for body in bodies {
                            sim = sim.add_body(body).unwrap();
                        }
                        
                        let mut sim = sim.build().unwrap();
                        
                        b.iter(|| {
                            black_box(sim.step(0.001).unwrap());
                        });
                    },
                );
            }
        }
        
        group.finish();
    }
}

/// Integration algorithm benchmarks
mod integrator_benchmarks {
    use super::*;
    
    pub fn bench_velocity_verlet(c: &mut Criterion) {
        let mut group = c.benchmark_group("integrators/velocity_verlet");
        
        for &particle_count in PARTICLE_COUNTS {
            group.throughput(Throughput::Elements(particle_count as u64));
            
            group.bench_with_input(
                BenchmarkId::new("particles", particle_count),
                &particle_count,
                |b, &count| {
                    let bodies = setup::create_random_system(count);
                    let mut sim = SimulationBuilder::new()
                        .with_integrator(VelocityVerlet::new())
                        .with_force_calculator(DirectGravity::new());
                    
                    for body in bodies {
                        sim = sim.add_body(body).unwrap();
                    }
                    
                    let mut sim = sim.build().unwrap();
                    
                    b.iter(|| {
                        black_box(sim.step(0.001).unwrap());
                    });
                },
            );
        }
        
        group.finish();
    }
    
    pub fn bench_leapfrog(c: &mut Criterion) {
        let mut group = c.benchmark_group("integrators/leapfrog");
        
        for &particle_count in PARTICLE_COUNTS {
            group.throughput(Throughput::Elements(particle_count as u64));
            
            group.bench_with_input(
                BenchmarkId::new("particles", particle_count),
                &particle_count,
                |b, &count| {
                    let bodies = setup::create_random_system(count);
                    let mut sim = SimulationBuilder::new()
                        .with_integrator(Leapfrog::new())
                        .with_force_calculator(DirectGravity::new());
                    
                    for body in bodies {
                        sim = sim.add_body(body).unwrap();
                    }
                    
                    let mut sim = sim.build().unwrap();
                    
                    b.iter(|| {
                        black_box(sim.step(0.001).unwrap());
                    });
                },
            );
        }
        
        group.finish();
    }
    
    pub fn bench_rk4(c: &mut Criterion) {
        let mut group = c.benchmark_group("integrators/runge_kutta_4");
        
        for &particle_count in &[100, 500, 1000] { // RK4 is expensive, test smaller sets
            group.throughput(Throughput::Elements(particle_count as u64));
            
            group.bench_with_input(
                BenchmarkId::new("particles", particle_count),
                &particle_count,
                |b, &count| {
                    let bodies = setup::create_random_system(count);
                    let mut sim = SimulationBuilder::new()
                        .with_integrator(RungeKutta4::new())
                        .with_force_calculator(DirectGravity::new());
                    
                    for body in bodies {
                        sim = sim.add_body(body).unwrap();
                    }
                    
                    let mut sim = sim.build().unwrap();
                    
                    b.iter(|| {
                        black_box(sim.step(0.001).unwrap());
                    });
                },
            );
        }
        
        group.finish();
    }
}

/// Full simulation benchmarks - real world performance scenarios
mod simulation_benchmarks {
    use super::*;
    
    pub fn bench_60fps_target(c: &mut Criterion) {
        let mut group = c.benchmark_group("simulation/60fps_target");
        group.measurement_time(Duration::from_secs(10));
        
        // Small system with direct gravity - 60 FPS target
        group.bench_function("small_system_1000_direct", |b| {
            let bodies = setup::create_random_system(1000);
            let mut builder = SimulationBuilder::new()
                .with_integrator(VelocityVerlet::new())
                .with_force_calculator(DirectGravity::new());
            for body in bodies { builder = builder.add_body(body).unwrap(); }
            let mut sim = builder.build().unwrap();
            
            let target_dt = 1.0 / 60.0; // 60 FPS timestep
            
            b.iter(|| {
                black_box(sim.step(target_dt).unwrap());
            });
        });
        
        // Medium system with Barnes-Hut - 60 FPS target
        group.bench_function("medium_system_2000_barnes_hut", |b| {
            let bodies = setup::create_random_system(2000);
            let mut builder = SimulationBuilder::new()
                .with_integrator(VelocityVerlet::new())
                .with_force_calculator(BarnesHut::new().theta(0.5));
            for body in bodies { builder = builder.add_body(body).unwrap(); }
            let mut sim = builder.build().unwrap();
            
            let target_dt = 1.0 / 60.0; // 60 FPS timestep
            
            b.iter(|| {
                black_box(sim.step(target_dt).unwrap());
            });
        });
        
        // Large system with Barnes-Hut - 60 FPS target
        group.bench_function("large_system_5000_barnes_hut", |b| {
            let bodies = setup::create_random_system(5000);
            let mut builder = SimulationBuilder::new()
                .with_integrator(VelocityVerlet::new())
                .with_force_calculator(BarnesHut::new().theta(0.5));
            for body in bodies { builder = builder.add_body(body).unwrap(); }
            let mut sim = builder.build().unwrap();
            
            let target_dt = 1.0 / 60.0; // 60 FPS timestep
            
            b.iter(|| {
                black_box(sim.step(target_dt).unwrap());
            });
        });
        
        group.finish();
    }
    
    pub fn bench_solar_system_simulation(c: &mut Criterion) {
        let mut group = c.benchmark_group("simulation/solar_system");
        

        
        // Velocity Verlet solar system benchmark
        group.bench_function("velocity_verlet", |b| {
            let bodies = setup::create_solar_system();
            let mut builder = SimulationBuilder::new()
                .with_integrator(VelocityVerlet::new())
                .with_force_calculator(DirectGravity::new());
            for body in bodies { builder = builder.add_body(body).unwrap(); }
            let mut sim = builder.build().unwrap();
            
            let timestep = 3600.0; // 1 hour timestep
            
            b.iter(|| {
                black_box(sim.step(timestep).unwrap());
            });
        });
        
        // Leapfrog solar system benchmark
        group.bench_function("leapfrog", |b| {
            let bodies = setup::create_solar_system();
            let mut builder = SimulationBuilder::new()
                .with_integrator(Leapfrog::new())
                .with_force_calculator(DirectGravity::new());
            for body in bodies { builder = builder.add_body(body).unwrap(); }
            let mut sim = builder.build().unwrap();
            
            let timestep = 3600.0; // 1 hour timestep
            
            b.iter(|| {
                black_box(sim.step(timestep).unwrap());
            });
        });
        
        // RK4 solar system benchmark
        group.bench_function("rk4", |b| {
            let bodies = setup::create_solar_system();
            let mut builder = SimulationBuilder::new()
                .with_integrator(RungeKutta4::new())
                .with_force_calculator(DirectGravity::new());
            for body in bodies { builder = builder.add_body(body).unwrap(); }
            let mut sim = builder.build().unwrap();
            
            let timestep = 3600.0; // 1 hour timestep
            
            b.iter(|| {
                black_box(sim.step(timestep).unwrap());
            });
        });
        
        group.finish();
    }
    
    pub fn bench_long_term_stability(c: &mut Criterion) {
        let mut group = c.benchmark_group("simulation/long_term_stability");
        group.sample_size(10); // Fewer samples for long benchmarks
        group.measurement_time(Duration::from_secs(30));
        
        group.bench_function("earth_moon_1000_steps", |b| {
            b.iter(|| {
                // Create Earth-Moon system for long-term stability test
                let mut sim = SimulationBuilder::new()
                    .with_integrator(VelocityVerlet::new())
                    .with_force_calculator(DirectGravity::new())
                    .add_body(Body::new()
                        .with_mass(EARTH_MASS)
                        .with_position([0.0, 0.0, 0.0])
                        .with_velocity([0.0, 0.0, 0.0]))
                    .unwrap()
                    .add_body(Body::new()
                        .with_mass(7.342e22) // Moon mass
                        .with_position([384400000.0, 0.0, 0.0]) // ~384,400 km
                        .with_velocity([0.0, 1022.0, 0.0])) // Moon orbital velocity
                    .unwrap()
                    .build()
                    .unwrap();
                
                let initial_energy = sim.total_energy();
                
                // Simulate 1000 steps
                for _ in 0..LONG_BENCHMARK_STEPS {
                    black_box(sim.step(3600.0).unwrap()); // 1 hour timesteps
                }
                
                let final_energy = sim.total_energy();
                let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();
                
                // Ensure stability (energy conservation)
                assert!(energy_error < 1e-6, "Energy conservation failed: {:.2e}", energy_error);
            });
        });
        
        group.finish();
    }
}

/// Memory allocation benchmarks
mod memory_benchmarks {
    use super::*;
    
    pub fn bench_particle_set_operations(c: &mut Criterion) {
        let mut group = c.benchmark_group("memory/particle_set");
        
        group.bench_function("add_1000_particles", |b| {
            b.iter(|| {
                let mut builder = SimulationBuilder::new()
                    .with_integrator(VelocityVerlet::new())
                    .with_force_calculator(DirectGravity::new());
                
                for i in 0..1000 {
                    let body = Body::new()
                        .with_mass(EARTH_MASS)
                        .with_position([i as f64 * 1000.0, 0.0, 0.0])
                        .with_velocity([0.0, 0.0, 0.0]);
                    builder = black_box(builder.add_body(body).unwrap());
                }
                
                black_box(builder.build().unwrap());
            });
        });
        
        group.bench_function("simulation_creation_overhead", |b| {
            let bodies = setup::create_random_system(1000);
            
            b.iter(|| {
                let mut builder = SimulationBuilder::new()
                    .with_integrator(VelocityVerlet::new())
                    .with_force_calculator(DirectGravity::new());
                
                for body in bodies.clone() {
                    builder = builder.add_body(body).unwrap();
                }
                
                black_box(builder.build().unwrap());
            });
        });
        
        group.finish();
    }
}

/// Throughput and scaling benchmarks
mod scaling_benchmarks {
    use super::*;
    
    pub fn bench_algorithm_scaling(c: &mut Criterion) {
        let mut group = c.benchmark_group("scaling/algorithm_comparison");
        
        // Compare O(N²) vs O(N log N) scaling
        let large_particle_counts = vec![500, 1000, 2000, 4000];
        
        for &particle_count in &large_particle_counts {
            // Direct O(N²) algorithm
            group.bench_with_input(
                BenchmarkId::new("direct_gravity", particle_count),
                &particle_count,
                |b, &count| {
                    let bodies = setup::create_random_system(count);
                    let mut sim = SimulationBuilder::new()
                        .with_integrator(VelocityVerlet::new())
                        .with_force_calculator(DirectGravity::new());
                    
                    for body in bodies {
                        sim = sim.add_body(body).unwrap();
                    }
                    
                    let mut sim = sim.build().unwrap();
                    
                    b.iter(|| {
                        black_box(sim.step(0.001).unwrap());
                    });
                },
            );
            
            // Barnes-Hut O(N log N) algorithm
            group.bench_with_input(
                BenchmarkId::new("barnes_hut", particle_count),
                &particle_count,
                |b, &count| {
                    let bodies = setup::create_random_system(count);
                    let mut sim = SimulationBuilder::new()
                        .with_integrator(VelocityVerlet::new())
                        .with_force_calculator(BarnesHut::new().theta(0.5));
                    
                    for body in bodies {
                        sim = sim.add_body(body).unwrap();
                    }
                    
                    let mut sim = sim.build().unwrap();
                    
                    b.iter(|| {
                        black_box(sim.step(0.001).unwrap());
                    });
                },
            );
        }
        
        group.finish();
    }
    
    pub fn bench_timestep_scaling(c: &mut Criterion) {
        let mut group = c.benchmark_group("scaling/timestep_impact");
        
        for &timestep in TIMESTEPS {
            group.bench_with_input(
                BenchmarkId::new("timestep", timestep),
                &timestep,
                |b, &dt| {
                    let bodies = setup::create_random_system(1000);
                    let mut sim = SimulationBuilder::new()
                        .with_integrator(VelocityVerlet::new())
                        .with_force_calculator(DirectGravity::new());
                    
                    for body in bodies {
                        sim = sim.add_body(body).unwrap();
                    }
                    
                    let mut sim = sim.build().unwrap();
                    
                    b.iter(|| {
                        black_box(sim.step(dt).unwrap());
                    });
                },
            );
        }
        
        group.finish();
    }
}

// Benchmark group registration
criterion_group!(
    force_benches,
    force_benchmarks::bench_direct_gravity,
    force_benchmarks::bench_barnes_hut,
);

criterion_group!(
    integrator_benches,
    integrator_benchmarks::bench_velocity_verlet,
    integrator_benchmarks::bench_leapfrog,
    integrator_benchmarks::bench_rk4,
);

criterion_group!(
    simulation_benches,
    simulation_benchmarks::bench_60fps_target,
    simulation_benchmarks::bench_solar_system_simulation,
    simulation_benchmarks::bench_long_term_stability,
);

criterion_group!(
    memory_benches,
    memory_benchmarks::bench_particle_set_operations,
);

criterion_group!(
    scaling_benches,
    scaling_benchmarks::bench_algorithm_scaling,
    scaling_benchmarks::bench_timestep_scaling,
);

criterion_main!(
    force_benches,
    integrator_benches, 
    simulation_benches,
    memory_benches,
    scaling_benches,
);