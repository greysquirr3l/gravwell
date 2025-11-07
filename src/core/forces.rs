//! Trait definition for force calculation algorithms.

use crate::{
    core::particle::ParticleSet,
    error::Result,
    types::{Acceleration, Force},
};

/// Trait for force calculation algorithms.
///
/// Force calculators compute gravitational forces between all particles
/// in the system. Different algorithms have different computational
/// complexity and accuracy characteristics.
pub trait ForceCalculator {
    /// Calculate gravitational forces for all particles.
    ///
    /// # Arguments
    /// * `particles` - The particle system
    /// * `forces` - Output array for computed forces (same length as particles)
    ///
    /// # Returns
    /// * `Result<()>` - Success or force calculation error
    ///
    /// # Errors
    /// Returns `GravwellError::ForceCalculation` if numerical issues occur,
    /// such as division by zero, infinite forces, or other computational problems.
    fn calculate_forces(&self, particles: &ParticleSet, forces: &mut [Force]) -> Result<()>;

    /// Calculate accelerations directly (convenience method).
    ///
    /// This is equivalent to calculating forces and dividing by mass,
    /// but may be optimized in some implementations.
    fn calculate_accelerations(
        &self,
        particles: &ParticleSet,
        accelerations: &mut [Acceleration],
    ) -> Result<()> {
        // Default implementation calculates forces first
        let mut forces = vec![Force::zeros(); particles.len()];
        self.calculate_forces(particles, &mut forces)?;

        for (i, acc) in accelerations.iter_mut().enumerate() {
            let mass = particles.mass(i);
            if mass > 0.0 {
                *acc = forces[i] / mass;
            } else {
                *acc = Acceleration::zeros();
            }
        }

        Ok(())
    }

    /// Get the name of this force calculator.
    fn name(&self) -> &'static str;

    /// Get the computational complexity order (e.g., "O(N²)", "O(N log N)").
    fn complexity(&self) -> &'static str;

    /// Whether this calculator supports parallel computation.
    fn supports_parallel(&self) -> bool {
        false
    }

    /// Validate the force calculation setup.
    fn validate(&self, particles: &ParticleSet) -> Result<()> {
        use crate::error::GravwellError;

        if particles.len() < 2 {
            return Err(GravwellError::InsufficientParticles {
                required: 2,
                actual: particles.len(),
            });
        }

        Ok(())
    }
}
