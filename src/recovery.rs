//! Error recovery strategies for physics simulation failures.

use crate::{
    error::{GravwellError, Result},
    types::{Scalar, Vector3},
    validation::{SystemStatistics, Validator},
};

/// Recovery strategies for physics simulation errors.
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Attempt to recover from a numerical instability error.
    pub fn recover_from_instability(
        error: &GravwellError,
        current_timestep: Scalar,
        system_stats: &SystemStatistics,
    ) -> RecoveryAction {
        match error {
            GravwellError::NumericalInstability { .. } => {
                let suggested_dt = system_stats.suggest_timestep();
                RecoveryAction::ReduceTimestep {
                    new_timestep: suggested_dt.min(current_timestep * 0.1),
                    reason: "Numerical instability detected".to_string(),
                }
            }
            GravwellError::TimestepTooLarge { max_recommended, .. } => {
                RecoveryAction::ReduceTimestep {
                    new_timestep: *max_recommended * 0.5,
                    reason: "Timestep exceeds stability limit".to_string(),
                }
            }
            GravwellError::EnergyConservationViolation { .. } => {
                RecoveryAction::ReduceTimestep {
                    new_timestep: current_timestep * 0.5,
                    reason: "Energy conservation violated".to_string(),
                }
            }
            _ => RecoveryAction::NoRecovery,
        }
    }

    /// Attempt to fix invalid particle data.
    pub fn fix_invalid_particles(
        positions: &mut [Vector3],
        velocities: &mut [Vector3],
        masses: &mut [Scalar],
    ) -> RecoveryResult {
        let mut fixes_applied = Vec::new();
        let mut fatal_errors = Vec::new();

        // Fix invalid positions
        for (i, position) in positions.iter_mut().enumerate() {
            if !crate::core::math::Math::is_valid_vector(position) {
                if position.iter().any(|&x| x.is_nan()) {
                    // NaN positions are usually fatal
                    fatal_errors.push(format!("Particle {} has NaN position", i));
                } else if position.iter().any(|&x| x.is_infinite()) {
                    // Try to clamp infinite positions
                    *position = Vector3::new(
                        clamp_infinite(position.x),
                        clamp_infinite(position.y),
                        clamp_infinite(position.z),
                    );
                    fixes_applied.push(format!("Clamped infinite position for particle {}", i));
                }
            }
        }

        // Fix invalid velocities
        for (i, velocity) in velocities.iter_mut().enumerate() {
            if !crate::core::math::Math::is_valid_vector(velocity) {
                if velocity.iter().any(|&x| x.is_nan()) {
                    // NaN velocities can sometimes be fixed by setting to zero
                    *velocity = Vector3::zeros();
                    fixes_applied.push(format!("Reset NaN velocity to zero for particle {}", i));
                } else if velocity.iter().any(|&x| x.is_infinite()) {
                    // Clamp infinite velocities
                    *velocity = Vector3::new(
                        clamp_infinite(velocity.x),
                        clamp_infinite(velocity.y),
                        clamp_infinite(velocity.z),
                    );
                    fixes_applied.push(format!("Clamped infinite velocity for particle {}", i));
                }
            }
        }

        // Fix invalid masses
        for (i, mass) in masses.iter_mut().enumerate() {
            if !mass.is_finite() || *mass <= 0.0 {
                if mass.is_nan() || *mass <= 0.0 {
                    *mass = 1.0; // Default to solar mass
                    fixes_applied.push(format!("Reset invalid mass to 1.0 for particle {}", i));
                } else if mass.is_infinite() {
                    *mass = 1e30; // Large but finite mass
                    fixes_applied.push(format!("Clamped infinite mass for particle {}", i));
                }
            }
        }

        if !fatal_errors.is_empty() {
            RecoveryResult::Fatal { errors: fatal_errors }
        } else if !fixes_applied.is_empty() {
            RecoveryResult::Fixed { fixes: fixes_applied }
        } else {
            RecoveryResult::NoActionNeeded
        }
    }

    /// Handle particle collision by separation.
    pub fn separate_colliding_particles(
        positions: &mut [Vector3],
        velocities: &mut [Vector3],
        particle1: usize,
        particle2: usize,
        separation_distance: Scalar,
    ) -> Result<()> {
        if particle1 >= positions.len() || particle2 >= positions.len() {
            return Err(GravwellError::configuration("Invalid particle indices for collision separation"));
        }

        // Calculate current separation vector
        let current_separation = positions[particle2] - positions[particle1];
        let current_distance = current_separation.magnitude();

        let separation_vector = if current_distance < Scalar::EPSILON {
            // Particles are at identical positions, use random separation
            Vector3::new(
                separation_distance * (rand::random::<f64>() - 0.5),
                separation_distance * (rand::random::<f64>() - 0.5),
                separation_distance * (rand::random::<f64>() - 0.5),
            )
        } else {
            // Use existing separation direction
            current_separation.normalize() * separation_distance
        };

        // Move particles apart (equal displacement)
        let half_separation = separation_vector * 0.5;
        positions[particle1] -= half_separation;
        positions[particle2] += half_separation;

        // Optionally, also separate velocities to prevent immediate re-collision
        let relative_velocity = velocities[particle2] - velocities[particle1];
        if relative_velocity.dot(&separation_vector) < 0.0 {
            // Particles are moving toward each other, separate their velocities
            let velocity_separation = separation_vector.normalize() * 0.1; // Small velocity push
            velocities[particle1] -= velocity_separation;
            velocities[particle2] += velocity_separation;
        }

        Ok(())
    }

    /// Adaptive recovery strategy that applies multiple recovery techniques.
    pub fn adaptive_recovery(
        error: &GravwellError,
        positions: &mut [Vector3],
        velocities: &mut [Vector3],
        masses: &mut [Scalar],
        current_timestep: Scalar,
    ) -> AdaptiveRecoveryResult {
        let mut actions_taken = Vec::new();
        let mut new_timestep = current_timestep;

        // First, try to fix any invalid particle data
        match Self::fix_invalid_particles(positions, velocities, masses) {
            RecoveryResult::Fixed { fixes } => {
                actions_taken.extend(fixes);
            }
            RecoveryResult::Fatal { errors } => {
                return AdaptiveRecoveryResult::Failed { reasons: errors };
            }
            RecoveryResult::NoActionNeeded => {}
        }

        // Then, handle specific error types
        if error.is_recoverable() {
            let system_stats = Validator::compute_system_statistics(positions, velocities);
            
            match Self::recover_from_instability(error, current_timestep, &system_stats) {
                RecoveryAction::ReduceTimestep { new_timestep: suggested_dt, reason } => {
                    new_timestep = suggested_dt;
                    actions_taken.push(format!("Reduced timestep to {}: {}", suggested_dt, reason));
                }
                RecoveryAction::NoRecovery => {
                    actions_taken.push("No automatic recovery available for this error".to_string());
                }
            }
        }

        // Check for particle collisions and separate them
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let distance = crate::core::math::Math::distance(&positions[i], &positions[j]);
                if distance < Scalar::EPSILON * 1000.0 {
                    if let Ok(()) = Self::separate_colliding_particles(
                        positions, 
                        velocities, 
                        i, 
                        j, 
                        0.001 // 1mm separation
                    ) {
                        actions_taken.push(format!("Separated colliding particles {} and {}", i, j));
                    }
                }
            }
        }

        AdaptiveRecoveryResult::Recovered {
            actions_taken,
            new_timestep: if new_timestep != current_timestep { Some(new_timestep) } else { None },
        }
    }
}

