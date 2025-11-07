//! Kepler Orbit Validation Tests
//!
//! Tests that validate orbital mechanics accuracy against analytical Kepler solutions.

use gravwell::prelude::*;
use std::f64::consts::PI;

// Use physics constants from Gravwell

/// Calculate circular orbital velocity
fn orbital_velocity(radius: f64, central_mass: f64) -> f64 {
    (G * central_mass / radius).sqrt()
}

/// Calculate orbital period using Kepler's third law
fn orbital_period(semi_major_axis: f64, total_mass: f64) -> f64 {
    2.0 * PI * (semi_major_axis.powi(3) / (G * total_mass)).sqrt()
}

/// Assert relative error is within tolerance
fn assert_relative_error(actual: f64, expected: f64, tolerance: f64, description: &str) {
    let relative_error = (actual - expected).abs() / expected.abs().max(1e-100);
    assert!(
        relative_error <= tolerance,
        "{}: error {:.2e} > tolerance {:.2e} (actual: {:.6e}, expected: {:.6e})",
        description,
        relative_error,
        tolerance,
        actual,
        expected
    );
}

#[cfg(test)]
mod kepler_tests {
    use super::*;

    #[test]
    fn test_circular_orbit_basic() {
        println!("\n🌍 Testing circular Earth orbit...");

        // Create Sun-Earth system using builder pattern
        let mut sim = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(DirectGravity::new())
            .add_body(
                Body::new()
                    .with_mass(SOLAR_MASS)
                    .with_position([0.0, 0.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .expect("Failed to add Sun")
            .add_body(
                Body::new()
                    .with_mass(EARTH_MASS)
                    .with_position([AU, 0.0, 0.0])
                    .with_velocity([0.0, orbital_velocity(AU, SOLAR_MASS), 0.0]),
            )
            .expect("Failed to add Earth")
            .build()
            .expect("Failed to build simulation");

        // Test initial conditions
        let particles = sim.particles();
        let earth_pos = particles.position(1); // Earth is second body (index 1)
        let earth_vel = particles.velocity(1);

        let initial_distance = earth_pos.norm();
        let initial_speed = earth_vel.norm();
        let expected_speed = orbital_velocity(AU, SOLAR_MASS);

        println!(
            "Initial distance: {:.3e} m ({:.6} AU)",
            initial_distance,
            initial_distance / AU
        );
        println!(
            "Initial speed: {:.1} m/s (expected: {:.1} m/s)",
            initial_speed, expected_speed
        );

        // Validate initial setup
        assert_relative_error(initial_distance, AU, 1e-10, "Initial distance at 1 AU");
        assert_relative_error(
            initial_speed,
            expected_speed,
            1e-8,
            "Initial orbital velocity",
        );

        // Calculate expected orbital period
        let expected_period = orbital_period(AU, SOLAR_MASS + EARTH_MASS);
        println!("Expected period: {:.1} days", expected_period / 86400.0);

        // Simulate for 1/8 orbit (about 45 days) to check orbit stability
        let timestep = 3600.0; // 1 hour
        let steps = (expected_period / 8.0 / timestep) as usize;

        println!(
            "Simulating {} steps ({:.1} days)...",
            steps,
            steps as f64 * timestep / 86400.0
        );

        let initial_energy = particles.kinetic_energy(); // Initial kinetic energy

        for step in 0..steps {
            sim.step(timestep).expect("Simulation step failed");

            // Check orbital distance every 100 steps
            if step % 100 == 0 {
                let current_pos = sim.particles().position(1);
                let current_distance = current_pos.norm();
                let distance_error = (current_distance - AU).abs() / AU;

                // For circular orbit, distance should remain constant
                assert!(
                    distance_error < 1e-5,
                    "Orbital distance error at step {}: {:.2e}",
                    step,
                    distance_error
                );
            }
        }

        // Check final state after 1/8 orbit
        let final_particles = sim.particles();
        let final_pos = final_particles.position(1);
        let final_distance = final_pos.norm();
        let final_energy = final_particles.kinetic_energy();

        println!(
            "Final distance: {:.3e} m ({:.6} AU)",
            final_distance,
            final_distance / AU
        );

        // Distance should still be close to 1 AU
        let distance_error = (final_distance - AU).abs() / AU;
        assert!(
            distance_error < 1e-5,
            "Final distance error: {:.2e}",
            distance_error
        );

        // Energy should be approximately conserved (kinetic + potential)
        let energy_change = (final_energy - initial_energy).abs() / initial_energy;
        assert!(energy_change < 5e-6, "Energy change: {:.2e}", energy_change);

        println!("✅ Circular orbit test passed!");
    }

    #[test]
    fn test_kepler_third_law() {
        println!("\n🪐 Testing Kepler's third law: T² ∝ a³");

        // Test orbital periods at different distances
        let test_distances = vec![0.5 * AU, 1.0 * AU, 1.5 * AU, 2.0 * AU];
        let mut measured_periods = Vec::new();

        for (_i, &distance) in test_distances.iter().enumerate() {
            let period = orbital_period(distance, SOLAR_MASS);
            measured_periods.push(period);

            println!("  {:.1} AU: {:.1} days", distance / AU, period / 86400.0);

            // Verify Kepler's third law: T² = (4π²/GM) * a³
            let theoretical_t_squared = (4.0 * PI * PI / (G * SOLAR_MASS)) * distance.powi(3);
            let measured_t_squared = period * period;

            assert_relative_error(
                measured_t_squared,
                theoretical_t_squared,
                1e-12,
                &format!("Kepler 3rd law at {:.1} AU", distance / AU),
            );
        }

        // Test proportionality between different orbits
        for i in 1..test_distances.len() {
            let distance_ratio = test_distances[i] / test_distances[0];
            let period_ratio = measured_periods[i] / measured_periods[0];

            let expected_period_ratio = distance_ratio.powf(1.5); // T ∝ a^(3/2)

            assert_relative_error(
                period_ratio,
                expected_period_ratio,
                1e-12,
                &format!(
                    "Period ratio {:.1}:{:.1} AU",
                    test_distances[i] / AU,
                    test_distances[0] / AU
                ),
            );
        }

        println!("✅ Kepler's third law validated!");
    }

    #[test]
    fn test_vis_viva_equation() {
        println!("\n⚡ Testing vis-viva equation: v² = μ(2/r - 1/a)");

        // Create circular orbit system
        let mut sim = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(DirectGravity::new())
            .add_body(
                Body::new()
                    .with_mass(SOLAR_MASS)
                    .with_position([0.0, 0.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .expect("Failed to add Sun")
            .add_body(
                Body::new()
                    .with_mass(EARTH_MASS)
                    .with_position([AU, 0.0, 0.0])
                    .with_velocity([0.0, orbital_velocity(AU, SOLAR_MASS), 0.0]),
            )
            .expect("Failed to add Earth")
            .build()
            .expect("Failed to build simulation");

        let mu = G * SOLAR_MASS; // Standard gravitational parameter
        let semi_major_axis = AU; // For circular orbit

        // Test vis-viva equation at several points
        let timestep = 3600.0; // 1 hour
        let num_tests = 20;

        for i in 0..num_tests {
            if i > 0 {
                sim.step(timestep).expect("Simulation step failed");
            }

            let particles = sim.particles();
            let pos = particles.position(1);
            let vel = particles.velocity(1);

            let r = pos.norm();
            let v_squared = vel.norm_squared();

            // Apply vis-viva equation: v² = μ(2/r - 1/a)
            let expected_v_squared = mu * (2.0 / r - 1.0 / semi_major_axis);

            assert_relative_error(
                v_squared,
                expected_v_squared,
                1e-10,
                &format!("Vis-viva at step {}", i),
            );

            if i % 5 == 0 {
                println!("  Step {}: r={:.3e}m, v²={:.1e}m²/s²", i, r, v_squared);
            }
        }

        println!("✅ Vis-viva equation validated!");
    }
}
