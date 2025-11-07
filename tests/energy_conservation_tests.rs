//! Energy Conservation Analysis
//!
//! Long-term energy conservation tests for validating numerical integrator accuracy
//! and identifying potential stability issues in extended simulations.

use gravwell::prelude::*;

/// Energy drift tracking data structure
#[derive(Debug, Clone)]
pub struct EnergyTracker {
    initial_energy: f64,
    energy_history: Vec<f64>,
    time_history: Vec<f64>,
    max_drift: f64,
    max_drift_time: f64,
}

impl EnergyTracker {
    pub fn new(initial_energy: f64) -> Self {
        Self {
            initial_energy,
            energy_history: vec![initial_energy],
            time_history: vec![0.0],
            max_drift: 0.0,
            max_drift_time: 0.0,
        }
    }

    pub fn record(&mut self, time: f64, energy: f64) {
        self.energy_history.push(energy);
        self.time_history.push(time);

        let drift = (energy - self.initial_energy).abs() / self.initial_energy.abs();
        if drift > self.max_drift {
            self.max_drift = drift;
            self.max_drift_time = time;
        }
    }

    pub fn relative_drift(&self) -> f64 {
        if let Some(&latest_energy) = self.energy_history.last() {
            (latest_energy - self.initial_energy).abs() / self.initial_energy.abs()
        } else {
            0.0
        }
    }

    pub fn summary(&self) -> EnergyConservationSummary {
        EnergyConservationSummary {
            total_steps: self.energy_history.len() - 1,
            total_time: *self.time_history.last().unwrap_or(&0.0),
            initial_energy: self.initial_energy,
            final_energy: *self.energy_history.last().unwrap_or(&self.initial_energy),
            max_relative_drift: self.max_drift,
            max_drift_time: self.max_drift_time,
        }
    }
}

#[derive(Debug)]
pub struct EnergyConservationSummary {
    pub total_steps: usize,
    pub total_time: f64,
    pub initial_energy: f64,
    pub final_energy: f64,
    pub max_relative_drift: f64,
    pub max_drift_time: f64,
}

