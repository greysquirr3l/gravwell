//! Trait definition for numerical integrators.

use crate::{
    core::forces::ForceCalculator,
    core::particle::ParticleSet,
    error::Result,
    types::{Acceleration, Time},
    validation::Validator,
};

/// Trait for numerical integration methods.
///
/// Integrators advance the simulation state forward in time by solving
/// the equations of motion. Different integrators have different properties
/// regarding accuracy, stability, and energy conservation.
pub trait Integrator {
    /// Advance the particle system by one timestep.
    ///
    /// # Arguments
    /// * `particles` - Mutable reference to the particle system
    /// * `forces` - Force calculator for computing gravitational forces
    /// * `dt` - Timestep duration
    ///
    /// # Returns
    /// * `Result<()>` - Success or integration error
    ///
    /// # Errors
    /// Returns `GravwellError::Integration` if numerical instabilities occur,
    /// such as NaN values, infinite forces, or other numerical issues.
    fn step<F>(&mut self, particles: &mut ParticleSet, forces: &F, dt: Time) -> Result<()>
    where
        F: ForceCalculator;

    /// Get the name of this integrator (for debugging/logging).
    fn name(&self) -> &'static str;

    /// Whether this integrator is symplectic (conserves phase space volume).
    ///
    /// Symplectic integrators are preferred for long-term simulations as they
    /// better preserve energy and angular momentum over many timesteps.
    fn is_symplectic(&self) -> bool;

    /// Get the order of accuracy of this integrator.
    ///
    /// Higher order integrators generally provide better accuracy but may
    /// require more computational work per timestep.
    fn order(&self) -> u8;

    /// Reset the integrator state (if any internal state exists).
    ///
    /// Some adaptive integrators maintain internal state that may need
    /// to be reset when starting a new simulation or after large changes.
    fn reset(&mut self) {
        // Default implementation does nothing for stateless integrators
    }
}

/// Helper function to validate timestep.
pub fn validate_timestep(dt: Time) -> Result<()> {
    Validator::validate_timestep(dt)
}

/// Helper function to validate accelerations for numerical stability.
pub fn validate_accelerations(accelerations: &[Acceleration]) -> Result<()> {
    use crate::{core::math::Math, error::GravwellError};

    for (i, acc) in accelerations.iter().enumerate() {
        if !Math::is_valid_vector(acc) {
            return Err(GravwellError::integration(format!(
                "Invalid acceleration for particle {}: [{}, {}, {}]",
                i, acc.x, acc.y, acc.z
            )));
        }
    }
    Ok(())
}
