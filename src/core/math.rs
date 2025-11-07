//! Mathematical utilities and vector operations.

use crate::types::{Scalar, Vector3};

/// Re-export common vector operations.
pub use nalgebra::{Point3, Vector3 as Vec3};

/// Mathematical constants and utility functions.
pub struct Math;

impl Math {
    /// Calculate the distance between two points.
    #[inline]
    pub fn distance(a: &Vector3, b: &Vector3) -> Scalar {
        (b - a).magnitude()
    }

    /// Calculate the squared distance between two points (faster than distance).
    #[inline]
    pub fn distance_squared(a: &Vector3, b: &Vector3) -> Scalar {
        (b - a).magnitude_squared()
    }

    /// Normalize a vector safely (returns zero vector if input is zero).
    #[inline]
    pub fn safe_normalize(v: &Vector3) -> Vector3 {
        let mag_sq = v.magnitude_squared();
        if mag_sq > Scalar::EPSILON {
            v / mag_sq.sqrt()
        } else {
            Vector3::zeros()
        }
    }

    /// Check if a scalar is finite and valid for physics calculations.
    #[inline]
    pub fn is_valid_scalar(s: Scalar) -> bool {
        s.is_finite() && !s.is_nan()
    }

    /// Check if a vector is valid for physics calculations.
    #[inline]
    pub fn is_valid_vector(v: &Vector3) -> bool {
        v.iter().all(|&s| Self::is_valid_scalar(s))
    }

    /// Clamp a scalar to a given range.
    #[inline]
    pub fn clamp(value: Scalar, min: Scalar, max: Scalar) -> Scalar {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
}
