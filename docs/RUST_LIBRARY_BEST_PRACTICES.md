# Rust Library Best Practices for Gravity Simulation Crate

## Executive Summary

This document outlines **comprehensive Rust library design principles** for building a production-grade, reusable gravity simulation crate. Following these practices ensures the library is ergonomic, maintainable, performant, and idiomatic for downstream consumers.

**Core Principles**:

1. **Minimal, Composable API** - Easy things easy, complex things possible
2. **Zero-Cost Abstractions** - Performance without runtime overhead
3. **Clear Stability Guarantees** - Semantic versioning with explicit stability markers
4. **Excellent Documentation** - Self-documenting code with comprehensive examples
5. **Opt-In Complexity** - Feature flags for optional dependencies
6. **Safe by Default** - Leverage Rust's type system for correctness

---

## 1. Crate Structure and Organization

### 1.1 Workspace Layout

```
gravity_sim/
├── Cargo.toml              # Workspace root
├── README.md
├── LICENSE
├── CHANGELOG.md
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
│
├── crates/
│   ├── gravity_sim/        # Main library crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs      # Public API surface
│   │   │   ├── core/       # Core abstractions (no_std)
│   │   │   ├── integrators/
│   │   │   ├── forces/
│   │   │   └── ...
│   │   ├── tests/          # Integration tests
│   │   ├── benches/        # Benchmarks
│   │   └── examples/       # Executable examples
│   │
│   ├── gravity_sim_derive/ # Procedural macros (optional)
│   │   └── src/lib.rs
│   │
│   └── gravity_sim_benches/ # Separate benchmark suite
│       └── ...
│
├── examples/               # Workspace-level examples
│   ├── basic_orbit.rs
│   └── solar_system.rs
│
└── xtask/                  # Development automation
    └── src/main.rs
```

### 1.2 Cargo.toml Structure

```toml
[package]
name = "gravity_sim"
version = "0.1.0"
edition = "2021"
rust-version = "1.70"  # MSRV - Minimum Supported Rust Version
authors = ["Your Name <email@example.com>"]
license = "MIT OR Apache-2.0"  # Dual license is standard
repository = "https://github.com/username/gravity_sim"
homepage = "https://github.com/username/gravity_sim"
documentation = "https://docs.rs/gravity_sim"
description = "Ultra-realistic gravity simulation for games and scientific computing"
keywords = ["physics", "gravity", "simulation", "n-body", "game-physics"]
categories = ["game-development", "science", "simulation"]
readme = "README.md"
include = [
    "src/**/*",
    "Cargo.toml",
    "LICENSE-*",
    "README.md",
    "CHANGELOG.md",
]

[workspace]
members = ["crates/*"]
resolver = "2"

[lib]
name = "gravity_sim"
path = "src/lib.rs"

# Optimize for library use
[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3

[profile.bench]
inherits = "release"
debug = true

# Core dependencies - minimal and stable
[dependencies]
# Linear algebra - feature-gated for no_std
nalgebra = { version = "0.32", default-features = false, features = ["libm"] }
num-traits = { version = "0.2", default-features = false }

# Optional dependencies
rayon = { version = "1.7", optional = true }
serde = { version = "1.0", optional = true, features = ["derive"] }
wgpu = { version = "0.18", optional = true }
bytemuck = { version = "1.14", optional = true, features = ["derive"] }

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.0"
approx = "0.5"

# Feature flags - additive only
[features]
default = ["std"]

# Core features
std = ["nalgebra/std", "num-traits/std"]
libm = ["nalgebra/libm"]  # For no_std environments

# Performance features
parallel = ["std", "rayon"]
simd = []
gpu = ["wgpu", "bytemuck", "std"]

# Utility features
serde = ["dep:serde", "nalgebra/serde-serialize"]
arbitrary-precision = ["dep:rug"]

# Meta-features
performance-60fps = ["simd", "parallel"]
full = ["std", "parallel", "simd", "gpu", "serde"]

# Internal features (not public API)
_internal_test_utils = []

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
targets = ["x86_64-unknown-linux-gnu"]

[[bench]]
name = "force_calculation"
harness = false

[[example]]
name = "basic_orbit"
required-features = ["std"]
```

---

## 2. API Design Principles

### 2.1 The Prelude Pattern

Provide a prelude module for convenient imports:

