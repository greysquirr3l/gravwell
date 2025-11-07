//! SIMD Performance Demonstration
//!
//! This example demonstrates the performance improvements available through
//! SIMD vectorization in Gravwell. It compares scalar and vectorized force
//! calculations across different particle counts and reports the speedup.

use gravwell::prelude::*;
use std::time::Instant;

fn main() -> Result<()> {
    println!("🧮 SIMD Performance Demonstration");
    println!("=================================");

    // Detect SIMD capabilities
    #[cfg(feature = "simd")]
    {
        let simd_capabilities = gravwell::simd::detect_cpu_features();
        println!("🖥️  CPU SIMD Capabilities:");
        println!(
            "   • Best SIMD Level: {}",
            simd_capabilities.best_simd_level().description()
        );
        println!(
            "   • Expected Speedup: {:.1}x",
            simd_capabilities.best_simd_level().speedup_factor()
        );
        println!();
    }

    #[cfg(not(feature = "simd"))]
    {
        println!("⚠️  SIMD feature not enabled. Enable with --features simd");
        println!();
    }

    let particle_counts = vec![100, 500, 1000, 2000];

    for &count in &particle_counts {
        println!("🧪 Testing {} particles:", count);
        test_force_calculation_performance(count)?;
        println!();
    }

    println!("📊 Summary:");
    println!("==========");
    println!("• SIMD provides the most benefit for larger particle counts (1000+)");
    #[cfg(feature = "simd")]
    println!("• VectorizedGravity automatically selects the best SIMD implementation");
    println!("• For optimal performance, ensure your CPU supports AVX2 or AVX-512");

    Ok(())
}

fn test_force_calculation_performance(particle_count: usize) -> Result<()> {
    // Create test particle system
    let mut particles = ParticleSet::new();

    // Generate particles in a spherical distribution
    for i in 0..particle_count {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / particle_count as f64;
        let radius = 1000.0 + (i as f64 * 10.0);

        particles.add_body(
            Body::new()
                .with_position([
                    radius * angle.cos(),
                    radius * angle.sin(),
                    (i as f64 * 0.1) % 100.0,
                ])
                .with_velocity([-10.0 * angle.sin(), 10.0 * angle.cos(), 0.0])
                .mass(1.0e20 + (i as f64 * 1e18))
                .with_radius(10.0),
        )?;
    }

    let _n = particles.len();
    let steps = 100;

    // Test scalar direct gravity
    println!("  📊 Scalar Force Calculation:");
    let direct_calc = DirectGravity::new();
    let scalar_time = benchmark_force_calculator(&direct_calc, &particles, steps)?;
    println!(
        "     Direct Gravity in {:.0}ms ({:.2}μs per step)",
        scalar_time.as_millis(),
        scalar_time.as_micros() as f64 / steps as f64
    );

    // Test SIMD vectorized gravity (if available)
    #[cfg(feature = "simd")]
    {
        println!("  ⚡ SIMD Force Calculation:");
        let vectorized_calc = VectorizedGravity::new();
        let simd_time = benchmark_force_calculator(&vectorized_calc, &particles, steps)?;
        println!(
            "     Vectorized Gravity in {:.0}ms ({:.2}μs per step)",
            simd_time.as_millis(),
            simd_time.as_micros() as f64 / steps as f64
        );

        let speedup = scalar_time.as_secs_f64() / simd_time.as_secs_f64();
        println!("  📈 Results:");
        println!("     Speedup: {:.2}x", speedup);
        println!("     Scalar time: {:.0}ms", scalar_time.as_millis());
        println!("     SIMD time: {:.0}ms", simd_time.as_millis());

        if speedup >= 2.0 {
            println!("     ✅ Excellent SIMD performance!");
        } else if speedup >= 1.5 {
            println!("     ✅ Good SIMD performance");
        } else if speedup >= 1.1 {
            println!("     ⚠️  Modest SIMD improvement");
        } else {
            println!("     ⚠️  SIMD overhead > benefit (try larger particle counts)");
        }
    }

    #[cfg(not(feature = "simd"))]
    {
        println!("  ⚠️  SIMD benchmarks skipped (feature not enabled)");
        println!("     Enable with: cargo run --example simd_performance --features simd");
    }

    Ok(())
}

fn benchmark_force_calculator<F>(
    calculator: &F,
    particles: &ParticleSet,
    steps: usize,
) -> Result<std::time::Duration>
where
    F: ForceCalculator,
{
    let mut forces = vec![Vector3::zeros(); particles.len()];

    let start = Instant::now();
    for _ in 0..steps {
        calculator.calculate_forces(particles, &mut forces)?;

        // Prevent optimization from eliminating the calculation
        std::hint::black_box(&forces);
    }
    let duration = start.elapsed();

    Ok(duration)
}

#[allow(dead_code)]
fn create_galaxy_distribution(particle_count: usize) -> Result<ParticleSet> {
    let mut particles = ParticleSet::new();

    for i in 0..particle_count {
        // Spiral galaxy distribution
        let arm_angle = 2.0 * std::f64::consts::PI * (i as f64 / particle_count as f64) * 3.0; // 3 spiral arms
        let radius = 1000.0 + (i as f64).sqrt() * 100.0; // Increasing radius
        let height = (fastrand::f64() - 0.5) * 50.0; // Small vertical scatter

        let x = radius * arm_angle.cos();
        let y = radius * arm_angle.sin();
        let z = height;

        // Orbital velocity (simplified)
        let orbital_speed = 100.0 / radius.sqrt();
        let vx = -orbital_speed * arm_angle.sin();
        let vy = orbital_speed * arm_angle.cos();
        let vz = 0.0;

        // Mass proportional to position in galaxy (more massive near center)
        let mass = 1e20 * (10000.0 / (radius + 100.0));

        particles.add_body(
            Body::new()
                .with_position([x, y, z])
                .with_velocity([vx, vy, vz])
                .mass(mass)
                .with_radius(1.0),
        )?;
    }

    Ok(particles)
}
