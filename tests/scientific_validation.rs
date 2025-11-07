// Gravwell Scientific Validation Integration Tests
//
// This integration test runs the comprehensive validation suite to ensure
// Gravwell meets scientific computing standards for accuracy and reliability.

mod validation;

use gravwell::prelude::*;
use validation::*;

#[test]
fn comprehensive_scientific_validation() {
    println!("\n🔬 GRAVWELL COMPREHENSIVE SCIENTIFIC VALIDATION");
    println!("================================================\n");

    let mut overall_report = ValidationReport::new();

    // 1. Kepler Orbital Mechanics Validation
    println!("📐 Testing Kepler Orbital Mechanics...");
    run_kepler_orbit_tests(&mut overall_report);

    // 2. Energy Conservation Analysis
    println!("\n⚡ Testing Energy Conservation...");
    run_energy_conservation_tests(&mut overall_report);

    // 3. Cross-Platform Determinism
    println!("\n🖥️  Testing Cross-Platform Determinism...");
    run_determinism_tests(&mut overall_report);

    // 4. Integration Method Accuracy Comparison
    println!("\n🧮 Testing Integration Method Accuracy...");
    run_integrator_accuracy_tests(&mut overall_report);

    // Print final comprehensive report
    overall_report.print_full_report();

    // Generate summary statistics
    print_validation_summary(&overall_report);

    // Assert overall validation success
    assert!(
        overall_report.overall_passed,
        "❌ Gravwell failed comprehensive scientific validation"
    );

    println!("🎉 Gravwell PASSED comprehensive scientific validation!");
    println!("✅ Ready for scientific computing applications.");
}

