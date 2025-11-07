//! Binary Orbit Simulation Example
//!
//! Demonstrates a high-precision binary star system simulation with analytical
//! validation against Kepler's laws and energy conservation.

use gravwell::prelude::*;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌟 Gravwell Binary Orbit Simulation");
    println!("===================================");

    // Binary system parameters (similar to Alpha Centauri A & B)
    let mass_a = 1.1 * SOLAR_MASS; // Primary star mass
    let mass_b = 0.907 * SOLAR_MASS; // Secondary star mass
    let separation = 23.4 * AU; // Semi-major axis
    let eccentricity = 0.519; // Orbital eccentricity
    let orbital_period = 79.91 * YEAR_IN_SECONDS; // Orbital period

    println!("Binary System Parameters:");
    println!("  Primary mass: {:.2} M☉", mass_a / SOLAR_MASS);
    println!("  Secondary mass: {:.2} M☉", mass_b / SOLAR_MASS);
    println!("  Semi-major axis: {:.1} AU", separation / AU);
    println!("  Eccentricity: {:.3}", eccentricity);
    println!("  Period: {:.1} years", orbital_period / YEAR_IN_SECONDS);
    println!();

    // Calculate center of mass and reduced mass
    let total_mass = mass_a + mass_b;
    let reduced_mass = (mass_a * mass_b) / total_mass;
    let r_a = mass_b * separation / total_mass; // Distance of A from barycenter
    let r_b = mass_a * separation / total_mass; // Distance of B from barycenter

    println!("Calculated Properties:");
    println!("  Total mass: {:.2} M☉", total_mass / SOLAR_MASS);
    println!("  Reduced mass: {:.2} M☉", reduced_mass / SOLAR_MASS);
    println!("  Distance A from barycenter: {:.1} AU", r_a / AU);
    println!("  Distance B from barycenter: {:.1} AU", r_b / AU);
    println!();

    // Create high-precision simulation
    let mut sim = Simulation::builder()
        .integrator(
            IAS15::new() // Adaptive 15th order integrator
                .tolerance(1e-12) // Very high precision
                .min_timestep(0.01 * DAYS_TO_SECONDS)
                .max_timestep(0.1 * DAYS_TO_SECONDS),
        )
        .forces(DirectGravity::new()) // Exact two-body calculation
        .gravity_constant(GRAVITATIONAL_CONSTANT)
        .energy_conservation_check(true) // Enable energy monitoring
        .build()?;

    // Calculate initial conditions for circular orbit at periapsis
    let periapsis = separation * (1.0 - eccentricity);
    let apoapsis = separation * (1.0 + eccentricity);

    // Orbital velocity at periapsis (using vis-viva equation)
    let v_periapsis =
        (GRAVITATIONAL_CONSTANT * total_mass * (2.0 / periapsis - 1.0 / separation)).sqrt();

    println!("Orbital Mechanics:");
    println!("  Periapsis: {:.2} AU", periapsis / AU);
    println!("  Apoapsis: {:.2} AU", apoapsis / AU);
    println!("  Velocity at periapsis: {:.1} km/s", v_periapsis / 1000.0);
    println!();

    // Place stars at periapsis with center of mass at origin
    let star_a = sim.add_body(
        Body::new()
            .name("Alpha Centauri A")
            .mass(mass_a)
            .position([-r_a * (1.0 - eccentricity), 0.0, 0.0])
            .velocity([0.0, -mass_b * v_periapsis / total_mass, 0.0])
            .radius(1.22 * SOLAR_RADIUS)
            .color([1.0, 1.0, 0.8]), // Slightly yellowish
    )?;

    let star_b = sim.add_body(
        Body::new()
            .name("Alpha Centauri B")
            .mass(mass_b)
            .position([r_b * (1.0 - eccentricity), 0.0, 0.0])
            .velocity([0.0, mass_a * v_periapsis / total_mass, 0.0])
            .radius(0.86 * SOLAR_RADIUS)
            .color([1.0, 0.8, 0.6]), // Orange dwarf
    )?;

    println!("Initial Conditions:");
    println!(
        "  Star A position: [{:.2}, {:.2}, {:.2}] AU",
        sim.position(star_a)?[0] / AU,
        sim.position(star_a)?[1] / AU,
        sim.position(star_a)?[2] / AU
    );
    println!(
        "  Star A velocity: [{:.1}, {:.1}, {:.1}] km/s",
        sim.velocity(star_a)?[0] / 1000.0,
        sim.velocity(star_a)?[1] / 1000.0,
        sim.velocity(star_a)?[2] / 1000.0
    );
    println!();

    // Calculate theoretical values
    let initial_energy = sim.total_energy();
    let angular_momentum = sim.angular_momentum()?;
    let am_magnitude = angular_momentum.norm();

    // Theoretical orbital energy (negative for bound orbit)
    let theoretical_energy = -GRAVITATIONAL_CONSTANT * mass_a * mass_b / (2.0 * separation);

    println!("Energy Analysis:");
    println!("  Initial total energy: {:.6e} J", initial_energy);
    println!("  Theoretical energy: {:.6e} J", theoretical_energy);
    println!(
        "  Energy error: {:.3e}",
        (initial_energy - theoretical_energy).abs() / theoretical_energy.abs()
    );
    println!("  Angular momentum: {:.6e} kg⋅m²⋅s⁻¹", am_magnitude);
    println!();

    // Simulation parameters
    let simulation_time = 3.0 * orbital_period; // 3 complete orbits
    let output_interval = orbital_period / 100.0; // 100 outputs per orbit

    println!(
        "Running simulation for {:.1} orbital periods ({:.1} years)...",
        simulation_time / orbital_period,
        simulation_time / YEAR_IN_SECONDS
    );

    // Tracking variables
    let mut max_separation = 0.0;
    let mut min_separation = f64::INFINITY;
    let mut max_velocity = 0.0;
    let mut min_velocity = f64::INFINITY;
    let mut orbit_count = 0;
    let mut last_angle = 0.0;
    let mut total_angle = 0.0;

    let start_time = Instant::now();
    let mut next_output = output_interval;

    // Main simulation loop with detailed orbital analysis
    while sim.time() < simulation_time {
        sim.adaptive_step()?; // Use adaptive timestep

        // Output at regular intervals
        if sim.time() >= next_output {
            let pos_a = sim.position(star_a)?;
            let pos_b = sim.position(star_b)?;
            let vel_a = sim.velocity(star_a)?;
            let vel_b = sim.velocity(star_b)?;

            // Calculate relative motion
            let relative_pos = [
                pos_a[0] - pos_b[0],
                pos_a[1] - pos_b[1],
                pos_a[2] - pos_b[2],
            ];
            let relative_vel = [
                vel_a[0] - vel_b[0],
                vel_a[1] - vel_b[1],
                vel_a[2] - vel_b[2],
            ];

            let separation =
                (relative_pos[0].powi(2) + relative_pos[1].powi(2) + relative_pos[2].powi(2))
                    .sqrt();
            let velocity =
                (relative_vel[0].powi(2) + relative_vel[1].powi(2) + relative_vel[2].powi(2))
                    .sqrt();

            // Track extrema
            max_separation = max_separation.max(separation);
            min_separation = min_separation.min(separation);
            max_velocity = max_velocity.max(velocity);
            min_velocity = min_velocity.min(velocity);

            // Calculate orbital angle
            let angle = relative_pos[1].atan2(relative_pos[0]);

            // Detect completed orbits
            if angle < -2.0 && last_angle > 2.0 {
                orbit_count += 1;
                total_angle += 2.0 * std::f64::consts::PI;

                let measured_period = sim.time() / orbit_count as f64;
                let period_error =
                    (measured_period - orbital_period).abs() / orbital_period * 100.0;

                println!(
                    "🔄 Orbit {} completed at t = {:.2} years",
                    orbit_count,
                    sim.time() / YEAR_IN_SECONDS
                );
                println!(
                    "   Period: {:.2} years (error: {:.3}%)",
                    measured_period / YEAR_IN_SECONDS,
                    period_error
                );
            }

            last_angle = angle;

            // Physics validation every 10 outputs
            if (next_output / output_interval) as i32 % 10 == 0 {
                let current_energy = sim.total_energy();
                let energy_drift = (current_energy - initial_energy).abs() / initial_energy.abs();
                let current_am = sim.angular_momentum()?.norm();
                let am_drift = (current_am - am_magnitude).abs() / am_magnitude.abs();

                println!(
                    "t = {:.1} yr: r = {:.2} AU, v = {:.1} km/s, ΔE = {:.2e}, ΔL = {:.2e}",
                    sim.time() / YEAR_IN_SECONDS,
                    separation / AU,
                    velocity / 1000.0,
                    energy_drift,
                    am_drift
                );

                if energy_drift > 1e-9 {
                    println!("⚠️  Warning: Energy drift exceeds tolerance!");
                }
            }

            next_output += output_interval;
        }
    }

    // Final analysis
    let elapsed = start_time.elapsed();
    println!("\n🎉 Binary Orbit Simulation Completed!");
    println!("=====================================");

    // Orbital mechanics validation
    let theoretical_max = apoapsis;
    let theoretical_min = periapsis;
    let separation_error_max = (max_separation - theoretical_max).abs() / theoretical_max * 100.0;
    let separation_error_min = (min_separation - theoretical_min).abs() / theoretical_min * 100.0;

    println!("Orbital Analysis:");
    println!("  Completed orbits: {}", orbit_count);
    println!(
        "  Maximum separation: {:.3} AU (theoretical: {:.3} AU, error: {:.2}%)",
        max_separation / AU,
        theoretical_max / AU,
        separation_error_max
    );
    println!(
        "  Minimum separation: {:.3} AU (theoretical: {:.3} AU, error: {:.2}%)",
        min_separation / AU,
        theoretical_min / AU,
        separation_error_min
    );
    println!("  Maximum velocity: {:.1} km/s", max_velocity / 1000.0);
    println!("  Minimum velocity: {:.1} km/s", min_velocity / 1000.0);

    // Energy and momentum conservation
    let final_energy = sim.total_energy();
    let final_am = sim.angular_momentum()?.norm();
    let energy_conservation = (final_energy - initial_energy).abs() / initial_energy.abs();
    let momentum_conservation = (final_am - am_magnitude).abs() / am_magnitude.abs();

    println!("\nConservation Laws:");
    println!(
        "  Energy conservation: {:.2e} (should be < 1e-12)",
        energy_conservation
    );
    println!(
        "  Angular momentum conservation: {:.2e} (should be < 1e-12)",
        momentum_conservation
    );

    if energy_conservation < 1e-12 && momentum_conservation < 1e-12 {
        println!("  ✅ Conservation laws satisfied!");
    } else {
        println!("  ❌ Conservation violations detected!");
    }

    // Performance metrics
    let total_steps = sim.step_count();
    let steps_per_second = total_steps as f64 / elapsed.as_secs_f64();

    println!("\nPerformance Metrics:");
    println!("  Wall clock time: {:.2} seconds", elapsed.as_secs_f64());
    println!("  Total integration steps: {}", total_steps);
    println!(
        "  Average timestep: {:.2} hours",
        (sim.time() / total_steps as f64) / 3600.0
    );
    println!("  Performance: {:.1} steps/second", steps_per_second);

    // Kepler's Third Law validation
    if orbit_count > 0 {
        let measured_period = sim.time() / orbit_count as f64;
        let kepler_period = 2.0
            * std::f64::consts::PI
            * (separation.powi(3) / (GRAVITATIONAL_CONSTANT * total_mass)).sqrt();
        let kepler_error = (measured_period - kepler_period).abs() / kepler_period * 100.0;

        println!("\nKepler's Third Law Validation:");
        println!(
            "  Measured period: {:.2} years",
            measured_period / YEAR_IN_SECONDS
        );
        println!(
            "  Kepler's law period: {:.2} years",
            kepler_period / YEAR_IN_SECONDS
        );
        println!("  Error: {:.3}% (should be < 0.1%)", kepler_error);

        if kepler_error < 0.1 {
            println!("  ✅ Kepler's Third Law validated!");
        } else {
            println!("  ❌ Kepler's Third Law violation detected!");
        }
    }

    println!("\n🌟 Binary system simulation demonstrates Gravwell's precision!");

    Ok(())
}