```rust
// src/prelude.rs
//! Convenient re-exports for common use cases.
//!
//! # Examples
//!
//! ```
//! use gravity_sim::prelude::*;
//!
//! let mut sim = Simulation::builder()
//!     .integrator(SemiImplicitEuler::new())
//!     .build();
//! ```

// Re-export core types
pub use crate::core::{
    Body, BodyHandle, ParticleSet,
    Vector3, Scalar,
};

// Re-export builders
pub use crate::Simulation;
pub use crate::SimulationBuilder;

// Re-export common integrators
pub use crate::integrators::{
    SemiImplicitEuler,
    VelocityVerlet,
    Leapfrog,
};

// Re-export common force calculators
pub use crate::forces::{
    DirectGravity,
    BarnesHut,
};

// Re-export traits for advanced use
pub use crate::traits::{
    Integrator,
    ForceCalculator,
    CollisionHandler,
};
```

### 2.2 Builder Pattern for Complex Configuration

```rust
// src/builder.rs
use crate::*;

/// Builder for configuring a [`Simulation`].
///
/// # Examples
///
/// ```
/// use gravity_sim::prelude::*;
///
/// let sim = Simulation::builder()
///     .integrator(VelocityVerlet::new())
///     .gravity(BarnesHut::new().theta(0.5))
///     .timestep(0.01)
///     .build()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug)]
pub struct SimulationBuilder<I = DefaultIntegrator, F = DefaultForce> {
    integrator: Option<I>,
    force_calculator: Option<F>,
    timestep: Option<f64>,
    collision_handler: Option<Box<dyn CollisionHandler>>,
    config: SimulationConfig,
    _phantom: PhantomData<(I, F)>,
}

impl SimulationBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            integrator: None,
            force_calculator: None,
            timestep: None,
            collision_handler: None,
            config: SimulationConfig::default(),
            _phantom: PhantomData,
        }
    }
}

impl<I, F> SimulationBuilder<I, F> {
    /// Set the numerical integrator.
    ///
    /// # Examples
    ///
    /// ```
    /// use gravity_sim::prelude::*;
    ///
    /// let builder = Simulation::builder()
    ///     .integrator(Leapfrog::new());
    /// ```
    pub fn integrator<I2>(self, integrator: I2) -> SimulationBuilder<I2, F>
    where
        I2: Integrator,
    {
        SimulationBuilder {
            integrator: Some(integrator),
            force_calculator: self.force_calculator,
            timestep: self.timestep,
            collision_handler: self.collision_handler,
            config: self.config,
            _phantom: PhantomData,
        }
    }

    /// Set the force calculator.
    pub fn gravity<F2>(self, force_calculator: F2) -> SimulationBuilder<I, F2>
    where
        F2: ForceCalculator,
    {
        SimulationBuilder {
            integrator: self.integrator,
            force_calculator: Some(force_calculator),
            timestep: self.timestep,
            collision_handler: self.collision_handler,
            config: self.config,
            _phantom: PhantomData,
        }
    }

    /// Set the timestep in seconds.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `dt <= 0.0`.
    pub fn timestep(mut self, dt: f64) -> Self {
        debug_assert!(dt > 0.0, "timestep must be positive");
        self.timestep = Some(dt);
        self
    }

    /// Build the simulation.
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are not set.
    pub fn build(self) -> Result<Simulation<I, F>, BuildError>
    where
        I: Integrator,
        F: ForceCalculator,
    {
        let integrator = self.integrator.ok_or(BuildError::MissingIntegrator)?;
        let force_calculator = self.force_calculator.ok_or(BuildError::MissingForceCalculator)?;
        let timestep = self.timestep.unwrap_or(1.0 / 60.0);

        Ok(Simulation {
            particles: ParticleSet::new(),
            integrator,
            force_calculator,
            collision_handler: self.collision_handler,
            timestep,
            time: 0.0,
            config: self.config,
        })
    }
}

/// Error type for simulation building.
#[derive(Debug, Clone, thiserror::Error)]
pub enum BuildError {
    #[error("integrator not specified")]
    MissingIntegrator,
    #[error("force calculator not specified")]
    MissingForceCalculator,
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}
```

### 2.3 Trait-Based Design

```rust
// src/traits.rs
//! Core traits for extensibility.

