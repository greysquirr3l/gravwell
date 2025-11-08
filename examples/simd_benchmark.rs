//! SIMD performance benchmark example.
//!
//! This example demonstrates the performance benefits of SIMD optimization
//! for gravity force calculations by comparing different SIMD levels.

use gravwell::prelude::*;
use gravwell::simd::{detect_cpu_features, SimdLevel, VectorizedGravity};
use std::time::Instant;

fn main() -> Result<()> {
    println!("🚀 SIMD Performance Benchmark");
    println!("=============================\n");

    // Detect CPU capabilities
    let features = detect_cpu_features();
    let best_level = features.best_simd_level();

    println!("🔍 CPU Feature Detection:");
    println!("  SSE2:     {}", features.has_sse2);
    println!("  AVX2:     {}", features.has_avx2);
    println!("  AVX-512:  {}", features.has_avx512f);
    println!("  NEON:     {}", features.has_neon);
    println!(
        "  Best:     {} ({}x speedup)\n",
        best_level.description(),
        best_level.speedup_factor()
    );

    // Create test systems of different sizes
    let particle_counts = vec![100, 500, 1000];

    for &n in &particle_counts {
        println!("📊 Benchmarking {} particles:", n);
        benchmark_particle_count(n)?;
        println!();
    }

    // Test accuracy consistency
    println!("🎯 Testing SIMD Accuracy Consistency:");
    test_simd_accuracy()?;

    Ok(())
}

fn benchmark_particle_count(n: usize) -> Result<()> {
    // Create test particle system
    let particles = create_test_system(n)?;

    // Test different SIMD levels
    let simd_levels = vec![
        (SimdLevel::Scalar, "Scalar"),
        (SimdLevel::Sse2, "SSE2"),
        (SimdLevel::Avx2, "AVX2"),
        (SimdLevel::Avx512, "AVX-512"),
        (SimdLevel::Neon, "NEON"),
    ];

    let mut results = Vec::new();

    for (level, name) in simd_levels {
        let calc = VectorizedGravity::with_simd_level(level);
        let mut forces = vec![Vector3::zeros(); n];

        // Warmup
        for _ in 0..10 {
            calc.calculate_forces(&particles, &mut forces)?;
        }

        // Benchmark
        let iterations = 100;
        let start = Instant::now();

        for _ in 0..iterations {
            calc.calculate_forces(&particles, &mut forces)?;
        }

        let duration = start.elapsed();
        let avg_time = duration.as_secs_f64() / iterations as f64;
        let steps_per_sec = 1.0 / avg_time;

        results.push((name, avg_time, steps_per_sec));

        println!(
            "  {:<8}: {:.3} ms/step, {:.0} steps/sec",
            name,
            avg_time * 1000.0,
            steps_per_sec
        );
    }

    // Calculate speedup vs scalar
    if let Some((_, scalar_time, _)) = results.iter().find(|(name, _, _)| *name == "Scalar") {
        println!("  Speedup vs Scalar:");
        for (name, time, _) in &results {
            if *name != "Scalar" {
                let speedup = scalar_time / time;
                println!("    {:<8}: {:.2}x", name, speedup);
            }
        }
    }

    Ok(())
}

fn test_simd_accuracy() -> Result<()> {
    let particles = create_test_system(4)?;

    let simd_levels = vec![
        SimdLevel::Scalar,
        SimdLevel::Sse2,
        SimdLevel::Avx2,
        SimdLevel::Avx512,
        SimdLevel::Neon,
    ];

    let mut all_forces = Vec::new();

    // Calculate forces with each SIMD level
    for level in &simd_levels {
        let calc = VectorizedGravity::with_simd_level(*level);
        let mut forces = vec![Vector3::zeros(); 4];
        calc.calculate_forces(&particles, &mut forces)?;
        all_forces.push(forces);

        println!(
            "  {} forces: [{:.3e}, {:.3e}, {:.3e}]",
            level.description(),
            forces[0].norm(),
            forces[1].norm(),
            forces[2].norm()
        );
    }

    // Compare against scalar reference
    let scalar_forces = &all_forces[0];
    let mut max_error: f64 = 0.0;

    for (i, forces) in all_forces.iter().enumerate().skip(1) {
        for j in 0..4 {
            let error = (forces[j] - scalar_forces[j]).norm() / scalar_forces[j].norm().max(1e-100);
            max_error = max_error.max(error);
        }
    }

    println!("  Maximum relative error: {:.3e}", max_error);

    if max_error < 1e-10 {
        println!("  ✅ All SIMD implementations match scalar reference");
    } else {
        println!("  ⚠️  SIMD implementations differ from scalar reference");
    }

    Ok(())
}

fn create_test_system(n: usize) -> Result<ParticleSet> {
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(42); // Fixed seed for reproducibility
    let mut particles = ParticleSet::with_capacity(n);

    for i in 0..n {
        // Create random but realistic particle system
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
        let radius = 1.0 + rng.gen::<f64>() * 5.0; // 1-6 units from center

        let position = [
            radius * angle.cos(),
            radius * angle.sin(),
            (rng.gen::<f64>() - 0.5) * 2.0, // Random z component
        ];

        let mass = 1e30 * (0.5 + rng.gen::<f64>() * 1.5); // 0.5 to 2.0 solar masses

        particles.add_body(
            Body::new()
                .with_mass(mass)
                .with_position(position)
                .with_velocity([0.0, 0.0, 0.0]),
        )?;
    }

    Ok(particles)
}
