//! Parallel Processing Performance Demonstration
//!
//! This example demonstrates the performance benefits of parallel force calculations
//! and integration using Rayon. It compares serial vs parallel performance across
//! different particle counts to show the scaling characteristics.
//!
//! Run with:
//! ```bash
//! cargo run --example parallel_performance --features parallel --release
//! ```

#[cfg(feature = "parallel")]
use gravwell::prelude::*;
#[cfg(feature = "parallel")]
use std::time::Instant;

#[cfg(feature = "parallel")]
fn main() -> Result<()> {
    // Configure rayon thread pool
    let thread_count = rayon::current_num_threads();
    println!(
        "Running parallel performance demo with {} threads",
        thread_count
    );
    println!("===============================================");

    // Test configurations
    let particle_counts = vec![100, 500, 1000, 2000, 5000];
    let simulation_steps = 100;

    for &particle_count in &particle_counts {
        println!("\n🚀 Testing {} particles:", particle_count);

        // Create test particle system (clustered distribution for realistic forces)
        let mut particles = ParticleSet::new();
        create_particle_cluster(&mut particles, particle_count)?;

        // Test serial force calculation
        println!("  📊 Serial Force Calculation:");
        let serial_forces_time =
            benchmark_force_calculation(&particles, &DirectGravity::new(), simulation_steps)?;

        // Test parallel force calculation
        println!("  ⚡ Parallel Force Calculation:");
        let parallel_forces_time = benchmark_force_calculation(
            &particles,
            &ParallelDirectGravity::new().with_parallel_threshold(100),
            simulation_steps,
        )?;

        // Test serial integration
        println!("  📊 Serial Integration:");
        let mut serial_particles = particles.clone();
        let serial_integration_time = benchmark_integration(
            &mut serial_particles,
            VelocityVerlet::new(),
            &DirectGravity::new(),
            simulation_steps,
        )?;

        // Test parallel integration
        println!("  ⚡ Parallel Integration:");
        let mut parallel_particles = particles.clone();
        let parallel_integration_time = benchmark_integration(
            &mut parallel_particles,
            ParallelVelocityVerlet::new().with_parallel_threshold(100),
            &ParallelDirectGravity::new().with_parallel_threshold(100),
            simulation_steps,
        )?;

        // Calculate and display speedups
        let force_speedup = serial_forces_time.as_secs_f64() / parallel_forces_time.as_secs_f64();
        let integration_speedup =
            serial_integration_time.as_secs_f64() / parallel_integration_time.as_secs_f64();

        println!("  📈 Results:");
        println!("     Force calculation speedup: {:.2}x", force_speedup);
        println!("     Integration speedup: {:.2}x", integration_speedup);
        println!(
            "     Serial forces: {:.2}ms",
            serial_forces_time.as_millis()
        );
        println!(
            "     Parallel forces: {:.2}ms",
            parallel_forces_time.as_millis()
        );
        println!(
            "     Serial integration: {:.2}ms",
            serial_integration_time.as_millis()
        );
        println!(
            "     Parallel integration: {:.2}ms",
            parallel_integration_time.as_millis()
        );

        // Performance validation
        if particle_count >= 1000 {
            if force_speedup < 2.0 {
                println!("     ⚠️  Force calculation speedup below target (2.0x)");
            } else {
                println!("     ✅ Force calculation speedup meets target");
            }

            if integration_speedup < 1.5 {
                println!("     ⚠️  Integration speedup below target (1.5x)");
            } else {
                println!("     ✅ Integration speedup meets target");
            }
        }
    }

    println!("\n🎯 Performance Summary:");
    println!("===============================================");
    println!("Parallel processing provides the most benefit for:");
    println!("• Force calculations with 1000+ particles");
    println!("• Integration steps with 2000+ particles");
    println!("• Multi-threaded environments ({}+ cores)", thread_count);
    println!("\nFor optimal performance, use:");
    println!("• ParallelDirectGravity for force calculations");
    println!("• ParallelVelocityVerlet for integration");
    println!("• Enable 'parallel' feature flag");

    Ok(())
}

#[cfg(feature = "parallel")]
fn create_particle_cluster(particles: &mut ParticleSet, count: usize) -> Result<()> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for i in 0..count {
        // Create a spherical cluster with some random distribution
        let radius = 100.0 * (i as f64 / count as f64).sqrt();
        let theta = rng.gen::<f64>() * 2.0 * std::f64::consts::PI;
        let phi = rng.gen::<f64>() * std::f64::consts::PI;

        let x = radius * phi.sin() * theta.cos();
        let y = radius * phi.sin() * theta.sin();
        let z = radius * phi.cos();

        // Add some orbital velocity for realistic dynamics
        let velocity_factor = 0.1;
        let vx = -y * velocity_factor + rng.gen_range(-0.1..0.1);
        let vy = x * velocity_factor + rng.gen_range(-0.1..0.1);
        let vz = rng.gen_range(-0.1..0.1);

        particles.add_body(
            Body::new()
                .with_position([x, y, z])
                .with_velocity([vx, vy, vz])
                .with_mass(1.0 + rng.gen::<f64>() * 10.0),
        )?;
    }

    Ok(())
}

#[cfg(feature = "parallel")]
fn benchmark_force_calculation<F>(
    particles: &ParticleSet,
    force_calc: &F,
    iterations: usize,
) -> Result<std::time::Duration>
where
    F: ForceCalculator,
{
    let mut forces = vec![Force::zeros(); particles.len()];

    let start = Instant::now();
    for _ in 0..iterations {
        force_calc.calculate_forces(particles, &mut forces)?;
    }
    let duration = start.elapsed();

    println!(
        "     {} in {:.2}ms ({:.2}μs per step)",
        force_calc.name(),
        duration.as_millis(),
        duration.as_micros() as f64 / iterations as f64
    );

    Ok(duration)
}

#[cfg(feature = "parallel")]
fn benchmark_integration<I, F>(
    particles: &mut ParticleSet,
    mut integrator: I,
    force_calc: &F,
    steps: usize,
) -> Result<std::time::Duration>
where
    I: Integrator,
    F: ForceCalculator,
{
    let dt = 0.01;

    let start = Instant::now();
    for _ in 0..steps {
        integrator.step(particles, force_calc, dt)?;
    }
    let duration = start.elapsed();

    println!(
        "     {} in {:.2}ms ({:.2}μs per step)",
        integrator.name(),
        duration.as_millis(),
        duration.as_micros() as f64 / steps as f64
    );

    Ok(duration)
}

#[cfg(not(feature = "parallel"))]
fn main() {
    println!("This example requires the 'parallel' feature to be enabled.");
    println!("Run with: cargo run --example parallel_performance --features parallel --release");
}