impl EnergyConservationSummary {
    pub fn print(&self) {
        println!("\n📊 Energy Conservation Analysis");
        println!("==============================");
        println!("Total Steps: {}", self.total_steps);
        println!(
            "Total Time: {:.2} years",
            self.total_time / (365.25 * 24.0 * 3600.0)
        );
        println!("Initial Energy: {:.6e} J", self.initial_energy);
        println!("Final Energy: {:.6e} J", self.final_energy);
        println!("Maximum Relative Drift: {:.2e}", self.max_relative_drift);
        println!(
            "Time of Max Drift: {:.2} years",
            self.max_drift_time / (365.25 * 24.0 * 3600.0)
        );

        // Pass/fail assessment
        let passes_strict = self.max_relative_drift < 1e-12;
        let passes_moderate = self.max_relative_drift < 1e-10;
        let passes_basic = self.max_relative_drift < 1e-8;

        println!("\n🎯 Assessment:");
        println!(
            "  Strict (1e-12): {}",
            if passes_strict {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        );
        println!(
            "  Moderate (1e-10): {}",
            if passes_moderate {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        );
        println!(
            "  Basic (1e-8): {}",
            if passes_basic { "✅ PASS" } else { "❌ FAIL" }
        );
    }
}

/// Calculate total energy (kinetic + potential) for any simulation
fn total_system_energy<I: Integrator, F: ForceCalculator>(
    sim: &gravwell::builder::Simulation<I, F>,
) -> f64 {
    let particles = sim.particles();
    let n = particles.len();

    // Kinetic energy (using built-in method)
    let kinetic_energy = particles.kinetic_energy();

    // Potential energy
    let mut potential_energy = 0.0;
    for i in 0..n {
        for j in (i + 1)..n {
            let pos1 = *particles.position(i);
            let pos2 = *particles.position(j);
            let mass1 = particles.mass(i);
            let mass2 = particles.mass(j);

            let r = (pos2 - pos1).magnitude();
            potential_energy -= G * mass1 * mass2 / r;
        }
    }

    kinetic_energy + potential_energy
}

#[cfg(test)]
mod energy_tests {
    use super::*;

    #[test]
    fn test_earth_sun_long_term_energy_conservation() {
        println!("\n🌍☀️ Testing Earth-Sun long-term energy conservation...");

        // Create Earth-Sun system with VelocityVerlet (symplectic)
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
                    .with_velocity([0.0, 29785.0, 0.0]),
            ) // Approximate circular velocity
            .expect("Failed to add Earth")
            .build()
            .expect("Failed to build simulation");

        let initial_energy = total_system_energy(&sim);
        let mut tracker = EnergyTracker::new(initial_energy);

        // Simulate for 10 years with 1 hour timesteps (reduced for testing)
        let timestep = 3600.0; // 1 hour
        let steps_per_year = (365.25 * 24.0) as usize;
        let years = 5; // Reduced for faster testing
        let total_steps = steps_per_year * years;

        println!(
            "Simulating {} years ({} steps) with {:.0}h timesteps...",
            years,
            total_steps,
            timestep / 3600.0
        );

        // Track energy every 1000 steps to avoid memory issues
        let track_interval = 1000;
        let mut step_count = 0;

        for year in 0..years {
            for _step in 0..steps_per_year {
                sim.step(timestep).expect("Simulation step failed");
                step_count += 1;

                if step_count % track_interval == 0 {
                    let current_time = step_count as f64 * timestep;
                    let current_energy = total_system_energy(&sim);
                    tracker.record(current_time, current_energy);
                }
            }

            if year % 1 == 0 {
                let current_energy = total_system_energy(&sim);
                let drift = (current_energy - initial_energy).abs() / initial_energy.abs();
                println!("  Year {}: Energy drift = {:.2e}", year + 1, drift);
            }
        }

        let summary = tracker.summary();
        summary.print();

        // Validate energy conservation
        assert!(
            summary.max_relative_drift < 1e-7,
            "Energy drift too large: {:.2e} > 1e-7",
            summary.max_relative_drift
        );

        println!("✅ Long-term energy conservation validated!");
    }

    #[test]
    fn test_solar_system_energy_conservation() {
        println!("\n🪐 Testing simplified solar system energy conservation...");

        // Create simplified solar system (Sun, Earth, Jupiter)
        let mut sim = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(DirectGravity::new())
            // Sun
            .add_body(
                Body::new()
                    .with_mass(SOLAR_MASS)
                    .with_position([0.0, 0.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .expect("Failed to add Sun")
            // Earth
            .add_body(
                Body::new()
                    .with_mass(EARTH_MASS)
                    .with_position([AU, 0.0, 0.0])
                    .with_velocity([0.0, 29785.0, 0.0]),
            )
            .expect("Failed to add Earth")
            // Jupiter (simplified orbit)
            .add_body(
                Body::new()
                    .with_mass(JUPITER_MASS)
                    .with_position([5.2 * AU, 0.0, 0.0])
                    .with_velocity([0.0, 13070.0, 0.0]),
            ) // Approximate circular velocity
            .expect("Failed to add Jupiter")
            .build()
            .expect("Failed to build simulation");

        let initial_energy = total_system_energy(&sim);
        let mut tracker = EnergyTracker::new(initial_energy);

        // Simulate for 6 years (reduced for testing)
        let timestep = 86400.0; // 1 day
        let total_days = 6 * 365; // 6 years

        println!(
            "Simulating {} days ({:.1} years) with 1-day timesteps...",
            total_days,
            total_days as f64 / 365.25
        );

        let track_interval = 365; // Track every year

        for day in 0..total_days {
            sim.step(timestep).expect("Simulation step failed");

            if day % track_interval == 0 {
                let current_time = day as f64 * timestep;
                let current_energy = total_system_energy(&sim);
                tracker.record(current_time, current_energy);

                if day % (365) == 0 {
                    let drift = (current_energy - initial_energy).abs() / initial_energy.abs();
                    println!("  Year {}: Energy drift = {:.2e}", day / 365, drift);
                }
            }
        }

        let summary = tracker.summary();
        summary.print();

        // Validate energy conservation (slightly relaxed for multi-body system)
        assert!(
            summary.max_relative_drift < 1e-5,
            "Energy drift too large: {:.2e} > 1e-5",
            summary.max_relative_drift
        );

        println!("✅ Solar system energy conservation validated!");
    }

    #[test]
    fn test_velocity_verlet_energy_conservation() {
        println!("\n⚖️ Testing Velocity Verlet energy conservation...");

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
                    .with_velocity([0.0, 29785.0, 0.0]),
            )
            .expect("Failed to add Earth")
            .build()
            .expect("Failed to build simulation");

        let initial_energy = total_system_energy(&sim);

        // Simulate for 5 years with 1-hour timesteps
        let timestep = 3600.0; // 1 hour
        let years = 5;
        let total_steps = years * 365 * 24;

        for _ in 0..total_steps {
            sim.step(timestep).expect("Simulation step failed");
        }

        let final_energy = total_system_energy(&sim);
        let drift = (final_energy - initial_energy).abs() / initial_energy.abs();

        println!("  Velocity Verlet energy drift: {:.2e}", drift);

        // Symplectic integrator should have excellent energy conservation
        assert!(
            drift < 1e-7,
            "Velocity Verlet energy drift too large: {:.2e} > 1e-7",
            drift
        );

        println!("✅ Velocity Verlet energy conservation validated!");
    }

    #[test]
    fn test_leapfrog_energy_conservation() {
        println!("\n⚖️ Testing Leapfrog energy conservation...");

        let mut sim = SimulationBuilder::new()
            .with_integrator(Leapfrog::new())
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
                    .with_velocity([0.0, 29785.0, 0.0]),
            )
            .expect("Failed to add Earth")
            .build()
            .expect("Failed to build simulation");

        let initial_energy = total_system_energy(&sim);

        // Simulate for 5 years with 1-hour timesteps
        let timestep = 3600.0; // 1 hour
        let years = 5;
        let total_steps = years * 365 * 24;

        for _ in 0..total_steps {
            sim.step(timestep).expect("Simulation step failed");
        }

        let final_energy = total_system_energy(&sim);
        let drift = (final_energy - initial_energy).abs() / initial_energy.abs();

        println!("  Leapfrog energy drift: {:.2e}", drift);

        // Symplectic integrator should have excellent energy conservation
        assert!(
            drift < 1e-6,
            "Leapfrog energy drift too large: {:.2e} > 1e-6",
            drift
        );

        println!("✅ Leapfrog energy conservation validated!");
    }

    #[test]
    fn test_rk4_energy_conservation() {
        println!("\n⚖️ Testing RK4 energy conservation...");

        let mut sim = SimulationBuilder::new()
            .with_integrator(RungeKutta4::new())
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
                    .with_velocity([0.0, 29785.0, 0.0]),
            )
            .expect("Failed to add Earth")
            .build()
            .expect("Failed to build simulation");

        let initial_energy = total_system_energy(&sim);

        // Simulate for 5 years with 1-hour timesteps
        let timestep = 3600.0; // 1 hour
        let years = 5;
        let total_steps = years * 365 * 24;

        for _ in 0..total_steps {
            sim.step(timestep).expect("Simulation step failed");
        }

        let final_energy = total_system_energy(&sim);
        let drift = (final_energy - initial_energy).abs() / initial_energy.abs();

        println!("  RK4 energy drift: {:.2e}", drift);

        // Non-symplectic integrator may have slightly worse energy conservation
        assert!(
            drift < 1e-5,
            "RK4 energy drift too large: {:.2e} > 1e-5",
            drift
        );

        println!("✅ RK4 energy conservation validated!");
    }
}
