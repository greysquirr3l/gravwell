//! Advanced Adaptive Timestep Control Demonstration
//!
//! This example showcases Gravwell's integrated adaptive timestep control system
//! using the new SimulationBuilder integration with comprehensive error metrics
//! and adaptation strategies.

use gravwell::adaptive::{AdaptationStrategy, AdaptiveTimestepController, ErrorMetric};
use gravwell::prelude::*;
use std::time::Instant;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    println!("🎯 Gravwell Advanced Adaptive Timestep Demo");
    println!("===========================================");

    demo_integrated_adaptive_controller()?;
    demo_error_metrics_comparison()?;
    demo_scientific_precision()?;

    println!("\n✅ All advanced adaptive demos completed successfully!");
    Ok(())
}

/// Demonstrate the new integrated adaptive timestep controller.
fn demo_integrated_adaptive_controller() -> Result<()> {
    println!("\n🚀 Demo 1: Integrated Adaptive Controller");
    println!("------------------------------------------");

    // Create an adaptive controller using the new integrated system
    let adaptive_controller = AdaptiveTimestepController::conservative(
        0.01,  // Initial timestep
        1e-10, // Very tight error tolerance
    )?;

    // Create simulation with integrated adaptive timestep
    let mut simulation = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(DirectGravity::new())
        .with_adaptive_timestep(adaptive_controller) // New integrated feature!
        .add_body(
            Body::new()
                .with_mass(1.0)
                .with_position([0.0, 0.0, 0.0])
                .with_velocity([0.0, 0.0, 0.0])
                .with_radius(0.1),
        )?
        .add_body(
            Body::new()
                .with_mass(1.0)
                .with_position([1.0, 0.0, 0.0])
                .with_velocity([0.0, 1.0, 0.0])
                .with_radius(0.1),
        )?
        .build()?;

    let initial_energy = simulation.total_energy();
    println!("Initial energy: {:.6e}", initial_energy);
    println!(
        "Initial timestep: {:.6}",
        simulation.current_adaptive_timestep().unwrap()
    );

    let start_time = Instant::now();
    let mut total_time = 0.0;
    let mut step_count = 0;
    let mut timestep_adjustments = 0;
    let mut previous_timestep = simulation.current_adaptive_timestep().unwrap();

    // Run simulation using the new step_adaptive() method
    for i in 0..50 {
        let actual_timestep = simulation.step_adaptive()?;
        total_time += actual_timestep;
        step_count += 1;

        // Track timestep adjustments
        if (actual_timestep - previous_timestep).abs() > 1e-10 {
            timestep_adjustments += 1;
        }
        previous_timestep = actual_timestep;

        if i % 10 == 0 {
            let current_energy = simulation.total_energy();
            let energy_drift = (current_energy - initial_energy).abs() / initial_energy.abs();

            println!(
                "Step {}: dt={:.3e}, time={:.3}, energy_drift={:.3e}",
                i + 1,
                actual_timestep,
                total_time,
                energy_drift
            );
        }
    }

    let elapsed = start_time.elapsed();
    let final_energy = simulation.total_energy();
    let energy_conservation = (final_energy - initial_energy).abs() / initial_energy.abs();

    println!("\nIntegrated Controller Results:");
    println!("  Total steps: {}", step_count);
    println!("  Total simulation time: {:.6}", total_time);
    println!("  Wall clock time: {:.3}ms", elapsed.as_millis());
    println!(
        "  Final timestep: {:.6}",
        simulation.current_adaptive_timestep().unwrap()
    );
    println!("  Timestep adjustments: {}", timestep_adjustments);
    println!("  Energy conservation: {:.3e}", energy_conservation);

    if energy_conservation < 1e-12 {
        println!("✅ Exceptional energy conservation!");
    } else if energy_conservation < 1e-10 {
        println!("✅ Excellent energy conservation!");
    } else {
        println!("✅ Good energy conservation");
    }

    Ok(())
}

