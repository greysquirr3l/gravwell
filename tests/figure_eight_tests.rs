//! Three-Body Figure-Eight Solution Tests
//!
//! Tests for validating the famous figure-eight solution discovered by Moore (1993)
//! and Chenciner & Montgomery (2000). This provides a rigorous test of numerical
//! accuracy for chaotic three-body dynamics.

use gravwell::prelude::*;

/// Figure-eight solution initial conditions (Moore 1993, normalized units)
/// These are precise initial conditions that produce a stable figure-eight orbit
const FIGURE_EIGHT_INITIAL_CONDITIONS: FigureEightSolution = FigureEightSolution {
    // Body 1 position and velocity
    x1: 0.9700436,
    y1: -0.24308753,
    vx1: 0.466203685,
    vy1: 0.43236573,

    // Body 2 position and velocity
    x2: -0.9700436,
    y2: 0.24308753,
    vx2: 0.466203685,
    vy2: 0.43236573,

    // Body 3 is at origin with opposite momentum
    x3: 0.0,
    y3: 0.0,
    vx3: -2.0 * 0.466203685,
    vy3: -2.0 * 0.43236573,

    // Each body has equal mass (normalized to 1.0)
    mass: 1.0,

    // Orbital period is approximately 6.32591398 time units
    period: 6.32591398,

    // Gravitational constant (normalized)
    g_constant: 1.0,
};

#[derive(Debug, Clone)]
pub struct FigureEightSolution {
    pub x1: f64,
    pub y1: f64,
    pub vx1: f64,
    pub vy1: f64,
    pub x2: f64,
    pub y2: f64,
    pub vx2: f64,
    pub vy2: f64,
    pub x3: f64,
    pub y3: f64,
    pub vx3: f64,
    pub vy3: f64,
    pub mass: f64,
    pub period: f64,
    pub g_constant: f64,
}

/// Analysis results for figure-eight orbit validation
#[derive(Debug)]
pub struct FigureEightAnalysis {
    pub max_deviation_from_path: f64,
    pub period_error: f64,
    pub energy_drift: f64,
    pub momentum_conservation_error: f64,
    pub simulation_time: f64,
    pub completed_orbits: f64,
}

