//! Comprehensive performance benchmarks for ParallelVectorizedGravity.
//!
//! This benchmark suite validates the 35-50x speedup target by comparing
//! ParallelVectorizedGravity against individual parallel and SIMD implementations
//! across different particle counts and system configurations.
//!
//! Run with: cargo bench --bench parallel_vectorized_performance --features="parallel,simd"

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gravwell::prelude::*;

#[cfg(all(feature = "parallel", feature = "simd"))]
use gravwell::forces::{ParallelDirectGravity, ParallelVectorizedGravity};

/// Create a galaxy-like particle distribution.
fn create_galaxy_system(particle_count: usize) -> ParticleSet {
    let mut particles = ParticleSet::with_capacity(particle_count);

    for i in 0..particle_count {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / particle_count as f64;
        let radius = 10.0 + (i as f64 / 100.0);

        let position = [
            radius * angle.cos(),
            radius * angle.sin(),
            0.1 * (i as f64).sin(),
        ];

        let orbital_velocity = (10.0 / radius).sqrt();
        let velocity = [
            -orbital_velocity * angle.sin(),
            orbital_velocity * angle.cos(),
            0.0,
        ];

        let body = Body::new()
            .with_position(position)
            .with_velocity(velocity)
            .with_mass(1.0);

        particles.add_body(body).unwrap();
    }

    particles
}

/// Create a clustered particle distribution.
#[cfg(all(feature = "parallel", feature = "simd"))]
fn create_clustered_system(particle_count: usize) -> ParticleSet {
    let mut particles = ParticleSet::with_capacity(particle_count);

    for i in 0..particle_count {
        let cluster_id = i / 100;
        let cluster_center_x = (cluster_id as f64) * 10.0;
        let local_offset = (i % 100) as f64 * 0.1;

        let position = [cluster_center_x + local_offset, 0.0, 0.0];
        let velocity = [0.0, 0.0, 0.0];

        let body = Body::new()
            .with_position(position)
            .with_velocity(velocity)
            .with_mass(1.0);

        particles.add_body(body).unwrap();
    }

    particles
}

/// Create a uniform particle distribution.
#[cfg(all(feature = "parallel", feature = "simd"))]
fn create_uniform_system(particle_count: usize) -> ParticleSet {
    let mut particles = ParticleSet::with_capacity(particle_count);

    for i in 0..particle_count {
        let x = (i as f64 % 10.0) - 5.0;
        let y = ((i / 10) as f64 % 10.0) - 5.0;
        let z = ((i / 100) as f64 % 10.0) - 5.0;

        let position = [x, y, z];
        let velocity = [0.0, 0.0, 0.0];

        let body = Body::new()
            .with_position(position)
            .with_velocity(velocity)
            .with_mass(1.0);

        particles.add_body(body).unwrap();
    }

    particles
}

/// Benchmark direct gravity calculation.
fn bench_direct_gravity(c: &mut Criterion) {
    let particles = create_galaxy_system(1000);
    let mut forces = vec![Force::zeros(); particles.len()];

    let calculator = DirectGravity::new();

    c.bench_function("direct_gravity_1000", |b| {
        b.iter(|| {
            calculator
                .calculate_forces(black_box(&particles), black_box(&mut forces))
                .unwrap();
        });
    });
}

/// Benchmark SIMD gravity calculation.
#[cfg(feature = "simd")]
fn bench_vectorized_gravity(c: &mut Criterion) {
    let particles = create_galaxy_system(1000);
    let mut forces = vec![Force::zeros(); particles.len()];

    let calculator = VectorizedGravity::new();

    c.bench_function("vectorized_gravity_1000", |b| {
        b.iter(|| {
            calculator
                .calculate_forces(black_box(&particles), black_box(&mut forces))
                .unwrap();
        });
    });
}

/// Benchmark parallel gravity calculation.
#[cfg(feature = "parallel")]
fn bench_parallel_gravity(c: &mut Criterion) {
    let particles = create_galaxy_system(1000);
    let mut forces = vec![Force::zeros(); particles.len()];

    let calculator = ParallelDirectGravity::new();

    c.bench_function("parallel_gravity_1000", |b| {
        b.iter(|| {
            calculator
                .calculate_forces(black_box(&particles), black_box(&mut forces))
                .unwrap();
        });
    });
}

/// Benchmark parallel vectorized gravity calculation.
#[cfg(all(feature = "parallel", feature = "simd"))]
fn bench_parallel_vectorized_gravity(c: &mut Criterion) {
    let particles = create_galaxy_system(1000);
    let mut forces = vec![Force::zeros(); particles.len()];

    let calculator = ParallelVectorizedGravity::new();

    c.bench_function("parallel_vectorized_gravity_1000", |b| {
        b.iter(|| {
            calculator
                .calculate_forces(black_box(&particles), black_box(&mut forces))
                .unwrap();
        });
    });
}

