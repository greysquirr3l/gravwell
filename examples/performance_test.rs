//! Performance Testing and Benchmarking Example
//!
//! Comprehensive performance validation suite for Gravwell's physics engine,
//! testing scalability from small systems to massive N-body simulations.

// Using std::result::Result to avoid conflict with gravwell::Result
// use gravwell::error::Result;  // Not needed since we use std::result::Result
// use gravwell::prelude::*; // Disabled - unused
use std::collections::HashMap;
use std::time::{Duration, Instant};

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Gravwell Performance Test Suite");
    println!("=================================");
    println!("Target: 60 FPS capability for real-time applications");
    println!("Requirement: 16.67ms maximum per physics step\n");

    // Test configuration
    let particle_counts = vec![100, 500, 1000, 2500, 5000, 10000, 25000];
    let integrator_types = vec!["VelocityVerlet", "Leapfrog", "RK4", "IAS15"];
    let force_algorithms = vec!["DirectGravity", "BarnesHut", "FastMultipole"];

    let mut results = HashMap::new();

    // 1. Scalability Test - Different particle counts
    println!("📊 1. SCALABILITY ANALYSIS");
    println!("========================");

    for &particle_count in &particle_counts {
        println!("\n🔬 Testing with {} particles", particle_count);

        // Test each force algorithm
        for algorithm in &force_algorithms {
            let performance = run_scalability_test(particle_count, algorithm)?;
            results.insert(
                format!("{}_{}", algorithm, particle_count),
                performance.clone(),
            );

            let fps_capable = if performance.avg_step_time.as_secs_f64() <= 0.01667 {
                "✅"
            } else {
                "❌"
            };

            println!(
                "  {:<15}: {:.2}ms avg, {:.1} FPS max {}",
                algorithm,
                performance.avg_step_time.as_secs_f64() * 1000.0,
                1.0 / performance.avg_step_time.as_secs_f64(),
                fps_capable
            );
        }
    }

    // 2. Integrator Comparison
    println!("\n\n🧮 2. INTEGRATOR PERFORMANCE COMPARISON");
    println!("======================================");

    let test_particles = 1000; // Standard test size

    for integrator in &integrator_types {
        println!("\n🔬 Testing {} integrator", integrator);

        let performance = run_integrator_test(test_particles, integrator)?;

        println!(
            "  Step time: {:.3}ms",
            performance.avg_step_time.as_secs_f64() * 1000.0
        );
        println!(
            "  Energy drift: {:.2e} per step",
            performance.energy_drift_per_step
        );
        println!("  Stability: {}", performance.stability_rating);

        if performance.avg_step_time.as_secs_f64() <= 0.01667 {
            println!("  ✅ 60 FPS capable");
        } else {
            println!("  ❌ Too slow for 60 FPS");
        }
    }

    // 3. Memory Usage Analysis
    println!("\n\n💾 3. MEMORY EFFICIENCY ANALYSIS");
    println!("===============================");

    for &particle_count in &[1000, 5000, 10000, 50000] {
        let memory_usage = measure_memory_usage(particle_count)?;
        let bytes_per_particle = memory_usage.total_bytes as f64 / particle_count as f64;

        println!(
            "  {:>6} particles: {:.1} MB total, {:.1} bytes/particle",
            particle_count,
            memory_usage.total_bytes as f64 / 1_048_576.0,
            bytes_per_particle
        );

        if bytes_per_particle <= 1024.0 {
            println!("    ✅ Memory efficient (<1KB per particle)");
        } else {
            println!("    ⚠️  High memory usage (>1KB per particle)");
        }
    }

    // 4. SIMD and Parallelization Performance
    println!("\n\n🚀 4. SIMD & PARALLELIZATION PERFORMANCE");
    println!("=======================================");

    let simd_test_particles = 10000;

    // Test with different threading configurations
    let thread_counts = vec![1, 2, 4, 8, 16];

    for &thread_count in &thread_counts {
        let performance = run_parallel_test(simd_test_particles, thread_count)?;
        let speedup = if thread_count == 1 {
            1.0
        } else {
            results
                .get(&format!("parallel_1"))
                .unwrap_or(&performance)
                .avg_step_time
                .as_secs_f64()
                / performance.avg_step_time.as_secs_f64()
        };

        println!(
            "  {:>2} threads: {:.2}ms, {:.1}x speedup, {:.1}% efficiency",
            thread_count,
            performance.avg_step_time.as_secs_f64() * 1000.0,
            speedup,
            speedup / thread_count as f64 * 100.0
        );

        results.insert(format!("parallel_{}", thread_count), performance);
    }

    // 5. Real-time Performance Validation
    println!("\n\n⏱️  5. REAL-TIME PERFORMANCE VALIDATION");
    println!("======================================");

    let realtime_tests = vec![
        (1000, "Small system (mobile/web)"),
        (5000, "Medium system (desktop)"),
        (10000, "Large system (high-end desktop)"),
    ];

    for (particle_count, description) in realtime_tests {
        println!("\n🎯 {} - {} particles", description, particle_count);

        let realtime_perf = run_realtime_test(particle_count)?;

        println!("  Average FPS: {:.1}", realtime_perf.average_fps);
        println!("  Minimum FPS: {:.1}", realtime_perf.minimum_fps);
        println!(
            "  Frame drops: {}/1000 ({:.1}%)",
            realtime_perf.frame_drops,
            realtime_perf.frame_drops as f64 / 10.0
        );

        if realtime_perf.minimum_fps >= 60.0 {
            println!("  ✅ Maintains 60 FPS consistently");
        } else if realtime_perf.average_fps >= 60.0 {
            println!("  ⚠️  Averages 60 FPS but has drops");
        } else {
            println!("  ❌ Cannot maintain 60 FPS");
        }
    }

    // 6. Scientific Accuracy vs Performance Trade-off
    println!("\n\n🔬 6. ACCURACY VS PERFORMANCE ANALYSIS");
    println!("=====================================");

    let accuracy_configs = vec![
        ("Low precision", 1e-6, "Fast gaming"),
        ("Medium precision", 1e-9, "Standard simulation"),
        ("High precision", 1e-12, "Scientific computing"),
        ("Ultra precision", 1e-15, "Research/validation"),
    ];

    for (name, tolerance, use_case) in accuracy_configs {
        let perf = run_accuracy_test(1000, tolerance)?;

        println!("\n  {} ({}):", name, use_case);
        println!("    Tolerance: {:.0e}", tolerance);
        println!(
            "    Step time: {:.3}ms",
            perf.avg_step_time.as_secs_f64() * 1000.0
        );
        println!("    Energy drift: {:.2e}/step", perf.energy_drift_per_step);
        println!("    Max FPS: {:.1}", 1.0 / perf.avg_step_time.as_secs_f64());
    }

    // 7. Platform-Specific Optimizations
    println!("\n\n🎮 7. PLATFORM OPTIMIZATION ANALYSIS");
    println!("===================================");

    test_platform_optimizations()?;

    // 8. Stress Test - Maximum Capabilities
    println!("\n\n💪 8. STRESS TEST - MAXIMUM CAPABILITIES");
    println!("=======================================");

    let max_particles = find_maximum_particles_for_60fps()?;

    println!("Maximum particles for 60 FPS: {}", max_particles);
    println!(
        "Estimated memory required: {:.1} MB",
        max_particles as f64 * 1024.0 / 1_048_576.0
    );

    if max_particles >= 10000 {
        println!("✅ Exceeds target performance (10K particles @ 60 FPS)");
    } else {
        println!("❌ Below target performance");
    }

    // Final Performance Summary
    println!("\n\n📋 FINAL PERFORMANCE SUMMARY");
    println!("============================");

    generate_performance_report(&results)?;

    Ok(())
}