/// Actions that can be taken to recover from errors.
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Reduce the timestep to improve numerical stability.
    ReduceTimestep {
        /// The new, smaller timestep.
        new_timestep: Scalar,
        /// Reason for the reduction.
        reason: String,
    },
    /// No recovery action can be taken.
    NoRecovery,
}

/// Result of attempting to fix invalid particle data.
#[derive(Debug, Clone)]
pub enum RecoveryResult {
    /// Successfully fixed some issues.
    Fixed {
        /// List of fixes applied.
        fixes: Vec<String>,
    },
    /// Fatal errors that cannot be recovered from.
    Fatal {
        /// List of fatal errors.
        errors: Vec<String>,
    },
    /// No action was needed.
    NoActionNeeded,
}

/// Result of adaptive recovery process.
#[derive(Debug, Clone)]
pub enum AdaptiveRecoveryResult {
    /// Successfully recovered from the error.
    Recovered {
        /// List of actions taken during recovery.
        actions_taken: Vec<String>,
        /// New timestep if it was changed.
        new_timestep: Option<Scalar>,
    },
    /// Recovery failed.
    Failed {
        /// Reasons why recovery failed.
        reasons: Vec<String>,
    },
}

/// Helper function to clamp infinite values to large but finite values.
fn clamp_infinite(value: f64) -> f64 {
    if value.is_infinite() {
        if value > 0.0 {
            1e15  // Large positive value
        } else {
            -1e15 // Large negative value
        }
    } else {
        value
    }
}

