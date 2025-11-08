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
