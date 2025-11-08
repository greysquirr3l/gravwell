//! Spatial Culling Performance Benchmarking Suite
//!
//! Comprehensive benchmarks for spatial culling systems including:
//! - Hash grid performance at various particle densities
//! - Frustum culling efficiency with different camera configurations
//! - Dynamic activation system throughput and accuracy
//! - Combined spatial optimization performance validation
//! - Real-world scenario stress testing for large particle counts

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use gravwell::{prelude::*, spatial::*};
use std::time::Duration;

/// Test configurations for spatial culling benchmarks
const PARTICLE_COUNTS: &[usize] = &[1_000, 5_000, 10_000, 25_000, 50_000];
const CELL_SIZES: &[f64] = &[50.0, 100.0, 200.0, 500.0];
const CAMERA_DISTANCES: &[f64] = &[1000.0, 5000.0, 15000.0, 50000.0];
const SEARCH_RADII: &[f64] = &[10.0, 50.0, 100.0, 200.0];

/// Generate test particle data
fn generate_test_particles(count: usize) -> (Vec<Vector3>, Vec<BodyHandle>) {
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(42); // Deterministic seed
    let mut positions = Vec::with_capacity(count);
    let mut handles = Vec::with_capacity(count);

    // Generate particles in a 3D space (-5000 to 5000 in each dimension)
    for i in 0..count {
        let position = Vector3::new(
            rng.gen_range(-5000.0..5000.0),
            rng.gen_range(-5000.0..5000.0),
            rng.gen_range(-5000.0..5000.0),
        );
        positions.push(position);
        handles.push(BodyHandle::new(i, 0));
    }

    (positions, handles)
}

/// Generate test camera positions for consistent benchmarking
fn generate_camera_positions(count: usize) -> Vec<Vector3> {
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(123);
    let mut cameras = Vec::with_capacity(count);

    for _ in 0..count {
        cameras.push(Vector3::new(
            rng.gen_range(-2000.0..2000.0),
            rng.gen_range(-2000.0..2000.0),
            rng.gen_range(-2000.0..2000.0),
        ));
    }

    cameras
}