/// Compare different error metrics with the integrated system.
fn demo_error_metrics_comparison() -> Result<()> {
    println!("\n📊 Demo 2: Error Metrics Comparison");
    println!("------------------------------------");

    let metrics = vec![
        ("Position", ErrorMetric::Position),
        ("Velocity", ErrorMetric::Velocity),
        ("Energy", ErrorMetric::Energy),
        ("Combined", ErrorMetric::Combined),
    ];

    for (name, metric) in metrics {
        println!("\nTesting {} error metric:", name);

        // Create controller with specific error metric
        let adaptive_controller = AdaptiveTimestepController::new(
            0.01,                         // Initial timestep
            1e-6,                         // Min timestep
            0.1,                          // Max timestep
            1e-9,                         // Error tolerance
            metric,                       // Error metric
            AdaptationStrategy::Balanced, // Adaptation strategy
        )?;

        // Create a binary star system
        let mut simulation = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(DirectGravity::new())
            .with_adaptive_timestep(adaptive_controller)
            .add_body(
                Body::new()
                    .with_mass(2.0)
                    .with_position([-0.5, 0.0, 0.0])
                    .with_velocity([0.0, -0.7, 0.0])
                    .with_radius(0.1),
            )?
            .add_body(
                Body::new()
                    .with_mass(1.0)
                    .with_position([1.0, 0.0, 0.0])
                    .with_velocity([0.0, 1.4, 0.0])
                    .with_radius(0.1),
            )?
            .build()?;

        let initial_energy = simulation.total_energy();
        let mut timestep_sum = 0.0;
        let mut min_timestep = f64::INFINITY;
        let mut max_timestep = 0.0f64;
        let step_count = 25;

        // Run simulation and collect timestep statistics
        for _ in 0..step_count {
            let dt = simulation.step_adaptive()?;
            timestep_sum += dt;
            min_timestep = min_timestep.min(dt);
            max_timestep = max_timestep.max(dt);
        }

        let final_energy = simulation.total_energy();
        let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();

        println!(
            "  Average timestep: {:.6}",
            timestep_sum / step_count as f64
        );
        println!(
            "  Min/Max timestep: {:.6} / {:.6}",
            min_timestep, max_timestep
        );
        println!("  Timestep range: {:.2}x", max_timestep / min_timestep);
        println!("  Energy error: {:.3e}", energy_error);

        // Access the controller to get additional statistics
        if let Some(controller) = simulation.adaptive_controller() {
            if let Some(analysis) = controller.last_stability_analysis() {
                println!("  Last error estimate: {:.3e}", analysis.current_error);
                println!(
                    "  Stability status: {}",
                    if analysis.is_stable {
                        "✅ Stable"
                    } else {
                        "⚠️ Unstable"
                    }
                );
            }
        }
    }

    Ok(())
}