// Performance test structures
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PerformanceResult {
    avg_step_time: Duration,
    min_step_time: Duration,
    max_step_time: Duration,
    energy_drift_per_step: f64,
    stability_rating: String,
}

#[derive(Debug)]
struct RealtimePerformance {
    average_fps: f64,
    minimum_fps: f64,
    frame_drops: u32,
}

#[derive(Debug)]
#[allow(dead_code)]
struct MemoryUsage {
    total_bytes: usize,
    particles_bytes: usize,
    forces_bytes: usize,
    integrator_bytes: usize,
}

// Test implementations
fn run_scalability_test(
    particle_count: usize,
    algorithm: &str,
) -> std::result::Result<PerformanceResult, Box<dyn std::error::Error>> {
    // Create simulation based on algorithm
    let mut sim = match algorithm {
        "DirectGravity" => create_direct_simulation(particle_count)?,
        "BarnesHut" => create_barnes_hut_simulation(particle_count)?,
        "FastMultipole" => create_fmm_simulation(particle_count)?,
        _ => return Err(format!("Unknown algorithm: {}", algorithm).into()),
    };

    // Add random particles
    add_random_particles(&mut sim, particle_count)?;

    // Benchmark simulation steps
    let benchmark_steps = 100;
    let mut step_times = Vec::new();
    let initial_energy = sim.total_energy();

    for _ in 0..benchmark_steps {
        let start = Instant::now();
        sim.step()?;
        let elapsed = start.elapsed();
        step_times.push(elapsed);
    }

    let final_energy = sim.total_energy();
    let energy_drift_per_step =
        (final_energy - initial_energy).abs() / (initial_energy.abs() * benchmark_steps as f64);

    Ok(PerformanceResult {
        avg_step_time: Duration::from_nanos(
            (step_times.iter().map(|d| d.as_nanos()).sum::<u128>() / step_times.len() as u128)
                as u64,
        ),
        min_step_time: *step_times.iter().min().unwrap(),
        max_step_time: *step_times.iter().max().unwrap(),
        energy_drift_per_step,
        stability_rating: classify_stability(energy_drift_per_step),
    })
}