/// Benchmark scaling comparison across all implementations.
#[cfg(all(feature = "parallel", feature = "simd"))]
fn bench_scaling_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling_comparison");
    let particle_counts = vec![100, 500, 1000, 2000];

    for &count in &particle_counts {
        let particles = create_galaxy_system(count);
        let mut forces = vec![Force::zeros(); particles.len()];

        // Direct gravity baseline
        let direct_calc = DirectGravity::new();
        group.bench_with_input(BenchmarkId::new("direct", count), &count, |b, _| {
            b.iter(|| {
                direct_calc
                    .calculate_forces(black_box(&particles), black_box(&mut forces))
                    .unwrap()
            })
        });

        // Vectorized gravity
        let vectorized_calc = VectorizedGravity::new();
        group.bench_with_input(BenchmarkId::new("vectorized", count), &count, |b, _| {
            b.iter(|| {
                vectorized_calc
                    .calculate_forces(black_box(&particles), black_box(&mut forces))
                    .unwrap()
            })
        });

        // Parallel gravity
        let parallel_calc = ParallelDirectGravity::new();
        group.bench_with_input(BenchmarkId::new("parallel", count), &count, |b, _| {
            b.iter(|| {
                parallel_calc
                    .calculate_forces(black_box(&particles), black_box(&mut forces))
                    .unwrap()
            })
        });

        // Parallel vectorized gravity
        let parallel_vectorized_calc = ParallelVectorizedGravity::new();
        group.bench_with_input(
            BenchmarkId::new("parallel_vectorized", count),
            &count,
            |b, _| {
                b.iter(|| {
                    parallel_vectorized_calc
                        .calculate_forces(black_box(&particles), black_box(&mut forces))
                        .unwrap()
                })
            },
        );
    }

    group.finish();
}

/// Comprehensive SIMD level comparison.
#[cfg(all(feature = "parallel", feature = "simd"))]
fn bench_simd_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_levels");

    let particles = create_galaxy_system(1000);
    let mut forces = vec![Force::zeros(); particles.len()];

    // Test different SIMD implementations
    group.bench_function("scalar", |b| {
        let calc = DirectGravity::new(); // Pure scalar implementation
        b.iter(|| {
            calc.calculate_forces(black_box(&particles), black_box(&mut forces))
                .unwrap()
        });
    });

    group.bench_function("auto_simd", |b| {
        let calc = VectorizedGravity::new(); // Auto-detect SIMD
        b.iter(|| {
            calc.calculate_forces(black_box(&particles), black_box(&mut forces))
                .unwrap()
        });
    });

    group.bench_function("parallel_auto_simd", |b| {
        let calc = ParallelVectorizedGravity::new(); // Parallel + Auto SIMD
        b.iter(|| {
            calc.calculate_forces(black_box(&particles), black_box(&mut forces))
                .unwrap()
        });
    });

    group.finish();
}

/// Test different particle distributions.
#[cfg(all(feature = "parallel", feature = "simd"))]
fn bench_particle_distributions(c: &mut Criterion) {
    let mut group = c.benchmark_group("particle_distributions");

    let calc = ParallelVectorizedGravity::new();

    // Galaxy distribution
    let galaxy_particles = create_galaxy_system(1000);
    let mut forces = vec![Force::zeros(); galaxy_particles.len()];

    group.bench_function("galaxy_distribution", |b| {
        b.iter(|| {
            calc.calculate_forces(black_box(&galaxy_particles), black_box(&mut forces))
                .unwrap()
        });
    });

    // Clustered distribution
    let clustered_particles = create_clustered_system(1000);
    let mut forces_2 = vec![Force::zeros(); clustered_particles.len()];

    group.bench_function("clustered_distribution", |b| {
        b.iter(|| {
            calc.calculate_forces(black_box(&clustered_particles), black_box(&mut forces_2))
                .unwrap()
        });
    });

    // Uniform distribution
    let uniform_particles = create_uniform_system(1000);
    let mut forces_3 = vec![Force::zeros(); uniform_particles.len()];

    group.bench_function("uniform_distribution", |b| {
        b.iter(|| {
            calc.calculate_forces(black_box(&uniform_particles), black_box(&mut forces_3))
                .unwrap()
        });
    });

    group.finish();
}

// Define all benchmark groups with proper feature gates
#[cfg(all(feature = "parallel", feature = "simd"))]
criterion_group!(
    comprehensive_benchmarks,
    bench_direct_gravity,
    bench_vectorized_gravity,
    bench_parallel_gravity,
    bench_parallel_vectorized_gravity,
    bench_scaling_comparison,
    bench_simd_levels,
    bench_particle_distributions
);

#[cfg(all(feature = "simd", not(feature = "parallel")))]
criterion_group!(
    simd_benchmarks,
    bench_direct_gravity,
    bench_vectorized_gravity
);

#[cfg(all(feature = "parallel", not(feature = "simd")))]
criterion_group!(
    parallel_benchmarks,
    bench_direct_gravity,
    bench_parallel_gravity
);

#[cfg(not(any(feature = "parallel", feature = "simd")))]
criterion_group!(basic_benchmarks, bench_direct_gravity);

// Main criterion entry point with conditional compilation
#[cfg(all(feature = "parallel", feature = "simd"))]
criterion_main!(comprehensive_benchmarks);

#[cfg(all(feature = "simd", not(feature = "parallel")))]
criterion_main!(simd_benchmarks);

#[cfg(all(feature = "parallel", not(feature = "simd")))]
criterion_main!(parallel_benchmarks);

#[cfg(not(any(feature = "parallel", feature = "simd")))]
criterion_main!(basic_benchmarks);