/// A numerical integrator for advancing the simulation.
///
/// Implement this trait to create custom integration schemes.
///
/// # Examples
///
/// ```
/// use gravity_sim::prelude::*;
///
/// struct MyIntegrator;
///
/// impl Integrator for MyIntegrator {
///     fn step(&mut self, system: &mut System, dt: f64) {
///         // Custom integration logic
///     }
/// }
/// ```
pub trait Integrator {
    /// Advance the system by one timestep.
    ///
    /// # Arguments
    ///
    /// * `system` - The system to advance
    /// * `dt` - The timestep in seconds
    fn step(&mut self, system: &mut System, dt: f64);

    /// Get the recommended timestep for stability.
    ///
    /// Returns `None` if the integrator is adaptive.
    fn recommended_timestep(&self, _system: &System) -> Option<f64> {
        None
    }

    /// Returns `true` if this integrator is symplectic (energy-conserving).
    fn is_symplectic(&self) -> bool {
        false
    }

    /// Returns the order of accuracy.
    fn order(&self) -> u32 {
        1
    }
}

/// Calculate gravitational forces between particles.
pub trait ForceCalculator: Send + Sync {
    /// Compute forces for all particles.
    ///
    /// # Arguments
    ///
    /// * `particles` - The particle system
    /// * `forces` - Output buffer for computed forces
    fn compute_forces(&self, particles: &ParticleSet, forces: &mut [Vector3<f64>]);

    /// Get the computational complexity.
    fn complexity(&self) -> Complexity {
        Complexity::ON2
    }

    /// Returns `true` if this calculator supports SIMD.
    fn supports_simd(&self) -> bool {
        false
    }
}

/// Computational complexity categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Complexity {
    /// O(N) - Linear
    ON,
    /// O(N log N) - Tree-based methods
    ONLogN,
    /// O(N²) - Direct summation
    ON2,
}
```

### 2.4 Error Handling

```rust
// src/error.rs
//! Error types for the gravity simulation library.

use std::fmt;

/// The main error type for simulation operations.
///
/// This type is marked `#[non_exhaustive]` to allow adding new
/// error variants without breaking changes.
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    /// Invalid body handle.
    #[error("invalid body handle: {0:?}")]
    InvalidHandle(BodyHandle),

    /// Physics error (energy not conserved, numerical instability, etc.)
    #[error("physics error: {0}")]
    Physics(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// I/O error (file loading, serialization, etc.)
    #[cfg(feature = "std")]
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// GPU error.
    #[cfg(feature = "gpu")]
    #[error("GPU error: {0}")]
    Gpu(String),
}

/// A specialized [`Result`] type for simulation operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Physics validation error.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub kind: ValidationKind,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationKind {
    EnergyDrift,
    MomentumDrift,
    NumericalInstability,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: {} (value: {:.3e}, threshold: {:.3e})",
            self.kind, self.message, self.value, self.threshold
        )
    }
}

impl std::error::Error for ValidationError {}
```

### 2.5 Type Aliases and Newtypes

```rust
// src/types.rs
//! Core type definitions.

use nalgebra as na;

/// Scalar type for calculations.
///
/// This is `f64` by default but can be changed for different
/// precision requirements.
pub type Scalar = f64;

/// 3D vector type.
pub type Vector3 = na::Vector3<Scalar>;

/// 3D point type.
pub type Point3 = na::Point3<Scalar>;

/// Body mass in kilograms.
///
/// This newtype provides type safety and prevents mixing up
/// masses with other scalar quantities.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mass(pub Scalar);

impl Mass {
    /// Create a new mass.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if mass is negative.
    pub fn new(value: Scalar) -> Self {
        debug_assert!(value >= 0.0, "mass cannot be negative");
        Self(value)
    }

    /// Get the mass value.
    pub fn value(&self) -> Scalar {
        self.0
    }

    /// Solar mass constant (1.989 × 10³⁰ kg).
    pub const SOLAR_MASS: Self = Self(1.989e30);

    /// Earth mass constant (5.972 × 10²⁴ kg).
    pub const EARTH_MASS: Self = Self(5.972e24);
}

/// Body handle - opaque reference to a body in the simulation.
///
/// Handles are stable across simulation steps and can be serialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyHandle(pub(crate) usize);