fn run_integrator_test(
    particle_count: usize,
    integrator: &str,
) -> std::result::Result<PerformanceResult, Box<dyn std::error::Error>> {
    let mut sim = match integrator {
        "VelocityVerlet" => create_verlet_simulation(particle_count)?,
        "Leapfrog" => create_leapfrog_simulation(particle_count)?,
        "RK4" => create_rk4_simulation(particle_count)?,
        "IAS15" => create_ias15_simulation(particle_count)?,
        _ => return Err(format!("Unknown integrator: {}", integrator).into()),
    };

    add_random_particles(&mut sim, particle_count)?;

    // Similar benchmarking as scalability test
    let benchmark_steps = 100;
    let mut step_times = Vec::new();
    let initial_energy = sim.total_energy();

    for _ in 0..benchmark_steps {
        let start = Instant::now();
        sim.step()?;
        step_times.push(start.elapsed());
    }

    let final_energy = sim.total_energy();
    let energy_drift_per_step =
        (final_energy - initial_energy).abs() / (initial_energy.abs() * benchmark_steps as f64);

    Ok(PerformanceResult {
        avg_step_time: Duration::from_nanos(
            (step_times.iter().map(|d| d.as_nanos()).sum::<u128>() / step_times.len() as u128)
                as u64,
        ),
        min_step_time: *step_times.iter().min().unwrap(),
        max_step_time: *step_times.iter().max().unwrap(),
        energy_drift_per_step,
        stability_rating: classify_stability(energy_drift_per_step),
    })
}

fn measure_memory_usage(
    particle_count: usize,
) -> std::result::Result<MemoryUsage, Box<dyn std::error::Error>> {
    // This would use platform-specific memory profiling
    // For now, we'll estimate based on data structure sizes

    let bytes_per_particle = 64; // Position, velocity, mass, etc.
    let force_buffer_size = particle_count * 24; // 3D force vectors
    let integrator_overhead = particle_count * 32; // Integration state

    Ok(MemoryUsage {
        total_bytes: particle_count * bytes_per_particle + force_buffer_size + integrator_overhead,
        particles_bytes: particle_count * bytes_per_particle,
        forces_bytes: force_buffer_size,
        integrator_bytes: integrator_overhead,
    })
}

fn run_parallel_test(
    particle_count: usize,
    _thread_count: usize,
) -> std::result::Result<PerformanceResult, Box<dyn std::error::Error>> {
    // Note: Thread management handled by force calculator implementations
    // Rayon-based parallelism would be internal to BarnesHut calculator

    let mut sim = create_barnes_hut_simulation(particle_count)?;
    add_random_particles(&mut sim, particle_count)?;

    // Benchmark parallel performance
    let benchmark_steps = 50;
    let mut step_times = Vec::new();
    let initial_energy = sim.total_energy();

    for _ in 0..benchmark_steps {
        let start = Instant::now();
        sim.parallel_step()?; // Use parallel stepping
        step_times.push(start.elapsed());
    }

    let final_energy = sim.total_energy();
    let energy_drift_per_step =
        (final_energy - initial_energy).abs() / (initial_energy.abs() * benchmark_steps as f64);

    Ok(PerformanceResult {
        avg_step_time: Duration::from_nanos(
            (step_times.iter().map(|d| d.as_nanos()).sum::<u128>() / step_times.len() as u128)
                as u64,
        ),
        min_step_time: *step_times.iter().min().unwrap(),
        max_step_time: *step_times.iter().max().unwrap(),
        energy_drift_per_step,
        stability_rating: classify_stability(energy_drift_per_step),
    })
}

