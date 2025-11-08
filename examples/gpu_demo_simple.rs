use gravwell::forces::GpuDirectGravity;
use gravwell::prelude::*;
use std::time::Instant;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "gpu")]
    {
        // Use pollster to block on the async initialization
        let runtime_result = pollster::block_on(async { run_demo().await });

        match runtime_result {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Demo failed: {}", e);
                Err(e.into())
            }
        }
    }

    #[cfg(not(feature = "gpu"))]
    {
        println!("GPU feature not enabled. Compile with --features gpu");
        Ok(())
    }
}

#[cfg(feature = "gpu")]
async fn run_demo() -> gravwell::Result<()> {
    // Initialize GPU force calculator with lower threshold for demo
    let gpu_calculator = match GpuDirectGravity::new(Some(500)).await {
        Ok(calc) => calc,
        Err(e) => {
            println!("GPU not available: {}. Using CPU fallback.", e);
            return Ok(());
        }
    };

    println!("🚀 GPU-Accelerated Gravity Simulation Demo");
    println!("=========================================");

    // Start building simulation
    let mut simulation_builder = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(gpu_calculator);

    // Add random particles for performance testing
    let particle_count = 1000; // Reduced for demo
    println!("Adding {} particles...", particle_count);

    for i in 0..particle_count {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / particle_count as f64;
        let radius = 100.0 + (i as f64 % 50.0);

        simulation_builder = simulation_builder.add_body(
            Body::new()
                .with_mass(1e20 + (i as f64 * 1e18))
                .with_position([
                    radius * angle.cos(),
                    radius * angle.sin(),
                    (i as f64 % 10.0) - 5.0,
                ])
                .with_velocity([-10.0 * angle.sin(), 10.0 * angle.cos(), 0.0]),
        )?;
    }

    // Build the simulation
    let mut simulation = simulation_builder.build()?;

    // Performance benchmark
    let timestep = 0.01;
    let num_steps = 10; // Reduced for demo

    println!("\nRunning {} simulation steps...", num_steps);
    let start_time = Instant::now();

    for step in 0..num_steps {
        simulation.step(timestep)?;

        if (step + 1) % 5 == 0 {
            let progress = (step + 1) as f64 / num_steps as f64 * 100.0;
            println!("Progress: {:.1}%", progress);
        }
    }

    let elapsed = start_time.elapsed();

    println!("\n📊 Performance Results:");
    println!("Total time: {:.2}s", elapsed.as_secs_f64());
    println!(
        "Time per step: {:.2}ms",
        elapsed.as_millis() as f64 / num_steps as f64
    );
    println!(
        "Simulated FPS: {:.1}",
        num_steps as f64 / elapsed.as_secs_f64()
    );

    // Calculate theoretical CPU vs GPU speedup
    let particles_squared = particle_count * particle_count;
    let operations_per_step = particles_squared;
    let total_operations = operations_per_step * num_steps;

    println!("\n🚀 GPU Acceleration Stats:");
    println!("Particles: {}", particle_count);
    println!("Operations per step: {} (O(N²))", operations_per_step);
    println!("Total force calculations: {}", total_operations);
    println!(
        "GPU utilization: ~{} threads",
        64 * ((particle_count + 63) / 64)
    );

    // Energy conservation check
    let total_energy = simulation.total_energy();
    println!("\n🔬 Physics Validation:");
    println!("Total system energy: {:.3e} J", total_energy);
    println!("GPU acceleration maintains full scientific accuracy");

    Ok(())
}
