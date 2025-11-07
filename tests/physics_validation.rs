// Quick Scientific Validation Test
//
// This test provides essential validation for Gravwell's scientific accuracy
// in a format compatible with the current API.

use gravwell::prelude::*;
use gravwell::SimulationBuilder;

#[test]
fn test_basic_physics_accuracy() {
    println!("🔬 Testing basic physics accuracy...");

    // Create Earth-Sun system
    let mut sim = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(DirectGravity::new())
        .add_body(
            Body::new()
                .with_mass(1.98847e30) // Solar mass in kg
                .with_position([0.0, 0.0, 0.0])
                .with_velocity([0.0, 0.0, 0.0]),
        )
        .expect("Failed to add Sun")
        .add_body(
            Body::new()
                .with_mass(5.97219e24) // Earth mass in kg
                .with_position([1.495978707e11, 0.0, 0.0]) // 1 AU in meters
                .with_velocity([0.0, 29780.0, 0.0]), // Earth orbital velocity
        )
        .expect("Failed to add Earth")
        .build()
        .expect("Failed to create simulation");
    let timestep = 3600.0; // 1 hour

    println!("✓ Earth-Sun system created");

    // Get handles for the bodies we added (Sun=0, Earth=1)
    let earth = sim.get_body_handle(1);

    // Store initial conditions
    let initial_energy = sim.total_energy();
    let initial_pos = sim.position(earth);
    let initial_vel = sim.velocity(earth);

    println!("Initial energy: {:.3e} J", initial_energy);
    println!(
        "Initial Earth position: ({:.3e}, {:.3e}, {:.3e}) m",
        initial_pos.x, initial_pos.y, initial_pos.z
    );

    // Simulate for 100 steps (roughly 4 days)
    for step in 0..100 {
        sim.step(timestep).unwrap();

        if step % 25 == 0 {
            let current_energy = sim.total_energy();
            let energy_drift = (current_energy - initial_energy).abs() / initial_energy.abs();
            println!("Step {}: Energy drift = {:.3e}", step, energy_drift);
        }
    }

    // Check final state
    let final_energy = sim.total_energy();
    let final_pos = sim.position(earth);
    let final_vel = sim.velocity(earth);

    // Energy conservation test
    let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();
    println!("Final energy drift: {:.3e}", energy_drift);
    assert!(
        energy_drift < 1e-10,
        "Energy not conserved: drift = {:.3e}",
        energy_drift
    );

    // Position should change (Earth is orbiting)
    let position_change = (final_pos - initial_pos).norm();
    println!("Position change: {:.3e} m", position_change);
    assert!(position_change > 1e6, "Earth should move during orbit");

    // Velocity should also change (circular motion)
    let velocity_change = (final_vel - initial_vel).norm();
    println!("Velocity change: {:.3e} m/s", velocity_change);
    assert!(
        velocity_change > 1e2,
        "Earth velocity should change during orbit"
    );

    // Distance from Sun should remain approximately 1 AU (circular orbit)
    let final_distance = final_pos.norm();
    let au_error = (final_distance - 1.495978707e11).abs() / 1.495978707e11;
    println!(
        "Distance from Sun: {:.3e} m (error: {:.3e})",
        final_distance, au_error
    );
    assert!(au_error < 0.01, "Earth should stay near 1 AU");

    println!("✅ All physics accuracy tests passed!");
}

#[test]
fn test_integrator_stability() {
    println!("⚖️  Testing integrator stability...");

    // Test Velocity Verlet integrator
    let name = "Velocity Verlet";
    let integrator = VelocityVerlet::new();
    println!("Testing integrator: {}", name);

    let mut sim = SimulationBuilder::new()
        .with_integrator(integrator)
        .with_force_calculator(DirectGravity::new())
        .add_body(
            Body::new()
                .with_mass(1.98847e30)
                .with_position([0.0, 0.0, 0.0])
                .with_velocity([0.0, 0.0, 0.0]),
        )
        .expect("Failed to add Sun")
        .add_body(
            Body::new()
                .with_mass(5.97219e24)
                .with_position([1.495978707e11, 0.0, 0.0])
                .with_velocity([0.0, 29780.0, 0.0]),
        )
        .expect("Failed to add Earth")
        .build()
        .expect("Failed to create simulation");
    let timestep = 1800.0; // 30 minutes

    let initial_energy = sim.total_energy();

    // Short simulation
    for _ in 0..50 {
        sim.step(timestep).unwrap();
    }

    let final_energy = sim.total_energy();
    let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();

    println!("  {} energy drift: {:.3e}", name, energy_drift);
    assert!(
        energy_drift < 1e-8,
        "{} energy drift too large: {:.3e}",
        name,
        energy_drift
    );

    println!("✅ All integrator stability tests passed!");
}