impl BodyHandle {
    /// Get the raw index (for internal use).
    #[doc(hidden)]
    pub fn index(&self) -> usize {
        self.0
    }
}
```

---

## 3. Documentation Standards

### 3.1 Module-Level Documentation

```rust
// src/integrators/mod.rs
//! Numerical integrators for advancing the simulation.
//!
//! This module provides various integration schemes with different
//! trade-offs between accuracy, stability, and performance.
//!
//! # Choosing an Integrator
//!
//! | Integrator | Order | Symplectic | Use Case |
//! |------------|-------|------------|----------|
//! | [`SemiImplicitEuler`] | 1st | No | Real-time games |
//! | [`VelocityVerlet`] | 2nd | Yes | General purpose |
//! | [`Leapfrog`] | 2nd | Yes | Long-term stability |
//! | [`RK4`] | 4th | No | High accuracy |
//!
//! # Examples
//!
//! ```
//! use gravity_sim::prelude::*;
//!
//! // For games - fast and stable
//! let game_integrator = SemiImplicitEuler::new();
//!
//! // For science - energy conserving
//! let science_integrator = VelocityVerlet::new();
//! ```
//!
//! # References
//!
//! - Hairer, E., et al. "Geometric Numerical Integration"

pub mod semi_implicit_euler;
pub mod velocity_verlet;
pub mod leapfrog;
pub mod rk4;

pub use semi_implicit_euler::SemiImplicitEuler;
pub use velocity_verlet::VelocityVerlet;
pub use leapfrog::Leapfrog;
pub use rk4::RK4;
```

### 3.2 Function-Level Documentation

```rust
/// Add a body to the simulation.
///
/// Returns a handle that can be used to query or modify the body later.
///
/// # Examples
///
/// ```
/// use gravity_sim::prelude::*;
///
/// let mut sim = Simulation::builder().build()?;
///
/// let earth = sim.add_body(
///     Body::new()
///         .mass(5.972e24)
///         .position([0.0, 0.0, 0.0])
///         .velocity([0.0, 0.0, 0.0])
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if the simulation is at maximum capacity.
///
/// # See Also
///
/// * [`remove_body`](Self::remove_body) - Remove a body
/// * [`get_body`](Self::get_body) - Query body properties
pub fn add_body(&mut self, body: Body) -> Result<BodyHandle> {
    // Implementation
}
```

### 3.3 Inline Examples

Every public function should have at least one working example:

```rust
/// Calculate orbital period for a circular orbit.
///
/// # Examples
///
/// ```
/// use gravity_sim::orbital;
///
/// // Earth's orbital period around the Sun
/// let period = orbital::period(
///     1.989e30,     // Sun mass (kg)
///     1.496e11      // 1 AU (m)
/// );
///
/// assert!((period - 365.25 * 86400.0).abs() < 1e6);  // ~1 year
/// ```
///
/// # Panics
///
/// Panics if `radius <= 0.0` or `central_mass <= 0.0`.
pub fn period(central_mass: f64, radius: f64) -> f64 {
    debug_assert!(radius > 0.0 && central_mass > 0.0);
    2.0 * PI * (radius.powi(3) / (G * central_mass)).sqrt()
}
```

### 3.4 Documentation Features

```rust
// Mark unstable APIs
#[doc(hidden)]
pub fn internal_function() {}

// Document features required
#[cfg(feature = "gpu")]
#[doc(cfg(feature = "gpu"))]
pub mod gpu {
    //! GPU acceleration support.
    //!
    //! This module is only available with the `gpu` feature enabled.
}

// Add doctests that shouldn't run
/// ```no_run
/// // This example requires user interaction
/// let input = read_user_input();
/// ```

// Add doctests that should fail
/// ```should_panic
/// // This should panic
/// let invalid = Body::new().mass(-1.0);
/// ```

// Compile-only doctests
/// ```compile_fail
/// // This shouldn't compile
/// let sim: Simulation = "not a simulation";
/// ```
```

---

## 4. Feature Flag Management

### 4.1 Feature Guidelines

**Principles:**

- Features should be **additive only** (enabling features should never break code)
- Features should be **independent** where possible
- Features should **minimize cascading dependencies**
- Use **feature-gated imports** with proper cfg attributes

```rust
// src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// Core always available
pub mod core;
pub mod integrators;
pub mod forces;

// Feature-gated modules
#[cfg(feature = "parallel")]
#[cfg_attr(docsrs, doc(cfg(feature = "parallel")))]
pub mod parallel;

#[cfg(feature = "gpu")]
#[cfg_attr(docsrs, doc(cfg(feature = "gpu")))]
pub mod gpu;

