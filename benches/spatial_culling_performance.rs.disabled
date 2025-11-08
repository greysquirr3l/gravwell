// TEMPORARILY DISABLED - API under development
#![allow(dead_code, unused_imports)]
/*
//! Spatial Culling Performance Benchmarking Suite
//!
//! Comprehensive benchmarks for spatial culling systems including:
//! - Hash grid performance at various particle densities
//! - Frustum culling efficiency with different camera configurations
//! - Dynamic activation system throughput and accuracy
//! - Combined spatial optimization performance validation
//! - Real-world scenario stress testing for 100K+ particles

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use gravwell::{prelude::*, spatial::*};
use nalgebra::Vector3;
use std::time::Duration;

/// Test configurations for spatial culling benchmarks
const PARTICLE_COUNTS: &[usize] = &[1_000, 5_000, 10_000, 25_000, 50_000, 100_000];
const CELL_SIZES: &[f64] = &[50.0, 100.0, 200.0, 500.0];
const CAMERA_DISTANCES: &[f64] = &[1000.0, 5000.0, 15000.0, 50000.0];

/// Benchmark duration for longer stress tests
const STRESS_TEST_DURATION: Duration = Duration::from_secs(10);

/// Simulation setup utilities
mod spatial_setup {
    use super::*;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    /// Create a galaxy-like particle distribution for realistic spatial testing
    pub fn create_galaxy_distribution(particle_count: usize) -> Vec<Body> {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut bodies = Vec::with_capacity(particle_count);

        // Central supermassive black hole
        bodies.push(Body {
            mass: SOLAR_MASS * 1e6,
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
        });

        // Spiral galaxy structure with realistic distribution
        for i in 1..particle_count {
            let arm_angle = (i as f64 / particle_count as f64) * 4.0 * std::f64::consts::PI;
            let radius = (i as f64 / particle_count as f64).sqrt() * 25000.0 + 100.0;

            // Add spiral structure
            let spiral_offset = arm_angle * 0.3;
            let final_angle = arm_angle + spiral_offset;

            // Add vertical distribution
            let height = rng.gen_range(-500.0..500.0) * (1.0 - radius / 25000.0);

            let position = Vector3::new(
                radius * final_angle.cos(),
                radius * final_angle.sin(),
                height,
            );

            // Orbital velocity with some random variation
            let orbital_speed = (G * SOLAR_MASS * 1e6 / radius).sqrt() * rng.gen_range(0.8..1.2);
            let velocity = Vector3::new(
                -orbital_speed * final_angle.sin(),
                orbital_speed * final_angle.cos(),
                rng.gen_range(-50.0..50.0),
            );

            let mass = SOLAR_MASS * rng.gen_range(0.1..5.0);

            bodies.push(Body {
                mass,
                position,
                velocity,
            });
        }

        bodies
    }

    /// Create clustered particle distribution for spatial hash testing
    pub fn create_clustered_distribution(particle_count: usize, cluster_count: usize) -> Vec<Body> {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut bodies = Vec::with_capacity(particle_count);
        let particles_per_cluster = particle_count / cluster_count;

        for cluster in 0..cluster_count {
            // Random cluster center
            let cluster_center = Vector3::new(
                rng.gen_range(-10000.0..10000.0),
                rng.gen_range(-10000.0..10000.0),
                rng.gen_range(-1000.0..1000.0),
            );

            let cluster_size = rng.gen_range(500.0..2000.0);

            for _ in 0..particles_per_cluster {
                // Gaussian distribution within cluster
                let offset = Vector3::new(
                    rng.gen_range(-cluster_size..cluster_size),
                    rng.gen_range(-cluster_size..cluster_size),
                    rng.gen_range(-cluster_size * 0.1..cluster_size * 0.1),
                );

                let position = cluster_center + offset;
                let velocity = Vector3::new(
                    rng.gen_range(-100.0..100.0),
                    rng.gen_range(-100.0..100.0),
                    rng.gen_range(-10.0..10.0),
                );

                bodies.push(Body {
                    mass: EARTH_MASS * rng.gen_range(0.1..10.0),
                    position,
                    velocity,
                });
            }
        }

        // Fill remaining particles if any
        let remaining = particle_count - (particles_per_cluster * cluster_count);
        for _ in 0..remaining {
            bodies.push(Body {
                mass: EARTH_MASS,
                position: Vector3::new(
                    rng.gen_range(-20000.0..20000.0),
                    rng.gen_range(-20000.0..20000.0),
                    rng.gen_range(-2000.0..2000.0),
                ),
                velocity: Vector3::new(
                    rng.gen_range(-200.0..200.0),
                    rng.gen_range(-200.0..200.0),
                    rng.gen_range(-20.0..20.0),
                ),
            });
        }

        bodies
    }

    /// Create camera for frustum culling tests
    pub fn create_test_camera(distance: f64, fov: f64) -> Camera {
        Camera {
            position: Vector3::new(distance, distance * 0.5, distance * 0.3),
            target: Vector3::zeros(),
            up: Vector3::z(),
            fov_radians: fov.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near_distance: 1.0,
            far_distance: distance * 10.0,
        }
    }
}

/// Benchmark spatial hash grid performance
fn bench_hash_grid_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_hash_grid");
    group.throughput(Throughput::Elements(1));

    for &particle_count in PARTICLE_COUNTS.iter() {
        for &cell_size in CELL_SIZES.iter() {
            let bodies = spatial_setup::create_clustered_distribution(particle_count, 10);
            let positions: Vec<Vector3<f64>> = bodies.iter().map(|b| b.position).collect();

            group.bench_with_input(
                BenchmarkId::new(
                    "insertion",
                    format!("{}p_{}cs", particle_count, cell_size as u32),
                ),
                &(&positions, cell_size),
                |b, (positions, cell_size)| {
                    b.iter(|| {
                        let mut grid = SpatialHashGrid::new(*cell_size);
                        for (i, &position) in positions.iter().enumerate() {
                            black_box(grid.insert(i, position));
                        }
                        grid
                    })
                },
            );

            // Test neighbor finding performance
            let mut grid = SpatialHashGrid::new(cell_size);
            for (i, &position) in positions.iter().enumerate() {
                grid.insert(i, position);
            }

            let query_position = Vector3::new(0.0, 0.0, 0.0);
            let query_radius = cell_size * 3.0;

            group.bench_with_input(
                BenchmarkId::new(
                    "neighbor_query",
                    format!("{}p_{}cs", particle_count, cell_size as u32),
                ),
                &(&grid, query_position, query_radius),
                |b, (grid, pos, radius)| b.iter(|| black_box(grid.find_neighbors(*pos, *radius))),
            );
        }
    }

    group.finish();
}

/// Benchmark frustum culling performance
fn bench_frustum_culling(c: &mut Criterion) {
    let mut group = c.benchmark_group("frustum_culling");
    group.throughput(Throughput::Elements(1));

    for &particle_count in PARTICLE_COUNTS.iter() {
        for &distance in CAMERA_DISTANCES.iter() {
            let bodies = spatial_setup::create_galaxy_distribution(particle_count);
            let positions: Vec<Vector3<f64>> = bodies.iter().map(|b| b.position).collect();
            let radii: Vec<f64> = vec![10.0; particle_count]; // Uniform radii for testing

            let camera = spatial_setup::create_test_camera(distance, 60.0);
            let frustum = Frustum::from_camera(&camera);

            group.bench_with_input(
                BenchmarkId::new(
                    "visibility_test",
                    format!("{}p_{}d", particle_count, distance as u32),
                ),
                &(&frustum, &positions, &radii),
                |b, (frustum, positions, radii)| {
                    b.iter(|| {
                        let mut visible_count = 0;
                        for (position, radius) in positions.iter().zip(radii.iter()) {
                            if black_box(frustum.intersects_sphere(*position, *radius)) {
                                visible_count += 1;
                            }
                        }
                        visible_count
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark dynamic activation system
fn bench_activation_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("activation_system");
    group.throughput(Throughput::Elements(1));

    for &particle_count in PARTICLE_COUNTS.iter() {
        let bodies = spatial_setup::create_galaxy_distribution(particle_count);
        let camera = spatial_setup::create_test_camera(5000.0, 60.0);

        let config = ActivationConfig {
            max_active_particles: (particle_count as f64 * 0.1) as usize, // 10% budget
            distance_thresholds: vec![1000.0, 3000.0, 10000.0, 30000.0],
            importance_weights: ImportanceWeights {
                distance: 1.0,
                mass: 0.5,
                velocity: 0.3,
            },
            hysteresis_factor: 0.15,
        };

        let mut activation_manager = ActivationManager::new(config);

        // Initialize with particle data
        for (i, body) in bodies.iter().enumerate() {
            activation_manager.add_particle(i, body.position, body.mass, body.velocity.norm());
        }

        group.bench_with_input(
            BenchmarkId::new("update_activation", format!("{}p", particle_count)),
            &(&mut activation_manager, &camera),
            |b, (manager, camera)| b.iter(|| black_box(manager.update_activation(camera.position))),
        );
    }

    group.finish();
}

/// Benchmark combined spatial optimization systems
fn bench_combined_spatial_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_spatial_optimization");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(STRESS_TEST_DURATION);

    for &particle_count in &[10_000, 25_000, 50_000, 100_000] {
        let bodies = spatial_setup::create_galaxy_distribution(particle_count);

        // Setup complete spatial optimization system
        let spatial_config = SpatialCullingConfig {
            hash_grid_cell_size: 200.0,
            activation_budget: (particle_count as f64 * 0.1) as usize,
            importance_weights: ImportanceWeights {
                distance: 1.0,
                mass: 0.5,
                velocity: 0.3,
            },
            hysteresis_factor: 0.15,
        };

        let mut spatial_culler = SpatialCuller::new(spatial_config);

        // Initialize spatial systems
        for (i, body) in bodies.iter().enumerate() {
            spatial_culler.add_particle(i, body.position, body.mass, body.velocity.norm());
        }

        let camera = spatial_setup::create_test_camera(5000.0, 60.0);

        group.bench_with_input(
            BenchmarkId::new("full_culling_pass", format!("{}p", particle_count)),
            &(&mut spatial_culler, &camera),
            |b, (culler, camera)| {
                b.iter(|| {
                    // Complete spatial culling pass
                    culler.update_positions(&bodies.iter().map(|b| b.position).collect::<Vec<_>>());
                    black_box(culler.cull_particles(camera))
                })
            },
        );
    }

    group.finish();
}

/// Real-world scenario stress testing
fn bench_real_world_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_scenarios");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(STRESS_TEST_DURATION);

    // Galaxy merger scenario
    let galaxy1 = spatial_setup::create_galaxy_distribution(25_000);
    let mut galaxy2 = spatial_setup::create_galaxy_distribution(25_000);

    // Offset second galaxy
    for body in &mut galaxy2 {
        body.position += Vector3::new(50000.0, 30000.0, 5000.0);
        body.velocity += Vector3::new(-500.0, 200.0, 0.0);
    }

    let mut combined_bodies = galaxy1;
    combined_bodies.extend(galaxy2);

    let spatial_config = SpatialCullingConfig {
        hash_grid_cell_size: 300.0,
        activation_budget: 5000, // 10% of 50K particles
        importance_weights: ImportanceWeights {
            distance: 1.0,
            mass: 0.8,
            velocity: 0.2,
        },
        hysteresis_factor: 0.2,
    };

    let mut spatial_culler = SpatialCuller::new(spatial_config);

    for (i, body) in combined_bodies.iter().enumerate() {
        spatial_culler.add_particle(i, body.position, body.mass, body.velocity.norm());
    }

    // Moving camera scenario
    let camera_start = spatial_setup::create_test_camera(10000.0, 75.0);

    group.bench_function("galaxy_merger_50k", |b| {
        b.iter(|| {
            // Simulate camera movement
            let time = (b.elapsed().as_secs_f64() * 0.1) % (2.0 * std::f64::consts::PI);
            let camera_pos = Vector3::new(
                10000.0 * time.cos(),
                10000.0 * time.sin(),
                5000.0 + 2000.0 * (time * 2.0).sin(),
            );

            let mut camera = camera_start.clone();
            camera.position = camera_pos;

            spatial_culler.update_positions(
                &combined_bodies
                    .iter()
                    .map(|b| b.position)
                    .collect::<Vec<_>>(),
            );
            black_box(spatial_culler.cull_particles(&camera))
        })
    });

    group.finish();
}

/// Memory usage benchmarks for spatial systems
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.throughput(Throughput::Elements(1));

    for &particle_count in PARTICLE_COUNTS.iter() {
        let bodies = spatial_setup::create_galaxy_distribution(particle_count);

        group.bench_with_input(
            BenchmarkId::new("memory_allocation", format!("{}p", particle_count)),
            &particle_count,
            |b, &count| {
                b.iter(|| {
                    // Test memory allocation patterns
                    let spatial_config = SpatialCullingConfig {
                        hash_grid_cell_size: 200.0,
                        activation_budget: count / 10,
                        importance_weights: ImportanceWeights::default(),
                        hysteresis_factor: 0.15,
                    };

                    let mut culler = SpatialCuller::new(spatial_config);

                    for (i, body) in bodies.iter().enumerate() {
                        culler.add_particle(i, body.position, body.mass, body.velocity.norm());
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
    bench_hash_grid_performance,
    bench_frustum_culling,
    bench_activation_system,
    bench_combined_spatial_optimization,
    bench_real_world_scenarios,
    bench_memory_usage
);

criterion_main!(spatial_culling_benches);
*/
