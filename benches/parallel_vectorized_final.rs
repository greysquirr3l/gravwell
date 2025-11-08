//! Benchmark for ParallelVectorizedGravity performance validation.
//!
//! This benchmark requires the "parallel" and "simd" features to be enabled:
//! cargo bench --bench parallel_vectorized_final --features="parallel,simd"

use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(all(feature = "parallel", feature = "simd"))]
use gravwell::prelude::*;

#[cfg(all(feature = "parallel", feature = "simd"))]
use gravwell::forces::{ParallelDirectGravity, ParallelVectorizedGravity};

#[cfg(all(feature = "parallel", feature = "simd"))]
fn bench_parallel_vectorized_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_vectorized_performance");

    for n_particles in [100, 500, 1000].iter() {
        let mut particles = ParticleSet::with_capacity(*n_particles);

        // Add particles in a galaxy-like spiral
        for i in 0..*n_particles {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (*n_particles as f64);
            let radius = 10.0 + (i as f64 / 100.0);

            let x = radius * angle.cos();
            let y = radius * angle.sin();
            let z = 0.1 * (i as f64).sin();

            let orbital_velocity = (10.0 / radius).sqrt();
            let vx = -orbital_velocity * angle.sin();
            let vy = orbital_velocity * angle.cos();

            let body = Body::new()
                .with_mass(1.0)
                .with_position([x, y, z])
                .with_velocity([vx, vy, 0.0]);

            particles.add_body(body).unwrap();
        }

        let mut forces = vec![Force::zeros(); particles.len()];

        // DirectGravity baseline
        let direct_calc = DirectGravity::new();
        group.bench_with_input(
            BenchmarkId::new("direct_gravity", n_particles),
            n_particles,
            |b, _| {
                b.iter(|| {
                    direct_calc
                        .calculate_forces(&particles, &mut forces)
                        .unwrap();
                });
            },
        );

        // VectorizedGravity SIMD
        let vectorized_calc = VectorizedGravity::new();
        group.bench_with_input(
            BenchmarkId::new("vectorized_gravity", n_particles),
            n_particles,
            |b, _| {
                b.iter(|| {
                    vectorized_calc
                        .calculate_forces(&particles, &mut forces)
                        .unwrap();
                });
            },
        );

        // ParallelDirectGravity multi-threading
        let parallel_calc = ParallelDirectGravity::new();
        group.bench_with_input(
            BenchmarkId::new("parallel_gravity", n_particles),
            n_particles,
            |b, _| {
                b.iter(|| {
                    parallel_calc
                        .calculate_forces(&particles, &mut forces)
                        .unwrap();
                });
            },
        );

        // ParallelVectorizedGravity ultimate optimization
        let parallel_vectorized_calc = ParallelVectorizedGravity::new();
        group.bench_with_input(
            BenchmarkId::new("parallel_vectorized_gravity", n_particles),
            n_particles,
            |b, _| {
                b.iter(|| {
                    parallel_vectorized_calc
                        .calculate_forces(&particles, &mut forces)
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

#[cfg(all(feature = "parallel", feature = "simd"))]
criterion_group!(benches, bench_parallel_vectorized_performance);

#[cfg(not(all(feature = "parallel", feature = "simd")))]
fn bench_empty(_c: &mut Criterion) {}

#[cfg(not(all(feature = "parallel", feature = "simd")))]
criterion_group!(benches, bench_empty);

criterion_main!(benches);