#[cfg(feature = "serde")]
mod serde_impl;
```

### 4.2 Feature Combinations Testing

```toml
# .github/workflows/features.yml
name: Feature Combinations

on: [push, pull_request]

jobs:
  test-features:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        features:
          - ""                    # no features
          - "std"                 # default
          - "parallel"
          - "gpu"
          - "std,parallel"
          - "std,parallel,simd"
          - "full"                # all features
    steps:
      - uses: actions/checkout@v3
      - run: cargo test --no-default-features --features ${{ matrix.features }}
```

### 4.3 Conditional Compilation

```rust
// Feature-specific implementations
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use std::arch::x86_64::*;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

// Different implementations based on features
#[cfg(feature = "parallel")]
fn calculate_forces(particles: &ParticleSet) -> Vec<Vector3> {
    particles.par_iter().map(|p| calculate_force(p)).collect()
}

#[cfg(not(feature = "parallel"))]
fn calculate_forces(particles: &ParticleSet) -> Vec<Vector3> {
    particles.iter().map(|p| calculate_force(p)).collect()
}

// Feature-specific types
#[cfg(feature = "std")]
pub type DynError = Box<dyn std::error::Error + Send + Sync>;

#[cfg(not(feature = "std"))]
pub type DynError = &'static str;
```

---

## 5. Semantic Versioning and Stability

### 5.1 Version Management

Follow [SemVer 2.0](https://semver.org/):

- **0.1.x** - Initial development, API unstable
- **0.2.x** - Major API revision, still unstable
- **1.0.0** - Stable API, breaking changes bump major version
- **1.x.y** - Bug fixes (patch) and backward-compatible additions (minor)

### 5.2 Stability Attributes

```rust
/// This API is stable and follows SemVer.
#[stable(since = "1.0.0")]
pub struct Simulation { }

/// This API is experimental and may change.
#[unstable(feature = "experimental_integrators")]
pub struct ExperimentalIntegrator { }

/// This function is deprecated.
#[deprecated(since = "1.2.0", note = "use `new_function` instead")]
pub fn old_function() { }
```

### 5.3 CHANGELOG.md Format

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New feature X

### Changed
- Improved performance of Y

### Deprecated
- Function Z (use W instead)

### Removed
- Old API (breaking change)

### Fixed
- Bug in collision detection

### Security
- Fixed vulnerability in serialization

## [1.2.0] - 2024-01-15

### Added
- Barnes-Hut tree optimization (#42)
- SIMD support for x86_64 (#45)

...
```

---

## 6. Testing Strategy

### 6.1 Test Organization

```
tests/
├── integration/          # Integration tests
│   ├── basic_orbit.rs
│   ├── collision.rs
│   └── serialization.rs
├── validation/           # Physics validation
│   ├── kepler.rs
│   └── energy_conservation.rs
└── property/             # Property-based tests
    └── conservation_laws.rs
```

### 6.2 Unit Tests

```rust
// src/integrators/velocity_verlet.rs
impl VelocityVerlet {
    // Implementation...
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_single_step() {
        let mut integrator = VelocityVerlet::new();
        // Test implementation
    }

    #[test]
    fn test_energy_conservation() {
        // Test energy conservation for harmonic oscillator
    }

    #[test]
    #[should_panic(expected = "timestep must be positive")]
    fn test_negative_timestep() {
        let mut integrator = VelocityVerlet::new();
        // Should panic
    }
}
```

### 6.3 Integration Tests

```rust
// tests/integration/basic_orbit.rs
use gravity_sim::prelude::*;

#[test]
fn test_circular_orbit() -> Result<()> {
    let mut sim = Simulation::builder()
        .integrator(VelocityVerlet::new())
        .gravity(DirectGravity::new())
        .timestep(0.01)
        .build()?;

    // Add Sun and Earth
    sim.add_body(Body::sun())?;
    sim.add_body(Body::earth())?;

    // Run for one year
    for _ in 0..365 {
        sim.step();
    }

    // Validate orbit closure
    assert!(sim.validate_orbit_closure()?);

    Ok(())
}
```

### 6.4 Property-Based Tests

