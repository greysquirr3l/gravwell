//! Comprehensive input validation and stability analysis.

use crate::{
    core::math::Math,
    error::{GravwellError, Result},
    types::{Mass, Scalar, Vector3},
};

/// Validation utilities for physics simulations.
pub struct Validator;

impl Validator {
    /// Validate a mass value.
    pub fn validate_mass(mass: Mass, particle_index: Option<usize>) -> Result<()> {
        if !Math::is_valid_scalar(mass) || mass <= 0.0 {
            Err(GravwellError::invalid_mass(mass, particle_index))
        } else {
            Ok(())
        }
    }

    /// Validate a position vector.
    pub fn validate_position(position: &Vector3, particle_index: usize) -> Result<()> {
        if !Math::is_valid_vector(position) {
            Err(GravwellError::invalid_position(
                particle_index,
                position.x,
                position.y,
                position.z,
            ))
        } else {
            Ok(())
        }
    }

    /// Validate a velocity vector.
    pub fn validate_velocity(velocity: &Vector3, particle_index: usize) -> Result<()> {
        if !Math::is_valid_vector(velocity) {
            Err(GravwellError::invalid_velocity(
                particle_index,
                velocity.x,
                velocity.y,
                velocity.z,
            ))
        } else {
            Ok(())
        }
    }

    /// Validate a radius value.
    pub fn validate_radius(radius: Scalar, particle_index: usize) -> Result<()> {
        if !Math::is_valid_scalar(radius) || radius <= 0.0 {
            Err(GravwellError::invalid_radius(radius, particle_index))
        } else {
            Ok(())
        }
    }

    /// Validate a timestep value.
    pub fn validate_timestep(dt: Scalar) -> Result<()> {
        if !Math::is_valid_scalar(dt) || dt <= 0.0 {
            Err(GravwellError::InvalidTimestep { timestep: dt })
        } else {
            Ok(())
        }
    }

    /// Perform timestep stability analysis.
    pub fn analyze_timestep_stability(
        dt: Scalar,
        max_velocity: Scalar,
        min_distance: Scalar,
    ) -> Result<()> {
        // Basic CFL-like condition for gravitational systems
        let velocity_timestep_limit = if max_velocity > 0.0 {
            min_distance / max_velocity * 0.1 // Conservative factor
        } else {
            f64::INFINITY
        };

        // Gravitational force timestep limit (rough estimate)
        let force_timestep_limit = if min_distance > 0.0 {
            (min_distance.powi(3) / (4.0 * std::f64::consts::PI)).sqrt() * 0.01
        } else {
            f64::INFINITY
        };

        let max_recommended = velocity_timestep_limit.min(force_timestep_limit);

        if dt > max_recommended {
            Err(GravwellError::timestep_too_large(dt, max_recommended))
        } else {
            Ok(())
        }
    }

    /// Check for particle collisions (identical positions).
    pub fn check_particle_collisions(positions: &[Vector3]) -> Result<()> {
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let distance = Math::distance(&positions[i], &positions[j]);
                if distance < Scalar::EPSILON * 1000.0 {
                    return Err(GravwellError::particle_collision(i, j));
                }
            }
        }
        Ok(())
    }

    /// Validate energy conservation within tolerance.
    pub fn validate_energy_conservation(
        initial_energy: Scalar,
        current_energy: Scalar,
        threshold: Scalar,
    ) -> Result<()> {
        let energy_drift = (current_energy - initial_energy).abs() / initial_energy.abs();
        if energy_drift > threshold {
            Err(GravwellError::energy_conservation_violation(energy_drift, threshold))
        } else {
            Ok(())
        }
    }

    /// Comprehensive particle system validation.
    pub fn validate_particle_system(
        positions: &[Vector3],
        velocities: &[Vector3],
        masses: &[Mass],
        radii: Option<&[Scalar]>,
    ) -> Result<()> {
        // Check array lengths match
        if positions.len() != velocities.len() || positions.len() != masses.len() {
            return Err(GravwellError::configuration(
                "Position, velocity, and mass arrays must have the same length",
            ));
        }

        if let Some(radii) = radii {
            if radii.len() != positions.len() {
                return Err(GravwellError::configuration(
                    "Radius array length must match position array length",
                ));
            }
        }

        // Validate each particle
        for i in 0..positions.len() {
            Self::validate_position(&positions[i], i)?;
            Self::validate_velocity(&velocities[i], i)?;
            Self::validate_mass(masses[i], Some(i))?;

            if let Some(radii) = radii {
                Self::validate_radius(radii[i], i)?;
            }
        }

        // Check for particle collisions
        Self::check_particle_collisions(positions)?;

        Ok(())
    }

    /// Get system statistics for stability analysis.
    pub fn compute_system_statistics(
        positions: &[Vector3],
        velocities: &[Vector3],
    ) -> SystemStatistics {
        let mut max_velocity = 0.0;
        let mut min_distance = f64::INFINITY;
        let mut max_acceleration_estimate = 0.0;

        // Compute maximum velocity
        for velocity in velocities {
            let speed = velocity.magnitude();
            if speed > max_velocity {
                max_velocity = speed;
            }
        }

        // Compute minimum distance and estimate maximum acceleration
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let distance = Math::distance(&positions[i], &positions[j]);
                if distance < min_distance {
                    min_distance = distance;
                }

                // Rough acceleration estimate (assumes unit masses)
                if distance > 0.0 {
                    let acceleration_estimate = 1.0 / (distance * distance);
                    if acceleration_estimate > max_acceleration_estimate {
                        max_acceleration_estimate = acceleration_estimate;
                    }
                }
            }
        }

        SystemStatistics {
            max_velocity,
            min_distance,
            max_acceleration_estimate,
            particle_count: positions.len(),
        }
    }
}