/// Benchmark hash grid insertion performance
fn bench_hash_grid_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_grid_insertion");

    for &particle_count in PARTICLE_COUNTS {
        for &cell_size in CELL_SIZES {
            let (positions, handles) = generate_test_particles(particle_count);

            group.throughput(Throughput::Elements(particle_count as u64));
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("{}p_{}cell", particle_count, cell_size)),
                &(positions, handles, cell_size),
                |b, (positions, handles, cell_size)| {
                    b.iter(|| {
                        let mut grid = SpatialHashGrid::new(*cell_size);
                        for (i, &position) in positions.iter().enumerate() {
                            black_box(grid.insert_particle(handles[i], position));
                        }
                        black_box(grid)
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark neighbor finding performance
fn bench_neighbor_finding(c: &mut Criterion) {
    let mut group = c.benchmark_group("neighbor_finding");

    for &particle_count in PARTICLE_COUNTS {
        for &search_radius in SEARCH_RADII {
            let (positions, handles) = generate_test_particles(particle_count);
            let mut grid = SpatialHashGrid::new(100.0); // Fixed cell size

            // Pre-populate grid
            for (i, &position) in positions.iter().enumerate() {
                grid.insert_particle(handles[i], position);
            }

            let search_positions = generate_camera_positions(100);

            group.throughput(Throughput::Elements(100)); // 100 searches per benchmark
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("{}p_{}r", particle_count, search_radius)),
                &(grid, search_positions, search_radius),
                |b, (grid, search_positions, search_radius)| {
                    b.iter(|| {
                        for &search_pos in search_positions.iter() {
                            let neighbors = grid.find_neighbors(search_pos, *search_radius);
                            black_box(neighbors);
                        }
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark frustum culling performance
fn bench_frustum_culling(c: &mut Criterion) {
    let mut group = c.benchmark_group("frustum_culling");

    for &particle_count in PARTICLE_COUNTS {
        let (positions, _handles) = generate_test_particles(particle_count);

        // Create test frustum
        let frustum = Frustum::from_camera(
            Vector3::new(0.0, 0.0, 0.0), // position
            Vector3::new(0.0, 0.0, 1.0), // forward
            Vector3::new(0.0, 1.0, 0.0), // up
            60.0,                        // fov degrees
            16.0 / 9.0,                  // aspect ratio
            1.0,                         // near
            10000.0,                     // far
        );

        group.throughput(Throughput::Elements(particle_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}p", particle_count)),
            &(positions, frustum),
            |b, (positions, frustum)| {
                b.iter(|| {
                    let mut visible_count = 0;
                    for &position in positions.iter() {
                        if frustum.contains_point(position) {
                            visible_count += 1;
                        }
                    }
                    black_box(visible_count)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark activation manager performance
fn bench_activation_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("activation_manager");

    for &particle_count in PARTICLE_COUNTS {
        for &camera_distance in CAMERA_DISTANCES {
            let (positions, handles) = generate_test_particles(particle_count);
            let camera_positions = generate_camera_positions(10);

            group.throughput(Throughput::Elements(10)); // 10 activation updates per benchmark
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("{}p_{}d", particle_count, camera_distance)),
                &(positions, handles, camera_positions),
                |b, (_positions, _handles, camera_positions)| {
                    b.iter(|| {
                        let mut manager = ActivationManager::new();

                        for &camera_pos in camera_positions.iter() {
                            let active = manager.update_activation(camera_pos);
                            black_box(active);
                        }
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark complete spatial culler performance
fn bench_spatial_culler_complete(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_culler_complete");

    for &particle_count in PARTICLE_COUNTS {
        let (positions, handles) = generate_test_particles(particle_count);
        let camera_positions = generate_camera_positions(10);

        // Create test frustum
        let frustum = Frustum::from_camera(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 0.0),
            60.0,
            16.0 / 9.0,
            1.0,
            10000.0,
        );

        group.throughput(Throughput::Elements(10)); // 10 complete culling passes
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}p", particle_count)),
            &(positions, handles, camera_positions, frustum),
            |b, (positions, handles, camera_positions, frustum)| {
                b.iter(|| {
                    let mut culler = SpatialCuller::new()
                        .with_cell_size(100.0)
                        .with_activation_distance(2000.0)
                        .with_frustum_culling(true);

                    culler.update_particles(positions, handles);

                    for &camera_pos in camera_positions.iter() {
                        let active = culler.cull_particles(
                            camera_pos,
                            Some(frustum),
                            particle_count / 2, // Limit to half particles
                        );
                        black_box(active);
                    }

                    black_box(culler)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark memory efficiency of spatial structures
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    for &particle_count in PARTICLE_COUNTS {
        let (positions, handles) = generate_test_particles(particle_count);

        group.throughput(Throughput::Elements(particle_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}p", particle_count)),
            &(positions, handles),
            |b, (positions, handles)| {
                b.iter(|| {
                    // Measure allocation and deallocation overhead
                    let mut culler = SpatialCuller::new().with_cell_size(100.0);
                    culler.update_particles(positions, handles);

                    let stats = culler.get_statistics();
                    let hash_stats = culler.get_hash_grid_stats();

                    black_box((stats, hash_stats));
                    black_box(culler)
                })
            },
        );
    }

    group.finish();
}

/// Real-world scenario benchmark simulating game loop
fn bench_real_world_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_scenario");
    group.measurement_time(Duration::from_secs(10));

    for &particle_count in &[1_000, 5_000, 10_000, 25_000] {
        let (positions, handles) = generate_test_particles(particle_count);

        // Simulate moving camera path
        let camera_path: Vec<Vector3> = (0..60)
            .map(|i| {
                let t = i as f64 / 60.0 * 2.0 * std::f64::consts::PI;
                Vector3::new((t.cos() * 3000.0) as f64, (t.sin() * 3000.0) as f64, 1000.0)
            })
            .collect();

        let frustum = Frustum::from_camera(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 0.0),
            75.0, // Wider FOV for games
            16.0 / 9.0,
            1.0,
            15000.0,
        );

        group.throughput(Throughput::Elements(60)); // 60 FPS simulation
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}p_60fps", particle_count)),
            &(positions, handles, camera_path, frustum),
            |b, (positions, handles, camera_path, frustum)| {
                b.iter(|| {
                    let mut culler = SpatialCuller::new()
                        .with_cell_size(150.0)
                        .with_activation_distance(5000.0)
                        .with_frustum_culling(true);

                    // Simulate 60 frames (1 second at 60 FPS)
                    for (frame, &camera_pos) in camera_path.iter().enumerate() {
                        // Simulate slight particle movement each frame
                        let time_factor = frame as f64 * 0.016; // 16ms per frame
                        let mut frame_positions = positions.clone();
                        for pos in frame_positions.iter_mut() {
                            pos.y += (time_factor * 10.0).sin() * 5.0; // Subtle oscillation
                        }

                        culler.update_particles(&frame_positions, handles);
                        let active = culler.cull_particles(
                            camera_pos,
                            Some(frustum),
                            particle_count.min(10_000), // Reasonable active limit
                        );

                        black_box(active);
                    }

                    black_box(culler)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    spatial_culling_benches,
    bench_hash_grid_insertion,
    bench_neighbor_finding,
    bench_frustum_culling,
    bench_activation_manager,
    bench_spatial_culler_complete,
    bench_memory_usage,
    bench_real_world_scenario
);

criterion_main!(spatial_culling_benches);
