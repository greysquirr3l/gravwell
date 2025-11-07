//! Physical and mathematical constants.

use crate::types::Scalar;

/// Gravitational constant (m³ kg⁻¹ s⁻²).
pub const G: Scalar = 6.674_30e-11;

/// Speed of light in vacuum (m/s).
pub const C: Scalar = 299_792_458.0;

/// Astronomical unit (m) - average Earth-Sun distance.
pub const AU: Scalar = 1.495_978_707e11;

/// Solar mass (kg).
pub const SOLAR_MASS: Scalar = 1.988_47e30;

/// Earth mass (kg).
pub const EARTH_MASS: Scalar = 5.972_16e24;

/// Moon mass (kg).
pub const LUNAR_MASS: Scalar = 7.342e22;

/// Jupiter mass (kg).
pub const JUPITER_MASS: Scalar = 1.898_13e27;

/// Parsec (m).
pub const PARSEC: Scalar = 3.085_677_581e16;

/// Light year (m).
pub const LIGHT_YEAR: Scalar = 9.460_730_473e15;

/// Standard gravitational parameter for the Sun (m³/s²).
pub const GM_SUN: Scalar = 1.327_124_4e20;

/// Standard gravitational parameter for Earth (m³/s²).
pub const GM_EARTH: Scalar = 3.986_004_418e14;
