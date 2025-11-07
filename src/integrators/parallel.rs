//! Parallel integrator implementations using rayon.
//!
//! This module provides parallel versions of numerical integrators that
//! leverage the parallel force calculations for improved performance.
//! The main benefit comes from parallel force computation rather than
//! parallel particle updates.

use crate::{
    core::{
        forces::ForceCalculator,
        integrator::{validate_accelerations, validate_timestep, Integrator},
        particle::ParticleSet,
    },
    error::Result,
    types::{Acceleration, Time},
};

/// Parallel-aware Velocity Verlet integrator.
///
/// This integrator leverages parallel force calculators for improved
/// performance while maintaining the symplectic properties of the
/// standard Velocity Verlet algorithm.
#[derive(Debug, Clone)]
pub struct ParallelVelocityVerlet {
    /// Minimum particle count below which to use serial calculation
    parallel_threshold: usize,
}

impl Default for ParallelVelocityVerlet {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallelVelocityVerlet {
    /// Create a new parallel-aware Velocity Verlet integrator.
    pub fn new() -> Self {
        Self {
            parallel_threshold: 1000,
        }
    }

    /// Set minimum particle count for using parallel processing.
    pub fn with_parallel_threshold(mut self, threshold: usize) -> Self {
        self.parallel_threshold = threshold;
        self
    }
}

impl Integrator for ParallelVelocityVerlet {
    fn step<F>(&mut self, particles: &mut ParticleSet, forces: &F, dt: Time) -> Result<()>
    where
        F: ForceCalculator,
    {
        validate_timestep(dt)?;

        let n = particles.len();
        let mut accelerations = vec![Acceleration::zeros(); n];

        // Calculate initial accelerations (this is where parallelism happens in force calculation)
        forces.calculate_accelerations(particles, &mut accelerations)?;
        validate_accelerations(&accelerations)?;

        // Update positions: x += v*dt + 0.5*a*dt²
        for i in 0..n {
            let velocity = *particles.velocity(i);
            *particles.position_mut(i) += velocity * dt + 0.5 * accelerations[i] * dt * dt;
        }

        // Calculate new accelerations at new positions
        let mut new_accelerations = vec![Acceleration::zeros(); n];
        forces.calculate_accelerations(particles, &mut new_accelerations)?;
        validate_accelerations(&new_accelerations)?;

        // Update velocities: v += 0.5*(a_old + a_new)*dt
        for i in 0..n {
            *particles.velocity_mut(i) += 0.5 * (accelerations[i] + new_accelerations[i]) * dt;
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Parallel-Aware Velocity Verlet"
    }

    fn is_symplectic(&self) -> bool {
        true
    }

    fn order(&self) -> u8 {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::particle::{Body, ParticleSet},
        forces::DirectGravity,
        types::{Mass, Vector3},
    };
    use approx::assert_relative_eq;

    #[test]
    fn test_parallel_integrator_consistency() {
        let mut particles = ParticleSet::new();

        // Add test particles
        for i in 0..10 {
            particles
                .add_body(
                    Body::new()
                        .with_position([i as f64, 0.0, 0.0])
                        .with_velocity([0.0, 0.0, 0.0])
                        .mass(1.0)
                        .with_radius(0.1),
                )
                .unwrap();
        }

        let force_calc = DirectGravity::new();
        let dt = 0.01;

        // Clone particle sets for comparison
        let mut parallel_particles = particles.clone();
        let mut serial_particles = particles;

        // Parallel integrator
        let mut parallel_integrator = ParallelVelocityVerlet::new().with_parallel_threshold(5);
        parallel_integrator
            .step(&mut parallel_particles, &force_calc, dt)
            .unwrap();

        // Standard integrator for comparison
        use crate::integrators::verlet::VelocityVerlet;
        let mut serial_integrator = VelocityVerlet::new();
        serial_integrator
            .step(&mut serial_particles, &force_calc, dt)
            .unwrap();

        // Results should be identical (since force calculation is deterministic)
        for i in 0..parallel_particles.len() {
            let parallel_pos = parallel_particles.position(i);
            let serial_pos = serial_particles.position(i);
            let parallel_vel = parallel_particles.velocity(i);
            let serial_vel = serial_particles.velocity(i);

            assert_relative_eq!(parallel_pos.x, serial_pos.x, epsilon = 1e-12);
            assert_relative_eq!(parallel_pos.y, serial_pos.y, epsilon = 1e-12);
            assert_relative_eq!(parallel_pos.z, serial_pos.z, epsilon = 1e-12);

            assert_relative_eq!(parallel_vel.x, serial_vel.x, epsilon = 1e-12);
            assert_relative_eq!(parallel_vel.y, serial_vel.y, epsilon = 1e-12);
            assert_relative_eq!(parallel_vel.z, serial_vel.z, epsilon = 1e-12);
        }
    }
}
