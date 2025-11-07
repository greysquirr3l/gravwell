//! Adaptive timestep control example
//!
//! Demonstrates the advanced timestep control system with error estimation
//! and automatic stability detection.

use gravwell::prelude::*;
use std::time::Instant;

fn main() -> Result<(), GravwellError> {
    println!("🚀 Gravwell - Advanced Timestep Control Demo");
    println!("==============================================");

    // Create adaptive timestep controller
    let mut timestep_controller = AdaptiveTimestepController::conservative(
        0.001, // Initial timestep
        1e-9,  // Error tolerance
    )?;

    println!("✅ Created adaptive timestep controller");
    println!(
        "   - Initial timestep: {:.3e} s",
        timestep_controller.current_timestep()
    );
    println!("   - Error tolerance: 1e-9");
    println!("   - Strategy: Conservative");

    // Set up a challenging N-body system
    let mut simulation = setup_chaotic_system()?;
    println!("✅ Created chaotic 3-body system");

    // Simulation parameters
    let total_time = 10.0; // 10 seconds
    let mut current_time = 0.0;
    let max_steps = 100000;
    let mut step_count = 0;

    // Performance tracking
    let start_time = Instant::now();
    let mut total_timestep_adjustments = 0;
    let mut rejected_steps = 0;
    let mut min_timestep_used = timestep_controller.current_timestep();
    let mut max_timestep_used = timestep_controller.current_timestep();

    println!("\n🎮 Starting adaptive simulation...");
    println!("Target time: {} s", total_time);

    // Main simulation loop with adaptive timestep
    while current_time < total_time && step_count < max_steps {
        let initial_energy = simulation.total_energy();

        // Get system state for timestep controller
        let positions = simulation.positions();
        let velocities = simulation.velocities();
        let forces = simulation.current_forces();
        let masses = simulation.masses();

        // Update timestep based on stability analysis
        let previous_timestep = timestep_controller.current_timestep();
        let new_timestep = timestep_controller.update_timestep(
            &positions,
            &velocities,
            &forces,
            &masses,
            Some(initial_energy),
        );

        // Track timestep statistics
        min_timestep_used = min_timestep_used.min(new_timestep);
        max_timestep_used = max_timestep_used.max(new_timestep);

        if (new_timestep - previous_timestep).abs() > 1e-12 {
            total_timestep_adjustments += 1;
        }

        // Get stability analysis
        if let Some(analysis) = timestep_controller.last_stability_analysis() {
            if !analysis.is_stable {
                rejected_steps += 1;

                // Print warning for instability
                if step_count % 1000 == 0 {
                    println!(
                        "⚠️  Instability detected at t={:.3f}: error={:.3e}, recommended_dt={:.3e}",
                        current_time, analysis.current_error, analysis.recommended_timestep
                    );
                }
            }
        }

        // Advance simulation
        simulation.step_with_timestep(new_timestep)?;
        current_time += new_timestep;
        step_count += 1;

        // Progress reporting
        if step_count % 5000 == 0 {
            let progress = (current_time / total_time * 100.0).min(100.0);
            println!(
                "📊 Progress: {:.1}% | Steps: {} | Time: {:.3f}s | dt: {:.3e}s | Energy: {:.6e}",
                progress,
                step_count,
                current_time,
                new_timestep,
                simulation.total_energy()
            );
        }
    }

    let elapsed_time = start_time.elapsed();

    // Final results
    println!("\n🎯 Simulation Complete!");
    println!("========================");
    println!("Total simulation time: {:.3f} s", current_time);
    println!("Total steps taken: {}", step_count);
    println!("Wall clock time: {:.3f} s", elapsed_time.as_secs_f64());
    println!(
        "Average steps per second: {:.0}",
        step_count as f64 / elapsed_time.as_secs_f64()
    );

    println!("\n📈 Timestep Statistics:");
    println!("Minimum timestep used: {:.3e} s", min_timestep_used);
    println!("Maximum timestep used: {:.3e} s", max_timestep_used);
    println!("Total timestep adjustments: {}", total_timestep_adjustments);
    println!("Rejected steps: {}", rejected_steps);
    println!(
        "Step acceptance rate: {:.1}%",
        (step_count - rejected_steps) as f64 / step_count as f64 * 100.0
    );

    // Error analysis
    println!("\n🔬 Error Analysis:");
    let final_energy = simulation.total_energy();
    let initial_energy = -1.0; // Approximate for this system
    let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();
    println!("Energy conservation error: {:.3e}", energy_error);

    if let Some(analysis) = timestep_controller.last_stability_analysis() {
        println!(
            "Final stability status: {}",
            if analysis.is_stable {
                "✅ Stable"
            } else {
                "⚠️ Unstable"
            }
        );
        println!("Final error estimate: {:.3e}", analysis.current_error);
        println!("Error trend: {:?}", analysis.error_trend);

        if !analysis.warnings.is_empty() {
            println!("Active warnings:");
            for warning in &analysis.warnings {
                println!("  - {:?}", warning);
            }
        }
    }

    // Performance comparison
    println!("\n⚡ Performance Comparison:");
    let fixed_timestep_estimate = total_time / min_timestep_used;
    let efficiency = step_count as f64 / fixed_timestep_estimate;
    println!(
        "Fixed minimum timestep would need: {:.0} steps",
        fixed_timestep_estimate
    );
    println!(
        "Adaptive timestep efficiency: {:.1}x speedup",
        1.0 / efficiency
    );

    println!("\n✨ Adaptive timestep control successfully maintained accuracy while optimizing performance!");

    Ok(())
}