```rust
// tests/property/conservation_laws.rs
use proptest::prelude::*;
use gravity_sim::prelude::*;

proptest! {
    #[test]
    fn momentum_conserved(
        masses in prop::collection::vec(0.1f64..100.0, 2..10),
        velocities in prop::collection::vec(prop::array::uniform3(-10.0f64..10.0), 2..10)
    ) {
        let mut sim = create_test_sim(masses, velocities);
        let initial_momentum = sim.total_momentum();

        for _ in 0..100 {
            sim.step();
        }

        let final_momentum = sim.total_momentum();
        prop_assert!((initial_momentum - final_momentum).norm() < 1e-10);
    }
}
```

### 6.5 Documentation Tests

All examples in documentation should compile and run:

```rust
/// Calculate escape velocity.
///
/// ```
/// use gravity_sim::orbital;
///
/// let escape_vel = orbital::escape_velocity(5.972e24, 6.371e6);
/// assert!((escape_vel - 11186.0).abs() < 1.0);  // ~11.2 km/s
/// ```
pub fn escape_velocity(mass: f64, radius: f64) -> f64 {
    (2.0 * G * mass / radius).sqrt()
}
```

---

## 7. Performance Considerations

### 7.1 Benchmarking

```rust
// benches/force_calculation.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use gravity_sim::prelude::*;

fn benchmark_force_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("force_calculation");

    for n in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::new("direct", n), n, |b, &n| {
            let particles = create_random_particles(n);
            let calculator = DirectGravity::new();

            b.iter(|| {
                let forces = calculator.compute_forces(black_box(&particles));
                black_box(forces)
            });
        });

        group.bench_with_input(BenchmarkId::new("barnes_hut", n), n, |b, &n| {
            let particles = create_random_particles(n);
            let calculator = BarnesHut::new().theta(0.5);

            b.iter(|| {
                let forces = calculator.compute_forces(black_box(&particles));
                black_box(forces)
            });
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_force_calculation);
criterion_main!(benches);
```

### 7.2 Performance Guidelines

```rust
// Use #[inline] strategically
#[inline]
pub fn distance_squared(a: &Vector3, b: &Vector3) -> f64 {
    (a - b).norm_squared()
}

// Hot paths should avoid allocations
#[inline(never)]  // Prevent inlining to get better profiles
pub fn calculate_forces_hot_path(&self, particles: &ParticleSet, forces: &mut [Vector3]) {
    // Pre-allocated buffers, no allocations in loop
    for i in 0..particles.len() {
        forces[i] = self.calculate_single_force(i, particles);
    }
}

// Use const where possible
pub const G: f64 = 6.67430e-11;
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;
```

---

## 8. Safety and no_std Support

### 8.1 no_std Compatibility

```rust
// src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{vec::Vec, boxed::Box, string::String};

#[cfg(feature = "std")]
use std::{vec::Vec, boxed::Box, string::String};

// Core functionality should work without std
pub mod core {
    // Implementation using only libcore/alloc
}
```

### 8.2 Unsafe Code Guidelines

```rust
/// # Safety
///
/// The caller must ensure that:
/// - `ptr` is valid for reads of `len` elements
/// - `ptr` is properly aligned
/// - The memory referenced by `ptr` lives for the entire duration of the slice
pub unsafe fn from_raw_parts(ptr: *const f64, len: usize) -> &'static [f64] {
    // SAFETY: Caller guarantees are documented above
    std::slice::from_raw_parts(ptr, len)
}

// Prefer safe abstractions
pub fn safe_alternative(data: &[f64]) -> Vec<f64> {
    // No unsafe needed
    data.to_vec()
}
```

---

## 9. Examples and Documentation

### 9.1 Example Structure

```rust
// examples/basic_orbit.rs
//! Simulate a simple two-body orbit.
//!
//! Run with: cargo run --example basic_orbit

use gravity_sim::prelude::*;

fn main() -> Result<()> {
    // Setup simulation
    let mut sim = Simulation::builder()
        .integrator(VelocityVerlet::new())
        .gravity(DirectGravity::new())
        .timestep(3600.0)  // 1 hour
        .build()?;

    // Add Sun
    let sun = sim.add_body(
        Body::new()
            .mass(1.989e30)
            .position([0.0, 0.0, 0.0])
            .name("Sun")
    )?;

    // Add Earth
    let earth = sim.add_body(
        Body::new()
            .mass(5.972e24)
            .position([1.496e11, 0.0, 0.0])
            .velocity([0.0, 29780.0, 0.0])
            .name("Earth")
    )?;

    // Run simulation for one year
    println!("Simulating Earth's orbit...");
    for day in 0..365 {
        for _ in 0..24 {
            sim.step();
        }

        if day % 30 == 0 {
            let pos = sim.position(earth)?;
            let vel = sim.velocity(earth)?;
            println!("Day {}: pos = {:?}, vel = {:.2} km/s",
                day, pos, vel.norm() / 1000.0);
        }
    }

    // Validate energy conservation
    let energy_drift = sim.energy_drift();
    println!("Energy drift: {:.3e}", energy_drift);

    Ok(())
}
```

### 9.2 README.md Template

```markdown
# gravity_sim

