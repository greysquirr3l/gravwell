//! GPU Barnes-Hut Algorithm Benchmarks
//!
//! Performance benchmarks targeting 50,000+ particles @ 60 FPS

use criterion::{criterion_group, Criterion};
use gravwell::prelude::*;

#[cfg(feature = "gpu")]
use gravwell::forces::GpuBarnesHut;

#[allow(dead_code)]
fn setup_particle_system(n: usize) -> ParticleSet {
    let mut particle_set = ParticleSet::new();

    // Create a spherical distribution of particles
    for i in 0..n {
        let phi = 2.0 * std::f64::consts::PI * (i as f64 / n as f64);
        let costheta = 2.0 * (i as f64 / n as f64) - 1.0;
        let theta = costheta.acos();
        let r = 100.0 * (i as f64 / n as f64).cbrt(); // Cubic root for volume distribution

        let x = r * theta.sin() * phi.cos();
        let y = r * theta.sin() * phi.sin();
        let z = r * costheta;

        let body = Body::new()
            .with_position([x, y, z])
            .with_velocity([0.0, 0.0, 0.0])
            .with_mass(1.0)
            .with_radius(0.1);

        particle_set.add_body(body).unwrap();
    }

    particle_set
}

#[cfg(feature = "gpu")]
fn bench_gpu_barnes_hut_scales(c: &mut Criterion) {
    let mut group = c.benchmark_group("GPU Barnes-Hut Scaling");

    // Test different particle counts to measure O(N log N) scaling
    let particle_counts = vec![1_000, 2_000, 5_000, 10_000, 20_000, 50_000];

    for &n in &particle_counts {
        let particle_set = setup_particle_system(n);
        let gpu_barnes_hut = GpuBarnesHut::new()
            .theta(0.5)
            .max_depth(20)
            .min_particles(8);

        group.bench_with_input(BenchmarkId::new("GPU Barnes-Hut", n), &n, |b, _| {
            b.iter(|| {
                let mut forces = vec![Vector3::zeros(); n];
                let result = gpu_barnes_hut.calculate_forces(&particle_set, &mut forces);
                black_box(result);
                black_box(&forces);
            });
        });
    }

    group.finish();
}

#[cfg(feature = "gpu")]
fn bench_gpu_vs_cpu_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("GPU vs CPU Barnes-Hut");

    let particle_count = 10_000;
    let (positions, masses) = setup_particle_system(particle_count);

    // GPU Barnes-Hut
    let gpu_barnes_hut = GpuBarnesHut::new().theta(0.5);

    // CPU Barnes-Hut
    let cpu_barnes_hut = BarnesHut::new().theta(0.5);

    let mut particle_set = ParticleSet::new();
    for i in 0..particle_count {
        particle_set.add_particle(positions[i], Vector3::zeros(), masses[i], 0.1);
    }

    group.bench_function("GPU Barnes-Hut", |b| {
        b.iter(|| {
            let mut forces = vec![Vector3::zeros(); particle_count];
            let result = gpu_barnes_hut.calculate_forces(&particle_set, &mut forces);
            black_box(result);
            black_box(&forces);
        });
    });

    group.bench_function("CPU Barnes-Hut", |b| {
        b.iter(|| {
            let mut forces = vec![Vector3::zeros(); particle_count];
            let result = cpu_barnes_hut.calculate_forces(&particle_set, &mut forces);
            black_box(result);
            black_box(&forces);
        });
    });

    group.finish();
}

#[cfg(feature = "gpu")]
fn bench_gpu_barnes_hut_theta_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("GPU Barnes-Hut Theta Parameter");

    let particle_count = 5_000;
    let (positions, masses) = setup_particle_system(particle_count);

    let mut particle_set = ParticleSet::new();
    for i in 0..particle_count {
        particle_set.add_particle(positions[i], Vector3::zeros(), masses[i], 0.1);
    }

    // Test different theta values
    let theta_values = vec![0.3, 0.5, 0.7, 1.0];

    for &theta in &theta_values {
        let gpu_barnes_hut = GpuBarnesHut::new().theta(theta);

        group.bench_with_input(
            BenchmarkId::new("GPU Barnes-Hut", format!("theta_{}", theta)),
            &theta,
            |b, _| {
                b.iter(|| {
                    let mut forces = vec![Vector3::zeros(); particle_count];
                    let result = gpu_barnes_hut.calculate_forces(&particle_set, &mut forces);
                    black_box(result);
                    black_box(&forces);
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "gpu")]
fn bench_fps_target_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("60 FPS Target Validation");
    group.measurement_time(std::time::Duration::from_secs(30)); // Longer measurement for accuracy

    // Target: 50,000 particles @ 60 FPS = ~16.67ms per frame
    let particle_count = 50_000;
    let (positions, masses) = setup_particle_system(particle_count);

    let gpu_barnes_hut = GpuBarnesHut::new()
        .theta(0.6) // Slightly more aggressive for performance
        .max_depth(18)
        .min_particles(16);

    let mut particle_set = ParticleSet::new();
    for i in 0..particle_count {
        particle_set.add_particle(positions[i], Vector3::zeros(), masses[i], 0.1);
    }

    group.bench_function("50K particles (60 FPS target)", |b| {
        b.iter(|| {
            let mut forces = vec![Vector3::zeros(); particle_count];
            let result = gpu_barnes_hut.calculate_forces(&particle_set, &mut forces);
            black_box(result);
            black_box(&forces);
        });
    });

    group.finish();
}

// Only compile these benchmarks when GPU feature is enabled
#[cfg(feature = "gpu")]
criterion_group!(
    gpu_benches,
    bench_gpu_barnes_hut_scales,
    bench_gpu_vs_cpu_comparison,
    bench_gpu_barnes_hut_theta_values,
    bench_fps_target_validation
);

#[cfg(not(feature = "gpu"))]
fn dummy_gpu_bench(_c: &mut Criterion) {
    // Placeholder benchmark when GPU features are disabled
}

#[cfg(not(feature = "gpu"))]
criterion_group!(gpu_benches, dummy_gpu_bench);

#[cfg(feature = "gpu")]
criterion_main!(gpu_benches);

#[cfg(not(feature = "gpu"))]
fn main() {
    println!("GPU benchmarks require --features gpu");
}
