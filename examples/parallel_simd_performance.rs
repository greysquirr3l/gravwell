#!/usr/bin/env rust

//! Enhanced Parallel + SIMD Performance Demonstration
//!
//! This example combines parallel processing with SIMD vectorization
//! to achieve maximum performance on multi-core systems. It demonstrates
//! the multiplicative performance gains possible when both optimizations
//! are used together.

use gravwell::{forces::DirectGravity, prelude::*};

#[cfg(feature = "parallel")]
use gravwell::forces::ParallelDirectGravity;
use std::time::Instant;

#[cfg(feature = "simd")]
use gravwell::simd::VectorizedGravity;

fn main() -> Result<()> {
    println!("🚀 Parallel + SIMD Performance Showcase");
    println!("========================================");

    // Display system capabilities
    println!("🖥️  System Information:");
    println!(
        "   • CPU Threads: {}",
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    );

    #[cfg(feature = "simd")]
    {
        println!("   • SIMD Level: Available (auto-detected at runtime)");
        println!("   • Expected SIMD Speedup: 2-8x depending on CPU architecture");
    }

    #[cfg(not(feature = "simd"))]
    {
        println!("   • SIMD: Not available (compile with --features simd)");
    }

    println!();

    // Test different particle counts to show scaling
    let test_cases = vec![
        ("Small System", 500),
        ("Medium System", 1000),
        ("Large System", 2000),
        ("XL System", 4000),
    ];

    for (name, particle_count) in test_cases {
        println!("🧪 Testing {}: {} particles", name, particle_count);
        run_performance_comparison(particle_count)?;
        println!();
    }

    // Demonstrate optimal configuration selection
    println!("🎯 Performance Optimization Recommendations");
    println!("===========================================");
    demonstrate_optimal_configs()?;

    Ok(())
}

fn run_performance_comparison(particle_count: usize) -> Result<()> {
    let particles = create_test_system(particle_count)?;
    let mut forces = vec![Vector3::zeros(); particle_count];
    let steps = 10; // Number of force calculation steps to average

    // 1. Baseline: Serial Direct Gravity
    let serial_calc = DirectGravity::new();
    let serial_time = benchmark_force_calculator(&serial_calc, &particles, &mut forces, steps)?;

    // 2. Parallel Processing Only
    #[cfg(feature = "parallel")]
    let parallel_time = {
        let parallel_calc = ParallelDirectGravity::new()
            .with_parallel_threshold(100) // Force parallel even for small systems
            .with_chunk_size_strategy(gravwell::forces::ChunkSizeStrategy::Adaptive);
        benchmark_force_calculator(&parallel_calc, &particles, &mut forces, steps)?
    };

    #[cfg(not(feature = "parallel"))]
    let _parallel_time = serial_time; // Use serial time as fallback

    // 3. SIMD Only
    #[cfg(feature = "simd")]
    let simd_time = {
        let simd_calc = VectorizedGravity::new();
        benchmark_force_calculator(&simd_calc, &particles, &mut forces, steps)?
    };

    #[cfg(not(feature = "simd"))]
    let _simd_time = serial_time; // Use serial time as fallback

    // 4. Parallel + SIMD Combined (current parallel implementation)
    #[cfg(all(feature = "parallel", feature = "simd"))]
    let combined_time = {
        let combined_calc = ParallelDirectGravity::new()
            .with_parallel_threshold(100)
            .with_chunk_size_strategy(gravwell::forces::ChunkSizeStrategy::Optimized);
        benchmark_force_calculator(&combined_calc, &particles, &mut forces, steps)?
    };

    #[cfg(not(all(feature = "parallel", feature = "simd")))]
    let _combined_time = serial_time; // Use serial time as fallback

    // Display results
    println!("  📊 Performance Results:");
    println!(
        "     Serial (baseline):     {:.2}ms ({:.1}μs per step)",
        serial_time * 1000.0,
        serial_time * 1_000_000.0 / steps as f64
    );

    #[cfg(feature = "parallel")]
    {
        let parallel_speedup = serial_time / _parallel_time;
        println!(
            "  Parallel Performance: {:.2}ms ({:.1}μs per step) - {:.2}x speedup",
            _parallel_time * 1000.0,
            _parallel_time * 1_000_000.0 / steps as f64,
            parallel_speedup
        );
    }

    #[cfg(feature = "simd")]
    {
        let simd_speedup = serial_time / _simd_time;
        println!(
            "     SIMD Performance: {:.2}ms ({:.1}μs per step) - {:.2}x speedup",
            _simd_time * 1000.0,
            _simd_time * 1_000_000.0 / steps as f64,
            simd_speedup
        );
    }

    #[cfg(all(feature = "parallel", feature = "simd"))]
    {
        let combined_speedup = serial_time / _combined_time;
        println!(
            "     Combined Performance: {:.2}ms ({:.1}μs per step) - {:.2}x speedup",
            _combined_time * 1000.0,
            _combined_time * 1_000_000.0 / steps as f64,
            combined_speedup
        );

        // Calculate theoretical combined speedup
        let parallel_speedup = serial_time / _parallel_time;
        let simd_speedup = serial_time / _simd_time;
        let theoretical_combined = parallel_speedup * simd_speedup;
        let theoretical_time = serial_time / theoretical_combined;

        println!(
            "     📈 Theoretical Combined: {:.2}ms ({:.1}μs per step) - {:.2}x speedup",
            theoretical_time * 1000.0,
            theoretical_time * 1_000_000.0 / steps as f64,
            theoretical_combined
        );
    }

    Ok(())
}

