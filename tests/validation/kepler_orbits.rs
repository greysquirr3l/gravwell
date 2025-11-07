// Kepler Orbital Mechanics Validation Tests
//
// These tests validate Gravwell's accuracy against analytical solutions
// for two-body orbital mechanics problems.

use super::*;
use gravwell::prelude::*;
use std::f64::consts::PI;

/// Orbital elements structure for orbit characterization
#[derive(Debug, Clone)]
pub struct OrbitalElements {
    pub semi_major_axis: f64,             // a (meters)
    pub eccentricity: f64,                // e (dimensionless)
    pub inclination: f64,                 // i (radians)
    pub longitude_of_ascending_node: f64, // Ω (radians)
    pub argument_of_periapsis: f64,       // ω (radians)
    pub true_anomaly: f64,                // ν (radians)
    pub orbital_period: f64,              // T (seconds)
    pub specific_energy: f64,             // ε (J/kg)
}

impl OrbitalElements {
    /// Calculate orbital elements from position and velocity vectors
    pub fn from_state_vectors(
        position: Vector3,
        velocity: Vector3,
        mu: f64, // Standard gravitational parameter (G * M)
    ) -> Self {
        let r = position.norm();
        let v_squared = velocity.norm_squared();

        // Specific orbital energy
        let specific_energy = v_squared / 2.0 - mu / r;

        // Semi-major axis (negative energy for bound orbits)
        let semi_major_axis = -mu / (2.0 * specific_energy);

        // Specific angular momentum vector
        let h_vec = position.cross(&velocity);
        let h = h_vec.norm();

        // Eccentricity
        let eccentricity = if h > 0.0 {
            ((1.0 + 2.0 * specific_energy * h * h / (mu * mu)).max(0.0)).sqrt()
        } else {
            0.0
        };

        // Inclination (angle between orbital plane and reference plane)
        let inclination = if h > 0.0 { (h_vec.z / h).acos() } else { 0.0 };

        // Node vector (intersection of orbital and reference planes)
        let n_vec = Vector3::new(-h_vec.y, h_vec.x, 0.0);
        let n = n_vec.norm();

        // Longitude of ascending node
        let longitude_of_ascending_node = if n > 1e-10 {
            let lon = (n_vec.x / n).acos();
            if n_vec.y >= 0.0 {
                lon
            } else {
                2.0 * PI - lon
            }
        } else {
            0.0
        };

        // Eccentricity vector
        let e_vec = if mu > 0.0 {
            (velocity.cross(&h_vec) / mu) - (position / r)
        } else {
            Vector3::zeros()
        };

        // Argument of periapsis
        let argument_of_periapsis = if n > 1e-10 && eccentricity > 1e-10 {
            let arg = (n_vec.dot(&e_vec) / (n * eccentricity))
                .clamp(-1.0, 1.0)
                .acos();
            if e_vec.z >= 0.0 {
                arg
            } else {
                2.0 * PI - arg
            }
        } else if eccentricity > 1e-10 {
            let arg = (e_vec.x / eccentricity).clamp(-1.0, 1.0).acos();
            if e_vec.y >= 0.0 {
                arg
            } else {
                2.0 * PI - arg
            }
        } else {
            0.0
        };

        // True anomaly
        let true_anomaly = if eccentricity > 1e-10 {
            let cos_nu = (e_vec.dot(&position) / (eccentricity * r)).clamp(-1.0, 1.0);
            let nu = cos_nu.acos();
            if position.dot(&velocity) >= 0.0 {
                nu
            } else {
                2.0 * PI - nu
            }
        } else {
            // For circular orbits, use longitude from ascending node
            if n > 1e-10 {
                let cos_u = (n_vec.dot(&position) / (n * r)).clamp(-1.0, 1.0);
                let u = cos_u.acos();
                if position.z >= 0.0 {
                    u
                } else {
                    2.0 * PI - u
                }
            } else {
                position.y.atan2(position.x)
            }
        };

        // Orbital period (for elliptical orbits)
        let orbital_period = if semi_major_axis > 0.0 && mu > 0.0 {
            2.0 * PI * (semi_major_axis.powi(3) / mu).sqrt()
        } else {
            f64::INFINITY
        };

        Self {
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_of_ascending_node,
            argument_of_periapsis,
            true_anomaly,
            orbital_period,
            specific_energy,
        }
    }
}

/// Test circular orbit stability and accuracy over multiple periods
#[cfg(test)]
mod tests {
    use super::*;
    use approx::{assert_abs_diff_eq, assert_relative_eq};

