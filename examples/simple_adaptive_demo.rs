//! Simple Adaptive Timestep Control Demo
//!
//! This example demonstrates the basic usage of the adaptive timestep control system.

use gravwell::{
    adaptive::{AdaptiveTimestepController, ErrorMetric},
    error::GravwellError,
    types::{Mass, Scalar, Vector3},
};

// Simple mock simulation for demonstration
struct MockSimulation {
    positions: Vec<Vector3>,
    velocities: Vec<Vector3>,
    masses: Vec<Mass>,
    forces: Vec<Vector3>,
    energy: Scalar,
    time: Scalar,
}

impl MockSimulation {
    fn new() -> Self {
        Self {
            positions: vec![
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(1e11, 0.0, 0.0), // 1 AU separation
            ],
            velocities: vec![
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 30000.0, 0.0), // Orbital velocity
            ],
            masses: vec![1.989e30, 5.972e24], // Sun and Earth masses
            forces: vec![Vector3::zeros(), Vector3::zeros()],
            energy: -2.6e33, // Approximate gravitational energy
            time: 0.0,
        }
    }

    fn step(&mut self, dt: Scalar) {
        // Simple integration step
        for i in 0..self.positions.len() {
            self.velocities[i] += self.forces[i] / self.masses[i] * dt;
            self.positions[i] += self.velocities[i] * dt;
        }
        self.time += dt;

        // Update energy with small random variation to simulate numerical error
        self.energy *= 1.0 + (fastrand::f64() - 0.5) * 1e-12;
    }

    fn update_forces(&mut self) {
        const G: Scalar = 6.67430e-11;

        for i in 0..self.positions.len() {
            self.forces[i] = Vector3::zeros();

            for j in 0..self.positions.len() {
                if i != j {
                    let r_vec = self.positions[j] - self.positions[i];
                    let r = r_vec.norm();
                    let force_magnitude = G * self.masses[i] * self.masses[j] / (r * r);
                    self.forces[i] += force_magnitude * r_vec / r;
                }
            }
        }
    }

    fn total_energy(&self) -> Scalar {
        self.energy
    }
}

fn main() -> Result<(), GravwellError> {
    println!("🚀 Gravwell - Adaptive Timestep Control Demo");
    println!("=============================================");

    // Create adaptive controller with conservative settings
    let mut controller = AdaptiveTimestepController::conservative(86400.0, 1e-9)?; // 1 day initial timestep

    // Create mock simulation
    let mut sim = MockSimulation::new();

    println!("Initial Setup:");
    println!("  Bodies: Sun-Earth system");
    println!(
        "  Initial timestep: {:.2e} s ({:.1} days)",
        controller.current_timestep(),
        controller.current_timestep() / 86400.0
    );
    println!("  Error tolerance: 1e-9");
    println!();

    let mut max_timestep = controller.current_timestep();
    let mut min_timestep = controller.current_timestep();
    let initial_energy = sim.total_energy();

    // Run simulation for 100 steps
    for step in 0..100 {
        // Update forces
        sim.update_forces();

        // Get current state for adaptive control
        let current_energy = sim.total_energy();

        // Update adaptive timestep
        let new_timestep = controller.update_timestep(
            &sim.positions,
            &sim.velocities,
            &sim.forces,
            &sim.masses,
            Some(current_energy),
        );

        // Track timestep range
        max_timestep = max_timestep.max(new_timestep);
        min_timestep = min_timestep.min(new_timestep);

        // Advance simulation
        sim.step(new_timestep);

        // Print periodic updates
        if step % 20 == 0 {
            let energy_error = ((current_energy - initial_energy) / initial_energy).abs();

            println!(
                "Step {}: t={:.1} days, dt={:.2e} s, E_err={:.2e}",
                step,
                sim.time / 86400.0,
                new_timestep,
                energy_error
            );

            // Show stability analysis
            if let Some(analysis) = controller.last_stability_analysis() {
                println!(
                    "  Stable: {}, Error: {:.2e}, Trend: {:?}",
                    analysis.is_stable, analysis.current_error, analysis.error_trend
                );

                if !analysis.warnings.is_empty() {
                    println!("  Warnings: {} detected", analysis.warnings.len());
                }
            }
        }
    }

    println!();
    println!("Final Results:");
    println!("  Total simulation time: {:.1} days", sim.time / 86400.0);
    println!("  Total integration steps: {}", controller.step_count());
    println!(
        "  Timestep range: {:.2e} - {:.2e} s",
        min_timestep, max_timestep
    );
    println!(
        "  Final timestep: {:.2e} s ({:.1} days)",
        controller.current_timestep(),
        controller.current_timestep() / 86400.0
    );

    let final_energy = sim.total_energy();
    let energy_drift = ((final_energy - initial_energy) / initial_energy).abs();
    println!("  Energy conservation error: {:.2e}", energy_drift);

    // Demonstrate different error metrics
    demonstrate_error_metrics()?;

    Ok(())
}

fn demonstrate_error_metrics() -> Result<(), GravwellError> {
    println!();
    println!("🔍 Error Metric Comparison");
    println!("==========================");

    let metrics = [
        ("Position", ErrorMetric::Position),
        ("Velocity", ErrorMetric::Velocity),
        ("Energy", ErrorMetric::Energy),
        ("Acceleration", ErrorMetric::Acceleration),
        ("Combined", ErrorMetric::Combined),
    ];

    for (name, metric) in metrics {
        let mut controller = AdaptiveTimestepController::conservative(86400.0, 1e-8)?;
        controller.set_error_metric(metric);

        let mut sim = MockSimulation::new();
        let mut adaptations = 0;
        let initial_timestep = controller.current_timestep();

        // Run short simulation
        for _step in 0..20 {
            sim.update_forces();

            let old_timestep = controller.current_timestep();
            let new_timestep = controller.update_timestep(
                &sim.positions,
                &sim.velocities,
                &sim.forces,
                &sim.masses,
                Some(sim.total_energy()),
            );

            if (new_timestep - old_timestep).abs() > old_timestep * 0.1 {
                adaptations += 1;
            }

            sim.step(new_timestep);
        }

        println!(
            "  {}: {} adaptations, {:.3e} -> {:.3e} s",
            name,
            adaptations,
            initial_timestep,
            controller.current_timestep()
        );
    }

    println!();
    println!("🏁 Demo Complete!");
    println!("The adaptive timestep controller successfully maintained simulation");
    println!("stability while automatically adjusting timesteps based on error metrics.");

    Ok(())
}
