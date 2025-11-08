//! Error types for Gravwell operations.

use thiserror::Error;

/// Result type alias for Gravwell operations.
pub type Result<T> = core::result::Result<T, GravwellError>;

/// Errors that can occur during gravity simulation operations.
#[derive(Error, Debug)]
pub enum GravwellError {
    /// Simulation configuration error.
    #[error("Simulation configuration error: {0}")]
    Configuration(String),

    /// Invalid particle data error.
    #[error("Invalid particle data: {0}")]
    InvalidParticle(String),

    /// Integration error (numerical instability, NaN values, etc.).
    #[error("Integration error: {0}")]
    Integration(String),

    /// Force calculation error.
    #[error("Force calculation error: {0}")]
    ForceCalculation(String),

    /// Collision detection error.
    #[cfg(feature = "std")]
    #[error("Collision detection error: {0}")]
    Collision(String),

    /// Invalid timestep error.
    #[error("Invalid timestep: {timestep}. Timestep must be positive and finite")]
    InvalidTimestep {
        /// The invalid timestep value.
        timestep: f64,
    },

    /// Insufficient particles error.
    #[error("Insufficient particles: need at least {required}, got {actual}")]
    InsufficientParticles {
        /// The required number of particles.
        required: usize,
        /// The actual number of particles.
        actual: usize,
    },

    /// Numerical overflow or underflow error.
    #[error("Numerical error: {0}")]
    Numerical(String),

    /// GPU acceleration error.
    #[cfg(feature = "gpu")]
    #[error("GPU error: {0}")]
    GpuError(String),

    /// Invalid mass error - mass must be positive and finite.
    #[error("Invalid mass: {mass}. Mass must be positive and finite")]
    InvalidMass {
        /// The invalid mass value.
        mass: f64,
        /// Optional particle index.
        particle_index: Option<usize>,
    },

    /// Invalid position error - position contains NaN or infinite values.
    #[error(
        "Invalid position at particle {particle_index}: [{x}, {y}, {z}]. Position must be finite"
    )]
    InvalidPosition {
        /// The particle index with invalid position.
        particle_index: usize,
        /// X coordinate.
        x: f64,
        /// Y coordinate.
        y: f64,
        /// Z coordinate.
        z: f64,
    },

    /// Invalid velocity error - velocity contains NaN or infinite values.
    #[error(
        "Invalid velocity at particle {particle_index}: [{x}, {y}, {z}]. Velocity must be finite"
    )]
    InvalidVelocity {
        /// The particle index with invalid velocity.
        particle_index: usize,
        /// X velocity component.
        x: f64,
        /// Y velocity component.
        y: f64,
        /// Z velocity component.
        z: f64,
    },

    /// Numerical instability detected.
    #[error("Numerical instability detected: {reason}. Consider reducing timestep or checking initial conditions")]
    NumericalInstability {
        /// Description of the instability.
        reason: String,
        /// Suggested recovery action.
        recovery_suggestion: String,
    },

    /// Timestep too large for stability.
    #[error("Timestep {timestep} is too large for stable integration. Maximum recommended: {max_recommended}")]
    TimestepTooLarge {
        /// The problematic timestep.
        timestep: f64,
        /// Maximum recommended timestep.
        max_recommended: f64,
    },

    /// Zero or negative radius error.
    #[error("Invalid radius: {radius} at particle {particle_index}. Radius must be positive")]
    InvalidRadius {
        /// The invalid radius value.
        radius: f64,
        /// The particle index.
        particle_index: usize,
    },

    /// Particle collision with zero distance.
    #[error("Particle collision detected: particles {particle1} and {particle2} have identical positions")]
    ParticleCollision {
        /// First particle index.
        particle1: usize,
        /// Second particle index.
        particle2: usize,
    },

    /// Energy conservation violation.
    #[error(
        "Energy conservation violated: energy drift {energy_drift} exceeds threshold {threshold}"
    )]
    EnergyConservationViolation {
        /// The energy drift amount.
        energy_drift: f64,
        /// The conservation threshold.
        threshold: f64,
    },

    /// Memory allocation error.
    #[error("Memory allocation failed: {reason}")]
    MemoryAllocation {
        /// Reason for allocation failure.
        reason: String,
    },

    /// Hardware compatibility error.
    #[error("Hardware compatibility error: {feature} is not supported on this system")]
    HardwareCompatibility {
        /// The unsupported feature.
        feature: String,
    },
}

impl GravwellError {
    /// Create a configuration error.
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create an invalid particle error.
    pub fn invalid_particle(msg: impl Into<String>) -> Self {
        Self::InvalidParticle(msg.into())
    }

    /// Create an integration error.
    pub fn integration(msg: impl Into<String>) -> Self {
        Self::Integration(msg.into())
    }

    /// Create a force calculation error.
    pub fn force_calculation(msg: impl Into<String>) -> Self {
        Self::ForceCalculation(msg.into())
    }

    /// Create a collision detection error.
    #[cfg(feature = "std")]
    pub fn collision(msg: impl Into<String>) -> Self {
        Self::Collision(msg.into())
    }

