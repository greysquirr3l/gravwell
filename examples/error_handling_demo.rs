//! Example demonstrating comprehensive error handling and recovery.

use gravwell::prelude::*;

fn main() -> Result<()> {
    println!("🔧 Gravwell Error Handling & Recovery Demo");
    println!("==========================================");

    // Demo 1: Input validation
    demo_input_validation()?;

    // Demo 2: Automatic error recovery
    demo_error_recovery()?;

    // Demo 3: Timestep stability analysis
    demo_timestep_stability()?;

    // Demo 4: Robust simulation with recovery
    demo_robust_simulation()?;

    println!("\n✅ All error handling demos completed successfully!");
    Ok(())
}

fn demo_input_validation() -> Result<()> {
    println!("\n📋 Demo 1: Input Validation");
    println!("----------------------------");

    // Try to create invalid bodies and show proper error handling
    println!("Testing invalid mass...");
    match Body::new().with_mass(-1.0).validate() {
        Err(e) => println!("✅ Caught invalid mass: {}", e),
        Ok(_) => println!("❌ Should have failed for negative mass"),
    }

    println!("Testing invalid position...");
    match Body::new().with_position([f64::NAN, 0.0, 0.0]).validate() {
        Err(e) => println!("✅ Caught invalid position: {}", e),
        Ok(_) => println!("❌ Should have failed for NaN position"),
    }

    println!("Testing invalid velocity...");
    match Body::new()
        .with_velocity([0.0, f64::INFINITY, 0.0])
        .validate()
    {
        Err(e) => println!("✅ Caught invalid velocity: {}", e),
        Ok(_) => println!("❌ Should have failed for infinite velocity"),
    }

    // Show timestep validation
    println!("Testing invalid timestep...");
    match Validator::validate_timestep(-0.01) {
        Err(e) => println!("✅ Caught invalid timestep: {}", e),
        Ok(_) => println!("❌ Should have failed for negative timestep"),
    }

    Ok(())
}

fn demo_error_recovery() -> Result<()> {
    println!("\n🔄 Demo 2: Automatic Error Recovery");
    println!("------------------------------------");

    // Create a system with some problematic data
    let mut positions = vec![
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(f64::INFINITY, 0.0, 0.0), // Invalid
        Vector3::new(1.0, 1.0, 1.0),
    ];
    let mut velocities = vec![
        Vector3::new(f64::NAN, 0.0, 0.0), // Invalid
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
    ];
    let mut masses = vec![1.0, -1.0, 1.0]; // Invalid mass

    println!("Original problematic data:");
    println!(
        "  Position 1: [{}, {}, {}]",
        positions[1].x, positions[1].y, positions[1].z
    );
    println!(
        "  Velocity 0: [{}, {}, {}]",
        velocities[0].x, velocities[0].y, velocities[0].z
    );
    println!("  Mass 1: {}", masses[1]);

    // Apply error recovery
    match ErrorRecovery::fix_invalid_particles(&mut positions, &mut velocities, &mut masses) {
        RecoveryResult::Fixed { fixes } => {
            println!("\n✅ Successfully applied fixes:");
            for fix in fixes {
                println!("  - {}", fix);
            }
        }
        RecoveryResult::Fatal { errors } => {
            println!("\n❌ Fatal errors found:");
            for error in errors {
                println!("  - {}", error);
            }
        }
        RecoveryResult::NoActionNeeded => {
            println!("\n✅ No fixes needed");
        }
    }

    println!("\nFixed data:");
    println!(
        "  Position 1: [{}, {}, {}]",
        positions[1].x, positions[1].y, positions[1].z
    );
    println!(
        "  Velocity 0: [{}, {}, {}]",
        velocities[0].x, velocities[0].y, velocities[0].z
    );
    println!("  Mass 1: {}", masses[1]);

    Ok(())
}

