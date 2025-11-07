//! Convenient re-exports for common Gravwell usage.
//!
//! This module provides a convenient way to import the most commonly used
//! types and traits from Gravwell with a single `use` statement.

pub use crate::{
    builder::{BodyHandle, SimulationBuilder},
    core::{
        forces::ForceCalculator,
        integrator::Integrator,
        math::Math,
        particle::{Body, ParticleSet},
    },
    error::{GravwellError, Result},
    forces::{barnes_hut::BarnesHut, direct::DirectGravity},
    integrators::{leapfrog::Leapfrog, rk4::RungeKutta4, verlet::VelocityVerlet},
    simd::{SimdLevel, VectorizedGravity},
    types::*,
    types::{Scalar, Vector3},
    utils::constants::*,
};

#[cfg(feature = "std")]
pub use crate::collision::{aabb::AabbTree, spatial_hash::SpatialHash};
