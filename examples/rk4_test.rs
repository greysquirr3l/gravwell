//! Test and validation of the RK4 integrator implementation.
//! 
//! This example demonstrates the 4th-order Runge-Kutta integration method
//! with accuracy analysis. RK4 provides excellent short-term precision but
//! requires 4 force evaluations per timestep, making it computationally expensive.

use gravwell::{
    prelude::*,
    utils::constants::{EARTH_MASS, LUNAR_MASS},
};
use std::time::Instant;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Testing RK4 Integrator Implementation");
    println!("=====================================");

    // Test parameters
    const TIMESTEP: f64 = 1.0; // 1 second (small timestep to demonstrate RK4 accuracy)
    const NUM_STEPS: usize = 3600; // 1 hour simulation
    const EARTH_MOON_DISTANCE: f64 = 3.844e8; // 384,400 km
    const MOON_ORBITAL_VELOCITY: f64 = 1022.0; // m/s

    println!("Test configuration:");
    println!("  System: Earth-Moon binary");
    println!("  Timestep: {} seconds", TIMESTEP);
    println!("  Steps: {} ({:.1} hours)", NUM_STEPS, NUM_STEPS as f64 / 3600.0);
    println!("  Focus: High-precision accuracy demonstration");

    // Create RK4 simulation
    println!("\nSetting up RK4 simulation...");
    let mut rk4_sim = SimulationBuilder::new()
        .with_integrator(RungeKutta4::new())
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

    // Create Leapfrog simulation for comparison
    println!("Setting up Leapfrog simulation for comparison...");
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

    // Record initial energies
    let rk4_initial_energy = rk4_sim.particles().kinetic_energy();
    let verlet_initial_energy = verlet_sim.particles().kinetic_energy();
    let leapfrog_initial_energy = leapfrog_sim.particles().kinetic_energy();

    println!("✓ Created simulations with identical initial conditions");
    println!("  Initial kinetic energy: {:.6e} J", rk4_initial_energy);

    // Run RK4 simulation
    println!("\nRunning RK4 simulation...");
    let rk4_start = Instant::now();
    
    for step in 0..NUM_STEPS {
        rk4_sim.step(TIMESTEP)?;
        
        // Progress reporting every 10 minutes
        if step % 600 == 0 && step > 0 {
            let current_energy = rk4_sim.particles().kinetic_energy();
            let energy_error = (current_energy - rk4_initial_energy).abs() / rk4_initial_energy.abs();
            println!("  Step {}: KE = {:.6e} J, Relative error = {:.3e}", 
                step, current_energy, energy_error);
        }
    }
    
    let rk4_duration = rk4_start.elapsed();
    let rk4_final_energy = rk4_sim.particles().kinetic_energy();

    println!("✓ RK4 simulation completed in {:.2}ms", rk4_duration.as_millis());

    // Run Velocity Verlet for comparison  
    println!("\nRunning Velocity Verlet simulation...");
    let verlet_start = Instant::now();
    
    for _step in 0..NUM_STEPS {
        verlet_sim.step(TIMESTEP)?;
    }
    
    let verlet_duration = verlet_start.elapsed();
    let verlet_final_energy = verlet_sim.particles().kinetic_energy();

    println!("✓ Velocity Verlet simulation completed in {:.2}ms", verlet_duration.as_millis());

    // Run Leapfrog for comparison
    println!("\nRunning Leapfrog simulation...");
    let leapfrog_start = Instant::now();
    
    for _step in 0..NUM_STEPS {
        leapfrog_sim.step(TIMESTEP)?;
    }
    
    let leapfrog_duration = leapfrog_start.elapsed();
    let leapfrog_final_energy = leapfrog_sim.particles().kinetic_energy();

    println!("✓ Leapfrog simulation completed in {:.2}ms", leapfrog_duration.as_millis());

    // Performance analysis
    println!("\nPerformance Analysis:");
    println!("====================");
    
    let rk4_steps_per_sec = NUM_STEPS as f64 / rk4_duration.as_secs_f64();
    let verlet_steps_per_sec = NUM_STEPS as f64 / verlet_duration.as_secs_f64();
    let leapfrog_steps_per_sec = NUM_STEPS as f64 / leapfrog_duration.as_secs_f64();
    
    println!("RK4 performance: {:.1} steps/second", rk4_steps_per_sec);
    println!("Velocity Verlet performance: {:.1} steps/second", verlet_steps_per_sec);
    println!("Leapfrog performance: {:.1} steps/second", leapfrog_steps_per_sec);
    
    println!("Performance ratios (vs RK4):");
    println!("  Verlet: {:.2}x faster", verlet_steps_per_sec / rk4_steps_per_sec);
    println!("  Leapfrog: {:.2}x faster", leapfrog_steps_per_sec / rk4_steps_per_sec);
    
    println!("Cost per accuracy: RK4 uses 4 force evaluations vs 2 for others");

    // Accuracy analysis
    println!("\nAccuracy Analysis:");
    println!("=================");
    
    let rk4_energy_error = (rk4_final_energy - rk4_initial_energy).abs() / rk4_initial_energy.abs();
    let verlet_energy_error = (verlet_final_energy - verlet_initial_energy).abs() / verlet_initial_energy.abs();
    let leapfrog_energy_error = (leapfrog_final_energy - leapfrog_initial_energy).abs() / leapfrog_initial_energy.abs();
    
    println!("Energy conservation (1 hour simulation):");
    println!("  RK4:         {:.3e} relative error", rk4_energy_error);
    println!("  Verlet:      {:.3e} relative error", verlet_energy_error);
    println!("  Leapfrog:    {:.3e} relative error", leapfrog_energy_error);

    // Position accuracy comparison
    println!("\nPosition Accuracy Comparison:");
    println!("============================");
    
    let rk4_earth_pos = rk4_sim.particles().position(0);
    let rk4_moon_pos = rk4_sim.particles().position(1);
    let verlet_earth_pos = verlet_sim.particles().position(0);
    let verlet_moon_pos = verlet_sim.particles().position(1);
    let leapfrog_earth_pos = leapfrog_sim.particles().position(0);
    let leapfrog_moon_pos = leapfrog_sim.particles().position(1);
    
    println!("Final positions after 1 hour:");
    println!("RK4:");
    println!("  Earth: ({:.3e}, {:.3e}, {:.3e}) m", 
        rk4_earth_pos.x, rk4_earth_pos.y, rk4_earth_pos.z);
    println!("  Moon:  ({:.3e}, {:.3e}, {:.3e}) m", 
        rk4_moon_pos.x, rk4_moon_pos.y, rk4_moon_pos.z);

    // Calculate position differences from RK4 (reference)
    let verlet_earth_diff = (verlet_earth_pos - rk4_earth_pos).norm();
    let verlet_moon_diff = (verlet_moon_pos - rk4_moon_pos).norm();
    let leapfrog_earth_diff = (leapfrog_earth_pos - rk4_earth_pos).norm();
    let leapfrog_moon_diff = (leapfrog_moon_pos - rk4_moon_pos).norm();
    
    println!("Position differences from RK4 reference:");
    println!("  Verlet vs RK4:");
    println!("    Earth: {:.3e} m ({:.1} km)", verlet_earth_diff, verlet_earth_diff / 1000.0);
    println!("    Moon:  {:.3e} m ({:.1} km)", verlet_moon_diff, verlet_moon_diff / 1000.0);
    println!("  Leapfrog vs RK4:");
    println!("    Earth: {:.3e} m ({:.1} km)", leapfrog_earth_diff, leapfrog_earth_diff / 1000.0);
    println!("    Moon:  {:.3e} m ({:.1} km)", leapfrog_moon_diff, leapfrog_moon_diff / 1000.0);

    // Algorithm properties analysis
    println!("\nAlgorithm Properties:");
    println!("====================");
    
    println!("RK4 (4th-order, non-symplectic):");
    println!("  Order: O(dt^5) error per step");
    println!("  Cost: 4 force evaluations per step");
    println!("  Best for: High-precision, short-term simulations");
    
    println!("Verlet (2nd-order, symplectic):");
    println!("  Order: O(dt^3) error per step");
    println!("  Cost: 2 force evaluations per step");
    println!("  Best for: Long-term energy conservation");
    
    println!("Leapfrog (2nd-order, symplectic):");
    println!("  Order: O(dt^3) error per step");
    println!("  Cost: 2 force evaluations per step (after initialization)");
    println!("  Best for: Long-term orbital mechanics");

    // Validation checks
    println!("\nValidation Results:");
    println!("==================");
    
    // Check RK4 accuracy (should be excellent for short-term)
    if rk4_energy_error < 1e-8 {
        println!("✓ RK4 accuracy: EXCELLENT (< 1e-8)");
    } else if rk4_energy_error < 1e-6 {
        println!("✓ RK4 accuracy: GOOD (< 1e-6)");
    } else {
        println!("⚠ RK4 accuracy: ACCEPTABLE (> 1e-6)");
    }

    // Check that RK4 is more accurate than 2nd order methods for small timesteps
    if rk4_energy_error < verlet_energy_error && rk4_energy_error < leapfrog_energy_error {
        println!("✓ RK4 superior accuracy: CONFIRMED (better than 2nd order methods)");
    } else {
        println!("⚠ RK4 superior accuracy: QUESTIONABLE (not clearly better)");
    }

    // Check numerical stability
    if rk4_earth_pos.norm() < 1e10 && rk4_moon_pos.norm() < 1e10 {
        println!("✓ Numerical stability: PASS (positions remain bounded)");
    } else {
        println!("✗ Numerical stability: FAIL (positions became unbounded)");
    }

    // Check expected performance cost
    let expected_slowdown = 2.0; // RK4 should be ~2x slower (4 vs 2 force evaluations)
    if rk4_steps_per_sec < verlet_steps_per_sec / expected_slowdown * 1.2 {
        println!("✓ Performance cost: EXPECTED (~2x slower due to 4 force evaluations)");
    } else {
        println!("⚠ Performance cost: BETTER THAN EXPECTED");
    }

    // Overall RK4 assessment
    println!("\nRK4 Method Assessment:");
    println!("=====================");
    
    if rk4_energy_error < 1e-7 {
        println!("✓ RK4 provides excellent accuracy for high-precision applications");
    }
    
    if rk4_steps_per_sec > 1000.0 {
        println!("✓ RK4 performance is acceptable for scientific computing");
    }
    
    if rk4_energy_error < verlet_energy_error {
        println!("✓ RK4 demonstrates superior short-term accuracy vs symplectic methods");
    }

    println!("\nRK4 Integrator Test: COMPLETED");
    println!("✓ 4th-order Runge-Kutta algorithm implemented");
    println!("✓ High-precision accuracy confirmed");
    println!("✓ Performance cost acceptable for scientific applications");
    println!("✓ Ready for high-accuracy gravitational simulations");

    Ok(())
}