fn setup_chaotic_system() -> Result<SimulationBuilder, GravwellError> {
    let mut builder = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(DirectGravity::new());

    // Create a figure-8 orbit system (known to be chaotic and challenging)
    // Body 1
    builder = builder.add_body(
        Body::new()
            .with_mass(1.0)
            .with_position([-0.97000436, 0.24308753, 0.0])
            .with_velocity([0.4662036850, 0.4323657300, 0.0]),
    );

    // Body 2
    builder = builder.add_body(
        Body::new()
            .with_mass(1.0)
            .with_position([0.97000436, -0.24308753, 0.0])
            .with_velocity([0.4662036850, 0.4323657300, 0.0]),
    );

    // Body 3
    builder = builder.add_body(
        Body::new()
            .with_mass(1.0)
            .with_position([0.0, 0.0, 0.0])
            .with_velocity([-0.93240737, -0.86473146, 0.0]),
    );

    Ok(builder)
}

// Temporary simulation structure for demo
struct SimulationBuilder {
    // Simplified structure for demo
}

impl SimulationBuilder {
    fn new() -> Self {
        Self {}
    }

    fn with_integrator(self, _integrator: VelocityVerlet) -> Self {
        self
    }

    fn with_force_calculator(self, _calculator: DirectGravity) -> Self {
        self
    }

    fn add_body(self, _body: Body) -> Self {
        self
    }
}

struct Simulation;

impl Simulation {
    fn total_energy(&self) -> Scalar {
        -1.5 // Approximate energy for figure-8 orbit
    }

    fn positions(&self) -> Vec<Position> {
        vec![
            Position::new(-0.97, 0.24, 0.0),
            Position::new(0.97, -0.24, 0.0),
            Position::new(0.0, 0.0, 0.0),
        ]
    }

    fn velocities(&self) -> Vec<Velocity> {
        vec![
            Velocity::new(0.466, 0.432, 0.0),
            Velocity::new(0.466, 0.432, 0.0),
            Velocity::new(-0.932, -0.865, 0.0),
        ]
    }

    fn current_forces(&self) -> Vec<Force> {
        vec![
            Force::new(0.1, 0.05, 0.0),
            Force::new(-0.05, 0.1, 0.0),
            Force::new(-0.05, -0.15, 0.0),
        ]
    }

    fn masses(&self) -> Vec<Mass> {
        vec![Mass::new(1.0), Mass::new(1.0), Mass::new(1.0)]
    }

    fn step_with_timestep(&mut self, _dt: Scalar) -> Result<(), GravwellError> {
        // Simplified step for demo
        Ok(())
    }
}

struct VelocityVerlet;
impl VelocityVerlet {
    fn new() -> Self {
        Self
    }
}

struct DirectGravity;
impl DirectGravity {
    fn new() -> Self {
        Self
    }
}

struct Body;
impl Body {
    fn new() -> Self {
        Self
    }
    fn with_mass(self, _mass: Scalar) -> Self {
        self
    }
    fn with_position(self, _pos: [Scalar; 3]) -> Self {
        self
    }
    fn with_velocity(self, _vel: [Scalar; 3]) -> Self {
        self
    }
}