/// Demonstrate scientific-precision simulation.
fn demo_scientific_precision() -> Result<()> {
    println!("\n🔬 Demo 3: Scientific Precision");
    println!("--------------------------------");

    // Create ultra-high precision controller for scientific computing
    let adaptive_controller = AdaptiveTimestepController::new(
        1e-5,                             // Very small initial timestep
        1e-12,                            // Extremely small minimum timestep
        1e-3,                             // Conservative maximum timestep
        1e-14,                            // Machine precision error tolerance
        ErrorMetric::Energy,              // Energy conservation focus
        AdaptationStrategy::Conservative, // Conservative adaptation
    )?;

    // Create a three-body system (restricted three-body problem)
    let mut simulation = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(DirectGravity::new())
        .with_adaptive_timestep(adaptive_controller)
        .add_body(
            Body::new() // Primary (like Sun)
                .with_mass(100.0)
                .with_position([0.0, 0.0, 0.0])
                .with_velocity([0.0, 0.0, 0.0])
                .with_radius(0.1),
        )?
        .add_body(
            Body::new() // Secondary (like Jupiter)
                .with_mass(1.0)
                .with_position([5.0, 0.0, 0.0])
                .with_velocity([0.0, 4.47, 0.0]) // Circular orbit velocity
                .with_radius(0.05),
        )?
        .add_body(
            Body::new() // Test particle (like asteroid)
                .with_mass(1e-6)
                .with_position([3.0, 0.0, 0.0])
                .with_velocity([0.0, 5.77, 0.0]) // Slightly different velocity
                .with_radius(0.01),
        )?
        .build()?;

    let initial_energy = simulation.total_energy();
    println!("Initial system energy: {:.12e}", initial_energy);
    println!(
        "Initial timestep: {:.3e}",
        simulation.current_adaptive_timestep().unwrap()
    );

    let start_time = Instant::now();
    let mut total_time = 0.0;
    let mut step_count = 0;
    let mut min_timestep = f64::INFINITY;
    let mut max_timestep = 0.0f64;
    let mut energy_errors = Vec::new();

    // Long-term integration for scientific accuracy assessment
    while total_time < 10.0 && step_count < 50000 {
        let dt = simulation.step_adaptive()?;
        total_time += dt;
        step_count += 1;

        min_timestep = min_timestep.min(dt);
        max_timestep = max_timestep.max(dt);

        // Track energy conservation over time
        if step_count % 1000 == 0 {
            let current_energy = simulation.total_energy();
            let energy_error = (current_energy - initial_energy).abs() / initial_energy.abs();
            energy_errors.push(energy_error);

            println!(
                "Step {}: t={:.3}, dt={:.3e}, energy_error={:.3e}",
                step_count, total_time, dt, energy_error
            );
        }

        // Stop if we're taking too long
        if start_time.elapsed().as_secs() > 5 {
            println!("  (Stopping after 5 seconds for demo purposes)");
            break;
        }
    }

    let elapsed = start_time.elapsed();
    let final_energy = simulation.total_energy();
    let total_energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();

    println!("\nScientific Precision Results:");
    println!("  Total steps: {}", step_count);
    println!("  Simulation time: {:.6}", total_time);
    println!("  Computation time: {:.3}s", elapsed.as_secs_f64());
    println!(
        "  Steps per second: {:.0}",
        step_count as f64 / elapsed.as_secs_f64()
    );
    println!(
        "  Timestep range: {:.3e} to {:.3e}",
        min_timestep, max_timestep
    );
    println!("  Adaptation ratio: {:.1}x", max_timestep / min_timestep);
    println!("  Final energy error: {:.3e}", total_energy_error);

    // Analyze energy conservation trend
    if energy_errors.len() > 1 {
        let max_error = energy_errors.iter().copied().fold(0.0f64, f64::max);
        let final_error = energy_errors[energy_errors.len() - 1];

        println!("  Maximum energy error: {:.3e}", max_error);
        println!(
            "  Energy error growth: {:.2}x",
            final_error / energy_errors[0]
        );
    }

    // Scientific quality assessment
    if total_energy_error < 1e-14 {
        println!("🏆 Machine precision energy conservation achieved!");
    } else if total_energy_error < 1e-12 {
        println!("✅ Exceptional scientific precision!");
    } else if total_energy_error < 1e-10 {
        println!("✅ Excellent scientific quality");
    } else if total_energy_error < 1e-8 {
        println!("✅ Good scientific accuracy");
    } else {
        println!("⚠️  Consider tighter error tolerance for scientific work");
    }

    // Access controller for detailed analysis
    if let Some(controller) = simulation.adaptive_controller() {
        println!("\nController Statistics:");
        println!(
            "  Error history length: {}",
            controller.error_history().len()
        );
        println!("  Step count: {}", controller.step_count());

        if let Some(analysis) = controller.last_stability_analysis() {
            println!("  Current error estimate: {:.3e}", analysis.current_error);
            println!(
                "  Recommended timestep: {:.3e}",
                analysis.recommended_timestep
            );
            println!("  Warnings: {}", analysis.warnings.len());

            for warning in &analysis.warnings {
                println!("    {:?}", warning);
            }
        }
    }

    Ok(())
}
