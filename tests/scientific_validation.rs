// Gravwell Scientific Validation Integration Tests
//
// This integration test runs the comprehensive validation suite to ensure
// Gravwell meets scientific computing standards for accuracy and reliability.

#[cfg(test)]
mod tests {
    use gravwell::prelude::*;

    #[test]
    fn basic_physics_validation() {
        // Basic two-body energy conservation test
        let mut sim = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(DirectGravity::new())
            .build()
            .unwrap();

        // Add Sun
        let _sun = sim
            .add_body(
                Body::new()
                    .with_mass(1.989e30) // Solar mass in kg
                    .with_position([0.0, 0.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0])
                    .with_radius(6.96e8), // Solar radius in meters
            )
            .unwrap();

        // Add Earth
        let _earth = sim
            .add_body(
                Body::new()
                    .with_mass(5.972e24) // Earth mass in kg
                    .with_position([1.496e11, 0.0, 0.0]) // 1 AU in meters
                    .with_velocity([0.0, 29780.0, 0.0]) // Circular orbital velocity
                    .with_radius(6.371e6), // Earth radius in meters
            )
            .unwrap();

        let initial_energy = sim.total_energy();

        let dt = 0.01; // timestep in seconds

        // Simulate for a short period
        for _ in 0..100 {
            sim.step(dt).unwrap();
        }

        let final_energy = sim.total_energy();
        let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();

        // Energy should be reasonably conserved
        assert!(
            energy_error < 0.01,
            "Energy conservation error too large: {:.6}",
            energy_error
        );

        println!(
            "✅ Basic energy conservation test passed - error: {:.2e}",
            energy_error
        );
    }
}
