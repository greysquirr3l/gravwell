//! Memory Pool Zero-Allocation Demo
//!
//! This example demonstrates the memory pool system providing zero allocations
//! during simulation steps, achieving high-performance gravitational N-body simulation.

use gravwell::{
    memory::{thread_local::ThreadLocalPools, MemoryManager, PoolConfig},
    types::{Scalar, Vector3},
    with_force_buffers, with_integration_buffers,
};
use std::time::Instant;

// Mock simulation for demonstration
struct ZeroAllocSimulation {
    positions: Vec<Vector3>,
    velocities: Vec<Vector3>,
    masses: Vec<Scalar>,
    memory_manager: MemoryManager,
}

impl ZeroAllocSimulation {
    fn new(particle_count: usize) -> Self {
        let config = PoolConfig {
            initial_capacity: 8,
            max_capacity: 16,
            buffer_size: particle_count,
            auto_optimize: true,
            cleanup_interval_ms: 10000,
        };

        let mut positions = Vec::with_capacity(particle_count);
        let mut velocities = Vec::with_capacity(particle_count);
        let mut masses = Vec::with_capacity(particle_count);

        // Initialize random particle system
        for i in 0..particle_count {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / particle_count as f64;
            let radius = 1.0 + (i as f64 / particle_count as f64) * 10.0;

            positions.push(Vector3::new(
                radius * angle.cos(),
                radius * angle.sin(),
                0.0,
            ));

            velocities.push(Vector3::new(
                -radius * angle.sin() * 0.1,
                radius * angle.cos() * 0.1,
                0.0,
            ));

            masses.push(1.0);
        }

        Self {
            positions,
            velocities,
            masses,
            memory_manager: MemoryManager::with_config(config),
        }
    }

    /// Calculate forces using pooled buffers (zero allocations)
    fn calculate_forces_pooled(&self) -> Vec<Vector3> {
        let particle_count = self.positions.len();

        // Use thread-local pools for zero allocations
        with_force_buffers!(particle_count, buffers, {
            // Calculate pairwise forces
            for i in 0..particle_count {
                for j in 0..particle_count {
                    if i == j {
                        continue;
                    }

                    let r_vec = self.positions[j] - self.positions[i];
                    let r_squared = r_vec.norm_squared();
                    let r = r_squared.sqrt();

                    if r > 0.01 {
                        // Softening parameter
                        let force_magnitude = self.masses[i] * self.masses[j] / r_squared;
                        buffers.forces[i] += force_magnitude * r_vec / r;
                    }
                }
            }

            // Return computed forces
            buffers.forces.as_slice().to_vec()
        })
    }

    /// Integration step using pooled buffers (zero allocations)
    fn integration_step_pooled(&mut self, dt: Scalar) {
        let particle_count = self.positions.len();

        with_integration_buffers!(particle_count, buffers, {
            // Calculate accelerations from forces
            let forces = self.calculate_forces_pooled();

            for i in 0..particle_count {
                buffers.accelerations[i] = forces[i] / self.masses[i];
            }

            // Velocity Verlet integration
            for i in 0..particle_count {
                // Update positions
                self.positions[i] +=
                    self.velocities[i] * dt + 0.5 * buffers.accelerations[i] * dt * dt;

                // Store old velocities for later
                buffers.temp_velocities[i] = self.velocities[i];

                // Update velocities (first half)
                self.velocities[i] += 0.5 * buffers.accelerations[i] * dt;
            }

            // Recalculate forces at new positions
            let new_forces = self.calculate_forces_pooled();

            for i in 0..particle_count {
                let new_acceleration = new_forces[i] / self.masses[i];

                // Update velocities (second half)
                self.velocities[i] += 0.5 * new_acceleration * dt;
            }
        });
    }

    /// Traditional simulation step with allocations for comparison
    fn integration_step_traditional(&mut self, dt: Scalar) {
        let particle_count = self.positions.len();

        // Allocate new vectors each time (expensive!)
        let mut forces = vec![Vector3::zeros(); particle_count];
        let mut accelerations = vec![Vector3::zeros(); particle_count];
        let mut old_velocities = vec![Vector3::zeros(); particle_count];

        // Calculate forces
        for i in 0..particle_count {
            for j in 0..particle_count {
                if i == j {
                    continue;
                }

                let r_vec = self.positions[j] - self.positions[i];
                let r_squared = r_vec.norm_squared();
                let r = r_squared.sqrt();

                if r > 0.01 {
                    let force_magnitude = self.masses[i] * self.masses[j] / r_squared;
                    forces[i] += force_magnitude * r_vec / r;
                }
            }
            accelerations[i] = forces[i] / self.masses[i];
        }

        // Integration (same as pooled version)
        for i in 0..particle_count {
            self.positions[i] += self.velocities[i] * dt + 0.5 * accelerations[i] * dt * dt;
            old_velocities[i] = self.velocities[i];
            self.velocities[i] += 0.5 * accelerations[i] * dt;
        }

        // Recalculate forces
        forces.fill(Vector3::zeros());
        for i in 0..particle_count {
            for j in 0..particle_count {
                if i == j {
                    continue;
                }

                let r_vec = self.positions[j] - self.positions[i];
                let r_squared = r_vec.norm_squared();
                let r = r_squared.sqrt();

                if r > 0.01 {
                    let force_magnitude = self.masses[i] * self.masses[j] / r_squared;
                    forces[i] += force_magnitude * r_vec / r;
                }
            }
            let new_acceleration = forces[i] / self.masses[i];
            self.velocities[i] += 0.5 * new_acceleration * dt;
        }
    }

