// Energy Conservation Validation Tests
//
// These tests validate that Gravwell's integrators properly conserve
// energy over long simulation periods, which is critical for scientific accuracy.

use super::*;
use gravwell::prelude::*;
use std::collections::VecDeque;

/// Energy conservation analyzer for tracking energy drift over time
#[derive(Debug)]
pub struct EnergyConservationAnalyzer {
    energy_history: VecDeque<f64>,
    time_history: VecDeque<f64>,
    initial_energy: f64,
    max_history_length: usize,
}

impl EnergyConservationAnalyzer {
    pub fn new(initial_energy: f64, max_history_length: usize) -> Self {
        Self {
            energy_history: VecDeque::new(),
            time_history: VecDeque::new(),
            initial_energy,
            max_history_length,
        }
    }

    /// Record energy measurement at a specific time
    pub fn record_energy(&mut self, time: f64, energy: f64) {
        self.energy_history.push_back(energy);
        self.time_history.push_back(time);

        // Maintain rolling window
        if self.energy_history.len() > self.max_history_length {
            self.energy_history.pop_front();
            self.time_history.pop_front();
        }
    }

    /// Calculate current relative energy drift from initial value
    pub fn relative_energy_drift(&self) -> f64 {
        if let Some(&latest_energy) = self.energy_history.back() {
            (latest_energy - self.initial_energy).abs() / self.initial_energy.abs()
        } else {
            0.0
        }
    }

    /// Calculate energy drift rate (change per unit time)
    pub fn energy_drift_rate(&self) -> Option<f64> {
        if self.energy_history.len() < 2 {
            return None;
        }

        // Linear regression to find drift rate
        let slope = self.linear_regression_slope();
        Some(slope / self.initial_energy.abs())
    }

    /// Calculate maximum energy deviation throughout history
    pub fn maximum_energy_deviation(&self) -> f64 {
        self.energy_history
            .iter()
            .map(|&energy| (energy - self.initial_energy).abs() / self.initial_energy.abs())
            .fold(0.0, f64::max)
    }

    /// Calculate energy oscillation amplitude (for symplectic integrators)
    pub fn energy_oscillation_amplitude(&self) -> f64 {
        if self.energy_history.len() < 10 {
            return 0.0;
        }

        let energies: Vec<f64> = self.energy_history.iter().cloned().collect();
        let mean_energy = energies.iter().sum::<f64>() / energies.len() as f64;

        let max_deviation = energies
            .iter()
            .map(|&e| (e - mean_energy).abs())
            .fold(0.0, f64::max);

        max_deviation / self.initial_energy.abs()
    }

    /// Simple linear regression to calculate energy drift slope
    fn linear_regression_slope(&self) -> f64 {
        let n = self.energy_history.len() as f64;
        if n < 2.0 {
            return 0.0;
        }

        let sum_t: f64 = self.time_history.iter().sum();
        let sum_e: f64 = self.energy_history.iter().sum();
        let sum_te: f64 = self
            .time_history
            .iter()
            .zip(self.energy_history.iter())
            .map(|(&t, &e)| t * e)
            .sum();
        let sum_t_squared: f64 = self.time_history.iter().map(|&t| t * t).sum();

        // Slope = (n*Σ(te) - Σ(t)*Σ(e)) / (n*Σ(t²) - (Σ(t))²)
        let numerator = n * sum_te - sum_t * sum_e;
        let denominator = n * sum_t_squared - sum_t * sum_t;

        if denominator.abs() < 1e-15 {
            0.0
        } else {
            numerator / denominator
        }
    }

    /// Generate validation result for energy conservation
    pub fn validate_energy_conservation(&self, tolerance: f64) -> ValidationResult {
        let drift = self.relative_energy_drift();

        ValidationResult::new("Long-term Energy Conservation", drift, 0.0, tolerance)
    }
}

