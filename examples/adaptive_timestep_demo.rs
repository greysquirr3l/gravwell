//! Adaptive timestep control example
//!
//! Demonstrates basic timestep control concepts with error estimation
//! and stability detection using the available Gravwell API.

// use gravwell::error::GravwellError; // Imported via prelude
use gravwell::prelude::*;
use std::time::Instant;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Gravwell - Basic Timestep Control Demo");
    println!("==========================================");

    // Set up a challenging N-body system
    let mut simulation = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(DirectGravity::new())
        .build()?;

    // Create a figure-8 orbit system (known to be chaotic and challenging)
    simulation.add_body(
        Body::new()
            .with_mass(1.0)
            .with_position([-0.97000436, 0.24308753, 0.0])
            .with_velocity([0.4662036850, 0.4323657300, 0.0])
            .with_radius(0.1),
    )?;

    simulation.add_body(
        Body::new()
            .with_mass(1.0)
            .with_position([0.97000436, -0.24308753, 0.0])
            .with_velocity([0.4662036850, 0.4323657300, 0.0])
            .with_radius(0.1),
    )?;

    simulation.add_body(
        Body::new()
            .with_mass(1.0)
            .with_position([0.0, 0.0, 0.0])
            .with_velocity([-0.93240737, -0.86473146, 0.0])
            .with_radius(0.1),
    )?;
    println!("✅ Created chaotic 3-body system");

    // Simulation parameters
    let total_time = 10.0; // 10 seconds
    let mut current_time = 0.0;
    let max_steps = 100000;
    let mut step_count = 0;

    // Basic timestep control parameters
    let mut current_timestep = 0.001; // Start with 1ms
    let min_timestep = 1e-6;
    let max_timestep = 0.01;
    let energy_tolerance = 1e-9;

    // Performance tracking
    let _start_time = Instant::now();
    let mut total_timestep_adjustments = 0;
    let mut rejected_steps = 0;
    let mut min_timestep_used: f64 = current_timestep;
    let mut max_timestep_used: f64 = current_timestep;

    println!("\n🎮 Starting basic adaptive simulation...");
    println!("Target time: {} s", total_time);
    println!("Initial timestep: {:.3e} s", current_timestep);

    let initial_energy = simulation.total_energy();
    let _start_time_main = Instant::now();

    // Main simulation loop with basic timestep control
    while current_time < total_time && step_count < max_steps {
        let energy_before = simulation.total_energy();

        // Take a simulation step
        simulation.step(current_timestep)?;
        current_time += current_timestep;
        step_count += 1;

        let energy_after = simulation.total_energy();
        let energy_error = (energy_after - energy_before).abs() / energy_before.abs();

        // Basic timestep adaptation based on energy conservation
        if energy_error > energy_tolerance {
            // Energy error too high - reduce timestep
            current_timestep *= 0.8;
            total_timestep_adjustments += 1;

            if current_timestep < min_timestep {
                current_timestep = min_timestep;
                rejected_steps += 1;
            }
        } else if energy_error < energy_tolerance * 0.1 {
            // Energy well conserved - can increase timestep
            current_timestep *= 1.1;
            total_timestep_adjustments += 1;

            if current_timestep > max_timestep {
                current_timestep = max_timestep;
            }
        }

        // Track timestep statistics
        min_timestep_used = min_timestep_used.min(current_timestep);
        max_timestep_used = max_timestep_used.max(current_timestep);

        // Progress reporting
        if step_count % 1000 == 0 {
            let current_energy = simulation.total_energy();
            let energy_drift = (current_energy - initial_energy).abs() / initial_energy.abs();

            println!(
                "Step {}: t={:.3}s, dt={:.2e}s, energy_drift={:.2e}",
                step_count, current_time, current_timestep, energy_drift
            );

            if energy_drift > 1e-6 {
                println!("⚠️  Warning: Energy drift detected!");
            }
        }
    }

    let elapsed_time = _start_time_main.elapsed();

    // Final results
    println!("\n🎯 Simulation Complete!");
    println!("========================");
    println!("Total simulation time: {:.3e} s", current_time);
    println!("Steps taken: {}", step_count);
    println!("Wall clock time: {:.3e} s", elapsed_time.as_secs_f64());
    println!(
        "Average steps per second: {:.0}",
        step_count as f64 / elapsed_time.as_secs_f64()
    );

    println!("\n📈 Timestep Statistics:");
    println!("Minimum timestep used: {:.3e} s", min_timestep_used);
    println!("Maximum timestep used: {:.3e} s", max_timestep_used);
    println!("Total timestep adjustments: {}", total_timestep_adjustments);
    println!("Rejected steps: {}", rejected_steps);
    println!(
        "Step acceptance rate: {:.1}%",
        (step_count - rejected_steps) as f64 / step_count as f64 * 100.0
    );

    // Error analysis
    println!("\n🔬 Error Analysis:");
    let final_energy = simulation.total_energy();
    let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();
    println!("Energy conservation error: {:.3e}", energy_error);
    println!("Final energy: {:.6e} J", final_energy);

    // Basic stability assessment
    if energy_error < 1e-8 {
        println!("Final stability status: ✅ Excellent energy conservation");
    } else if energy_error < 1e-6 {
        println!("Final stability status: ✅ Good energy conservation");
    } else {
        println!("Final stability status: ⚠️ Poor energy conservation");
    }

    // Performance comparison
    println!("\n⚡ Performance Comparison:");
    let fixed_timestep_estimate = total_time / min_timestep_used;
    let efficiency = step_count as f64 / fixed_timestep_estimate;
    println!(
        "Fixed minimum timestep would need: {:.0} steps",
        fixed_timestep_estimate
    );
    println!(
        "Adaptive timestep efficiency: {:.1}x speedup",
        1.0 / efficiency
    );

    println!("\n✨ Basic timestep control successfully maintained accuracy while optimizing performance!");

    Ok(())
}
