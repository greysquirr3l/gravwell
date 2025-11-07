//! # Gravwell - Ultra-Realistic Gravity Simulation
//!
//! Gravwell is a high-performance Rust library for ultra-realistic gravity simulation
//! designed for games and astrophysics applications. It provides multiple integration
//! methods, force calculation algorithms, and optimization techniques to achieve
//! scientifically accurate simulations while maintaining 60 FPS performance.
//!
//! ## Quick Start
//!
//! ```rust
//! use gravwell::prelude::*;
//!
//! // Create a simple two-body system
//! let mut simulation = SimulationBuilder::new()
//!     .with_integrator(VelocityVerlet::new())
//!     .with_force_calculator(DirectGravity::new())
//!     .build();
//!
//! // Add bodies (e.g., Earth and Moon)
//! simulation.add_body(Body::new()
//!     .with_mass(5.972e24)  // Earth mass in kg
//!     .with_position([0.0, 0.0, 0.0])
//!     .with_velocity([0.0, 0.0, 0.0]));
//!
//! simulation.add_body(Body::new()
//!     .with_mass(7.342e22)  // Moon mass in kg
//!     .with_position([384400000.0, 0.0, 0.0])  // Moon distance in m
//!     .with_velocity([0.0, 1022.0, 0.0]));  // Moon orbital velocity in m/s
//!
//! // Run simulation
//! let timestep = 3600.0; // 1 hour
//! for _ in 0..8760 {  // 1 year
//!     simulation.step(timestep);
//! }
//! ```
//!
//! ## Features
//!
//! - **Multiple Integrators**: Velocity Verlet, Leapfrog, RK4, IAS15
//! - **Force Algorithms**: Direct O(N²), Barnes-Hut O(N log N), Fast Multipole O(N)
//! - **Performance**: SIMD vectorization, parallel computation, 60 FPS capable
//! - **Accuracy**: Energy conservation, symplectic integrators, adaptive timesteps
//! - **Flexibility**: Trait-based design, builder patterns, optional features

#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod adaptive;
pub mod core;
pub mod forces;
pub mod integrators;
pub mod lod;
pub mod memory;
pub mod simd;
pub mod utils;

#[cfg(feature = "std")]
pub mod collision;

pub mod builder;
pub mod error;
pub mod prelude;
pub mod types;

pub use builder::{BodyHandle, SimulationBuilder};
pub use error::{GravwellError, Result};
pub use types::*;
