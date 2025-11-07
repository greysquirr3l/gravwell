use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use gravwell::{core::particle::ParticleSet, forces::direct::DirectGravity, prelude::*};

fn bench_force_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("force_calculation");

    for n_particles in [10, 100, 1000].iter() {
        let mut particles = ParticleSet::with_capacity(*n_particles);

        // Add particles in a rough sphere
        for i in 0..*n_particles {
            let angle1 = 2.0 * std::f64::consts::PI * (i as f64) / (*n_particles as f64);
            let angle2 = std::f64::consts::PI * ((i * 2) as f64) / (*n_particles as f64);

            let r = 1e11; // 100 million km radius
            let x = r * angle1.cos() * angle2.sin();
            let y = r * angle1.sin() * angle2.sin();
            let z = r * angle2.cos();

            let body = Body::new()
                .with_mass(1e24) // Earth-like mass
                .with_position([x, y, z]);

            particles.add_body(body).unwrap();
        }

        let calculator = DirectGravity::new();
        let mut forces = vec![Force::zeros(); *n_particles];

        group.bench_with_input(
            BenchmarkId::new("direct_gravity", n_particles),
            n_particles,
            |b, _| {
                b.iter(|| {
                    calculator
                        .calculate_forces(&particles, &mut forces)
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_force_calculation);
criterion_main!(benches);