    #[test]
    fn test_circular_earth_orbit() {
        let mut report = ValidationReport::new();

        // Set up Earth-Sun system
        let (mut sim, _sun, earth) = setup_earth_sun_system();

        // Calculate expected orbital elements
        let mu = constants::G * constants::SOLAR_MASS;
        let expected_period = theoretical_orbital_period(constants::AU, constants::SOLAR_MASS);
        let expected_velocity = theoretical_orbital_velocity(constants::AU, constants::SOLAR_MASS);

        // Initial orbital elements
        let initial_pos = sim.position(earth);
        let initial_vel = sim.velocity(earth);
        let initial_elements = OrbitalElements::from_state_vectors(initial_pos, initial_vel, mu);

        // Validate initial conditions
        report.add_result(ValidationResult::new(
            "Initial Semi-Major Axis",
            initial_elements.semi_major_axis,
            constants::AU,
            constants::KEPLER_ORBIT_TOLERANCE,
        ));

        report.add_result(ValidationResult::new(
            "Initial Eccentricity (Circular)",
            initial_elements.eccentricity,
            0.0,
            1e-6,
        ));

        report.add_result(ValidationResult::new(
            "Initial Orbital Period",
            initial_elements.orbital_period,
            expected_period,
            constants::KEPLER_ORBIT_TOLERANCE,
        ));

        // Simulate for one complete orbit
        let steps_per_orbit = 1000;
        let dt = expected_period / steps_per_orbit as f64;
        sim.set_timestep(dt);

        let initial_energy = sim.total_energy();
        let mut max_position_error = 0.0;
        let mut max_velocity_error = 0.0;

        for step in 0..steps_per_orbit {
            sim.step();

            // Check every 100 steps
            if step % 100 == 0 {
                let current_pos = sim.position(earth);
                let current_vel = sim.velocity(earth);
                let current_elements =
                    OrbitalElements::from_state_vectors(current_pos, current_vel, mu);

                // Position should remain at 1 AU
                let position_error = (current_pos.norm() - constants::AU).abs() / constants::AU;
                max_position_error = max_position_error.max(position_error);

                // Velocity should remain at orbital velocity
                let velocity_error =
                    (current_vel.norm() - expected_velocity).abs() / expected_velocity;
                max_velocity_error = max_velocity_error.max(velocity_error);

                // Semi-major axis should remain constant
                let sma_error =
                    (current_elements.semi_major_axis - constants::AU).abs() / constants::AU;
                if sma_error > constants::KEPLER_ORBIT_TOLERANCE {
                    report.add_result(ValidationResult::new(
                        format!("Semi-Major Axis at Step {}", step),
                        current_elements.semi_major_axis,
                        constants::AU,
                        constants::KEPLER_ORBIT_TOLERANCE,
                    ));
                }

                // Eccentricity should remain near zero
                if current_elements.eccentricity > 1e-6 {
                    report.add_result(ValidationResult::new(
                        format!("Eccentricity at Step {}", step),
                        current_elements.eccentricity,
                        0.0,
                        1e-6,
                    ));
                }
            }
        }

        // Final position should be close to initial position
        let final_pos = sim.position(earth);
        let final_vel = sim.velocity(earth);
        let position_closure_error = (final_pos - initial_pos).norm() / constants::AU;
        let velocity_closure_error = (final_vel - initial_vel).norm() / expected_velocity;

        report.add_result(ValidationResult::new(
            "Orbital Position Closure",
            position_closure_error,
            0.0,
            constants::KEPLER_ORBIT_TOLERANCE,
        ));

        report.add_result(ValidationResult::new(
            "Orbital Velocity Closure",
            velocity_closure_error,
            0.0,
            constants::KEPLER_ORBIT_TOLERANCE,
        ));

        // Energy conservation
        let final_energy = sim.total_energy();
        let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();

        report.add_result(ValidationResult::new(
            "Energy Conservation",
            energy_error,
            0.0,
            constants::ENERGY_CONSERVATION_TOLERANCE,
        ));

        // Maximum errors throughout orbit
        report.add_result(ValidationResult::new(
            "Maximum Position Error",
            max_position_error,
            0.0,
            constants::KEPLER_ORBIT_TOLERANCE,
        ));

        report.add_result(ValidationResult::new(
            "Maximum Velocity Error",
            max_velocity_error,
            0.0,
            constants::KEPLER_ORBIT_TOLERANCE,
        ));

        // Print detailed report
        report.print_full_report();

        // Assert overall success
        assert!(report.overall_passed, "Circular orbit validation failed");
    }