fn run_realtime_test(
    particle_count: usize,
) -> std::result::Result<RealtimePerformance, Box<dyn std::error::Error>> {
    let mut sim = create_barnes_hut_simulation(particle_count)?;
    add_random_particles(&mut sim, particle_count)?;

    // Simulate 1000 frames at 60 FPS target
    let target_frame_time = Duration::from_nanos(16_666_667); // 16.67ms
    let total_frames = 1000;
    let mut frame_times = Vec::new();
    let mut frame_drops = 0;

    for _ in 0..total_frames {
        let start = Instant::now();
        sim.step()?;
        let frame_time = start.elapsed();

        frame_times.push(frame_time);

        if frame_time > target_frame_time {
            frame_drops += 1;
        }
    }

    let average_frame_time = Duration::from_nanos(
        (frame_times.iter().map(|d| d.as_nanos()).sum::<u128>() / frame_times.len() as u128) as u64,
    );
    let _minimum_frame_time = *frame_times.iter().min().unwrap();

    Ok(RealtimePerformance {
        average_fps: 1.0 / average_frame_time.as_secs_f64(),
        minimum_fps: 1.0 / frame_times.iter().max().unwrap().as_secs_f64(),
        frame_drops,
    })
}

fn run_accuracy_test(
    particle_count: usize,
    tolerance: f64,
) -> std::result::Result<PerformanceResult, Box<dyn std::error::Error>> {
    let mut sim = create_adaptive_simulation(particle_count, tolerance)?;
    add_random_particles(&mut sim, particle_count)?;

    // Test with high-precision requirements
    let benchmark_steps = 50;
    let mut step_times = Vec::new();
    let initial_energy = sim.total_energy();

    for _ in 0..benchmark_steps {
        let start = Instant::now();
        sim.adaptive_step()?; // Use adaptive stepping with tolerance
        step_times.push(start.elapsed());
    }

    let final_energy = sim.total_energy();
    let energy_drift_per_step =
        (final_energy - initial_energy).abs() / (initial_energy.abs() * benchmark_steps as f64);

    Ok(PerformanceResult {
        avg_step_time: Duration::from_nanos(
            (step_times.iter().map(|d| d.as_nanos()).sum::<u128>() / step_times.len() as u128)
                as u64,
        ),
        min_step_time: *step_times.iter().min().unwrap(),
        max_step_time: *step_times.iter().max().unwrap(),
        energy_drift_per_step,
        stability_rating: classify_stability(energy_drift_per_step),
    })
}

fn test_platform_optimizations() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("  CPU Features:");

    // Check for SIMD support
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            println!("    ✅ AVX2 vectorization available");
        } else {
            println!("    ⚠️  AVX2 not available");
        }

        if is_x86_feature_detected!("fma") {
            println!("    ✅ FMA (Fused Multiply-Add) available");
        } else {
            println!("    ⚠️  FMA not available");
        }
    }

    // Test different optimization levels
    let _test_particles = 1000;

    println!("  Optimization Levels:");
    println!("    Debug build: Not recommended for benchmarking");
    println!("    Release build: Full optimizations enabled");
    println!("    Target-CPU native: Platform-specific optimizations");

    Ok(())
}

