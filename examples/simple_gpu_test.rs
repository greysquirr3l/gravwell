//! Simple GPU Performance Test
//!
//! Tests the working SimpleGpuBarnesHut implementation

use std::time::Instant;
use gravwell::prelude::*;

#[cfg(feature = "gpu")]
use gravwell::forces::SimpleGpuBarnesHut;

fn create_test_system(n: usize) -> ParticleSet {
    let mut particle_set = ParticleSet::new();
    
    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * (i as f64 / n as f64);
        let radius = 10.0 + (i as f64 / n as f64) * 90.0;
        
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let z = 0.0;

        let body = Body::new()
            .with_position([x, y, z])
            .with_velocity([0.0, 0.0, 0.0])
            .with_mass(1.0)
            .with_radius(0.1);

        particle_set.add_body(body).unwrap();
    }
    
    particle_set
}

fn test_simple_gpu_performance(particle_count: usize) {
    println!("\n=== Testing Simple GPU with {} particles ===", particle_count);
    
    let particle_set = create_test_system(particle_count);
    
    // Test CPU Barnes-Hut
    let cpu_barnes_hut = BarnesHut::new().theta(0.5);
    let mut cpu_forces = vec![Vector3::zeros(); particle_count];
    
    let start = Instant::now();
    cpu_barnes_hut.calculate_forces(&particle_set, &mut cpu_forces).unwrap();
    let cpu_duration = start.elapsed();
    
    println!("CPU Barnes-Hut: {:.2}ms ({:.1} FPS)", 
        cpu_duration.as_millis(), 1000.0 / cpu_duration.as_millis() as f64);

    #[cfg(feature = "gpu")]
    {
        // Test Simple GPU
        let simple_gpu = SimpleGpuBarnesHut::new()
            .gravity_constant(6.67430e-11);
        let mut gpu_forces = vec![Vector3::zeros(); particle_count];
        
        let start = Instant::now();
        match simple_gpu.calculate_forces(&particle_set, &mut gpu_forces) {
            Ok(()) => {
                let gpu_duration = start.elapsed();
                
                println!("Simple GPU: {:.2}ms ({:.1} FPS)", 
                    gpu_duration.as_millis(), 1000.0 / gpu_duration.as_millis() as f64);
                
                let speedup = cpu_duration.as_millis() as f64 / gpu_duration.as_millis() as f64;
                println!("GPU Speedup: {:.1}x", speedup);
                
                // Check 60 FPS target (16.67ms)
                if gpu_duration.as_millis() <= 16 {
                    println!("✅ 60 FPS target achieved!");
                } else {
                    println!("❌ 60 FPS target missed (need ≤16ms)");
                }
                
                // Validate forces match
                let mut max_diff: f64 = 0.0;
                for i in 0..particle_count.min(10) {
                    let diff = (cpu_forces[i] - gpu_forces[i]).norm();
                    max_diff = max_diff.max(diff);
                }
                
                if max_diff < 1e-6 {
                    println!("✅ Force calculation matches CPU (max diff: {:.3e})", max_diff);
                } else {
                    println!("❌ Force calculation differs from CPU (max diff: {:.3e})", max_diff);
                }
            }
            Err(e) => {
                println!("❌ Simple GPU failed: {}", e);
            }
        }
    }

    #[cfg(not(feature = "gpu"))]
    {
        println!("GPU feature not enabled - compile with --features gpu");
    }
}

fn main() {
    println!("Simple GPU Barnes-Hut Performance Test");
    println!("Using direct GPU computation (O(N²))");

    // Progressive scaling test
    let test_cases = vec![100, 500, 1_000, 2_000, 5_000];

    for particle_count in test_cases {
        test_simple_gpu_performance(particle_count);
    }
    
    println!("\nNote: This is a direct O(N²) GPU implementation for validation.");
    println!("For large systems, use CPU Barnes-Hut O(N log N) algorithm.");
}