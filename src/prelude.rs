//! Convenient re-exports for common Gravwell usage.
//!
//! This module provides a convenient way to import the most commonly used
//! types and traits from Gravwell with a single `use` statement.

pub use crate::{
    adaptive::{
        AdaptationStrategy, AdaptiveTimestepController, ErrorMetric, ErrorTrend, StabilityAnalysis,
        StabilityWarning,
    },
    builder::{BodyHandle, SimulationBuilder},
    core::{
        forces::ForceCalculator,
        integrator::Integrator,
        math::Math,
        particle::{Body, ParticleSet},
    },
    error::{GravwellError, Result},
    forces::{barnes_hut::BarnesHut, direct::DirectGravity},
    integrators::{
        euler::SemiImplicitEuler, ias15::IAS15, leapfrog::Leapfrog, rk4::RungeKutta4,
        verlet::VelocityVerlet,
    },
    lod::{DetailLevel, LODPerformanceStats, LODSystem},
    simd::{SimdLevel, VectorizedGravity},
    types::*,
    types::{Scalar, Vector3},
    utils::constants::*,
};

// Parallel processing re-exports (when feature is enabled)
#[cfg(feature = "parallel")]
pub use crate::{
    forces::parallel::{ChunkSizeStrategy, ParallelDirectGravity},
    integrators::parallel::ParallelVelocityVerlet,
};

#[cfg(feature = "std")]
pub use crate::collision::{aabb::AabbTree, spatial_hash::SpatialHash};