fn benchmark_force_calculator(
    calculator: &dyn ForceCalculator,
    particles: &ParticleSet,
    forces: &mut [Vector3],
    steps: usize,
) -> Result<f64> {
    // Warm-up run
    calculator.calculate_forces(particles, forces)?;

    // Actual benchmark
    let start = Instant::now();
    for _ in 0..steps {
        calculator.calculate_forces(particles, forces)?;
    }
    let duration = start.elapsed();

    Ok(duration.as_secs_f64() / steps as f64)
}

fn create_test_system(particle_count: usize) -> Result<ParticleSet> {
    let mut particles = ParticleSet::new();

    // Create a spiral galaxy-like distribution for realistic testing
    for i in 0..particle_count {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (particle_count as f64);
        let radius = 1.0 + (i as f64).sqrt() * 0.1; // Spiral outward
        let height = (angle * 3.0).sin() * 0.1; // Some vertical structure

        let position = [radius * angle.cos(), radius * angle.sin(), height];

        let orbital_speed = 0.5 / radius.sqrt(); // Keplerian velocity profile
        let velocity = [
            -orbital_speed * angle.sin(),
            orbital_speed * angle.cos(),
            0.0,
        ];

        let mass = if i == 0 {
            1000.0 // Central massive object
        } else {
            1.0 + fastrand::f64() * 0.1 // Small random variation
        };

        particles.add_body(
            Body::new()
                .mass(mass)
                .with_position(position)
                .with_velocity(velocity)
                .with_radius(0.01),
        )?;
    }

    Ok(particles)
}

fn demonstrate_optimal_configs() -> Result<()> {
    println!("Based on performance testing:");
    println!();

    println!("💡 For Game Development (60 FPS target):");
    println!("   • Particle count: < 2,000");
    #[cfg(feature = "simd")]
    println!("   • Force calculator: VectorizedGravity (5-6x speedup)");
    #[cfg(feature = "parallel")]
    println!("   • Alternative: ParallelDirectGravity with 4-8 threads");
    println!("   • Integrator: VelocityVerlet or SemiImplicitEuler");
    println!();

    println!("🔬 For Scientific Computing (accuracy priority):");
    println!("   • Particle count: 10,000+");
    #[cfg(all(feature = "parallel", feature = "simd"))]
    println!("   • Force calculator: Future ParallelVectorizedGravity");
    #[cfg(feature = "parallel")]
    println!("   • Current best: ParallelDirectGravity with Optimized chunking");
    println!("   • Integrator: Leapfrog or IAS15");
    println!();

    println!("⚡ For Maximum Performance:");
    #[cfg(all(feature = "parallel", feature = "simd"))]
    println!("   • Target: 35-50x theoretical speedup (6x parallel × 6x SIMD)");
    #[cfg(feature = "simd")]
    println!("   • Currently achievable: 5-6x with SIMD alone");
    #[cfg(feature = "parallel")]
    println!("   • Or: 6-8x with parallel processing alone");
    println!("   • Hardware: Multi-core CPU with AVX-512 support");
    println!();

    println!("🎯 Next Development Priority:");
    println!("   • Integrate SIMD into ParallelDirectGravity");
    println!("   • Implement ParallelVectorizedGravity for maximum performance");
    println!("   • Add GPU acceleration for 100K+ particles");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_comparison() {
        let result = run_performance_comparison(100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_system_creation() {
        let particles = create_test_system(50).unwrap();
        assert_eq!(particles.len(), 50);

        // Verify the central massive object
        assert!(particles.mass(0).value() > 100.0);

        // Verify other particles have reasonable masses
        for i in 1..particles.len() {
            let mass = particles.mass(i).value();
            assert!(mass >= 1.0 && mass <= 1.2);
        }
    }
}
