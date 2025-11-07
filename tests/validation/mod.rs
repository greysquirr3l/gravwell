// Gravwell Scientific Validation Test Suite
//
// This module contains comprehensive physics accuracy validation tests
// that ensure Gravwell maintains scientific computing standards.

pub mod cross_platform_determinism;
pub mod energy_conservation;
pub mod kepler_orbits;
pub mod three_body_solutions;

use approx::{assert_abs_diff_eq, assert_relative_eq};
use gravwell::prelude::*;
use std::f64::consts::PI;

/// Physical and mathematical constants for validation tests
pub mod constants {
    use super::*;

    /// Gravitational constant (m³/kg⋅s²)
    pub const G: f64 = 6.67430e-11;

    /// Solar mass (kg)
    pub const SOLAR_MASS: f64 = 1.98847e30;

    /// Earth mass (kg)
    pub const EARTH_MASS: f64 = 5.97219e24;

    /// Astronomical unit (m)
    pub const AU: f64 = 1.495978707e11;

    /// Earth's orbital velocity (m/s)
    pub const EARTH_ORBITAL_VELOCITY: f64 = 29780.0;

    /// Tolerance thresholds for different validation levels
    pub const ENERGY_CONSERVATION_TOLERANCE: f64 = 1e-12;
    pub const KEPLER_ORBIT_TOLERANCE: f64 = 1e-8;
    pub const CROSS_PLATFORM_TOLERANCE: f64 = 1e-15; // Bit-identical
}

/// Validation result structure for consistent reporting
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub test_name: String,
    pub passed: bool,
    pub measured_value: f64,
    pub expected_value: f64,
    pub tolerance: f64,
    pub relative_error: f64,
    pub description: String,
}

impl ValidationResult {
    pub fn new(
        test_name: impl Into<String>,
        measured_value: f64,
        expected_value: f64,
        tolerance: f64,
    ) -> Self {
        let relative_error = if expected_value.abs() > 0.0 {
            (measured_value - expected_value).abs() / expected_value.abs()
        } else {
            (measured_value - expected_value).abs()
        };

        let passed = relative_error <= tolerance;

        Self {
            test_name: test_name.into(),
            passed,
            measured_value,
            expected_value,
            tolerance,
            relative_error,
            description: format!(
                "Measured: {:.6e}, Expected: {:.6e}, Error: {:.3e} (tol: {:.1e})",
                measured_value, expected_value, relative_error, tolerance
            ),
        }
    }

    pub fn print_summary(&self) {
        let status = if self.passed { "✓ PASS" } else { "✗ FAIL" };
        println!("{} {}: {}", status, self.test_name, self.description);
    }
}

/// Comprehensive validation report aggregator
#[derive(Debug)]
pub struct ValidationReport {
    pub results: Vec<ValidationResult>,
    pub overall_passed: bool,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            overall_passed: true,
        }
    }

    pub fn add_result(&mut self, result: ValidationResult) {
        if !result.passed {
            self.overall_passed = false;
        }
        self.results.push(result);
    }

    pub fn print_full_report(&self) {
        println!("\n═══════════════════════════════════");
        println!("  GRAVWELL VALIDATION REPORT");
        println!("═══════════════════════════════════");

        let passed_count = self.results.iter().filter(|r| r.passed).count();
        let total_count = self.results.len();

        for result in &self.results {
            result.print_summary();
        }

        println!("\n───────────────────────────────────");
        println!("Summary: {}/{} tests passed", passed_count, total_count);

        if self.overall_passed {
            println!("🎉 ALL VALIDATION TESTS PASSED! 🎉");
            println!("Gravwell meets scientific computing standards.");
        } else {
            println!("⚠️  Some validation tests failed.");
            println!("Review failed tests before production use.");
        }
        println!("═══════════════════════════════════\n");
    }
}

/// Helper function to create a standard Earth-Sun two-body system
pub fn setup_earth_sun_system() -> (Simulation, BodyHandle, BodyHandle) {
    let mut sim = Simulation::builder()
        .integrator(VelocityVerlet::new())
        .forces(DirectGravity::new())
        .timestep(3600.0) // 1 hour timestep
        .build()
        .expect("Failed to create simulation");

    // Add Sun at origin
    let sun = sim
        .add_body(Body {
            mass: constants::SOLAR_MASS,
            position: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            radius: 0.0,
        })
        .expect("Failed to add Sun");

    // Add Earth at 1 AU with circular orbital velocity
    let earth = sim
        .add_body(Body {
            mass: constants::EARTH_MASS,
            position: Vector3::new(constants::AU, 0.0, 0.0),
            velocity: Vector3::new(0.0, constants::EARTH_ORBITAL_VELOCITY, 0.0),
            radius: 0.0,
        })
        .expect("Failed to add Earth");

    (sim, sun, earth)
}

/// Calculate theoretical circular orbital period using Kepler's third law
pub fn theoretical_orbital_period(semi_major_axis: f64, total_mass: f64) -> f64 {
    2.0 * PI * (semi_major_axis.powi(3) / (constants::G * total_mass)).sqrt()
}

/// Calculate theoretical circular orbital velocity
pub fn theoretical_orbital_velocity(orbital_radius: f64, central_mass: f64) -> f64 {
    (constants::G * central_mass / orbital_radius).sqrt()
}
