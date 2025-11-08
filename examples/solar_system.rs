//! Solar System Simulation Example
//!
//! Demonstrates setting up and running a realistic solar system simulation
//! with the major planets using Gravwell's physics engine.

use gravwell::prelude::*;
use std::time::Instant;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🌌 Gravwell Solar System Simulation");
    println!("=====================================");

    // Create simulation with realistic physics
    let mut sim = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new()) // Symplectic integrator
        .with_force_calculator(BarnesHut::new()) // O(N log N) force calculation
        .build()?;

    // Add the Sun at the center
    let _sun = sim.add_body(
        Body::new()
            .with_mass(SOLAR_MASS)
            .with_position([0.0, 0.0, 0.0])
            .with_velocity([0.0, 0.0, 0.0])
            .with_radius(6.96e8), // Solar radius in meters
    )?;

    // Add Mercury
    let mercury = sim.add_body(
        Body::new()
            .with_mass(3.301e23)
            .with_position([0.387 * AU, 0.0, 0.0])
            .with_velocity([0.0, 47360.0, 0.0]) // Orbital velocity
            .with_radius(2.44e6),
    )?;

    // Add Venus
    let venus = sim.add_body(
        Body::new()
            .with_mass(4.867e24)
            .with_position([0.723 * AU, 0.0, 0.0])
            .with_velocity([0.0, 35020.0, 0.0])
            .with_radius(6.05e6),
    )?;

    // Add Earth
    let earth = sim.add_body(
        Body::new()
            .with_mass(EARTH_MASS)
            .with_position([1.0 * AU, 0.0, 0.0])
            .with_velocity([0.0, 29780.0, 0.0])
            .with_radius(6.371e6),
    )?;

    // Add Mars
    let mars = sim.add_body(
        Body::new()
            .with_mass(6.417e23)
            .with_position([1.524 * AU, 0.0, 0.0])
            .with_velocity([0.0, 24070.0, 0.0])
            .with_radius(3.39e6),
    )?;

    // Add Jupiter
    let jupiter = sim.add_body(
        Body::new()
            .with_mass(1.898e27)
            .with_position([5.204 * AU, 0.0, 0.0])
            .with_velocity([0.0, 13060.0, 0.0])
            .with_radius(6.99e7),
    )?;

    // Add Saturn
    let saturn = sim.add_body(
        Body::new()
            .with_mass(5.683e26)
            .with_position([9.537 * AU, 0.0, 0.0])
            .with_velocity([0.0, 9680.0, 0.0])
            .with_radius(5.82e7),
    )?;

    // Add Uranus
    let uranus = sim.add_body(
        Body::new()
            .with_mass(8.681e25)
            .with_position([19.191 * AU, 0.0, 0.0])
            .with_velocity([0.0, 6810.0, 0.0])
            .with_radius(2.54e7),
    )?;

    // Add Neptune
    let neptune = sim.add_body(
        Body::new()
            .with_mass(1.024e26)
            .with_position([30.069 * AU, 0.0, 0.0])
            .with_velocity([0.0, 5430.0, 0.0])
            .with_radius(2.46e7),
    )?;

    println!(
        "Added {} celestial bodies to the simulation",
        sim.particles().len()
    );
    println!("Initial total energy: {:.3e} J", sim.total_energy());
    println!();

    // Simulation parameters
    let timestep = 0.1 * DAYS_TO_SECONDS; // 0.1 day timestep
    let total_simulation_time = 10.0 * YEAR_IN_SECONDS; // 10 years
    let output_interval = 30.0 * DAYS_TO_SECONDS; // Output every 30 days
    let steps_per_output = (output_interval / timestep) as usize;

    println!(
        "Running simulation for {} years...",
        total_simulation_time / YEAR_IN_SECONDS
    );
    println!(
        "Output interval: {} days",
        output_interval / DAYS_TO_SECONDS
    );
    println!("Timestep: {:.1} hours", timestep / 3600.0);
    println!();

    let initial_energy = sim.total_energy();
    let start_time = Instant::now();
    let mut output_count = 0;
    let mut simulation_time = 0.0;

    // Main simulation loop
    while simulation_time < total_simulation_time {
        // Run simulation steps
        for _ in 0..steps_per_output {
            sim.step(timestep)?;
            simulation_time += timestep;
        }

        output_count += 1;
        let current_time_years = simulation_time / YEAR_IN_SECONDS;
        let current_energy = sim.total_energy();
        let energy_drift = (current_energy - initial_energy).abs() / initial_energy.abs();

        // Print status every 30 simulation days
        println!(
            "Year {:.2}: Energy drift = {:.3e}, Performance = {:.1} steps/sec",
            current_time_years,
            energy_drift,
            (output_count * steps_per_output) as f64 / start_time.elapsed().as_secs_f64()
        );

        // Print planetary positions (in AU)
        if output_count % 4 == 0 {
            // Every 4 months
            println!("  Planetary positions (AU from Sun):");
            let planet_handles = [
                mercury, venus, earth, mars, jupiter, saturn, uranus, neptune,
            ];
            let planet_names = [
                "Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune",
            ];

            for (&planet, name) in planet_handles.iter().zip(planet_names.iter()) {
                let pos = sim.position(planet);
                let distance_au = pos.norm() / AU;
                println!("    {:<8}: {:.3} AU", name, distance_au);
            }
            println!();
        }

        // Validate physics
        if energy_drift > 1e-6 {
            println!(
                "⚠️  Warning: Large energy drift detected: {:.3e}",
                energy_drift
            );
        }
    }

    // Final statistics
    let elapsed = start_time.elapsed();
    let total_steps = (simulation_time / timestep) as usize;
    let steps_per_second = total_steps as f64 / elapsed.as_secs_f64();

    println!("🎉 Simulation completed!");
    println!("========================");
    println!(
        "Total simulation time: {:.1} years",
        simulation_time / YEAR_IN_SECONDS
    );
    println!("Total physics steps: {}", total_steps);
    println!("Wall clock time: {:.2} seconds", elapsed.as_secs_f64());
    println!("Performance: {:.1} steps/second", steps_per_second);
    println!(
        "Final energy drift: {:.3e}",
        (sim.total_energy() - initial_energy).abs() / initial_energy.abs()
    );

    // Calculate final orbital periods (approximate)
    println!("\nApproximate orbital periods:");
    let planetary_data = [
        (mercury, "Mercury", 0.24),
        (venus, "Venus", 0.62),
        (earth, "Earth", 1.00),
        (mars, "Mars", 1.88),
        (jupiter, "Jupiter", 11.86),
        (saturn, "Saturn", 29.46),
        (uranus, "Uranus", 84.01),
        (neptune, "Neptune", 164.8),
    ];

    for (handle, name, expected_period) in planetary_data {
        let pos = sim.position(handle);
        let _vel = sim.velocity(handle);
        let distance = pos.norm();

        // Calculate orbital period using Kepler's third law
        let calculated_period = 2.0
            * std::f64::consts::PI
            * (distance.powi(3) / (GRAVITATIONAL_CONSTANT * SOLAR_MASS)).sqrt()
            / YEAR_IN_SECONDS;

        let error = (calculated_period - expected_period).abs() / expected_period * 100.0;

        println!(
            "  {:<8}: {:.2} years (expected: {:.2}, error: {:.1}%)",
            name, calculated_period, expected_period, error
        );
    }

    Ok(())
}

// Physical constants
const GRAVITATIONAL_CONSTANT: f64 = 6.67430e-11; // m³ kg⁻¹ s⁻²
const AU: f64 = 1.496e11; // Astronomical Unit (meters)
const SOLAR_MASS: f64 = 1.989e30; // kg
const EARTH_MASS: f64 = 5.972e24; // kg
const YEAR_IN_SECONDS: f64 = 365.25 * 24.0 * 3600.0;
const DAYS_TO_SECONDS: f64 = 24.0 * 3600.0;
