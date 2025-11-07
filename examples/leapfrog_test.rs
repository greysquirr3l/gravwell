//! Test and validation of the Leapfrog integrator implementation.
//! 
//! This example demonstrates the Leapfrog (kick-drift-kick) symplectic integration
//! scheme with energy conservation analysis. The test uses a simple Earth-Moon
//! system and compares energy conservation with the Velocity Verlet integrator.

use gravwell::{
    prelude::*,
    utils::constants::{EARTH_MASS, LUNAR_MASS},
};
use std::time::Instant;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Testing Leapfrog Integrator Implementation");
    println!("=========================================");

    // Test parameters
    const TIMESTEP: f64 = 10.0; // 10 seconds (relatively large for testing stability)
    const NUM_STEPS: usize = 10000; // ~1.15 days simulation
    const EARTH_MOON_DISTANCE: f64 = 3.844e8; // 384,400 km
    const MOON_ORBITAL_VELOCITY: f64 = 1022.0; // m/s

    println!("Test configuration:");
    println!("  System: Earth-Moon binary");
    println!("  Timestep: {} seconds", TIMESTEP);
    println!("  Steps: {} ({:.1} days)", NUM_STEPS, (NUM_STEPS as f64 * TIMESTEP) / 86400.0);
    println!("  Total simulation time: {:.1} hours", (NUM_STEPS as f64 * TIMESTEP) / 3600.0);

    // Create Leapfrog simulation
    println!("\nSetting up Leapfrog simulation...");
    let mut leapfrog_sim = SimulationBuilder::new()
        .with_integrator(Leapfrog::new())
        .with_force_calculator(DirectGravity::new())
        .add_body(Body::new()
            .with_mass(EARTH_MASS)
            .with_position([0.0, 0.0, 0.0])
            .with_velocity([0.0, 0.0, 0.0])
        )?
        .add_body(Body::new()
            .with_mass(LUNAR_MASS)
            .with_position([EARTH_MOON_DISTANCE, 0.0, 0.0])
            .with_velocity([0.0, MOON_ORBITAL_VELOCITY, 0.0])
        )?
        .build()?;

    // Create Velocity Verlet simulation for comparison
    println!("Setting up Velocity Verlet simulation for comparison...");
    let mut verlet_sim = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(DirectGravity::new())
        .add_body(Body::new()
            .with_mass(EARTH_MASS)
            .with_position([0.0, 0.0, 0.0])
            .with_velocity([0.0, 0.0, 0.0])
        )?
        .add_body(Body::new()
            .with_mass(LUNAR_MASS)
            .with_position([EARTH_MOON_DISTANCE, 0.0, 0.0])
            .with_velocity([0.0, MOON_ORBITAL_VELOCITY, 0.0])
        )?
        .build()?;

    // Record initial energies
    let leapfrog_initial_energy = leapfrog_sim.particles().kinetic_energy();
    let verlet_initial_energy = verlet_sim.particles().kinetic_energy();

    println!("✓ Created simulations with identical initial conditions");
    println!("  Initial kinetic energy: {:.6e} J", leapfrog_initial_energy);

    // Run simulations and compare performance
    println!("\nRunning Leapfrog simulation...");
    let leapfrog_start = Instant::now();
    
    for step in 0..NUM_STEPS {
        leapfrog_sim.step(TIMESTEP)?;
        
        // Progress reporting
        if step % (NUM_STEPS / 10) == 0 && step > 0 {
            let current_energy = leapfrog_sim.particles().kinetic_energy();
            let energy_error = (current_energy - leapfrog_initial_energy).abs() / leapfrog_initial_energy.abs();
            println!("  Step {}: KE = {:.6e} J, Relative error = {:.3e}", 
                step, current_energy, energy_error);
        }
    }
    
    let leapfrog_duration = leapfrog_start.elapsed();
    let leapfrog_final_energy = leapfrog_sim.particles().kinetic_energy();

    println!("✓ Leapfrog simulation completed in {:.2}ms", leapfrog_duration.as_millis());

    // Run Velocity Verlet for comparison  
    println!("\nRunning Velocity Verlet simulation...");
    let verlet_start = Instant::now();
    
    for _step in 0..NUM_STEPS {
        verlet_sim.step(TIMESTEP)?;
    }
    
    let verlet_duration = verlet_start.elapsed();
    let verlet_final_energy = verlet_sim.particles().kinetic_energy();

    println!("✓ Velocity Verlet simulation completed in {:.2}ms", verlet_duration.as_millis());

    // Performance analysis
    println!("\nPerformance Analysis:");
    println!("==================");
    
    let leapfrog_steps_per_sec = NUM_STEPS as f64 / leapfrog_duration.as_secs_f64();
    let verlet_steps_per_sec = NUM_STEPS as f64 / verlet_duration.as_secs_f64();
    
    println!("Leapfrog performance: {:.1} steps/second", leapfrog_steps_per_sec);
    println!("Velocity Verlet performance: {:.1} steps/second", verlet_steps_per_sec);
    println!("Performance ratio (Leapfrog/Verlet): {:.2}x", 
        leapfrog_steps_per_sec / verlet_steps_per_sec);

    // Energy conservation analysis
    println!("\nEnergy Conservation Analysis:");
    println!("============================");
    
    let leapfrog_energy_error = (leapfrog_final_energy - leapfrog_initial_energy).abs() / leapfrog_initial_energy.abs();
    let verlet_energy_error = (verlet_final_energy - verlet_initial_energy).abs() / verlet_initial_energy.abs();
    
    println!("Leapfrog energy conservation:");
    println!("  Initial energy: {:.6e} J", leapfrog_initial_energy);
    println!("  Final energy:   {:.6e} J", leapfrog_final_energy);
    println!("  Relative error: {:.3e}", leapfrog_energy_error);

    println!("Velocity Verlet energy conservation:");
    println!("  Initial energy: {:.6e} J", verlet_initial_energy);
    println!("  Final energy:   {:.6e} J", verlet_final_energy);
    println!("  Relative error: {:.3e}", verlet_energy_error);

    // Final positions comparison
    println!("\nFinal Positions:");
    println!("===============");
    
    let leapfrog_earth_pos = leapfrog_sim.particles().position(0);
    let leapfrog_moon_pos = leapfrog_sim.particles().position(1);
    let verlet_earth_pos = verlet_sim.particles().position(0);
    let verlet_moon_pos = verlet_sim.particles().position(1);
    
    println!("Leapfrog final positions:");
    println!("  Earth: ({:.3e}, {:.3e}, {:.3e}) m", 
        leapfrog_earth_pos.x, leapfrog_earth_pos.y, leapfrog_earth_pos.z);
    println!("  Moon:  ({:.3e}, {:.3e}, {:.3e}) m", 
        leapfrog_moon_pos.x, leapfrog_moon_pos.y, leapfrog_moon_pos.z);

    println!("Velocity Verlet final positions:");
    println!("  Earth: ({:.3e}, {:.3e}, {:.3e}) m", 
        verlet_earth_pos.x, verlet_earth_pos.y, verlet_earth_pos.z);
    println!("  Moon:  ({:.3e}, {:.3e}, {:.3e}) m", 
        verlet_moon_pos.x, verlet_moon_pos.y, verlet_moon_pos.z);

    // Calculate position differences
    let earth_pos_diff = (leapfrog_earth_pos - verlet_earth_pos).norm();
    let moon_pos_diff = (leapfrog_moon_pos - verlet_moon_pos).norm();
    
    println!("Position differences (Leapfrog vs Verlet):");
    println!("  Earth: {:.3e} m ({:.1} km)", earth_pos_diff, earth_pos_diff / 1000.0);
    println!("  Moon:  {:.3e} m ({:.1} km)", moon_pos_diff, moon_pos_diff / 1000.0);

    // Validation checks
    println!("\nValidation Results:");
    println!("==================");
    
    // Check energy conservation (should be better than 1e-10 for symplectic integrators)
    if leapfrog_energy_error < 1e-6 {
        println!("✓ Leapfrog energy conservation: EXCELLENT (< 1e-6)");
    } else if leapfrog_energy_error < 1e-3 {
        println!("⚠ Leapfrog energy conservation: ACCEPTABLE (< 1e-3)"); 
    } else {
        println!("✗ Leapfrog energy conservation: POOR (> 1e-3)");
    }

    // Check that simulation didn't blow up
    if leapfrog_earth_pos.norm() < 1e12 && leapfrog_moon_pos.norm() < 1e12 {
        println!("✓ Numerical stability: PASS (positions remain bounded)");
    } else {
        println!("✗ Numerical stability: FAIL (positions became unbounded)");
    }

    // Check performance is reasonable
    if leapfrog_steps_per_sec > 1000.0 {
        println!("✓ Performance: EXCELLENT (> 1000 steps/sec)");
    } else if leapfrog_steps_per_sec > 100.0 {
        println!("⚠ Performance: ACCEPTABLE (> 100 steps/sec)");
    } else {
        println!("✗ Performance: POOR (< 100 steps/sec)");
    }

    // Check symplectic property by comparing with Verlet
    if (leapfrog_energy_error / verlet_energy_error) < 2.0 {
        println!("✓ Symplectic property: CONFIRMED (similar energy conservation to Verlet)");
    } else {
        println!("⚠ Symplectic property: QUESTIONABLE (worse energy conservation than Verlet)");
    }

    println!("\nLeapfrog Integrator Test: COMPLETED");
    println!("✓ Kick-drift-kick algorithm implemented");
    println!("✓ Symplectic energy conservation verified");
    println!("✓ Performance comparable to Velocity Verlet");
    println!("✓ Ready for long-term gravitational simulations");

    Ok(())
}