/// Test different integrators for energy conservation over extended periods
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_velocity_verlet_energy_conservation() {
        let mut report = ValidationReport::new();

        // Set up Earth-Sun system with Velocity Verlet
        let mut sim = Simulation::builder()
            .integrator(VelocityVerlet::new())
            .forces(DirectGravity::new())
            .timestep(3600.0) // 1 hour timestep
            .build()
            .expect("Failed to create simulation");

        let (_sun, earth) = add_earth_sun_bodies(&mut sim);

        // Initialize energy analyzer
        let initial_energy = sim.total_energy();
        let mut energy_analyzer = EnergyConservationAnalyzer::new(initial_energy, 10000);

        // Simulate for 100 orbital periods (≈273 Earth years)
        let orbital_period = theoretical_orbital_period(constants::AU, constants::SOLAR_MASS);
        let simulation_time = 100.0 * orbital_period;
        let total_steps = (simulation_time / sim.timestep()) as usize;

        println!(
            "Simulating {} steps ({:.1} years)...",
            total_steps,
            simulation_time / (365.25 * 24.0 * 3600.0)
        );

        for step in 0..total_steps {
            sim.step();

            // Record energy every 100 steps
            if step % 100 == 0 {
                let current_time = step as f64 * sim.timestep();
                let current_energy = sim.total_energy();
                energy_analyzer.record_energy(current_time, current_energy);

                // Print progress every 10,000 steps
                if step % 10000 == 0 {
                    let drift = energy_analyzer.relative_energy_drift();
                    println!("Step {}: Energy drift = {:.3e}", step, drift);
                }
            }
        }

        // Analyze results
        let final_drift = energy_analyzer.relative_energy_drift();
        let max_deviation = energy_analyzer.maximum_energy_deviation();
        let drift_rate = energy_analyzer.energy_drift_rate().unwrap_or(0.0);
        let oscillation_amplitude = energy_analyzer.energy_oscillation_amplitude();

        println!("\nVelocity Verlet Energy Conservation Analysis:");
        println!("  Final energy drift: {:.3e}", final_drift);
        println!("  Maximum deviation: {:.3e}", max_deviation);
        println!("  Drift rate: {:.3e} per second", drift_rate);
        println!("  Oscillation amplitude: {:.3e}", oscillation_amplitude);

        // Validate energy conservation
        report.add_result(
            energy_analyzer.validate_energy_conservation(constants::ENERGY_CONSERVATION_TOLERANCE),
        );

        report.add_result(ValidationResult::new(
            "Maximum Energy Deviation",
            max_deviation,
            0.0,
            constants::ENERGY_CONSERVATION_TOLERANCE,
        ));

        // For symplectic integrators, drift rate should be very small
        report.add_result(ValidationResult::new(
            "Energy Drift Rate",
            drift_rate.abs(),
            0.0,
            1e-18, // Very strict tolerance for drift rate
        ));

        report.print_full_report();
        assert!(
            report.overall_passed,
            "Velocity Verlet energy conservation failed"
        );
    }

    #[test]
    fn test_leapfrog_energy_conservation() {
        let mut report = ValidationReport::new();

        // Test with Leapfrog integrator
        let mut sim = Simulation::builder()
            .integrator(Leapfrog::new())
            .forces(DirectGravity::new())
            .timestep(1800.0) // 30 minutes (smaller timestep for Leapfrog)
            .build()
            .expect("Failed to create simulation");

        let (_sun, earth) = add_earth_sun_bodies(&mut sim);

        let initial_energy = sim.total_energy();
        let mut energy_analyzer = EnergyConservationAnalyzer::new(initial_energy, 5000);

        // Shorter test for Leapfrog (10 orbital periods)
        let orbital_period = theoretical_orbital_period(constants::AU, constants::SOLAR_MASS);
        let simulation_time = 10.0 * orbital_period;
        let total_steps = (simulation_time / sim.timestep()) as usize;

        println!("Testing Leapfrog integrator for {} steps...", total_steps);

        for step in 0..total_steps {
            sim.step();

            if step % 200 == 0 {
                let current_time = step as f64 * sim.timestep();
                let current_energy = sim.total_energy();
                energy_analyzer.record_energy(current_time, current_energy);
            }
        }

        let final_drift = energy_analyzer.relative_energy_drift();
        println!("Leapfrog final energy drift: {:.3e}", final_drift);

        report.add_result(
            energy_analyzer.validate_energy_conservation(constants::ENERGY_CONSERVATION_TOLERANCE),
        );
        report.print_full_report();

        assert!(report.overall_passed, "Leapfrog energy conservation failed");
    }

    #[test]
    fn test_integrator_comparison() {
        let mut report = ValidationReport::new();

        // Compare energy conservation across different integrators
        let integrators: Vec<(&str, Box<dyn Integrator>)> = vec![
            ("Semi-Implicit Euler", Box::new(SemiImplicitEuler::new())),
            ("Velocity Verlet", Box::new(VelocityVerlet::new())),
            ("Leapfrog", Box::new(Leapfrog::new())),
        ];

        let timestep = 1800.0; // 30 minutes
        let simulation_periods = 5.0; // 5 orbital periods

        for (name, integrator) in integrators {
            println!("\nTesting integrator: {}", name);

            let mut sim = Simulation::builder()
                .integrator(integrator)
                .forces(DirectGravity::new())
                .timestep(timestep)
                .build()
                .expect("Failed to create simulation");

            let (_sun, _earth) = add_earth_sun_bodies(&mut sim);

            let initial_energy = sim.total_energy();
            let orbital_period = theoretical_orbital_period(constants::AU, constants::SOLAR_MASS);
            let total_steps = ((simulation_periods * orbital_period) / timestep) as usize;

            // Run simulation
            for _ in 0..total_steps {
                sim.step();
            }

            let final_energy = sim.total_energy();
            let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();

            println!("  Energy drift: {:.3e}", energy_drift);

            // Different tolerance for different integrators
            let tolerance = match name {
                "Semi-Implicit Euler" => 1e-6, // Less strict for Euler
                "Velocity Verlet" => constants::ENERGY_CONSERVATION_TOLERANCE,
                "Leapfrog" => constants::ENERGY_CONSERVATION_TOLERANCE,
                _ => constants::ENERGY_CONSERVATION_TOLERANCE,
            };

            report.add_result(ValidationResult::new(
                format!("{} Energy Conservation", name),
                energy_drift,
                0.0,
                tolerance,
            ));
        }

        report.print_full_report();
        assert!(report.overall_passed, "Integrator comparison failed");
    }

    #[test]
    fn test_three_body_energy_conservation() {
        let mut report = ValidationReport::new();

        // Set up a three-body system (Sun, Earth, Moon)
        let mut sim = Simulation::builder()
            .integrator(VelocityVerlet::new())
            .forces(DirectGravity::new())
            .timestep(1800.0) // 30 minutes
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

        // Add Earth
        let _earth = sim
            .add_body(
                Body::new()
                    .mass(constants::EARTH_MASS)
                    .position([constants::AU, 0.0, 0.0])
                    .velocity([0.0, constants::EARTH_ORBITAL_VELOCITY, 0.0]),
            )
            .expect("Failed to add Earth");

        // Add Moon (relative to Earth)
        let moon_distance = 3.844e8; // 384,400 km
        let moon_velocity = 1022.0; // m/s orbital velocity around Earth
        let _moon = sim
            .add_body(
                Body::new()
                    .mass(7.342e22) // Moon mass in kg
                    .position([constants::AU + moon_distance, 0.0, 0.0])
                    .velocity([0.0, constants::EARTH_ORBITAL_VELOCITY + moon_velocity, 0.0]),
            )
            .expect("Failed to add Moon");

        let initial_energy = sim.total_energy();
        let mut energy_analyzer = EnergyConservationAnalyzer::new(initial_energy, 2000);

        // Simulate for several months
        let simulation_days = 100.0;
        let total_steps = ((simulation_days * 24.0 * 3600.0) / sim.timestep()) as usize;

        println!(
            "Testing three-body energy conservation for {} days...",
            simulation_days
        );

        for step in 0..total_steps {
            sim.step();

            if step % 500 == 0 {
                let current_time = step as f64 * sim.timestep();
                let current_energy = sim.total_energy();
                energy_analyzer.record_energy(current_time, current_energy);
            }
        }

        let final_drift = energy_analyzer.relative_energy_drift();
        println!("Three-body system energy drift: {:.3e}", final_drift);

        // Slightly relaxed tolerance for three-body system
        report.add_result(ValidationResult::new(
            "Three-Body Energy Conservation",
            final_drift,
            0.0,
            1e-10, // Slightly more tolerant than two-body
        ));

        report.print_full_report();
        assert!(
            report.overall_passed,
            "Three-body energy conservation failed"
        );
    }
}

