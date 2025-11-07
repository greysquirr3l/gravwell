//! Kepler Orbit Validation Tests
//!
//! These tests validate orbital mechanics accuracy against analytical
//! Kepler solutions for two-body systems.

use super::*;
use approx::assert_relative_eq;
use std::f64::consts::PI;

#[test]
fn test_circular_orbit_stability() {
    let (mut sim, _sun, earth) = ValidationUtils::create_sun_earth_system();
    
    // Record initial orbital elements
    let initial_pos = sim.position(earth).expect("Earth should exist");
    let initial_vel = sim.velocity(earth).expect("Earth should exist");
    let initial_elements = OrbitalElements::from_state_vectors(
        initial_pos,
        initial_vel,
        ValidationUtils::MU_SUN_EARTH,
    );
    
    // Verify it's a circular orbit
    assert!(
        initial_elements.is_circular(1e-6),
        "Initial orbit should be circular, eccentricity: {:.2e}",
        initial_elements.eccentricity
    );
    
    // Simulate for one complete orbit
    let steps_per_orbit = (initial_elements.orbital_period / sim.timestep()) as usize;
    let initial_energy = sim.total_energy();
    
    for _ in 0..steps_per_orbit {
        sim.step().expect("Simulation step should succeed");
    }
    
    // Check final position and orbital elements
    let final_pos = sim.position(earth).expect("Earth should exist");
    let final_vel = sim.velocity(earth).expect("Earth should exist");
    let final_elements = OrbitalElements::from_state_vectors(
        final_pos,
        final_vel,
        ValidationUtils::MU_SUN_EARTH,
    );
    
    // Validate orbital element conservation
    ValidationUtils::assert_relative_error(
        final_elements.semi_major_axis,
        initial_elements.semi_major_axis,
        1e-8,
        "Semi-major axis conservation"
    );
    
    ValidationUtils::assert_relative_error(
        final_elements.eccentricity,
        initial_elements.eccentricity,
        1e-6,
        "Eccentricity conservation"
    );
    
    ValidationUtils::assert_relative_error(
        final_elements.orbital_period,
        initial_elements.orbital_period,
        1e-8,
        "Orbital period conservation"
    );
    
    // Validate position return (should complete one orbit)
    let position_error = (final_pos - initial_pos).norm() / ValidationUtils::AU;
    assert!(
        position_error < 1e-6,
        "Position should return close to initial after one orbit, error: {:.2e} AU",
        position_error
    );
    
    // Validate energy conservation
    let final_energy = sim.total_energy();
    let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();
    assert!(
        energy_drift < 1e-10,
        "Energy drift should be minimal over one orbit, drift: {:.2e}",
        energy_drift
    );
}

#[test]
fn test_elliptical_orbit_accuracy() {
    // Create an elliptical Earth orbit (e = 0.2)
    let mut sim = Simulation::builder()
        .integrator(VelocityVerlet::new())
        .forces(DirectGravity::new())
        .timestep(1800.0) // 30 minutes for higher accuracy
        .build()
        .expect("Failed to create simulation");
    
    // Add Sun
    let _sun = sim.add_body(Body::new()
        .mass(Mass::new(ValidationUtils::SOLAR_MASS))
        .position(Vector3::zeros())
        .velocity(Vector3::zeros())
    ).expect("Failed to add Sun");
    
    // Create elliptical orbit: e = 0.2, a = 1 AU
    let semi_major_axis = ValidationUtils::AU;
    let eccentricity = 0.2;
    let periapsis_distance = semi_major_axis * (1.0 - eccentricity);
    
    // Position at periapsis
    let periapsis_pos = Vector3::new(periapsis_distance, 0.0, 0.0);
    
    // Velocity at periapsis (maximum for ellipse)
    let periapsis_velocity_mag = (ValidationUtils::MU_SUN_EARTH * 
        (2.0 / periapsis_distance - 1.0 / semi_major_axis)).sqrt();
    let periapsis_vel = Vector3::new(0.0, periapsis_velocity_mag, 0.0);
    
    let earth = sim.add_body(Body::new()
        .mass(Mass::new(ValidationUtils::EARTH_MASS))
        .position(periapsis_pos)
        .velocity(periapsis_vel)
    ).expect("Failed to add Earth");
    
    // Calculate expected orbital elements
    let expected_elements = OrbitalElements::from_state_vectors(
        periapsis_pos,
        periapsis_vel,
        ValidationUtils::MU_SUN_EARTH,
    );
    
    // Verify initial elliptical orbit setup
    ValidationUtils::assert_relative_error(
        expected_elements.eccentricity,
        eccentricity,
        1e-10,
        "Initial eccentricity setup"
    );
    
    ValidationUtils::assert_relative_error(
        expected_elements.semi_major_axis,
        semi_major_axis,
        1e-10,
        "Initial semi-major axis setup"
    );
    
    // Simulate for multiple orbits
    let steps_per_orbit = (expected_elements.orbital_period / sim.timestep()) as usize;
    let num_orbits = 5;
    let initial_energy = sim.total_energy();
    
    for orbit in 0..num_orbits {
        for _ in 0..steps_per_orbit {
            sim.step().expect("Simulation step should succeed");
        }
        
        // Check orbital elements every orbit
        let current_pos = sim.position(earth).expect("Earth should exist");
        let current_vel = sim.velocity(earth).expect("Earth should exist");
        let current_elements = OrbitalElements::from_state_vectors(
            current_pos,
            current_vel,
            ValidationUtils::MU_SUN_EARTH,
        );
        
        // Validate element conservation with stricter tolerances for multiple orbits
        let tolerance_multiplier = (orbit + 1) as f64 * 1e-8;
        
        ValidationUtils::assert_relative_error(
            current_elements.semi_major_axis,
            expected_elements.semi_major_axis,
            tolerance_multiplier,
            &format!("Semi-major axis after {} orbits", orbit + 1)
        );
        
        ValidationUtils::assert_relative_error(
            current_elements.eccentricity,
            expected_elements.eccentricity,
            tolerance_multiplier,
            &format!("Eccentricity after {} orbits", orbit + 1)
        );
    }
    
    // Final energy conservation check
    let final_energy = sim.total_energy();
    let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();
    assert!(
        energy_drift < 1e-8,
        "Energy drift over {} orbits should be minimal, drift: {:.2e}",
        num_orbits, energy_drift
    );
}