impl FigureEightAnalysis {
    pub fn print_summary(&self) {
        println!("\n∞ Figure-Eight Solution Analysis");
        println!("================================");
        println!(
            "Simulation Time: {:.3} units ({:.2} periods)",
            self.simulation_time, self.completed_orbits
        );
        println!("Max Path Deviation: {:.2e}", self.max_deviation_from_path);
        println!(
            "Period Error: {:.2e} ({:.2e}%)",
            self.period_error,
            self.period_error * 100.0 / FIGURE_EIGHT_INITIAL_CONDITIONS.period
        );
        println!("Energy Drift: {:.2e}", self.energy_drift);
        println!("Momentum Error: {:.2e}", self.momentum_conservation_error);

        // Pass/fail assessment
        let path_accurate = self.max_deviation_from_path < 1e-3;
        let period_accurate = self.period_error.abs() < 1e-6;
        let energy_stable = self.energy_drift < 1e-10;
        let momentum_conserved = self.momentum_conservation_error < 1e-12;

        println!("\n🎯 Assessment:");
        println!(
            "  Path Accuracy (1e-3): {}",
            if path_accurate {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        );
        println!(
            "  Period Accuracy (1e-6): {}",
            if period_accurate {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        );
        println!(
            "  Energy Stability (1e-10): {}",
            if energy_stable {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        );
        println!(
            "  Momentum Conservation (1e-12): {}",
            if momentum_conserved {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        );

        let overall_pass = path_accurate && period_accurate && energy_stable && momentum_conserved;
        println!(
            "\n🏆 Overall: {}",
            if overall_pass {
                "✅ ALL TESTS PASSED"
            } else {
                "❌ TESTS FAILED"
            }
        );
    }
}

/// Calculate normalized gravitational potential energy
fn potential_energy_normalized(
    sim: &gravwell::builder::Simulation<impl Integrator, impl ForceCalculator>,
) -> f64 {
    let particles = sim.particles();
    let n = particles.len();

    let mut potential = 0.0;
    for i in 0..n {
        for j in (i + 1)..n {
            let pos1 = *particles.position(i);
            let pos2 = *particles.position(j);
            let mass1 = particles.mass(i);
            let mass2 = particles.mass(j);

            let r = (pos2 - pos1).magnitude();
            potential -= FIGURE_EIGHT_INITIAL_CONDITIONS.g_constant * mass1 * mass2 / r;
        }
    }

    potential
}

/// Calculate total energy (kinetic + potential) with normalized units
fn total_energy_normalized(
    sim: &gravwell::builder::Simulation<impl Integrator, impl ForceCalculator>,
) -> f64 {
    let kinetic = sim.particles().kinetic_energy();
    let potential = potential_energy_normalized(sim);
    kinetic + potential
}

/// Calculate center of mass of the three-body system
#[allow(dead_code)]
fn calculate_center_of_mass(
    sim: &gravwell::builder::Simulation<impl Integrator, impl ForceCalculator>,
) -> Vector3 {
    sim.particles().center_of_mass()
}

/// Calculate total linear momentum
fn calculate_total_momentum(
    sim: &gravwell::builder::Simulation<impl Integrator, impl ForceCalculator>,
) -> Vector3 {
    let particles = sim.particles();
    let mut momentum = Vector3::zeros();

    for i in 0..particles.len() {
        let vel = *particles.velocity(i);
        let mass = particles.mass(i);
        momentum += mass * vel;
    }

    momentum
}

#[cfg(test)]
mod figure_eight_tests {
    use super::*;

    #[test]
    fn test_figure_eight_basic_stability() {
        println!("\n∞ Testing Figure-Eight basic stability...");

        // Create three-body system with figure-eight initial conditions
        let sol = &FIGURE_EIGHT_INITIAL_CONDITIONS;
        let mut sim = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(DirectGravity::new())
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x1, sol.y1, 0.0])
                    .with_velocity([sol.vx1, sol.vy1, 0.0]),
            )
            .expect("Failed to add body 1")
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x2, sol.y2, 0.0])
                    .with_velocity([sol.vx2, sol.vy2, 0.0]),
            )
            .expect("Failed to add body 2")
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x3, sol.y3, 0.0])
                    .with_velocity([sol.vx3, sol.vy3, 0.0]),
            )
            .expect("Failed to add body 3")
            .build()
            .expect("Failed to build simulation");

        let initial_energy = total_energy_normalized(&sim);
        let initial_momentum = calculate_total_momentum(&sim);

        // Simulate for 1/4 period to check basic stability
        let timestep = sol.period / 1000.0; // 1000 steps per period
        let simulation_steps = 250; // 1/4 period

        println!(
            "Simulating {:.3} time units ({:.1} steps)...",
            timestep * simulation_steps as f64,
            simulation_steps
        );

        for _step in 0..simulation_steps {
            sim.step(timestep).expect("Simulation step failed");
        }

        let final_energy = total_energy_normalized(&sim);
        let final_momentum = calculate_total_momentum(&sim);

        // Check energy conservation
        let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();
        println!("  Energy drift: {:.2e}", energy_drift);

        // Check momentum conservation
        let momentum_error = (final_momentum - initial_momentum).magnitude()
            / (initial_momentum.magnitude() + 1e-15);
        println!("  Momentum error: {:.2e}", momentum_error);

        // Verify basic stability
        assert!(
            energy_drift < 1e-8,
            "Energy drift too large: {:.2e}",
            energy_drift
        );
        assert!(
            momentum_error < 1e-10,
            "Momentum not conserved: {:.2e}",
            momentum_error
        );

        println!("✅ Figure-eight basic stability validated!");
    }

    #[test]
    fn test_figure_eight_one_complete_orbit() {
        println!("\n∞ Testing Figure-Eight complete orbit...");

        // Create the figure-eight system
        let sol = &FIGURE_EIGHT_INITIAL_CONDITIONS;
        let mut sim = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(DirectGravity::new())
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x1, sol.y1, 0.0])
                    .with_velocity([sol.vx1, sol.vy1, 0.0]),
            )
            .expect("Failed to add body 1")
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x2, sol.y2, 0.0])
                    .with_velocity([sol.vx2, sol.vy2, 0.0]),
            )
            .expect("Failed to add body 2")
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x3, sol.y3, 0.0])
                    .with_velocity([sol.vx3, sol.vy3, 0.0]),
            )
            .expect("Failed to add body 3")
            .build()
            .expect("Failed to build simulation");

        // Store initial conditions for comparison
        let initial_positions: Vec<Vector3> =
            (0..3).map(|i| *sim.particles().position(i)).collect();
        let initial_velocities: Vec<Vector3> =
            (0..3).map(|i| *sim.particles().velocity(i)).collect();
        let initial_energy = total_energy_normalized(&sim);
        let initial_momentum = calculate_total_momentum(&sim);

        // Simulate for one complete period
        let timestep = sol.period / 2000.0; // 2000 steps per period for accuracy
        let period_steps = 2000;

        println!(
            "Simulating one complete orbit ({:.3} time units, {} steps)...",
            sol.period, period_steps
        );

        for step in 0..period_steps {
            sim.step(timestep).expect("Simulation step failed");

            if step % 400 == 0 {
                let current_energy = total_energy_normalized(&sim);
                let energy_drift = (current_energy - initial_energy).abs() / initial_energy.abs();
                println!("  Step {}: Energy drift = {:.2e}", step, energy_drift);
            }
        }

        // After one period, check how close we are to initial conditions
        let final_positions: Vec<Vector3> = (0..3).map(|i| *sim.particles().position(i)).collect();
        let final_velocities: Vec<Vector3> = (0..3).map(|i| *sim.particles().velocity(i)).collect();
        let final_energy = total_energy_normalized(&sim);
        let final_momentum = calculate_total_momentum(&sim);

        // Calculate position errors
        let mut max_position_error: f64 = 0.0;
        for i in 0..3 {
            let pos_error = (final_positions[i] - initial_positions[i]).magnitude();
            max_position_error = max_position_error.max(pos_error);
        }

        // Calculate velocity errors
        let mut max_velocity_error: f64 = 0.0;
        for i in 0..3 {
            let vel_error = (final_velocities[i] - initial_velocities[i]).magnitude();
            max_velocity_error = max_velocity_error.max(vel_error);
        }

        let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();
        let momentum_error = (final_momentum - initial_momentum).magnitude();

        println!("\n📊 Orbit Completion Analysis:");
        println!("  Max position error: {:.2e}", max_position_error);
        println!("  Max velocity error: {:.2e}", max_velocity_error);
        println!("  Energy drift: {:.2e}", energy_drift);
        println!("  Momentum error: {:.2e}", momentum_error);

        // The figure-eight solution is chaotic, so we expect some sensitivity
        // but should still maintain reasonable accuracy for one period
        assert!(
            max_position_error < 1e-2,
            "Position error too large: {:.2e}",
            max_position_error
        );
        assert!(
            energy_drift < 1e-6,
            "Energy drift too large: {:.2e}",
            energy_drift
        );
        assert!(
            momentum_error < 1e-10,
            "Momentum not conserved: {:.2e}",
            momentum_error
        );

        println!("✅ Figure-eight complete orbit validated!");
    }

    #[test]
    fn test_figure_eight_velocity_verlet() {
        println!("\n∞ Testing Figure-Eight with Velocity Verlet integrator...");

        let sol = &FIGURE_EIGHT_INITIAL_CONDITIONS;
        let mut sim = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(DirectGravity::new())
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x1, sol.y1, 0.0])
                    .with_velocity([sol.vx1, sol.vy1, 0.0]),
            )
            .expect("Failed to add body 1")
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x2, sol.y2, 0.0])
                    .with_velocity([sol.vx2, sol.vy2, 0.0]),
            )
            .expect("Failed to add body 2")
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x3, sol.y3, 0.0])
                    .with_velocity([sol.vx3, sol.vy3, 0.0]),
            )
            .expect("Failed to add body 3")
            .build()
            .expect("Failed to build simulation");

        let initial_energy = total_energy_normalized(&sim);

        // Simulate for 1/2 period
        let timestep = sol.period / 1000.0;
        let steps = 500; // Half period

        for _step in 0..steps {
            sim.step(timestep).expect("Simulation step failed");
        }

        let final_energy = total_energy_normalized(&sim);
        let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();

        println!("  Velocity Verlet energy drift: {:.2e}", energy_drift);

        assert!(
            energy_drift < 1e-7,
            "Velocity Verlet energy drift too large: {:.2e} > 1e-7",
            energy_drift
        );

        println!("✅ Velocity Verlet validated for figure-eight solution!");
    }

    #[test]
    fn test_figure_eight_leapfrog() {
        println!("\n∞ Testing Figure-Eight with Leapfrog integrator...");

        let sol = &FIGURE_EIGHT_INITIAL_CONDITIONS;
        let mut sim = SimulationBuilder::new()
            .with_integrator(Leapfrog::new())
            .with_force_calculator(DirectGravity::new())
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x1, sol.y1, 0.0])
                    .with_velocity([sol.vx1, sol.vy1, 0.0]),
            )
            .expect("Failed to add body 1")
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x2, sol.y2, 0.0])
                    .with_velocity([sol.vx2, sol.vy2, 0.0]),
            )
            .expect("Failed to add body 2")
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x3, sol.y3, 0.0])
                    .with_velocity([sol.vx3, sol.vy3, 0.0]),
            )
            .expect("Failed to add body 3")
            .build()
            .expect("Failed to build simulation");

        let initial_energy = total_energy_normalized(&sim);

        // Simulate for 1/2 period
        let timestep = sol.period / 1000.0;
        let steps = 500; // Half period

        for _step in 0..steps {
            sim.step(timestep).expect("Simulation step failed");
        }

        let final_energy = total_energy_normalized(&sim);
        let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();

        println!("  Leapfrog energy drift: {:.2e}", energy_drift);

        assert!(
            energy_drift < 1e-6,
            "Leapfrog energy drift too large: {:.2e} > 1e-6",
            energy_drift
        );

        println!("✅ Leapfrog validated for figure-eight solution!");
    }

    #[test]
    fn test_figure_eight_rk4() {
        println!("\n∞ Testing Figure-Eight with RK4 integrator...");

        let sol = &FIGURE_EIGHT_INITIAL_CONDITIONS;
        let mut sim = SimulationBuilder::new()
            .with_integrator(RungeKutta4::new())
            .with_force_calculator(DirectGravity::new())
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x1, sol.y1, 0.0])
                    .with_velocity([sol.vx1, sol.vy1, 0.0]),
            )
            .expect("Failed to add body 1")
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x2, sol.y2, 0.0])
                    .with_velocity([sol.vx2, sol.vy2, 0.0]),
            )
            .expect("Failed to add body 2")
            .add_body(
                Body::new()
                    .with_mass(sol.mass)
                    .with_position([sol.x3, sol.y3, 0.0])
                    .with_velocity([sol.vx3, sol.vy3, 0.0]),
            )
            .expect("Failed to add body 3")
            .build()
            .expect("Failed to build simulation");

        let initial_energy = total_energy_normalized(&sim);

        // Simulate for 1/2 period
        let timestep = sol.period / 1000.0;
        let steps = 500; // Half period

        for _step in 0..steps {
            sim.step(timestep).expect("Simulation step failed");
        }

        let final_energy = total_energy_normalized(&sim);
        let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();

        println!("  RK4 energy drift: {:.2e}", energy_drift);

        assert!(
            energy_drift < 1e-9,
            "RK4 energy drift too large: {:.2e} > 1e-9",
            energy_drift
        );

        println!("✅ RK4 validated for figure-eight solution!");
    }
}
