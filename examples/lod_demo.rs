//! Level of Detail (LOD) System Demonstration
//! 
//! This example demonstrates the basic LOD system for optimizing
//! massive particle count simulations.

use gravwell::lod::LODSystem;
use gravwell::prelude::*;
use std::time::Instant;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🌌 Gravwell LOD System Demonstration");
    println!("=====================================");
    
    // Create a large particle system
    let particle_count = 5000;
    let mut particles = ParticleSet::new();
    
    println!("📊 Creating {} particles for LOD demonstration...", particle_count);
    
    // Add particles at varying distances from origin
    for i in 0..particle_count {
        let angle = 2.0 * std::f64::consts::PI * (i as f64 / particle_count as f64);
        let distance = 50.0 + (i as f64 * 2.0); // Distances from 50 to 10,050
        
        let x = distance * angle.cos();
        let y = distance * angle.sin();
        let z = 0.0;
        
        particles.add_body(
            Body::new()
                .with_mass(1.0)
                .with_position([x, y, z])
                .with_velocity([0.0, 0.0, 0.0])
        )?;
    }
    
    println!("✅ Created {} particles", particles.len());
    
    // Create LOD system with camera at origin
    let mut lod_system = LODSystem::new();
    lod_system.set_camera_position([0.0, 0.0, 0.0].into());
    
    // Demonstrate basic LOD assignment
    println!("\n🎯 Distance-Based LOD Assignment:");
    println!("================================");
    
    let start_time = Instant::now();
    lod_system.update_lod(&particles);
    let lod_time = start_time.elapsed();
    
    // Show performance stats
    let stats = lod_system.performance_stats();
    println!("Total particles: {}", stats.total_particles);
    println!("Active particles: {}", stats.active_particles);
    println!("Full Detail: {} particles", stats.particles_per_level[0]);
    println!("Reduced Detail: {} particles", stats.particles_per_level[1]);
    println!("Minimal Detail: {} particles", stats.particles_per_level[2]);
    println!("Culled: {} particles", stats.particles_per_level[3]);
    println!("Performance gain: {:.2}x", stats.performance_gain);
    println!("LOD Assignment Time: {:.2}ms", lod_time.as_secs_f64() * 1000.0);
    
    // Demonstrate frame-based update frequency
    println!("\n⏱️  Update Frequency Optimization:");
    println!("=================================");
    
    // Show which particles update each frame
    for frame in 1..=4 {
        let updating_count = (0..std::cmp::min(10, particles.len()))
            .filter(|&i| lod_system.should_update_particle(i))
            .count();
        
        let particle_0_lod = lod_system.particle_detail_level(0);
        let particle_1_lod = lod_system.particle_detail_level(1);
        
        println!("Frame {}: {} out of first 10 particles updating (P0: {:?}, P1: {:?})", 
            frame, updating_count, particle_0_lod, particle_1_lod);
        
        // Advance to next frame
        lod_system.update_lod(&particles);
    }
    
    // Demonstrate camera movement effect
    println!("\n📹 Camera Movement Effect:");
    println!("==========================");
    
    let camera_positions = vec![
        [0.0, 0.0, 0.0],
        [1000.0, 0.0, 0.0],
        [2000.0, 0.0, 0.0],
        [3000.0, 0.0, 0.0],
    ];
    
    for &pos in camera_positions.iter() {
        lod_system.set_camera_position(pos.into());
        
        let start_time = Instant::now();
        lod_system.update_lod(&particles);
        let update_time = start_time.elapsed();
        
        let stats = lod_system.performance_stats();
        println!("Camera at ({:.0}, {:.0}, {:.0}): Full: {}, Reduced: {}, Minimal: {}, Culled: {} ({:.2}ms)",
            pos[0], pos[1], pos[2],
            stats.particles_per_level[0], stats.particles_per_level[1], 
            stats.particles_per_level[2], stats.particles_per_level[3],
            update_time.as_secs_f64() * 1000.0);
    }
    
    println!("\n🎯 LOD System Features Demonstrated:");
    println!("- ✅ Distance-based detail level assignment");
    println!("- ✅ Frame-based update frequency optimization");
    println!("- ✅ Camera position effect on LOD assignment");
    println!("- ✅ Real-time performance metrics");
    println!("- ✅ Significant performance improvements for large particle counts");
    
    Ok(())
}