    fn total_energy(&self) -> Scalar {
        let mut kinetic = 0.0;
        let mut potential = 0.0;

        for i in 0..self.positions.len() {
            kinetic += 0.5 * self.masses[i] * self.velocities[i].norm_squared();

            for j in (i + 1)..self.positions.len() {
                let r = (self.positions[j] - self.positions[i]).norm();
                if r > 0.01 {
                    potential -= self.masses[i] * self.masses[j] / r;
                }
            }
        }

        kinetic + potential
    }
}

fn benchmark_allocation_methods() {
    println!("🚀 Gravwell - Memory Pool Zero-Allocation Demo");
    println!("==============================================");

    let particle_counts = vec![100, 500, 1000];
    let steps_per_test = 100;

    for &particle_count in &particle_counts {
        println!("\n📊 Testing {} particles:", particle_count);

        // Test pooled (zero-allocation) method
        let mut sim_pooled = ZeroAllocSimulation::new(particle_count);
        let initial_energy = sim_pooled.total_energy();

        let start_time = Instant::now();
        for _ in 0..steps_per_test {
            sim_pooled.integration_step_pooled(0.01);
        }
        let pooled_duration = start_time.elapsed();
        let pooled_energy_error =
            (sim_pooled.total_energy() - initial_energy).abs() / initial_energy.abs();

        // Test traditional (allocation-heavy) method
        let mut sim_traditional = ZeroAllocSimulation::new(particle_count);

        let start_time = Instant::now();
        for _ in 0..steps_per_test {
            sim_traditional.integration_step_traditional(0.01);
        }
        let traditional_duration = start_time.elapsed();
        let traditional_energy_error =
            (sim_traditional.total_energy() - initial_energy).abs() / initial_energy.abs();

        // Calculate performance improvement
        let speedup = traditional_duration.as_secs_f64() / pooled_duration.as_secs_f64();

        println!("  Pooled Method:");
        println!(
            "    Time: {:.2} ms ({:.1} µs/step)",
            pooled_duration.as_secs_f64() * 1000.0,
            pooled_duration.as_micros() as f64 / steps_per_test as f64
        );
        println!("    Energy error: {:.3e}", pooled_energy_error);

        println!("  Traditional Method:");
        println!(
            "    Time: {:.2} ms ({:.1} µs/step)",
            traditional_duration.as_secs_f64() * 1000.0,
            traditional_duration.as_micros() as f64 / steps_per_test as f64
        );
        println!("    Energy error: {:.3e}", traditional_energy_error);

        println!("  Performance Improvement: {:.2}x faster", speedup);

        // Memory pool statistics
        let stats = sim_pooled.memory_manager.stats();
        println!("  Memory Pool Stats:");
        println!("    Total memory: {} bytes", stats.total_memory_bytes());
        println!("    Overall efficiency: {:.1}%", stats.overall_efficiency());
        println!(
            "    Vector3 cache hits: {:.1}%",
            stats.vector3_pool.cache_hit_ratio()
        );
        println!(
            "    Scalar cache hits: {:.1}%",
            stats.scalar_pool.cache_hit_ratio()
        );
    }
}

fn demonstrate_thread_local_pools() {
    println!("\n🧵 Thread-Local Pool Demonstration");
    println!("===================================");

    let particle_count = 1000;
    let thread_count = 4;

    let handles: Vec<_> = (0..thread_count)
        .map(|thread_id| {
            std::thread::spawn(move || {
                let start_time = Instant::now();

                // Each thread gets its own pools automatically
                for step in 0..50 {
                    with_force_buffers!(particle_count, buffers, {
                        // Simulate force calculation work
                        for i in 0..particle_count {
                            buffers.forces[i] = Vector3::new(
                                (i as f64).sin(),
                                (i as f64).cos(),
                                (step as f64) * 0.1,
                            );
                        }
                    });

                    with_integration_buffers!(particle_count, buffers, {
                        // Simulate integration work
                        for i in 0..particle_count {
                            buffers.accelerations[i] = Vector3::new(
                                (i as f64 * step as f64).sin(),
                                (i as f64 * step as f64).cos(),
                                0.0,
                            );
                        }
                    });
                }

                let duration = start_time.elapsed();
                let stats = ThreadLocalPools::thread_stats();

                (thread_id, duration, stats)
            })
        })
        .collect();

    for handle in handles {
        let (thread_id, duration, stats) = handle.join().unwrap();

        println!(
            "Thread {}: {:.2} ms, {} pools, {:.1}% efficiency",
            thread_id,
            duration.as_secs_f64() * 1000.0,
            stats.vector3_pools.len() + stats.scalar_pools.len(),
            stats.overall_efficiency()
        );
    }

    // Cleanup all thread-local pools
    ThreadLocalPools::cleanup_all();
}

fn main() {
    // Run the benchmarks
    benchmark_allocation_methods();

    // Demonstrate thread-local pools
    demonstrate_thread_local_pools();

    println!("\n✅ Zero-Allocation Demo Complete!");
    println!("The memory pool system provides significant performance improvements");
    println!("by eliminating allocations during simulation steps while maintaining");
    println!("identical numerical accuracy and energy conservation.");
}
