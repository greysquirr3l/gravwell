//! Momentum Conservation Tests
//!
//! Comprehensive validation of linear and angular momentum conservation
//! in multi-body gravitational systems. These tests verify that fundamental
//! conservation laws are maintained over extended simulations.

use gravwell::prelude::*;

/// Conservation analysis results
#[derive(Debug)]
pub struct ConservationAnalysis {
    pub initial_linear_momentum: Vector3,
    pub final_linear_momentum: Vector3,
    pub initial_angular_momentum: Vector3,
    pub final_angular_momentum: Vector3,
    pub linear_momentum_error: f64,
    pub angular_momentum_error: f64,
    pub simulation_time: f64,
    pub total_steps: usize,
}

impl ConservationAnalysis {
    pub fn print_summary(&self) {
        println!("\n⚖️ Momentum Conservation Analysis");
        println!("=================================");
        println!(
            "Simulation Time: {:.2} years",
            self.simulation_time / (365.25 * 24.0 * 3600.0)
        );
        println!("Total Steps: {}", self.total_steps);

        println!("\n📊 Linear Momentum:");
        println!(
            "  Initial: [{:.2e}, {:.2e}, {:.2e}] kg⋅m/s",
            self.initial_linear_momentum.x,
            self.initial_linear_momentum.y,
            self.initial_linear_momentum.z
        );
        println!(
            "  Final:   [{:.2e}, {:.2e}, {:.2e}] kg⋅m/s",
            self.final_linear_momentum.x,
            self.final_linear_momentum.y,
            self.final_linear_momentum.z
        );
        println!("  Error: {:.2e} (relative)", self.linear_momentum_error);

        println!("\n🌀 Angular Momentum:");
        println!(
            "  Initial: [{:.2e}, {:.2e}, {:.2e}] kg⋅m²/s",
            self.initial_angular_momentum.x,
            self.initial_angular_momentum.y,
            self.initial_angular_momentum.z
        );
        println!(
            "  Final:   [{:.2e}, {:.2e}, {:.2e}] kg⋅m²/s",
            self.final_angular_momentum.x,
            self.final_angular_momentum.y,
            self.final_angular_momentum.z
        );
        println!("  Error: {:.2e} (relative)", self.angular_momentum_error);

        // Pass/fail assessment
        let linear_conserved = self.linear_momentum_error < 1e-12;
        let angular_conserved = self.angular_momentum_error < 1e-12;

        println!("\n🎯 Assessment:");
        println!(
            "  Linear Momentum Conservation (1e-12): {}",
            if linear_conserved {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        );
        println!(
            "  Angular Momentum Conservation (1e-12): {}",
            if angular_conserved {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        );

        let overall_pass = linear_conserved && angular_conserved;
        println!(
            "\n🏆 Overall: {}",
            if overall_pass {
                "✅ ALL CONSERVATION LAWS VERIFIED"
            } else {
                "❌ CONSERVATION VIOLATED"
            }
        );
    }
}

/// Calculate total linear momentum of the system
fn calculate_linear_momentum(
    sim: &gravwell::builder::Simulation<impl Integrator, impl ForceCalculator>,
) -> Vector3 {
    let particles = sim.particles();
    let mut momentum = Vector3::zeros();

    for i in 0..particles.len() {
        let velocity = *particles.velocity(i);
        let mass = particles.mass(i);
        momentum += mass * velocity;
    }

    momentum
}

/// Calculate total angular momentum about the center of mass
fn calculate_angular_momentum(
    sim: &gravwell::builder::Simulation<impl Integrator, impl ForceCalculator>,
) -> Vector3 {
    let particles = sim.particles();
    let center_of_mass = particles.center_of_mass();
    let mut angular_momentum = Vector3::zeros();

    for i in 0..particles.len() {
        let position = *particles.position(i) - center_of_mass;
        let velocity = *particles.velocity(i);
        let mass = particles.mass(i);

        angular_momentum += mass * position.cross(&velocity);
    }

    angular_momentum
}

/// Analyze momentum conservation over a simulation
fn analyze_momentum_conservation(
    sim: &gravwell::builder::Simulation<impl Integrator, impl ForceCalculator>,
    initial_linear: Vector3,
    initial_angular: Vector3,
    simulation_time: f64,
    total_steps: usize,
) -> ConservationAnalysis {
    let final_linear = calculate_linear_momentum(sim);
    let final_angular = calculate_angular_momentum(sim);

    let linear_momentum_error =
        (final_linear - initial_linear).magnitude() / (initial_linear.magnitude() + 1e-15); // Avoid division by zero

    let angular_momentum_error =
        (final_angular - initial_angular).magnitude() / initial_angular.magnitude();

    ConservationAnalysis {
        initial_linear_momentum: initial_linear,
        final_linear_momentum: final_linear,
        initial_angular_momentum: initial_angular,
        final_angular_momentum: final_angular,
        linear_momentum_error,
        angular_momentum_error,
        simulation_time,
        total_steps,
    }
}

#[cfg(test)]
mod momentum_tests {
    use super::*;

    #[test]
    fn test_two_body_momentum_conservation() {
        println!("\n🌍☀️ Testing two-body momentum conservation...");

        // Create Earth-Sun system
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

        let initial_linear = calculate_linear_momentum(&sim);
        let initial_angular = calculate_angular_momentum(&sim);

        // Simulate for 10 years
        let timestep = 3600.0; // 1 hour
        let years = 10;
        let total_steps = years * 365 * 24;
        let simulation_time = total_steps as f64 * timestep;

        println!(
            "Simulating {} years ({} steps) with 1h timesteps...",
            years, total_steps
        );

        for step in 0..total_steps {
            sim.step(timestep).expect("Simulation step failed");

            if step % (365 * 24) == 0 {
                // Every year
                let current_linear = calculate_linear_momentum(&sim);
                let linear_error = (current_linear - initial_linear).magnitude()
                    / (initial_linear.magnitude() + 1e-15);
                println!(
                    "  Year {}: Linear momentum error = {:.2e}",
                    step / (365 * 24),
                    linear_error
                );
            }
        }

        let analysis = analyze_momentum_conservation(
            &sim,
            initial_linear,
            initial_angular,
            simulation_time,
            total_steps,
        );
        analysis.print_summary();

        // Validate conservation
        assert!(
            analysis.linear_momentum_error < 1e-12,
            "Linear momentum not conserved: {:.2e} > 1e-12",
            analysis.linear_momentum_error
        );
        assert!(
            analysis.angular_momentum_error < 1e-12,
            "Angular momentum not conserved: {:.2e} > 1e-12",
            analysis.angular_momentum_error
        );

        println!("✅ Two-body momentum conservation validated!");
    }

    #[test]
    fn test_three_body_momentum_conservation() {
        println!("\n🪐 Testing three-body momentum conservation...");

        // Create Sun-Earth-Jupiter system
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
            // Jupiter
            .add_body(
                Body::new()
                    .with_mass(JUPITER_MASS)
                    .with_position([5.2 * AU, 0.0, 0.0])
                    .with_velocity([0.0, 13070.0, 0.0]),
            )
            .expect("Failed to add Jupiter")
            .build()
            .expect("Failed to build simulation");

        let initial_linear = calculate_linear_momentum(&sim);
        let initial_angular = calculate_angular_momentum(&sim);

        // Simulate for 5 years (shorter for 3-body system)
        let timestep = 86400.0; // 1 day
        let years = 5;
        let total_steps = years * 365;
        let simulation_time = total_steps as f64 * timestep;

        println!(
            "Simulating {} years ({} steps) with 1-day timesteps...",
            years, total_steps
        );

        for step in 0..total_steps {
            sim.step(timestep).expect("Simulation step failed");

            if step % 365 == 0 {
                // Every year
                let current_linear = calculate_linear_momentum(&sim);
                let linear_error = (current_linear - initial_linear).magnitude()
                    / (initial_linear.magnitude() + 1e-15);
                println!(
                    "  Year {}: Linear momentum error = {:.2e}",
                    step / 365,
                    linear_error
                );
            }
        }

        let analysis = analyze_momentum_conservation(
            &sim,
            initial_linear,
            initial_angular,
            simulation_time,
            total_steps,
        );
        analysis.print_summary();

        // Validate conservation (slightly relaxed for 3-body system)
        assert!(
            analysis.linear_momentum_error < 1e-10,
            "Linear momentum not conserved: {:.2e} > 1e-10",
            analysis.linear_momentum_error
        );
        assert!(
            analysis.angular_momentum_error < 1e-10,
            "Angular momentum not conserved: {:.2e} > 1e-10",
            analysis.angular_momentum_error
        );

        println!("✅ Three-body momentum conservation validated!");
    }

    #[test]
    fn test_momentum_conservation_velocity_verlet() {
        println!("\n⚖️ Testing momentum conservation with Velocity Verlet...");
        test_integrator_momentum_conservation_vv();
    }

    #[test]
    fn test_momentum_conservation_leapfrog() {
        println!("\n⚖️ Testing momentum conservation with Leapfrog...");
        test_integrator_momentum_conservation_lf();
    }

    #[test]
    fn test_momentum_conservation_rk4() {
        println!("\n⚖️ Testing momentum conservation with RK4...");
        test_integrator_momentum_conservation_rk4();
    }

    fn create_velocity_verlet_sim() -> gravwell::builder::Simulation<VelocityVerlet, DirectGravity>
    {
        SimulationBuilder::new()
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
            .expect("Failed to build simulation")
    }

    fn create_leapfrog_sim() -> gravwell::builder::Simulation<Leapfrog, DirectGravity> {
        SimulationBuilder::new()
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
            .expect("Failed to build simulation")
    }

    fn create_rk4_sim() -> gravwell::builder::Simulation<RungeKutta4, DirectGravity> {
        SimulationBuilder::new()
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
            .expect("Failed to build simulation")
    }

    fn test_integrator_momentum_conservation_vv() {
        let mut sim = create_velocity_verlet_sim();
        let initial_linear = calculate_linear_momentum(&sim);
        let initial_angular = calculate_angular_momentum(&sim);

        // Simulate for 3 years
        let timestep = 3600.0; // 1 hour
        let total_steps = 3 * 365 * 24;
        let simulation_time = total_steps as f64 * timestep;

        for _step in 0..total_steps {
            sim.step(timestep).expect("Simulation step failed");
        }

        let analysis = analyze_momentum_conservation(
            &sim,
            initial_linear,
            initial_angular,
            simulation_time,
            total_steps,
        );

        println!(
            "  Linear momentum error: {:.2e}",
            analysis.linear_momentum_error
        );
        println!(
            "  Angular momentum error: {:.2e}",
            analysis.angular_momentum_error
        );

        assert!(
            analysis.linear_momentum_error < 1e-12,
            "Linear momentum error too large: {:.2e} > 1e-12",
            analysis.linear_momentum_error
        );
        assert!(
            analysis.angular_momentum_error < 1e-12,
            "Angular momentum error too large: {:.2e} > 1e-12",
            analysis.angular_momentum_error
        );

        println!("✅ Velocity Verlet momentum conservation validated!");
    }

    fn test_integrator_momentum_conservation_lf() {
        let mut sim = create_leapfrog_sim();
        let initial_linear = calculate_linear_momentum(&sim);
        let initial_angular = calculate_angular_momentum(&sim);

        // Simulate for 3 years
        let timestep = 3600.0; // 1 hour
        let total_steps = 3 * 365 * 24;
        let simulation_time = total_steps as f64 * timestep;

        for _step in 0..total_steps {
            sim.step(timestep).expect("Simulation step failed");
        }

        let analysis = analyze_momentum_conservation(
            &sim,
            initial_linear,
            initial_angular,
            simulation_time,
            total_steps,
        );

        println!(
            "  Linear momentum error: {:.2e}",
            analysis.linear_momentum_error
        );
        println!(
            "  Angular momentum error: {:.2e}",
            analysis.angular_momentum_error
        );

        assert!(
            analysis.linear_momentum_error < 1e-12,
            "Linear momentum error too large: {:.2e} > 1e-12",
            analysis.linear_momentum_error
        );
        assert!(
            analysis.angular_momentum_error < 1e-12,
            "Angular momentum error too large: {:.2e} > 1e-12",
            analysis.angular_momentum_error
        );

        println!("✅ Leapfrog momentum conservation validated!");
    }

    fn test_integrator_momentum_conservation_rk4() {
        let mut sim = create_rk4_sim();
        let initial_linear = calculate_linear_momentum(&sim);
        let initial_angular = calculate_angular_momentum(&sim);

        // Simulate for 3 years
        let timestep = 3600.0; // 1 hour
        let total_steps = 3 * 365 * 24;
        let simulation_time = total_steps as f64 * timestep;

        for _step in 0..total_steps {
            sim.step(timestep).expect("Simulation step failed");
        }

        let analysis = analyze_momentum_conservation(
            &sim,
            initial_linear,
            initial_angular,
            simulation_time,
            total_steps,
        );

        println!(
            "  Linear momentum error: {:.2e}",
            analysis.linear_momentum_error
        );
        println!(
            "  Angular momentum error: {:.2e}",
            analysis.angular_momentum_error
        );

        assert!(
            analysis.linear_momentum_error < 1e-12,
            "Linear momentum error too large: {:.2e} > 1e-12",
            analysis.linear_momentum_error
        );
        assert!(
            analysis.angular_momentum_error < 1e-12,
            "Angular momentum error too large: {:.2e} > 1e-12",
            analysis.angular_momentum_error
        );

        println!("✅ RK4 momentum conservation validated!");
    }

    #[test]
    fn test_center_of_mass_stability() {
        println!("\n🎯 Testing center of mass stability...");

        // Create multi-body system with asymmetric initial conditions
        let mut sim = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(DirectGravity::new())
            // Central mass
            .add_body(
                Body::new()
                    .with_mass(SOLAR_MASS)
                    .with_position([1e10, 2e10, 3e10]) // Offset from origin
                    .with_velocity([1000.0, -500.0, 200.0]),
            ) // Non-zero velocity
            .expect("Failed to add central body")
            // Orbiting mass 1
            .add_body(
                Body::new()
                    .with_mass(EARTH_MASS)
                    .with_position([1e10 + AU, 2e10, 3e10])
                    .with_velocity([1000.0, -500.0 + 29785.0, 200.0]),
            )
            .expect("Failed to add orbiting body 1")
            // Orbiting mass 2
            .add_body(
                Body::new()
                    .with_mass(JUPITER_MASS)
                    .with_position([1e10 - AU, 2e10, 3e10])
                    .with_velocity([1000.0, -500.0 - 13070.0, 200.0]),
            )
            .expect("Failed to add orbiting body 2")
            .build()
            .expect("Failed to build simulation");

        let initial_com = sim.particles().center_of_mass();
        let initial_linear = calculate_linear_momentum(&sim);

        // Simulate for 2 years
        let timestep = 86400.0; // 1 day
        let total_steps = 2 * 365;

        println!(
            "Initial center of mass: [{:.2e}, {:.2e}, {:.2e}]",
            initial_com.x, initial_com.y, initial_com.z
        );

        for _step in 0..total_steps {
            sim.step(timestep).expect("Simulation step failed");
        }

        let final_com = sim.particles().center_of_mass();
        let final_linear = calculate_linear_momentum(&sim);

        println!(
            "Final center of mass:   [{:.2e}, {:.2e}, {:.2e}]",
            final_com.x, final_com.y, final_com.z
        );

        // In the absence of external forces, center of mass should move at constant velocity
        let expected_com_change = initial_linear * (total_steps as f64 * timestep)
            / sim.particles().masses().iter().sum::<f64>();
        let expected_final_com = initial_com + expected_com_change;

        let com_error = (final_com - expected_final_com).magnitude() / initial_com.magnitude();

        println!("Center of mass error: {:.2e}", com_error);

        // Center of mass motion should be predictable
        assert!(
            com_error < 1e-10,
            "Center of mass motion unpredictable: {:.2e} > 1e-10",
            com_error
        );

        // Linear momentum should still be conserved
        let momentum_error =
            (final_linear - initial_linear).magnitude() / initial_linear.magnitude();
        assert!(
            momentum_error < 1e-12,
            "Linear momentum not conserved: {:.2e} > 1e-12",
            momentum_error
        );

        println!("✅ Center of mass stability validated!");
    }
}
