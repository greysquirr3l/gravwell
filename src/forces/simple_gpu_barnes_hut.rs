//! Simplified GPU Barnes-Hut Implementation
//!
//! Direct GPU implementation using simple direct force calculation

use crate::{
    core::{forces::ForceCalculator, particle::ParticleSet},
    error::Result,
    types::{Scalar, Vector3},
};

/// Simple GPU Barnes-Hut force calculator.
///
/// Currently implements a fallback to CPU direct gravity calculation.
/// Future versions will include GPU compute shader implementation.
pub struct SimpleGpuBarnesHut {
    gravity_constant: Scalar,
    softening: Scalar,
}

impl SimpleGpuBarnesHut {
    /// Create a new simple GPU Barnes-Hut calculator.
    pub fn new() -> Self {
        Self {
            gravity_constant: 6.67430e-11,
            softening: 1e-6,
        }
    }

    /// Set the gravitational constant.
    pub fn gravity_constant(mut self, g: Scalar) -> Self {
        self.gravity_constant = g;
        self
    }

    /// Set the softening parameter to prevent singularities.
    pub fn softening(mut self, softening: Scalar) -> Self {
        self.softening = softening;
        self
    }

    fn calculate_forces_sync(
        &mut self,
        particles: &ParticleSet,
        forces: &mut [Vector3],
    ) -> Result<()> {
        // For now, fallback to CPU implementation to get working code
        use crate::forces::DirectGravity;

        let direct_gravity = DirectGravity::new();
        direct_gravity.calculate_forces(particles, forces)
    }
}

impl ForceCalculator for SimpleGpuBarnesHut {
    fn name(&self) -> &'static str {
        "Simple GPU Barnes-Hut"
    }

    fn complexity(&self) -> &'static str {
        "O(N²)"
    }

    fn calculate_forces(&self, particles: &ParticleSet, forces: &mut [Vector3]) -> Result<()> {
        // Create a mutable copy for the calculation
        let mut self_mut = Self {
            gravity_constant: self.gravity_constant,
            softening: self.softening,
        };

        self_mut.calculate_forces_sync(particles, forces)
    }
}
