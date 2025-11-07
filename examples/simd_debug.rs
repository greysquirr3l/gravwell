//! Simple SIMD test example to debug force calculation issues.

use gravwell::prelude::*;
use gravwell::simd::{SimdLevel, VectorizedGravity};

fn main() -> Result<()> {
    println!("🔧 SIMD Debug Test");
    println!("==================\n");

    // Create a simple 2-body system
    let mut particles = ParticleSet::new();

    // Add two particles 1 unit apart
    particles.add_body(
        Body::new()
            .with_mass(1.0e30)
            .with_position([-0.5, 0.0, 0.0])
            .with_velocity([0.0, 0.0, 0.0]),
    )?;

    particles.add_body(
        Body::new()
            .with_mass(1.0e30)
            .with_position([0.5, 0.0, 0.0])
            .with_velocity([0.0, 0.0, 0.0]),
    )?;

    println!("🌟 Test System:");
    println!(
        "  Particle 1: mass={:.1e} kg, position=[-0.5, 0, 0]",
        1.0e30
    );
    println!("  Particle 2: mass={:.1e} kg, position=[0.5, 0, 0]", 1.0e30);
    println!("  Distance: 1.0 unit\n");

    // Test scalar implementation
    println!("🧮 Scalar Calculation:");
    let scalar_calc = VectorizedGravity::with_simd_level(SimdLevel::Scalar);
    let mut scalar_forces = vec![Vector3::zeros(); 2];
    scalar_calc.calculate_forces(&particles, &mut scalar_forces)?;

    println!(
        "  Force on particle 1: [{:.3e}, {:.3e}, {:.3e}]",
        scalar_forces[0].x, scalar_forces[0].y, scalar_forces[0].z
    );
    println!(
        "  Force on particle 2: [{:.3e}, {:.3e}, {:.3e}]",
        scalar_forces[1].x, scalar_forces[1].y, scalar_forces[1].z
    );
    println!("  Force magnitude: {:.3e} N", scalar_forces[0].norm());

    // Calculate expected force manually
    let g_const = 6.67430e-11; // m³/(kg⋅s²)
    let m1 = 1.0e30;
    let m2 = 1.0e30;
    let r = 1.0;
    let expected_force = g_const * m1 * m2 / (r * r);

    println!("  Expected magnitude: {:.3e} N", expected_force);
    println!(
        "  Error: {:.3e}",
        (scalar_forces[0].norm() - expected_force).abs() / expected_force
    );

    // Test force direction (should be toward other particle)
    println!("  Force direction check:");
    println!(
        "    Particle 1 force should be positive X (toward particle 2): {}",
        scalar_forces[0].x > 0.0
    );
    println!(
        "    Particle 2 force should be negative X (toward particle 1): {}",
        scalar_forces[1].x < 0.0
    );

    // Test Newton's third law
    let force_diff = (scalar_forces[0] + scalar_forces[1]).norm();
    println!(
        "    Newton's 3rd law check (should be ~0): {:.3e}",
        force_diff
    );

    // Test momentum conservation (total force should be zero)
    let total_force: Vector3 = scalar_forces.iter().sum();
    println!(
        "    Momentum conservation (total force): [{:.3e}, {:.3e}, {:.3e}]",
        total_force.x, total_force.y, total_force.z
    );

    Ok(())
}
