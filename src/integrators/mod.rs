//! Numerical integration methods.

pub mod euler;
pub mod ias15;
pub mod leapfrog;
pub mod rk4;
pub mod verlet;

#[cfg(feature = "parallel")]
pub mod parallel;

pub use euler::SemiImplicitEuler;

#[cfg(feature = "parallel")]
pub use parallel::ParallelVelocityVerlet;
