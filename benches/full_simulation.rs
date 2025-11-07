use criterion::{criterion_group, criterion_main, Criterion};
use gravwell::{forces::direct::DirectGravity, integrators::verlet::VelocityVerlet, prelude::*};

fn bench_full_simulation(c: &mut Criterion) {
    c.bench_function("full_simulation_1000_steps", |b| {
        b.iter(|| {
            // Create a simple binary system
            let mut simulation = SimulationBuilder::new()
                .with_integrator(VelocityVerlet::new())
                .with_force_calculator(DirectGravity::new())
                .add_body(
                    Body::new()
                        .with_mass(5.972e24) // Earth mass
                        .with_position([0.0, 0.0, 0.0])
                        .with_velocity([0.0, 0.0, 0.0]),
                )
                .unwrap()
                .add_body(
                    Body::new()
                        .with_mass(7.342e22) // Moon mass
                        .with_position([384400000.0, 0.0, 0.0])
                        .with_velocity([0.0, 1022.0, 0.0]),
                )
                .unwrap()
                .build()
                .unwrap();

            // Run for 1000 timesteps
            let dt = 3600.0; // 1 hour
            for _ in 0..1000 {
                simulation.step(dt).unwrap();
            }
        });
    });
}

criterion_group!(benches, bench_full_simulation);
criterion_main!(benches);