// Constants
const GRAVITATIONAL_CONSTANT: f64 = 6.67430e-11;
const AU: f64 = 1.496e11;
const SOLAR_MASS: f64 = 1.989e30;
const SOLAR_RADIUS: f64 = 6.96e8;
const YEAR_IN_SECONDS: f64 = 365.25 * 24.0 * 3600.0;
const DAYS_TO_SECONDS: f64 = 24.0 * 3600.0;

// Placeholder traits and types (these would be defined in the actual Gravwell crate)
trait BodyExt {
    fn name(self, name: &str) -> Self;
    fn mass(self, mass: f64) -> Self;
    fn position(self, pos: [f64; 3]) -> Self;
    fn velocity(self, vel: [f64; 3]) -> Self;
    fn radius(self, radius: f64) -> Self;
    fn color(self, color: [f32; 3]) -> Self;
}

struct Body {
    // Implementation details...
}

impl Body {
    fn new() -> Self {
        Self {}
    }
}

impl BodyExt for Body {
    fn name(self, _name: &str) -> Self {
        self
    }
    fn mass(self, _mass: f64) -> Self {
        self
    }
    fn position(self, _pos: [f64; 3]) -> Self {
        self
    }
    fn velocity(self, _vel: [f64; 3]) -> Self {
        self
    }
    fn radius(self, _radius: f64) -> Self {
        self
    }
    fn color(self, _color: [f32; 3]) -> Self {
        self
    }
}
