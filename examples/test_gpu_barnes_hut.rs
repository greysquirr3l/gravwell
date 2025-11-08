//! Test GPU Barnes-Hut Algorithm
//!
//! Basic test to validate the GPU Barnes-Hut implementation

use gravwell::prelude::*;

#[cfg(feature = "gpu")]
use gravwell::forces::GpuBarnesHut;

fn main() {
    println!("Testing GPU Barnes-Hut Algorithm");

    // Create a small test system
    let mut particle_set = ParticleSet::new();
    
    for i in 0..10 {
        let angle = 2.0 * std::f64::consts::PI * (i as f64 / 10.0);
        let x = 10.0 * angle.cos();
        let y = 10.0 * angle.sin();
        let z = 0.0;

        let body = Body::new()
            .with_position([x, y, z])
            .with_velocity([0.0, 0.0, 0.0])
            .with_mass(1.0)
            .with_radius(0.1);

        if let Err(e) = particle_set.add_body(body) {
            println!("Failed to add body: {}", e);
            return;
        }
    }

    println!("Created particle system with {} particles", particle_set.len());

    // Test CPU Barnes-Hut first
    let cpu_barnes_hut = BarnesHut::new().theta(0.5);
    let mut cpu_forces = vec![Vector3::zeros(); particle_set.len()];
    
    match cpu_barnes_hut.calculate_forces(&particle_set, &mut cpu_forces) {
        Ok(()) => println!("CPU Barnes-Hut calculation successful"),
        Err(e) => println!("CPU Barnes-Hut failed: {}", e),
    }

    #[cfg(feature = "gpu")]
    {
        // Test GPU Barnes-Hut
        println!("Testing GPU Barnes-Hut...");
        
        let gpu_barnes_hut = GpuBarnesHut::new().theta(0.5);
        let mut gpu_forces = vec![Vector3::zeros(); particle_set.len()];
        
        match gpu_barnes_hut.calculate_forces(&particle_set, &mut gpu_forces) {
            Ok(()) => {
                println!("GPU Barnes-Hut calculation successful");
                
                // Compare first few forces
                println!("Force comparison:");
                for i in 0..std::cmp::min(3, particle_set.len()) {
                    println!("  Particle {}: CPU {:?} vs GPU {:?}", 
                        i, cpu_forces[i], gpu_forces[i]);
                }
            }
            Err(e) => println!("GPU Barnes-Hut failed: {}", e),
        }
    }

    #[cfg(not(feature = "gpu"))]
    {
        println!("GPU feature not enabled - compile with --features gpu");
    }
}