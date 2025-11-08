// Spatial Culling Performance Benchmarks - TEMPORARILY DISABLED
//
// This benchmark suite is temporarily disabled while the spatial culling API
// is under development. The implementation will be restored once the API
// stabilizes.

#![allow(dead_code, unused_imports)]

use criterion::{criterion_group, criterion_main, Criterion};

fn dummy_benchmark(_c: &mut Criterion) {
    // No-op benchmark placeholder
}

criterion_group!(benches, dummy_benchmark);
criterion_main!(benches);
