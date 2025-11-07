//! Simple working example for Gravwell library.
//!
//! This demonstrates a basic two-body gravitational simulation.

use gravwell::prelude::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create a simple two-body system (Earth and Moon)
    let mut simulation = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(DirectGravity::new())
        .add_body(
            Body::new()
                .with_mass(5.972e24) // Earth mass in kg
                .with_position([0.0, 0.0, 0.0])
                .with_velocity([0.0, 0.0, 0.0]),
        )?
        .add_body(
            Body::new()
                .with_mass(7.342e22) // Moon mass in kg
                .with_position([384400000.0, 0.0, 0.0]) // Moon distance in m
                .with_velocity([0.0, 1022.0, 0.0]),
        )? // Moon orbital velocity in m/s
        .build()?;

    println!("🌍 Starting Earth-Moon simulation...");
    println!("Integrator: {}", simulation.integrator_name());
    println!("Force Calculator: {}", simulation.force_calculator_name());

    // Initial state
    let particles = simulation.particles();
    println!("\nInitial state:");
    println!(
        "  Earth position: [{:.2e}, {:.2e}, {:.2e}]",
        particles.position(0).x,
        particles.position(0).y,
        particles.position(0).z
    );
    println!(
        "  Moon position: [{:.2e}, {:.2e}, {:.2e}]",
        particles.position(1).x,
        particles.position(1).y,
        particles.position(1).z
    );

    // Run simulation for 1 day (24 hours)
    let timestep = 3600.0; // 1 hour in seconds
    for hour in 1..=24 {
        simulation.step(timestep)?;

        if hour % 6 == 0 {
            let particles = simulation.particles();
            println!("\nAfter {} hours:", hour);
            println!(
                "  Moon position: [{:.2e}, {:.2e}, {:.2e}]",
                particles.position(1).x,
                particles.position(1).y,
                particles.position(1).z
            );

            let distance = (particles.position(1) - particles.position(0)).magnitude();
            println!("  Earth-Moon distance: {:.2e} m", distance);
        }
    }

    println!("\n✅ Simulation completed successfully!");
    Ok(())
}