fn demo_timestep_stability() -> Result<()> {
    println!("\n⏰ Demo 3: Timestep Stability Analysis");
    println!("--------------------------------------");

    // Create a tight binary system
    let positions = vec![
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.1, 0.0, 0.0), // Very close
    ];
    let velocities = vec![
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 10.0, 0.0), // High velocity
    ];

    let stats = Validator::compute_system_statistics(&positions, &velocities);

    println!("System statistics:");
    println!("  Particle count: {}", stats.particle_count);
    println!("  Maximum velocity: {:.3}", stats.max_velocity);
    println!("  Minimum distance: {:.6}", stats.min_distance);
    println!("  System stable: {}", stats.is_stable());

    let suggested_dt = stats.suggest_timestep();
    println!("  Suggested timestep: {:.6}", suggested_dt);

    // Test various timesteps
    let test_timesteps = vec![1.0, 0.1, 0.01, suggested_dt];

    for dt in test_timesteps {
        match Validator::analyze_timestep_stability(dt, stats.max_velocity, stats.min_distance) {
            Ok(_) => println!("  ✅ Timestep {:.6} is stable", dt),
            Err(e) => println!("  ❌ Timestep {:.6}: {}", dt, e),
        }
    }

    Ok(())
}

fn demo_robust_simulation() -> Result<()> {
    println!("\n🚀 Demo 4: Robust Simulation with Recovery");
    println!("------------------------------------------");

    // Create a simulation that might encounter problems
    let mut simulation = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(DirectGravity::new())
        .build()?;

    // Add some challenging particles
    simulation.add_body(
        Body::new()
            .with_mass(1.0)
            .with_position([0.0, 0.0, 0.0])
            .with_velocity([0.0, 0.0, 0.0])
            .with_radius(0.1),
    )?;

    simulation.add_body(
        Body::new()
            .with_mass(1.0)
            .with_position([1.0, 0.0, 0.0])
            .with_velocity([0.0, 1.0, 0.0])
            .with_radius(0.1),
    )?;

    let initial_energy = simulation.total_energy();
    println!("Initial energy: {:.6e}", initial_energy);

    let mut timestep = 0.1;
    let mut total_time = 0.0;
    let target_time = 1.0;
    let mut step_count = 0;

    while total_time < target_time && step_count < 1000 {
        step_count += 1;

        // Try to take a simulation step
        match simulation.step(timestep) {
            Ok(_) => {
                total_time += timestep;

                // Check energy conservation
                let current_energy = simulation.total_energy();
                let energy_drift = (current_energy - initial_energy).abs() / initial_energy.abs();

                if energy_drift > 1e-10 {
                    println!(
                        "⚠️  Warning: Energy drift detected at step {}: {:.3e}",
                        step_count, energy_drift
                    );

                    // Apply recovery by reducing timestep
                    timestep *= 0.5;
                    println!("   Reduced timestep to {:.6}", timestep);
                }
            }
            Err(e) => {
                println!("❌ Simulation error at step {}: {}", step_count, e);

                if e.is_recoverable() {
                    if let Some(suggestion) = e.recovery_suggestion() {
                        println!("   Recovery suggestion: {}", suggestion);
                        timestep *= 0.1; // Drastically reduce timestep
                        println!("   Reduced timestep to {:.6}", timestep);
                        continue;
                    }
                } else {
                    println!("   Error is not recoverable, terminating simulation");
                    break;
                }
            }
        }

        if step_count % 100 == 0 {
            let current_energy = simulation.total_energy();
            let energy_drift = (current_energy - initial_energy).abs() / initial_energy.abs();
            println!(
                "Step {}: time={:.3}, energy={:.6e}, drift={:.3e}",
                step_count, total_time, current_energy, energy_drift
            );
        }
    }

    let final_energy = simulation.total_energy();
    let final_drift = (final_energy - initial_energy).abs() / initial_energy.abs();

    println!("\nSimulation completed:");
    println!("  Total steps: {}", step_count);
    println!("  Final time: {:.3}", total_time);
    println!("  Final timestep: {:.6}", timestep);
    println!("  Final energy: {:.6e}", final_energy);
    println!("  Energy drift: {:.3e}", final_drift);

    if final_drift < 1e-8 {
        println!("  ✅ Excellent energy conservation!");
    } else if final_drift < 1e-6 {
        println!("  ✅ Good energy conservation");
    } else {
        println!("  ⚠️  Noticeable energy drift - consider smaller timesteps");
    }

    Ok(())
}