/// System statistics for stability analysis.
#[derive(Debug, Clone)]
pub struct SystemStatistics {
    /// Maximum velocity magnitude in the system.
    pub max_velocity: Scalar,
    /// Minimum distance between any two particles.
    pub min_distance: Scalar,
    /// Estimated maximum acceleration (rough approximation).
    pub max_acceleration_estimate: Scalar,
    /// Total number of particles.
    pub particle_count: usize,
}

impl SystemStatistics {
    /// Suggest a stable timestep based on system characteristics.
    pub fn suggest_timestep(&self) -> Scalar {
        let velocity_limit = if self.max_velocity > 0.0 {
            self.min_distance / self.max_velocity * 0.01
        } else {
            1.0
        };

        let acceleration_limit = if self.max_acceleration_estimate > 0.0 {
            (self.min_distance / self.max_acceleration_estimate).sqrt() * 0.01
        } else {
            1.0
        };

        velocity_limit.min(acceleration_limit).min(1.0) // Cap at 1.0 for safety
    }

    /// Check if the system appears stable for integration.
    pub fn is_stable(&self) -> bool {
        self.min_distance > Scalar::EPSILON * 1000.0
            && self.max_velocity.is_finite()
            && self.max_acceleration_estimate.is_finite()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mass_validation() {
        // Valid mass
        assert!(Validator::validate_mass(1.0, None).is_ok());

        // Invalid masses
        assert!(Validator::validate_mass(0.0, None).is_err());
        assert!(Validator::validate_mass(-1.0, None).is_err());
        assert!(Validator::validate_mass(f64::NAN, None).is_err());
        assert!(Validator::validate_mass(f64::INFINITY, None).is_err());
    }

    #[test]
    fn test_position_validation() {
        // Valid position
        let valid_pos = Vector3::new(1.0, 2.0, 3.0);
        assert!(Validator::validate_position(&valid_pos, 0).is_ok());

        // Invalid positions
        let nan_pos = Vector3::new(f64::NAN, 0.0, 0.0);
        assert!(Validator::validate_position(&nan_pos, 0).is_err());

        let inf_pos = Vector3::new(f64::INFINITY, 0.0, 0.0);
        assert!(Validator::validate_position(&inf_pos, 0).is_err());
    }

    #[test]
    fn test_timestep_validation() {
        // Valid timestep
        assert!(Validator::validate_timestep(0.01).is_ok());

        // Invalid timesteps
        assert!(Validator::validate_timestep(0.0).is_err());
        assert!(Validator::validate_timestep(-0.01).is_err());
        assert!(Validator::validate_timestep(f64::NAN).is_err());
        assert!(Validator::validate_timestep(f64::INFINITY).is_err());
    }

    #[test]
    fn test_particle_collision_detection() {
        // No collisions
        let positions = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ];
        assert!(Validator::check_particle_collisions(&positions).is_ok());

        // Collision detected
        let collision_positions = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0), // Same position
            Vector3::new(1.0, 0.0, 0.0),
        ];
        assert!(Validator::check_particle_collisions(&collision_positions).is_err());
    }

    #[test]
    fn test_system_statistics() {
        let positions = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ];
        let velocities = vec![
            Vector3::new(0.1, 0.0, 0.0),
            Vector3::new(0.0, 0.2, 0.0),
            Vector3::new(0.0, 0.0, 0.3),
        ];

        let stats = Validator::compute_system_statistics(&positions, &velocities);
        assert_eq!(stats.particle_count, 3);
        assert!(stats.max_velocity > 0.0);
        assert!(stats.min_distance > 0.0);
        assert!(stats.is_stable());

        let suggested_dt = stats.suggest_timestep();
        assert!(suggested_dt > 0.0);
        assert!(suggested_dt.is_finite());
    }
}