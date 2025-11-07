use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use gravwell::{forces::direct::DirectGravity, integrators::verlet::VelocityVerlet, prelude::*};

fn bench_integration_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("integration_step");

    for n_particles in [10, 100].iter() {
        let mut simulation = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(DirectGravity::new());

        // Add particles in a binary system
        for i in 0..*n_particles {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (*n_particles as f64);
            let r = 1e11 * (i + 1) as f64; // Varying orbital radii

            let body = Body::new()
                .with_mass(1e24)
                .with_position([r * angle.cos(), r * angle.sin(), 0.0])
                .with_velocity([-1000.0 * angle.sin(), 1000.0 * angle.cos(), 0.0]);

            simulation = simulation.add_body(body).unwrap();
        }

        let mut simulation = simulation.build().unwrap();
        let dt = 3600.0; // 1 hour timestep

        group.bench_with_input(
            BenchmarkId::new("velocity_verlet", n_particles),
            n_particles,
            |b, _| {
                b.iter(|| {
                    simulation.step(dt).unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_integration_step);
criterion_main!(benches);