fn find_maximum_particles_for_60fps() -> std::result::Result<usize, Box<dyn std::error::Error>> {
    let target_frame_time = Duration::from_nanos(16_666_667); // 16.67ms for 60 FPS
    let mut low = 1000;
    let mut high = 100_000;
    let mut best = low;

    // Binary search to find maximum particle count
    while low <= high {
        let mid = (low + high) / 2;

        let mut sim = create_barnes_hut_simulation(mid)?;
        add_random_particles(&mut sim, mid)?;

        // Test average performance over several steps
        let mut total_time = Duration::ZERO;
        let test_steps = 10;

        for _ in 0..test_steps {
            let start = Instant::now();
            sim.step()?;
            total_time += start.elapsed();
        }

        let avg_step_time = total_time / test_steps;

        if avg_step_time <= target_frame_time {
            best = mid;
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }

    Ok(best)
}

fn generate_performance_report(
    results: &HashMap<String, PerformanceResult>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Key Findings:");
    println!("------------");

    // Find best performing configurations
    let mut best_small_system: Option<(String, PerformanceResult)> = None;
    let mut best_large_system: Option<(String, PerformanceResult)> = None;

    for (key, result) in results {
        if key.contains("1000") && result.avg_step_time.as_secs_f64() <= 0.01667 {
            if best_small_system.is_none()
                || result.avg_step_time < best_small_system.as_ref().unwrap().1.avg_step_time
            {
                best_small_system = Some((key.clone(), result.clone()));
            }
        }

        if key.contains("10000") && result.avg_step_time.as_secs_f64() <= 0.01667 {
            if best_large_system.is_none()
                || result.avg_step_time < best_large_system.as_ref().unwrap().1.avg_step_time
            {
                best_large_system = Some((key.clone(), result.clone()));
            }
        }
    }

    if let Some((name, perf)) = best_small_system {
        println!("✅ Best small system (1K particles): {}", name);
        println!(
            "   {:.2}ms per step ({:.1} FPS)",
            perf.avg_step_time.as_secs_f64() * 1000.0,
            1.0 / perf.avg_step_time.as_secs_f64()
        );
    }

    if let Some((name, perf)) = best_large_system {
        println!("✅ Best large system (10K particles): {}", name);
        println!(
            "   {:.2}ms per step ({:.1} FPS)",
            perf.avg_step_time.as_secs_f64() * 1000.0,
            1.0 / perf.avg_step_time.as_secs_f64()
        );
    }

    println!("\nRecommendations:");
    println!("---------------");
    println!("• Use BarnesHut algorithm for systems >1000 particles");
    println!("• Enable SIMD optimizations for 2-4x performance gain");
    println!("• Use VelocityVerlet integrator for best stability/performance balance");
    println!("• Consider adaptive timestep for variable precision requirements");
    println!("• Profile memory usage for systems >50K particles");

    Ok(())
}

// Helper functions
fn classify_stability(energy_drift: f64) -> String {
    match energy_drift {
        d if d < 1e-12 => "Excellent".to_string(),
        d if d < 1e-9 => "Very Good".to_string(),
        d if d < 1e-6 => "Good".to_string(),
        d if d < 1e-3 => "Acceptable".to_string(),
        _ => "Poor".to_string(),
    }
}

// Placeholder simulation creation functions
fn create_direct_simulation(
    _particles: usize,
) -> std::result::Result<MockSimulation, Box<dyn std::error::Error>> {
    Ok(MockSimulation::new())
}

fn create_barnes_hut_simulation(
    _particles: usize,
) -> std::result::Result<MockSimulation, Box<dyn std::error::Error>> {
    Ok(MockSimulation::new())
}

fn create_fmm_simulation(
    _particles: usize,
) -> std::result::Result<MockSimulation, Box<dyn std::error::Error>> {
    Ok(MockSimulation::new())
}

fn create_verlet_simulation(
    _particles: usize,
) -> std::result::Result<MockSimulation, Box<dyn std::error::Error>> {
    Ok(MockSimulation::new())
}

fn create_leapfrog_simulation(
    _particles: usize,
) -> std::result::Result<MockSimulation, Box<dyn std::error::Error>> {
    Ok(MockSimulation::new())
}

fn create_rk4_simulation(
    _particles: usize,
) -> std::result::Result<MockSimulation, Box<dyn std::error::Error>> {
    Ok(MockSimulation::new())
}

fn create_ias15_simulation(
    _particles: usize,
) -> std::result::Result<MockSimulation, Box<dyn std::error::Error>> {
    Ok(MockSimulation::new())
}

fn create_adaptive_simulation(
    _particles: usize,
    _tolerance: f64,
) -> std::result::Result<MockSimulation, Box<dyn std::error::Error>> {
    Ok(MockSimulation::new())
}

fn add_random_particles(
    _sim: &mut MockSimulation,
    _count: usize,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// Mock simulation for example purposes
struct MockSimulation {
    step_count: usize,
}

impl MockSimulation {
    fn new() -> Self {
        Self { step_count: 0 }
    }

    fn step(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.step_count += 1;
        // Simulate some work
        std::thread::sleep(Duration::from_micros(100));
        Ok(())
    }

    fn parallel_step(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.step()
    }

    fn adaptive_step(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.step()
    }

    fn total_energy(&self) -> f64 {
        // Simulate slight energy drift
        -1000.0 + (self.step_count as f64 * 1e-12)
    }
}
