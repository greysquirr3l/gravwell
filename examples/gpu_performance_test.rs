//! GPU Barnes-Hut Performance Validation
//!
//! Tests to validate 60 FPS performance target with large particle systems

use std::time::Instant;
use gravwell::prelude::*;

#[cfg(feature = "gpu")]
use gravwell::forces::GpuBarnesHut;

fn create_particle_system(n: usize) -> ParticleSet {
    let mut particle_set = ParticleSet::new();
    
    println!("Creating {} particle system...", n);
    for i in 0..n {
        let phi = 2.0 * std::f64::consts::PI * (i as f64 / n as f64);
        let costheta = 2.0 * (i as f64 / n as f64) - 1.0;
        let theta = costheta.acos();
        let r = 100.0 * (i as f64 / n as f64).cbrt(); // Cubic root for volume distribution

        let x = r * theta.sin() * phi.cos();
        let y = r * theta.sin() * phi.sin();
        let z = r * costheta;

        let body = Body::new()
            .with_position([x, y, z])
            .with_velocity([0.0, 0.0, 0.0])
            .with_mass(1.0)
            .with_radius(0.1);

        particle_set.add_body(body).unwrap();
    }
    
    particle_set
}

fn test_performance(particle_count: usize, iterations: usize) {
    println!("\n=== Testing {} particles ===", particle_count);
    
    let particle_set = create_particle_system(particle_count);
    
    // Test CPU Barnes-Hut
    let cpu_barnes_hut = BarnesHut::new().theta(0.5);
    let mut cpu_forces = vec![Vector3::zeros(); particle_count];
    
    let start = Instant::now();
    for _ in 0..iterations {
        cpu_barnes_hut.calculate_forces(&particle_set, &mut cpu_forces).unwrap();
    }
    let cpu_duration = start.elapsed();
    let cpu_avg_ms = cpu_duration.as_millis() as f64 / iterations as f64;
    
    println!("CPU Barnes-Hut: {:.2}ms avg ({:.1} FPS)", cpu_avg_ms, 1000.0 / cpu_avg_ms);

    #[cfg(feature = "gpu")]
    {
        // Test GPU Barnes-Hut
        let gpu_barnes_hut = GpuBarnesHut::new().theta(0.5);
        let mut gpu_forces = vec![Vector3::zeros(); particle_count];
        
        // Warm up GPU
        gpu_barnes_hut.calculate_forces(&particle_set, &mut gpu_forces).unwrap();
        
        let start = Instant::now();
        for _ in 0..iterations {
            gpu_barnes_hut.calculate_forces(&particle_set, &mut gpu_forces).unwrap();
        }
        let gpu_duration = start.elapsed();
        let gpu_avg_ms = gpu_duration.as_millis() as f64 / iterations as f64;
        
        println!("GPU Barnes-Hut: {:.2}ms avg ({:.1} FPS)", gpu_avg_ms, 1000.0 / gpu_avg_ms);
        
        let speedup = cpu_avg_ms / gpu_avg_ms;
        println!("GPU Speedup: {:.1}x", speedup);
        
        // Check 60 FPS target (16.67ms)
        if gpu_avg_ms <= 16.67 {
            println!("✅ 60 FPS target achieved!");
        } else {
            println!("❌ 60 FPS target missed (need ≤16.67ms)");
        }
    }

    #[cfg(not(feature = "gpu"))]
    {
        println!("GPU feature not enabled - compile with --features gpu");
    }
}

fn main() {
    println!("GPU Barnes-Hut Performance Validation");
    println!("Target: 60 FPS (≤16.67ms per frame)");

    // Progressive scaling test
    let test_cases = vec![
        (1_000, 10),   // Small system
        (5_000, 5),    // Medium system
        (10_000, 3),   // Large system
        (25_000, 2),   // Very large system
    ];

    for (particle_count, iterations) in test_cases {
        test_performance(particle_count, iterations);
    }

    // Ultimate test: 50,000 particles (TODO.md target)
    println!("\n=== ULTIMATE TEST: 50,000 particles (TODO.md target) ===");
    test_performance(50_000, 1);
}