// Note: Using a simple random number generator for demonstration.
// In production, you might want to use a proper RNG crate.
mod rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn random<T: Default>() -> f64 {
        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        let hash = hasher.finish();
        (hash as f64) / (u64::MAX as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_infinite_positions() {
        let mut positions = vec![
            Vector3::new(f64::INFINITY, 0.0, 0.0),
            Vector3::new(0.0, f64::NEG_INFINITY, 0.0),
            Vector3::new(1.0, 2.0, 3.0), // Valid
        ];
        let mut velocities = vec![Vector3::zeros(); 3];
        let mut masses = vec![1.0; 3];

        let result = ErrorRecovery::fix_invalid_particles(&mut positions, &mut velocities, &mut masses);
        
        match result {
            RecoveryResult::Fixed { fixes } => {
                assert!(!fixes.is_empty());
                assert!(positions[0].x.is_finite());
                assert!(positions[1].y.is_finite());
            }
            _ => panic!("Expected fixes to be applied"),
        }
    }

    #[test]
    fn test_fix_nan_velocities() {
        let mut positions = vec![Vector3::zeros(); 2];
        let mut velocities = vec![
            Vector3::new(f64::NAN, 0.0, 0.0),
            Vector3::new(1.0, 2.0, 3.0), // Valid
        ];
        let mut masses = vec![1.0; 2];

        let result = ErrorRecovery::fix_invalid_particles(&mut positions, &mut velocities, &mut masses);
        
        match result {
            RecoveryResult::Fixed { fixes } => {
                assert!(!fixes.is_empty());
                assert_eq!(velocities[0], Vector3::zeros());
            }
            _ => panic!("Expected fixes to be applied"),
        }
    }

    #[test]
    fn test_separate_colliding_particles() {
        let mut positions = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0), // Same position
        ];
        let mut velocities = vec![Vector3::zeros(); 2];

        let result = ErrorRecovery::separate_colliding_particles(
            &mut positions,
            &mut velocities,
            0,
            1,
            1.0,
        );

        assert!(result.is_ok());
        let distance = crate::core::math::Math::distance(&positions[0], &positions[1]);
        assert!(distance > 0.5); // Should be approximately 1.0 (separation distance)
    }

    #[test]
    fn test_recovery_action_for_timestep_error() {
        let error = GravwellError::timestep_too_large(0.1, 0.01);
        let stats = SystemStatistics {
            max_velocity: 1.0,
            min_distance: 1.0,
            max_acceleration_estimate: 1.0,
            particle_count: 2,
        };

        let action = ErrorRecovery::recover_from_instability(&error, 0.1, &stats);
        
        match action {
            RecoveryAction::ReduceTimestep { new_timestep, .. } => {
                assert!(new_timestep < 0.1);
            }
            _ => panic!("Expected timestep reduction"),
        }
    }
}