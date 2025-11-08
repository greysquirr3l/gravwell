//! Direct gravitational force calculation (O(N²)).

use crate::{
    core::{forces::ForceCalculator, math::Math, particle::ParticleSet},
    error::{GravwellError, Result},
    types::{Force, Scalar, Vector3},
    utils::constants::G,
    validation::Validator,
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
    pub fn with_softening(softening: Scalar) -> Result<Self> {
        if !softening.is_finite() || softening < 0.0 {
            return Err(GravwellError::configuration(format!(
                "Invalid softening parameter: {}. Must be non-negative and finite",
                softening
            )));
        }
        Ok(Self { softening })
    }

    /// Set the softening parameter.
    pub fn set_softening(&mut self, softening: Scalar) -> Result<()> {
        if !softening.is_finite() || softening < 0.0 {
            return Err(GravwellError::configuration(format!(
                "Invalid softening parameter: {}. Must be non-negative and finite",
                softening
            )));
        }
        self.softening = softening;
        Ok(())
    }

    /// Get the current softening parameter.
    pub fn softening(&self) -> Scalar {
        self.softening
    }

    /// Validate the force calculator configuration.
    pub fn validate(&self) -> Result<()> {
        if !self.softening.is_finite() || self.softening < 0.0 {
            return Err(GravwellError::configuration(format!(
                "Invalid softening parameter: {}. Must be non-negative and finite",
                self.softening
            )));
        }
        Ok(())
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

        // Validate particle system before force calculation
        particles.validate()?;

        // Validate softening parameter
        if !self.softening.is_finite() || self.softening < 0.0 {
            return Err(GravwellError::force_calculation(format!(
                "Invalid softening parameter: {}. Must be non-negative and finite",
                self.softening
            )));
        }

        // Check for minimum particle count
        if n < 2 {
            // For 0 or 1 particles, forces are zero (already handled by initialization)
            for force in forces.iter_mut() {
                *force = Force::zeros();
            }
            return Ok(());
        }

        // Analyze system stability
        let system_stats = Validator::compute_system_statistics(
            particles.positions(),
            particles.velocities(),
        );

        if !system_stats.is_stable() {
            return Err(GravwellError::numerical_instability(
                "System appears unstable for force calculation",
                "Check for particle collisions or invalid positions",
            ));
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