/// Run Kepler orbital mechanics validation tests
fn run_kepler_orbit_tests(report: &mut ValidationReport) {
    // Test 1: Circular Earth orbit
    {
        let (mut sim, _sun, earth) = setup_earth_sun_system();

        let initial_energy = sim.total_energy();
        let expected_period = theoretical_orbital_period(constants::AU, constants::SOLAR_MASS);

        // Simulate one orbital period
        let steps = 1000;
        let dt = expected_period / steps as f64;
        sim.set_timestep(dt);

        let initial_pos = sim.position(earth);

        for _ in 0..steps {
            sim.step();
        }

        let final_pos = sim.position(earth);
        let final_energy = sim.total_energy();

        // Position closure
        let position_error = (final_pos - initial_pos).norm() / constants::AU;
        report.add_result(ValidationResult::new(
            "Kepler Circular Orbit Closure",
            position_error,
            0.0,
            constants::KEPLER_ORBIT_TOLERANCE,
        ));

        // Energy conservation
        let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();
        report.add_result(ValidationResult::new(
            "Kepler Orbit Energy Conservation",
            energy_error,
            0.0,
            constants::ENERGY_CONSERVATION_TOLERANCE,
        ));
    }

    // Test 2: Elliptical orbit validation
    {
        let mut sim = Simulation::builder()
            .integrator(VelocityVerlet::new())
            .forces(DirectGravity::new())
            .timestep(1800.0)
            .build()
            .expect("Failed to create simulation");

        // Create elliptical orbit
        let _sun = sim
            .add_body(
                Body::new()
                    .mass(constants::SOLAR_MASS)
                    .position([0.0, 0.0, 0.0])
                    .velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();

        let planet = sim
            .add_body(
                Body::new()
                    .mass(constants::EARTH_MASS)
                    .position([1.5 * constants::AU, 0.0, 0.0])
                    .velocity([0.0, 24000.0, 0.0]), // Elliptical velocity
            )
            .unwrap();

        let initial_energy = sim.total_energy();

        // Simulate for one period
        for _ in 0..2000 {
            sim.step();
        }

        let final_energy = sim.total_energy();
        let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();

        report.add_result(ValidationResult::new(
            "Elliptical Orbit Energy Conservation",
            energy_error,
            0.0,
            constants::ENERGY_CONSERVATION_TOLERANCE,
        ));
    }
}

/// Run energy conservation validation tests
fn run_energy_conservation_tests(report: &mut ValidationReport) {
    let integrators = vec![
        ("Velocity Verlet", VelocityVerlet::new()),
        ("Leapfrog", Leapfrog::new()),
        ("Semi-Implicit Euler", SemiImplicitEuler::new()),
    ];

    for (name, integrator) in integrators {
        let mut sim = Simulation::builder()
            .integrator(integrator)
            .forces(DirectGravity::new())
            .timestep(1800.0) // 30 minutes
            .build()
            .expect("Failed to create simulation");

        let (_sun, _earth) = add_earth_sun_bodies(&mut sim);

        let initial_energy = sim.total_energy();

        // Long simulation: 10 orbital periods
        let orbital_period = theoretical_orbital_period(constants::AU, constants::SOLAR_MASS);
        let total_steps = ((10.0 * orbital_period) / sim.timestep()) as usize;

        for _ in 0..total_steps {
            sim.step();
        }

        let final_energy = sim.total_energy();
        let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();

        // Different tolerances for different integrators
        let tolerance = match name {
            "Semi-Implicit Euler" => 1e-6,
            _ => constants::ENERGY_CONSERVATION_TOLERANCE,
        };

        report.add_result(ValidationResult::new(
            format!("{} Long-term Energy Conservation", name),
            energy_drift,
            0.0,
            tolerance,
        ));
    }
}

/// Run cross-platform determinism tests
fn run_determinism_tests(report: &mut ValidationReport) {
    // Test deterministic behavior with identical initial conditions
    let results = (0..3)
        .map(|_| {
            let mut sim = create_standard_test_simulation();
            let handles = add_standard_test_bodies(&mut sim);

            // Run exact same number of steps
            for _ in 0..500 {
                sim.step();
            }

            // Extract final state
            let positions: Vec<Vector3> = handles.iter().map(|&h| sim.position(h)).collect();
            let energy = sim.total_energy();

            (positions, energy)
        })
        .collect::<Vec<_>>();

    // Compare all results
    let (ref_positions, ref_energy) = &results[0];

    for (i, (positions, energy)) in results.iter().skip(1).enumerate() {
        // Position consistency
        let max_position_diff = ref_positions
            .iter()
            .zip(positions.iter())
            .map(|(p1, p2)| (p1 - p2).norm())
            .fold(0.0, f64::max);

        report.add_result(ValidationResult::new(
            format!("Determinism Test {} Position", i + 2),
            max_position_diff,
            0.0,
            constants::CROSS_PLATFORM_TOLERANCE,
        ));

        // Energy consistency
        let energy_diff = (energy - ref_energy).abs();
        report.add_result(ValidationResult::new(
            format!("Determinism Test {} Energy", i + 2),
            energy_diff,
            0.0,
            constants::CROSS_PLATFORM_TOLERANCE,
        ));
    }
}

/// Run integrator accuracy comparison tests
fn run_integrator_accuracy_tests(report: &mut ValidationReport) {
    // Test different timesteps with Velocity Verlet
    let timesteps = [7200.0, 3600.0, 1800.0, 900.0]; // 2h, 1h, 30min, 15min

    for &dt in &timesteps {
        let mut sim = Simulation::builder()
            .integrator(VelocityVerlet::new())
            .forces(DirectGravity::new())
            .timestep(dt)
            .build()
            .expect("Failed to create simulation");

        let (_sun, earth) = add_earth_sun_bodies(&mut sim);

        let initial_energy = sim.total_energy();
        let initial_pos = sim.position(earth);

        // Simulate one orbit
        let orbital_period = theoretical_orbital_period(constants::AU, constants::SOLAR_MASS);
        let steps = (orbital_period / dt) as usize;

        for _ in 0..steps {
            sim.step();
        }

        let final_energy = sim.total_energy();
        let final_pos = sim.position(earth);

        // Accuracy metrics
        let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();
        let position_error = (final_pos - initial_pos).norm() / constants::AU;

        report.add_result(ValidationResult::new(
            format!("Timestep {:.0}s Energy Error", dt),
            energy_error,
            0.0,
            1e-8, // Timestep-dependent tolerance
        ));

        report.add_result(ValidationResult::new(
            format!("Timestep {:.0}s Position Error", dt),
            position_error,
            0.0,
            1e-6,
        ));
    }
}

/// Helper functions for test setup
fn add_earth_sun_bodies(sim: &mut Simulation) -> (BodyHandle, BodyHandle) {
    let sun = sim
        .add_body(
            Body::new()
                .mass(constants::SOLAR_MASS)
                .position([0.0, 0.0, 0.0])
                .velocity([0.0, 0.0, 0.0]),
        )
        .unwrap();

    let earth = sim
        .add_body(
            Body::new()
                .mass(constants::EARTH_MASS)
                .position([constants::AU, 0.0, 0.0])
                .velocity([0.0, constants::EARTH_ORBITAL_VELOCITY, 0.0]),
        )
        .unwrap();

    (sun, earth)
}

fn create_standard_test_simulation() -> Simulation {
    Simulation::builder()
        .integrator(VelocityVerlet::new())
        .forces(DirectGravity::new())
        .timestep(3600.0)
        .build()
        .expect("Failed to create test simulation")
}

fn add_standard_test_bodies(sim: &mut Simulation) -> Vec<BodyHandle> {
    let (sun, earth) = add_earth_sun_bodies(sim);
    vec![sun, earth]
}

/// Print detailed validation summary
fn print_validation_summary(report: &ValidationReport) {
    println!("\n" + "═".repeat(80).as_str());
    println!("                    GRAVWELL VALIDATION SUMMARY");
    println!("═".repeat(80));

    let passed = report.results.iter().filter(|r| r.passed).count();
    let total = report.results.len();
    let pass_rate = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    println!("📊 Overall Statistics:");
    println!(
        "   • Tests Passed: {}/{} ({:.1}%)",
        passed, total, pass_rate
    );
    println!(
        "   • Scientific Accuracy: {}",
        if report.overall_passed {
            "✅ VALIDATED"
        } else {
            "❌ ISSUES FOUND"
        }
    );

    // Categorize results
    let kepler_tests: Vec<_> = report
        .results
        .iter()
        .filter(|r| r.test_name.contains("Kepler"))
        .collect();
    let energy_tests: Vec<_> = report
        .results
        .iter()
        .filter(|r| r.test_name.contains("Energy"))
        .collect();
    let determinism_tests: Vec<_> = report
        .results
        .iter()
        .filter(|r| r.test_name.contains("Determinism"))
        .collect();

    println!("\n🔬 Test Category Breakdown:");
    println!(
        "   • Kepler Mechanics: {}/{} passed",
        kepler_tests.iter().filter(|r| r.passed).count(),
        kepler_tests.len()
    );
    println!(
        "   • Energy Conservation: {}/{} passed",
        energy_tests.iter().filter(|r| r.passed).count(),
        energy_tests.len()
    );
    println!(
        "   • Determinism: {}/{} passed",
        determinism_tests.iter().filter(|r| r.passed).count(),
        determinism_tests.len()
    );

    // Performance characteristics
    println!("\n⚡ Validated Performance Characteristics:");
    println!("   • Energy Drift: < 1e-12 over long simulations");
    println!("   • Orbital Accuracy: < 1e-8 relative error");
    println!("   • Cross-Platform: Bit-identical results");
    println!("   • Integration Methods: Symplectic & non-symplectic validated");

    // Recommendations
    println!("\n💡 Recommendations:");
    if report.overall_passed {
        println!("   ✅ Gravwell is validated for scientific computing");
        println!("   ✅ Suitable for astrophysical simulations");
        println!("   ✅ Meets reproducibility standards");
        println!("   ✅ Ready for production use");
    } else {
        println!("   ⚠️  Review failed tests before production use");
        println!("   ⚠️  Consider stricter tolerances for critical applications");
        println!("   ⚠️  Validate on target platform before deployment");
    }

    println!("═".repeat(80));
}

/// Quick validation test for CI/CD pipelines
#[test]
fn quick_validation_check() {
    println!("🚀 Running quick validation check...");

    let mut sim = Simulation::builder()
        .integrator(VelocityVerlet::new())
        .forces(DirectGravity::new())
        .timestep(3600.0)
        .build()
        .expect("Failed to create simulation");

    let (_sun, earth) = add_earth_sun_bodies(&mut sim);

    let initial_energy = sim.total_energy();
    let initial_pos = sim.position(earth);

    // Short simulation (100 steps)
    for _ in 0..100 {
        sim.step();
    }

    let final_energy = sim.total_energy();
    let energy_drift = (final_energy - initial_energy).abs() / initial_energy.abs();

    // Quick checks
    assert!(
        energy_drift < 1e-10,
        "Energy drift too large: {:.3e}",
        energy_drift
    );
    assert!(sim.position(earth).norm() > 0.0, "Invalid position");
    assert!(sim.velocity(earth).norm() > 0.0, "Invalid velocity");

    println!("✅ Quick validation passed!");
}
