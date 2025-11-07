//! Direct gravitational force calculation (O(N²)).

use crate::{
    core::{forces::ForceCalculator, math::Math, particle::ParticleSet},
    error::{GravwellError, Result},
    types::{Force, Scalar, Vector3},
    utils::constants::G,
};

/// Direct N-body gravitational force calculator.
///
/// Computes forces between every pair of particles, resulting in O(N²)
/// computational complexity. This is exact but becomes expensive for
/// large numbers of particles.
#[derive(Debug, Clone, Default)]
pub struct DirectGravity {
    /// Softening parameter to prevent singularities at small distances.
    softening: Scalar,
}

impl DirectGravity {
    /// Create a new direct gravity calculator.
    pub fn new() -> Self {
        Self { softening: 0.0 }
    }

    /// Create a direct gravity calculator with softening.
    ///
    /// Softening prevents infinite forces when particles get very close.
    /// The softening parameter should be much smaller than typical
    /// inter-particle distances.
    pub fn with_softening(softening: Scalar) -> Self {
        Self { softening }
    }

    /// Calculate gravitational force between two particles.
    #[inline]
    fn pairwise_force(
        &self,
        pos1: &Vector3,
        mass1: Scalar,
        pos2: &Vector3,
        mass2: Scalar,
    ) -> Force {
        let dr = pos2 - pos1;
        let r_squared = dr.magnitude_squared() + self.softening * self.softening;
        let r = r_squared.sqrt();

        if r > Scalar::EPSILON {
            let force_magnitude = G * mass1 * mass2 / r_squared;
            let force_direction = dr / r;
            force_magnitude * force_direction
        } else {
            Force::zeros()
        }
    }
}

impl ForceCalculator for DirectGravity {
    fn calculate_forces(&self, particles: &ParticleSet, forces: &mut [Force]) -> Result<()> {
        let n = particles.len();

        if forces.len() != n {
            return Err(GravwellError::force_calculation(format!(
                "Force array length {} doesn't match particle count {}",
                forces.len(),
                n
            )));
        }

        // Initialize all forces to zero
        for force in forces.iter_mut() {
            *force = Force::zeros();
        }

        // Calculate pairwise forces
        for i in 0..n {
            for j in (i + 1)..n {
                let force_ij = self.pairwise_force(
                    particles.position(i),
                    particles.mass(i),
                    particles.position(j),
                    particles.mass(j),
                );

                // Newton's third law: F_ij = -F_ji
                forces[i] += force_ij;
                forces[j] -= force_ij;
            }
        }

        // Validate computed forces
        for (i, force) in forces.iter().enumerate() {
            if !Math::is_valid_vector(force) {
                return Err(GravwellError::force_calculation(format!(
                    "Invalid force computed for particle {}: [{}, {}, {}]",
                    i, force.x, force.y, force.z
                )));
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Direct Gravity"
    }

    fn complexity(&self) -> &'static str {
        "O(N²)"
    }

    fn supports_parallel(&self) -> bool {
        true
    }
}