    #[test]
    fn test_elliptical_orbit() {
        let mut report = ValidationReport::new();

        // Create elliptical orbit (e ≈ 0.5)
        let mut sim = Simulation::builder()
            .integrator(VelocityVerlet::new())
            .forces(DirectGravity::new())
            .timestep(3600.0) // 1 hour
            .build()
            .expect("Failed to create simulation");

        // Add Sun
        let _sun = sim
            .add_body(
                Body::new()
                    .mass(constants::SOLAR_MASS)
                    .position([0.0, 0.0, 0.0])
                    .velocity([0.0, 0.0, 0.0]),
            )
            .expect("Failed to add Sun");

        // Add planet in elliptical orbit
        // Start at aphelion (farthest point) with reduced velocity
        let aphelion_distance = 1.5 * constants::AU;
        let aphelion_velocity =
            theoretical_orbital_velocity(constants::AU, constants::SOLAR_MASS) * 0.8;

        let planet = sim
            .add_body(
                Body::new()
                    .mass(constants::EARTH_MASS)
                    .position([aphelion_distance, 0.0, 0.0])
                    .velocity([0.0, aphelion_velocity, 0.0]),
            )
            .expect("Failed to add planet");

        // Calculate expected orbital elements
        let mu = constants::G * constants::SOLAR_MASS;
        let initial_pos = sim.position(planet);
        let initial_vel = sim.velocity(planet);
        let expected_elements = OrbitalElements::from_state_vectors(initial_pos, initial_vel, mu);

        println!("Expected orbital elements:");
        println!(
            "  Semi-major axis: {:.3e} m ({:.3} AU)",
            expected_elements.semi_major_axis,
            expected_elements.semi_major_axis / constants::AU
        );
        println!("  Eccentricity: {:.6}", expected_elements.eccentricity);
        println!(
            "  Orbital period: {:.1} days",
            expected_elements.orbital_period / 86400.0
        );

        // Validate initial eccentricity
        report.add_result(ValidationResult::new(
            "Initial Eccentricity",
            expected_elements.eccentricity,
            0.4, // Approximately expected for this configuration
            0.1, // Allow some tolerance for numerical setup
        ));

        // Simulate for one complete orbit
        let steps_per_orbit = 2000;
        let dt = expected_elements.orbital_period / steps_per_orbit as f64;
        sim.set_timestep(dt);

        let initial_energy = sim.total_energy();
        let mut periapsis_distance = f64::INFINITY;
        let mut apoapsis_distance = 0.0;

        for step in 0..steps_per_orbit {
            sim.step();

            let current_pos = sim.position(planet);
            let distance = current_pos.norm();

            // Track periapsis and apoapsis
            periapsis_distance = periapsis_distance.min(distance);
            apoapsis_distance = apoapsis_distance.max(distance);
        }

        // Validate orbital characteristics
        let expected_periapsis =
            expected_elements.semi_major_axis * (1.0 - expected_elements.eccentricity);
        let expected_apoapsis =
            expected_elements.semi_major_axis * (1.0 + expected_elements.eccentricity);

        report.add_result(ValidationResult::new(
            "Periapsis Distance",
            periapsis_distance,
            expected_periapsis,
            constants::KEPLER_ORBIT_TOLERANCE,
        ));

        report.add_result(ValidationResult::new(
            "Apoapsis Distance",
            apoapsis_distance,
            expected_apoapsis,
            constants::KEPLER_ORBIT_TOLERANCE,
        ));

        // Energy conservation
        let final_energy = sim.total_energy();
        let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();

        report.add_result(ValidationResult::new(
            "Energy Conservation (Elliptical)",
            energy_error,
            0.0,
            constants::ENERGY_CONSERVATION_TOLERANCE,
        ));

        // Print report
        report.print_full_report();

        // Assert success
        assert!(report.overall_passed, "Elliptical orbit validation failed");
    }

    #[test]
    fn test_kepler_third_law() {
        let mut report = ValidationReport::new();

        // Test Kepler's third law: T² ∝ a³
        // Create multiple orbits at different distances

        let distances = vec![0.5, 1.0, 1.5, 2.0]; // In AU
        let mut periods = Vec::new();

        for &distance_au in &distances {
            let distance = distance_au * constants::AU;
            let velocity = theoretical_orbital_velocity(distance, constants::SOLAR_MASS);
            let period = theoretical_orbital_period(distance, constants::SOLAR_MASS);

            periods.push(period);

            // Verify theoretical relationship T² = (4π²/GM) * a³
            let theoretical_period_squared =
                (4.0 * PI * PI / (constants::G * constants::SOLAR_MASS)) * distance.powi(3);
            let calculated_period_squared = period * period;

            report.add_result(ValidationResult::new(
                format!("Kepler's Third Law at {:.1} AU", distance_au),
                calculated_period_squared,
                theoretical_period_squared,
                1e-10,
            ));
        }

        // Verify the proportionality relationship
        for i in 1..distances.len() {
            let distance_ratio_cubed = (distances[i] / distances[0]).powi(3);
            let period_ratio_squared = (periods[i] / periods[0]).powi(2);

            report.add_result(ValidationResult::new(
                format!(
                    "Period²/Distance³ Ratio {:.1}/{:.1} AU",
                    distances[i], distances[0]
                ),
                period_ratio_squared,
                distance_ratio_cubed,
                1e-12,
            ));
        }

        report.print_full_report();
        assert!(
            report.overall_passed,
            "Kepler's third law validation failed"
        );
    }
}

/// Run all Kepler orbit validation tests
pub fn run_kepler_validation() -> ValidationReport {
    let mut report = ValidationReport::new();

    println!("Running Kepler orbital mechanics validation...");

    // This would typically run the test functions programmatically
    // For now, we direct users to run: cargo test test_circular_earth_orbit

    println!("Run the following commands to execute Kepler validation:");
    println!("  cargo test test_circular_earth_orbit");
    println!("  cargo test test_elliptical_orbit");
    println!("  cargo test test_kepler_third_law");

    report
}