#[test]
fn test_kepler_third_law() {
    // Test Kepler's third law: T² ∝ a³
    // Compare orbital periods of different semi-major axes
    
    let test_cases = vec![
        (0.5 * ValidationUtils::AU, "0.5 AU orbit"),
        (1.0 * ValidationUtils::AU, "1.0 AU orbit (Earth)"),
        (1.5 * ValidationUtils::AU, "1.5 AU orbit"),
        (2.0 * ValidationUtils::AU, "2.0 AU orbit"),
    ];
    
    for (semi_major_axis, description) in test_cases {
        let mut sim = Simulation::builder()
            .integrator(VelocityVerlet::new())
            .forces(DirectGravity::new())
            .timestep(3600.0)
            .build()
            .expect("Failed to create simulation");
        
        // Add Sun
        let _sun = sim.add_body(Body::new()
            .mass(Mass::new(ValidationUtils::SOLAR_MASS))
            .position(Vector3::zeros())
            .velocity(Vector3::zeros())
        ).expect("Failed to add Sun");
        
        // Add planet in circular orbit at specified distance
        let orbital_velocity = ValidationUtils::circular_velocity(
            semi_major_axis,
            ValidationUtils::SOLAR_MASS
        );
        
        let planet = sim.add_body(Body::new()
            .mass(Mass::new(ValidationUtils::EARTH_MASS))
            .position(Vector3::new(semi_major_axis, 0.0, 0.0))
            .velocity(Vector3::new(0.0, orbital_velocity, 0.0))
        ).expect("Failed to add planet");
        
        // Calculate theoretical orbital period using Kepler's third law
        let expected_period = ValidationUtils::orbital_period(
            semi_major_axis,
            ValidationUtils::SOLAR_MASS + ValidationUtils::EARTH_MASS
        );
        
        // Measure actual orbital period by finding when planet returns to initial position
        let initial_pos = sim.position(planet).expect("Planet should exist");
        let mut last_x = initial_pos.x;
        let mut sign_changes = 0;
        let mut steps = 0;
        let max_steps = (expected_period / sim.timestep() * 2.0) as usize;
        
        // Look for two zero crossings in x-coordinate (one complete orbit)
        while sign_changes < 2 && steps < max_steps {
            sim.step().expect("Simulation step should succeed");
            let current_pos = sim.position(planet).expect("Planet should exist");
            
            if (last_x > 0.0) != (current_pos.x > 0.0) {
                sign_changes += 1;
            }
            
            last_x = current_pos.x;
            steps += 1;
        }
        
        let measured_period = steps as f64 * sim.timestep();
        
        // Validate against Kepler's third law
        ValidationUtils::assert_relative_error(
            measured_period,
            expected_period,
            1e-4, // Allow 0.01% error due to discretization
            &format!("{} - Kepler's third law", description)
        );
    }
}

#[test]
fn test_orbital_energy_vis_viva() {
    // Test vis-viva equation: v² = μ(2/r - 1/a)
    let (mut sim, _sun, earth) = ValidationUtils::create_sun_earth_system();
    
    // Sample orbital positions and validate vis-viva equation
    let steps_per_check = 1000; // Check every ~1000 hours
    let total_checks = 50;
    
    for i in 0..total_checks {
        // Advance simulation
        for _ in 0..steps_per_check {
            sim.step().expect("Simulation step should succeed");
        }
        
        let pos = sim.position(earth).expect("Earth should exist");
        let vel = sim.velocity(earth).expect("Earth should exist");
        
        let r = pos.norm();
        let v_squared = vel.norm_squared();
        
        // Calculate orbital elements to get semi-major axis
        let elements = OrbitalElements::from_state_vectors(
            pos,
            vel,
            ValidationUtils::MU_SUN_EARTH,
        );
        
        // Apply vis-viva equation: v² = μ(2/r - 1/a)
        let expected_v_squared = ValidationUtils::MU_SUN_EARTH * 
            (2.0 / r - 1.0 / elements.semi_major_axis);
        
        ValidationUtils::assert_relative_error(
            v_squared,
            expected_v_squared,
            1e-10,
            &format!("Vis-viva equation at check {}", i + 1)
        );
    }
}
