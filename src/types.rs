//! Core type definitions for Gravwell.

use nalgebra as na;

/// 3D vector type used throughout Gravwell.
pub type Vector3 = na::Vector3<f64>;

/// 3D point type used for positions.
pub type Point3 = na::Point3<f64>;

/// Scalar type used for masses, times, energies, etc.
pub type Scalar = f64;

/// Mass type for bodies.
pub type Mass = Scalar;

/// Time type for simulation timesteps.
pub type Time = Scalar;

/// Energy type for energy calculations.
pub type Energy = Scalar;

/// Momentum type for momentum calculations.
pub type Momentum = Vector3;

/// Force type for force calculations.
pub type Force = Vector3;

/// Acceleration type for accelerations.
pub type Acceleration = Vector3;

/// Velocity type for velocities.
pub type Velocity = Vector3;

/// Position type for positions.
pub type Position = Vector3;
