//! Semi-implicit Euler integrator implementation.

use crate::{
    core::{integrator::Integrator, particle::ParticleSet},
    error::Result,
    types::{Acceleration, Time},
};

/// Semi-implicit Euler integrator.
///
/// This is a simple first-order integrator that updates velocity first,
/// then position using the updated velocity. It's symplectic and stable
/// for orbital mechanics, making it suitable for game simulations.
#[derive(Debug, Clone)]
pub struct SemiImplicitEuler;

impl SemiImplicitEuler {
    /// Create a new semi-implicit Euler integrator.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SemiImplicitEuler {
    fn default() -> Self {
        Self::new()
    }
}

impl Integrator for SemiImplicitEuler {
    fn step<F>(&mut self, particles: &mut ParticleSet, forces: &F, dt: Time) -> Result<()>
    where
        F: crate::core::forces::ForceCalculator,
    {
        let n = particles.len();
        let mut accelerations = vec![Acceleration::zeros(); n];

        // Calculate accelerations at current positions
        forces.calculate_accelerations(particles, &mut accelerations)?;

        // Semi-implicit Euler: update velocities first, then positions
        for i in 0..n {
            // Update velocity using acceleration
            *particles.velocity_mut(i) += accelerations[i] * dt;

            // Update position using new velocity
            let velocity = *particles.velocity(i);
            *particles.position_mut(i) += velocity * dt;
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Semi-Implicit Euler"
    }

    fn order(&self) -> u8 {
        1
    }

    fn is_symplectic(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::particle::Body, forces::DirectGravity, utils::constants};

    #[test]
    fn test_euler_basic_step() {
        let mut integrator = SemiImplicitEuler::new();
        let mut particles = ParticleSet::new();

        // Add a simple test particle
        let _handle = particles.add_body(
            Body::new()
                .mass(1.0)
                .with_position([1.0, 0.0, 0.0])
                .with_velocity([0.0, 1.0, 0.0]),
        );

        let forces = DirectGravity::new();
        let dt = 0.01;

        // Should not panic
        integrator.step(&mut particles, &forces, dt).unwrap();
    }

    #[test]
    fn test_euler_properties() {
        let integrator = SemiImplicitEuler::new();
        assert_eq!(integrator.name(), "Semi-Implicit Euler");
        assert_eq!(integrator.order(), 1);
        assert!(integrator.is_symplectic());
    }
}