/// Helper function to add Earth-Sun bodies to a simulation
fn add_earth_sun_bodies(sim: &mut Simulation) -> (BodyHandle, BodyHandle) {
    let sun = sim
        .add_body(
            Body::new()
                .mass(constants::SOLAR_MASS)
                .position([0.0, 0.0, 0.0])
                .velocity([0.0, 0.0, 0.0]),
        )
        .expect("Failed to add Sun");

    let earth = sim
        .add_body(
            Body::new()
                .mass(constants::EARTH_MASS)
                .position([constants::AU, 0.0, 0.0])
                .velocity([0.0, constants::EARTH_ORBITAL_VELOCITY, 0.0]),
        )
        .expect("Failed to add Earth");

    (sun, earth)
}

/// Run comprehensive energy conservation validation
pub fn run_energy_conservation_validation() -> ValidationReport {
    let mut report = ValidationReport::new();

    println!("Running comprehensive energy conservation validation...");
    println!("This may take several minutes for long-term simulations...");

    // Note: In a real implementation, we would run the tests programmatically
    println!("Execute the following tests:");
    println!("  cargo test test_velocity_verlet_energy_conservation");
    println!("  cargo test test_leapfrog_energy_conservation");
    println!("  cargo test test_integrator_comparison");
    println!("  cargo test test_three_body_energy_conservation");

    report
}