#[test]
fn test_force_calculation_accuracy() {
    println!("🎯 Testing force calculation accuracy...");

    // Two masses separated by exactly 1 meter
    let mass1 = 1.0e20; // kg
    let mass2 = 2.0e20; // kg
    let separation = 1.0; // meters

    // Create a simple system where we can verify the gravitational force analytically
    let mut sim = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(DirectGravity::new())
        .add_body(
            Body::new()
                .with_mass(mass1)
                .with_position([0.0, 0.0, 0.0])
                .with_velocity([0.0, 0.0, 0.0]),
        )
        .expect("Failed to add body1")
        .add_body(
            Body::new()
                .with_mass(mass2)
                .with_position([separation, 0.0, 0.0])
                .with_velocity([0.0, 0.0, 0.0]),
        )
        .expect("Failed to add body2")
        .build()
        .expect("Failed to create simulation");
    let _timestep = 1.0; // 1 second (doesn't matter for force test)

    // Get body handles
    let body1 = sim.get_body_handle(0);
    let body2 = sim.get_body_handle(1);

    // Take one step to calculate forces
    sim.step(_timestep).unwrap();

    // Calculate expected gravitational force: F = G * m1 * m2 / r^2
    let g_const = 6.67430e-11; // Gravitational constant
    let expected_force_magnitude = g_const * mass1 * mass2 / (separation * separation);

    // Get velocities after one timestep to infer forces (F = m * a, a = v / t)
    let v1 = sim.velocity(body1);
    let v2 = sim.velocity(body2);

    // Force on body1 should be in positive x direction
    let force1_magnitude = mass1 * v1.x / 1.0; // F = m * a = m * v / t
    let force2_magnitude = mass2 * (-v2.x) / 1.0; // Force on body2 is in negative x direction

    println!("Expected force: {:.3e} N", expected_force_magnitude);
    println!("Calculated force on body1: {:.3e} N", force1_magnitude);
    println!("Calculated force on body2: {:.3e} N", force2_magnitude);

    // Check that forces are approximately equal to expected value
    let error1 = (force1_magnitude - expected_force_magnitude).abs() / expected_force_magnitude;
    let error2 = (force2_magnitude - expected_force_magnitude).abs() / expected_force_magnitude;

    println!("Force error on body1: {:.3e}", error1);
    println!("Force error on body2: {:.3e}", error2);

    // Allow some numerical error due to integration
    assert!(
        error1 < 1e-10,
        "Force calculation error too large for body1: {:.3e}",
        error1
    );
    assert!(
        error2 < 1e-10,
        "Force calculation error too large for body2: {:.3e}",
        error2
    );

    // Forces should be equal and opposite (Newton's 3rd law)
    let force_balance_error =
        (force1_magnitude - force2_magnitude).abs() / expected_force_magnitude;
    println!("Force balance error: {:.3e}", force_balance_error);
    assert!(
        force_balance_error < 1e-12,
        "Forces not balanced: {:.3e}",
        force_balance_error
    );

    println!("✅ Force calculation accuracy verified!");
}

#[test]
fn test_multi_body_energy_conservation() {
    println!("🌌 Testing multi-body energy conservation...");

    // Moon relative to Earth
    let moon_distance = 3.844e8; // 384,400 km from Earth
    let moon_velocity = 1022.0; // Moon's orbital velocity around Earth

    let mut sim = SimulationBuilder::new()
        .with_integrator(VelocityVerlet::new())
        .with_force_calculator(DirectGravity::new())
        .add_body(
            Body::new()
                .with_mass(1.98847e30)
                .with_position([0.0, 0.0, 0.0])
                .with_velocity([0.0, 0.0, 0.0]),
        )
        .expect("Failed to add Sun")
        .add_body(
            Body::new()
                .with_mass(5.97219e24)
                .with_position([1.495978707e11, 0.0, 0.0])
                .with_velocity([0.0, 29780.0, 0.0]),
        )
        .expect("Failed to add Earth")
        .add_body(
            Body::new()
                .with_mass(7.342e22)
                .with_position([1.495978707e11 + moon_distance, 0.0, 0.0])
                .with_velocity([0.0, 29780.0 + moon_velocity, 0.0]),
        )
        .expect("Failed to add Moon")
        .build()
        .expect("Failed to create simulation");
    let timestep = 3600.0; // 1 hour

    let initial_energy = sim.total_energy();
    println!("Initial three-body system energy: {:.3e} J", initial_energy);

    // Simulate for several days
    let steps = 120; // 5 days
    for step in 0..steps {
        sim.step(timestep).unwrap();

        if step % 24 == 0 {
            let current_energy = sim.total_energy();
            let energy_drift = (current_energy - initial_energy).abs() / initial_energy.abs();
            println!("Day {}: Energy drift = {:.3e}", step / 24, energy_drift);
        }
    }

    let final_energy = sim.total_energy();
    let total_energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();

    println!("Final three-body energy drift: {:.3e}", total_energy_drift);
    assert!(
        total_energy_drift < 1e-9,
        "Three-body energy drift too large: {:.3e}",
        total_energy_drift
    );

    println!("✅ Multi-body energy conservation verified!");
}