[![Crates.io](https://img.shields.io/crates/v/gravity_sim.svg)](https://crates.io/crates/gravity_sim)
[![Documentation](https://docs.rs/gravity_sim/badge.svg)](https://docs.rs/gravity_sim)
[![License](https://img.shields.io/crates/l/gravity_sim.svg)](LICENSE)
[![CI](https://github.com/user/gravity_sim/workflows/CI/badge.svg)](https://github.com/user/gravity_sim/actions)

Ultra-realistic gravity simulation for games and scientific computing.

## Features

- 🎮 **Game Mode**: Real-time performance with stable, bounded behavior
- 🔬 **Science Mode**: High-accuracy symplectic integrators
- ⚡ **Fast**: SIMD, multi-threading, GPU acceleration
- 🦀 **Pure Rust**: Memory-safe with zero-cost abstractions
- 📦 **Modular**: `no_std` core with optional features

## Quick Start

```rust
use gravity_sim::prelude::*;

fn main() -> Result<()> {
    let mut sim = Simulation::builder()
        .integrator(VelocityVerlet::new())
        .gravity(BarnesHut::new())
        .build()?;

    sim.add_body(Body::sun())?;
    sim.add_body(Body::earth())?;

    for _ in 0..365 {
        sim.step();
    }

    Ok(())
}
```

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
gravity_sim = "0.1"

# Enable optional features
gravity_sim = { version = "0.1", features = ["parallel", "simd"] }
```

## Documentation

- [API Documentation](https://docs.rs/gravity_sim)
- [User Guide](https://github.com/user/gravity_sim/blob/main/docs/guide.md)
- [Examples](https://github.com/user/gravity_sim/tree/main/examples)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

```

---

## 10. CI/CD and Automation

### 10.1 GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  RUSTFLAGS: -D warnings
  RUST_BACKTRACE: 1

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, nightly]
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      
      - name: Run tests
        run: cargo test --all-features

  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --all-features -- -D warnings

  doc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo doc --all-features --no-deps

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@cargo-tarpaulin
      - run: cargo tarpaulin --all-features --out Xml
      - uses: codecov/codecov-action@v3
```

---

## Summary: Best Practices Checklist

### API Design

- ✅ Provide a prelude module for convenient imports
- ✅ Use builder pattern for complex configuration
- ✅ Design traits for extensibility
- ✅ Use newtype pattern for type safety
- ✅ Provide handle-based API to avoid lifetime issues

### Error Handling

- ✅ Define custom error types with `thiserror`
- ✅ Use `Result<T, Error>` for fallible operations
- ✅ Mark error types as `#[non_exhaustive]`
- ✅ Provide helpful error messages

### Documentation

- ✅ Document every public item
- ✅ Include examples in doc comments
- ✅ Write integration examples
- ✅ Maintain comprehensive README
- ✅ Keep CHANGELOG.md updated

### Features

- ✅ Features are additive only
- ✅ Core is `no_std` compatible
- ✅ Optional dependencies behind feature flags
- ✅ Test all feature combinations

### Testing

- ✅ Unit tests for individual functions
- ✅ Integration tests for workflows
- ✅ Property-based tests for invariants
- ✅ Benchmark critical paths
- ✅ All doc examples compile and run

### Performance

- ✅ Use `#[inline]` strategically
- ✅ Avoid allocations in hot paths
- ✅ Benchmark regularly
- ✅ Profile before optimizing

### Safety

- ✅ Minimize unsafe code
- ✅ Document safety requirements
- ✅ Support `no_std` where possible
- ✅ Use `clippy` and `rustfmt`

### Publishing

- ✅ Follow SemVer
- ✅ Maintain CHANGELOG
- ✅ Set up CI/CD
- ✅ Use appropriate crate metadata
- ✅ Choose permissive license (MIT/Apache-2.0)

This document provides a comprehensive foundation for building a production-quality Rust library that follows ecosystem best practices!
