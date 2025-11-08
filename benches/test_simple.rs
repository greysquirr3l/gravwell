use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gravwell::prelude::*;

fn bench_simple(c: &mut Criterion) {
    let mut particles = ParticleSet::with_capacity(100);

    for i in 0..100 {
        let body = Body::new()
            .with_position([i as f64, 0.0, 0.0])
            .with_velocity([0.0, 0.0, 0.0])
            .with_mass(1.0);
        particles.add_body(body).unwrap();
    }

    let mut forces = vec![Force::zeros(); particles.len()];
    let calculator = DirectGravity::new();

    c.bench_function("simple_direct_gravity", |b| {
        b.iter(|| {
            calculator
                .calculate_forces(black_box(&particles), black_box(&mut forces))
                .unwrap();
        });
    });
}

criterion_group!(benches, bench_simple);
criterion_main!(benches);