    /// Create a numerical error.
    pub fn numerical(msg: impl Into<String>) -> Self {
        Self::Numerical(msg.into())
    }

    /// Create a GPU error.
    #[cfg(feature = "gpu")]
    pub fn gpu_error(msg: impl Into<String>) -> Self {
        Self::GpuError(msg.into())
    }

    /// Create an invalid mass error.
    pub fn invalid_mass(mass: f64, particle_index: Option<usize>) -> Self {
        Self::InvalidMass {
            mass,
            particle_index,
        }
    }

    /// Create an invalid position error.
    pub fn invalid_position(particle_index: usize, x: f64, y: f64, z: f64) -> Self {
        Self::InvalidPosition {
            particle_index,
            x,
            y,
            z,
        }
    }

    /// Create an invalid velocity error.
    pub fn invalid_velocity(particle_index: usize, x: f64, y: f64, z: f64) -> Self {
        Self::InvalidVelocity {
            particle_index,
            x,
            y,
            z,
        }
    }

    /// Create a numerical instability error.
    pub fn numerical_instability(
        reason: impl Into<String>,
        recovery_suggestion: impl Into<String>,
    ) -> Self {
        Self::NumericalInstability {
            reason: reason.into(),
            recovery_suggestion: recovery_suggestion.into(),
        }
    }

    /// Create a timestep too large error.
    pub fn timestep_too_large(timestep: f64, max_recommended: f64) -> Self {
        Self::TimestepTooLarge {
            timestep,
            max_recommended,
        }
    }

    /// Create an invalid radius error.
    pub fn invalid_radius(radius: f64, particle_index: usize) -> Self {
        Self::InvalidRadius {
            radius,
            particle_index,
        }
    }

    /// Create a particle collision error.
    pub fn particle_collision(particle1: usize, particle2: usize) -> Self {
        Self::ParticleCollision {
            particle1,
            particle2,
        }
    }

    /// Create an energy conservation violation error.
    pub fn energy_conservation_violation(energy_drift: f64, threshold: f64) -> Self {
        Self::EnergyConservationViolation {
            energy_drift,
            threshold,
        }
    }

    /// Create a memory allocation error.
    pub fn memory_allocation(reason: impl Into<String>) -> Self {
        Self::MemoryAllocation {
            reason: reason.into(),
        }
    }

    /// Create a hardware compatibility error.
    pub fn hardware_compatibility(feature: impl Into<String>) -> Self {
        Self::HardwareCompatibility {
            feature: feature.into(),
        }
    }

    /// Check if this error is recoverable.
    pub fn is_recoverable(&self) -> bool {
        match self {
            // These errors can potentially be recovered from
            Self::InvalidTimestep { .. } => true,
            Self::TimestepTooLarge { .. } => true,
            Self::NumericalInstability { .. } => true,
            Self::EnergyConservationViolation { .. } => true,
            Self::MemoryAllocation { .. } => true,

            // These errors are typically fatal
            Self::InvalidMass { .. } => false,
            Self::InvalidPosition { .. } => false,
            Self::InvalidVelocity { .. } => false,
            Self::InvalidRadius { .. } => false,
            Self::ParticleCollision { .. } => false,
            Self::HardwareCompatibility { .. } => false,

            // Legacy errors - assume recoverable for backward compatibility
            Self::Configuration(_) => true,
            Self::InvalidParticle(_) => false,
            Self::Integration(_) => true,
            Self::ForceCalculation(_) => true,
            Self::InsufficientParticles { .. } => false,
            Self::Numerical(_) => true,

            #[cfg(feature = "std")]
            Self::Collision(_) => true,
            #[cfg(feature = "gpu")]
            Self::GpuError(_) => true,
        }
    }

    /// Get a suggested recovery action for recoverable errors.
    pub fn recovery_suggestion(&self) -> Option<&str> {
        match self {
            Self::InvalidTimestep { .. } => Some("Use a positive, finite timestep value"),
            Self::TimestepTooLarge { .. } => Some("Reduce the timestep for numerical stability"),
            Self::NumericalInstability {
                recovery_suggestion,
                ..
            } => Some(recovery_suggestion),
            Self::EnergyConservationViolation { .. } => {
                Some("Reduce timestep or use a more stable integrator")
            }
            Self::MemoryAllocation { .. } => Some("Reduce particle count or free system memory"),
            #[cfg(feature = "gpu")]
            Self::GpuError(_) => Some("Fallback to CPU computation or check GPU drivers"),
            _ => None,
        }
    }
}

// From implementations for common error types that examples expect
impl From<String> for GravwellError {
    fn from(msg: String) -> Self {
        Self::Configuration(msg)
    }
}

impl From<&str> for GravwellError {
    fn from(msg: &str) -> Self {
        Self::Configuration(msg.to_string())
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for GravwellError {
    fn from(err: std::io::Error) -> Self {
        Self::Configuration(format!("IO error: {}", err))
    }
}

#[cfg(feature = "std")]
impl From<std::fmt::Error> for GravwellError {
    fn from(err: std::fmt::Error) -> Self {
        Self::Configuration(format!("Format error: {}", err))
    }
}
