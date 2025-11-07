# INITIAL_PROMPT.md: Gravwell - Ultra-Realistic Gravity Simulation

> 📘 **Companion Documents**:
>
> - [60FPS_REQUIREMENTS.md](./60FPS_REQUIREMENTS.md) - Achieving 60 FPS performance
> - [RUST_LIBRARY_BEST_PRACTICES.md](./RUST_LIBRARY_BEST_PRACTICES.md) - Rust library design patterns

## Project Vision

**Gravwell** is a production-grade Rust crate for ultra-realistic gravity simulation
that serves **both game development and scientific computing** communities through
a dual-mode architecture. This library follows **Rust ecosystem best practices**
for reusable, idiomatic, and maintainable library design.

**Tagline**: *Realistic gravity wells for games and astrophysics*

**Core Philosophy:** Rather than compromise between game performance and scientific
accuracy, implement distinct but composable modes that excel in their respective
domains while sharing common abstractions.

## Project Goals

### Primary Goals

1. **Multi-Scale Support**: Seamlessly handle planetary/solar system scale, local
    surface gravity, and intermediate scales
2. **Dual Performance Modes**:
   - **Game Mode**: Real-time performance (1000+ bodies at 30+ FPS) with bounded, stable behavior
   - **Science Mode**: High accuracy (energy conservation < 10⁻¹⁰) with symplectic integration
3. **Professional Architecture**: Clean separation of concerns, testable, extensible, well-documented
4. **Rust Ecosystem Integration**: `no_std` core, optional GPU acceleration, Bevy/other engine plugins

### Secondary Goals

- WASM compatibility for browser-based simulations
- Deterministic physics for networked games (optional mode)
- Built-in benchmarking and validation tools
- Comprehensive examples for both game devs and scientists

### Stretch Goals (60 FPS Performance)

- **60 FPS target** for game mode with optimized configurations (see [60FPS_REQUIREMENTS.md](./60FPS_REQUIREMENTS.md))
- Physics/render rate decoupling with interpolation for smooth 60 FPS visuals
- 1,000-5,000 particles at 60 FPS with full optimization stack
- VR-ready performance (90 FPS) for smaller particle counts (N ≤ 1,000)

## Critical Success Criteria

### Tier 1: Minimum Viable Product (MVP)

- ✅ Energy errors < 10⁻⁶ for two-body circular orbits over 100 periods
- ✅ Handle 1,000 particles at 30 FPS (single-threaded)
- ✅ Clean API requiring < 50 lines for basic solar system setup
- ✅ At least two integrators: Semi-implicit Euler and Velocity Verlet
- ✅ Basic collision detection and handling
- ✅ 80%+ test coverage with analytical validation

**Stretch (60 FPS)**:

- ⭐ 500 particles at 60 FPS with SIMD optimization
- ⭐ Physics/render rate decoupling implemented

### Tier 2: Production Ready

- ✅ Energy errors < 10⁻¹⁰ for symplectic integrators
- ✅ 10,000 particles at 30 FPS with multithreading (80%+ parallel efficiency)
- ✅ Barnes-Hut tree optimization (O(N log N) complexity)
- ✅ Trait-based extensibility for custom integrators and forces
- ✅ Comprehensive documentation with examples for both audiences
- ✅ 90%+ test coverage including property-based tests
- ✅ Published to crates.io with CI/CD pipeline

**Stretch (60 FPS)**:

- ⭐ 1,000-5,000 particles at 60 FPS with full optimization stack
- ⭐ LOD (Level of Detail) system for dynamic fidelity scaling
- ⭐ Spatial culling for large open-world scenarios
- ⭐ Frame time variance < 2ms for smooth gameplay

### Tier 3: Research Grade

- ✅ Parity with REBOUND on standard astrophysics benchmarks
- ✅ 100,000+ particles with GPU acceleration
- ✅ Advanced integrators: RK4, IAS15 (adaptive 15th-order), WHFast
- ✅ Post-Newtonian corrections for relativistic systems
- ✅ Built-in visualization and analysis tools
- ✅ Published paper demonstrating novel optimizations or applications

**Stretch (60 FPS)**:

- ⭐ 10,000+ particles at 60 FPS with GPU + LOD + culling
- ⭐ VR-ready performance (90 FPS) for N ≤ 1,000
- ⭐ Async physics thread for guaranteed smooth frame times
- ⭐ 50,000+ particles at 60 FPS in "visual showcase" mode

> 📘 **Note**: For detailed strategies, optimizations, and implementation guidance for achieving 60 FPS performance, see the supplementary document [60FPS_REQUIREMENTS.md](./60FPS_REQUIREMENTS.md).

## Technical Architecture

### Core Design Principles

> 📘 **For comprehensive Rust library design patterns, see [RUST_LIBRARY_BEST_PRACTICES.md](./RUST_LIBRARY_BEST_PRACTICES.md)**

1. **Trait-Based Abstraction**: Define core traits (`Integrator`, `ForceCalculator`, `CollisionHandler`) with multiple implementations
2. **Zero-Cost Abstractions**: Use generics for compile-time dispatch, trait objects only where runtime selection needed
3. **SoA Data Layout**: Structure-of-Arrays for SIMD-friendly vectorization
4. **No Premature GPU Optimization**: Build fast CPU implementation first, add GPU as optional feature
5. **`no_std` Core**: Keep physics engine free of std dependencies, add features behind flags
6. **Library-First Design**: Follow Rust ecosystem conventions for reusable library crates
7. **Minimal, Composable API**: Provide sensible defaults with escape hatches for experts
8. **Safe by Default**: Leverage Rust's type system to prevent misuse at compile time

### Module Structure

```plaintext
gravwell/                  # Workspace root
├── Cargo.toml             # Workspace manifest
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── CHANGELOG.md
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
│
├── crates/
│   └── gravwell/          # Main library crate
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs              # Public API and re-exports
│       │   ├── prelude.rs          # Convenient imports module
│       │   ├── error.rs            # Error types
│       │   ├── types.rs            # Core type definitions
│       │   ├── builder.rs          # Builder pattern implementation
│       │   ├── core/               # Core abstractions (no_std)
│       │   │   ├── mod.rs
│       │   │   ├── integrator.rs   # Integrator trait
│       │   │   ├── forces.rs       # ForceCalculator trait
│       │   │   ├── particle.rs     # ParticleSet and Body types
│       │   │   └── math.rs         # Vector math utilities
│       │   ├── integrators/        # Concrete integrator implementations
│       │   │   ├── mod.rs
│       │   │   ├── euler.rs        # Semi-implicit Euler (game mode)
│       │   │   ├── verlet.rs       # Velocity Verlet (basic science)
│       │   │   ├── leapfrog.rs     # Leapfrog (symplectic)
│       │   │   ├── rk4.rs          # Runge-Kutta 4th order
│       │   │   └── ias15.rs        # IAS15 adaptive (advanced)
│       │   ├── forces/             # Force calculation methods
│       │   │   ├── mod.rs
│       │   │   ├── direct.rs       # O(N²) brute force
│       │   │   ├── barnes_hut.rs   # O(N log N) tree method
│       │   │   └── fmm.rs          # Fast Multipole Method (future)
│       │   ├── collision/          # Collision detection and response
│       │   │   ├── mod.rs
│       │   │   ├── broad_phase.rs  # Spatial partitioning
│       │   │   ├── narrow_phase.rs # Exact collision tests
│       │   │   └── response.rs     # Collision response physics
│       │   ├── parallel/           # Parallelization (optional feature)
│       │   │   ├── mod.rs
│       │   │   ├── cpu.rs          # Rayon-based CPU parallelism
│       │   │   └── gpu.rs          # WGPU compute shaders
│       │   ├── performance/        # 60 FPS optimizations (optional)
│       │   │   ├── mod.rs
│       │   │   ├── lod.rs          # Level of Detail system
│       │   │   ├── culling.rs      # Spatial culling
│       │   │   ├── interpolation.rs # Physics/render rate decoupling
│       │   │   └── async_physics.rs # Async physics thread
│       │   └── utils/              # Utilities
│       │       ├── mod.rs
│       │       ├── validation.rs   # Energy/momentum conservation checks
│       │       └── benchmark.rs    # Built-in benchmarking
│       ├── tests/                  # Integration tests
│       │   ├── integration/
│       │   ├── validation/
│       │   └── property/
│       ├── benches/                # Criterion benchmarks
│       │   └── force_calculation.rs
│       └── examples/               # Executable examples
│           ├── basic_orbit.rs
│           ├── solar_system.rs
│           └── ...
│
├── examples/                       # Workspace-level examples
│   ├── game/                       # Game development examples
│   │   ├── solar_system.rs
│   │   ├── planetary_landing.rs
│   │   └── orbital_mechanics.rs
│   └── science/                    # Scientific computing examples
│       ├── kepler_validation.rs
│       ├── three_body.rs
│       └── galaxy_collision.rs
│
├── docs/                           # Documentation (mdBook)
│   ├── book.toml
│   ├── src/
│   │   ├── SUMMARY.md
│   │   ├── introduction.md
│   │   ├── getting_started.md
│   │   └── ...
│
└── xtask/                          # Development automation
    └── src/main.rs
```

### Key Traits

> 📘 **See [RUST_LIBRARY_BEST_PRACTICES.md](./RUST_LIBRARY_BEST_PRACTICES.md#trait-based-design) for detailed trait design patterns**

```rust
/// A numerical integrator for advancing the simulation.
///
/// Implement this trait to create custom integration schemes.
///
/// # Examples
///
/// ```
/// use gravwell::prelude::*;
///
/// struct MyIntegrator;
///
/// impl Integrator for MyIntegrator {
///     fn step(&mut self, system: &mut System, dt: f64) {
///         // Custom integration logic
///     }
/// }
/// ```
///
/// # Stability and Accuracy
///
/// Different integrators provide different trade-offs:
/// - **Order**: Higher order = better accuracy per timestep
/// - **Symplectic**: Preserves phase space structure (energy conservation)
/// - **Adaptive**: Can adjust timestep automatically
pub trait Integrator {
    /// Advance the system by one timestep.
    ///
    /// # Arguments
    ///
    /// * `system` - The system to advance
    /// * `dt` - The timestep in seconds (must be positive)
    ///
    /// # Examples
    ///
    /// ```
    /// use gravwell::prelude::*;
    ///
    /// let mut sim = Simulation::builder().build()?;
    /// sim.step(0.01);  // 10ms timestep
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn step(&mut self, system: &mut System, dt: f64);
    
    /// Get the recommended timestep for this integrator.
    ///
    /// Returns `None` for adaptive integrators.
    fn recommended_timestep(&self, system: &System) -> Option<f64> {
        None
    }
    
    /// Returns `true` if this integrator is symplectic (energy-conserving).
    fn is_symplectic(&self) -> bool {
        false
    }
    
    /// Returns the order of accuracy.
    ///
    /// Higher order means better accuracy per timestep.
    fn order(&self) -> u32 {
        1
    }
}

/// Calculate gravitational forces between particles.
///
/// # Examples
///
/// ```
/// use gravwell::prelude::*;
///
/// let calculator = BarnesHut::new()
///     .theta(0.5)
///     .softening(0.01);
///
/// let mut forces = vec![Vector3::zeros(); particles.len()];
/// calculator.compute_forces(&particles, &mut forces);
/// ```
pub trait ForceCalculator: Send + Sync {
    /// Compute forces for all particles.
    ///
    /// # Arguments
    ///
    /// * `particles` - The particle system
    /// * `forces` - Output buffer for computed forces (must be same length as particles)
    ///
    /// # Panics
    ///
    /// May panic if `forces.len() != particles.len()`.
    fn compute_forces(&self, particles: &ParticleSet, forces: &mut [Vector3<f64>]);
    
    /// Get the computational complexity.
    fn complexity(&self) -> Complexity {
        Complexity::ON2
    }
    
    /// Returns `true` if this calculator supports SIMD vectorization.
    fn supports_simd(&self) -> bool {
        false
    }
}

/// Computational complexity categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Complexity {
    /// O(N) - Linear complexity
    ON,
    /// O(N log N) - Tree-based methods like Barnes-Hut
    ONLogN,
    /// O(N²) - Direct summation (brute force)
    ON2,
}

/// Collision detection and response.
///
/// # Examples
///
/// ```
/// use gravwell::prelude::*;
///
/// let handler = InelasticMerger::new();
/// let collisions = handler.detect(&particles);
///
/// for collision in collisions {
///     handler.resolve(&mut particles, collision);
/// }
/// ```
pub trait CollisionHandler {
    /// Detect collisions and return pairs.
    ///
    /// # Returns
    ///
    /// A vector of collision pairs with collision details.
    fn detect(&self, particles: &ParticleSet) -> Vec<CollisionPair>;
    
    /// Resolve collision between two particles.
    ///
    /// # Arguments
    ///
    /// * `particles` - The particle system to modify
    /// * `collision` - The collision to resolve
    fn resolve(&self, particles: &mut ParticleSet, collision: &CollisionPair);
}
```

### Data Structures

> 📘 **See [RUST_LIBRARY_BEST_PRACTICES.md](./RUST_LIBRARY_BEST_PRACTICES.md#type-aliases-and-newtypes) for type safety patterns**

```rust
// Core type definitions for maximum type safety
use nalgebra as na;

/// Scalar type for calculations (f64 for scientific accuracy).
pub type Scalar = f64;

/// 3D vector type.
pub type Vector3 = na::Vector3<Scalar>;

/// 3D point type.
pub type Point3 = na::Point3<Scalar>;

/// Body mass with type safety.
///
/// This newtype prevents mixing masses with other scalar quantities.
///
/// # Examples
///
/// ```
/// use gravwell::prelude::*;
///
/// let earth_mass = Mass::EARTH_MASS;
/// let sun_mass = Mass::SOLAR_MASS;
/// ```
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

    /// Get the mass value in kilograms.
    pub fn value(&self) -> Scalar {
        self.0
    }

    /// Solar mass constant (1.989 × 10³⁰ kg).
    pub const SOLAR_MASS: Self = Self(1.989e30);

    /// Earth mass constant (5.972 × 10²⁴ kg).
    pub const EARTH_MASS: Self = Self(5.972e24);
}

/// Structure-of-Arrays layout for SIMD efficiency.
///
/// This layout stores each component in a contiguous array,
/// enabling efficient vectorization.
///
/// # Examples
///
/// ```
/// use gravwell::prelude::*;
///
/// let mut particles = ParticleSet::new();
/// let handle = particles.add(Body::new()
///     .mass(Mass::EARTH_MASS)
///     .position([0.0, 0.0, 0.0]));
/// ```
#[derive(Debug, Clone)]
pub struct ParticleSet {
    /// Positions (m)
    pub positions: Vec<Vector3>,
    /// Velocities (m/s)
    pub velocities: Vec<Vector3>,
    /// Accelerations (m/s²)
    pub accelerations: Vec<Vector3>,
    /// Masses (kg)
    pub masses: Vec<Mass>,
    /// Radii for collision detection (m)
    pub radii: Vec<Scalar>,
    /// Number of active particles
    count: usize,
}

impl ParticleSet {
    /// Create a new empty particle set.
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Create a particle set with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            positions: Vec::with_capacity(capacity),
            velocities: Vec::with_capacity(capacity),
            accelerations: Vec::with_capacity(capacity),
            masses: Vec::with_capacity(capacity),
            radii: Vec::with_capacity(capacity),
            count: 0,
        }
    }

    /// Get the number of particles.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns `true` if the set contains no particles.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Individual particle handle (opaque, stable reference).
///
/// Handles remain valid until the particle is explicitly removed.
/// They can be copied, stored, and compared.
///
/// # Examples
///
/// ```
/// use gravwell::prelude::*;
///
/// let mut sim = Simulation::builder().build()?;
/// let earth = sim.add_body(Body::earth())?;
///
/// // Use handle later
/// let position = sim.position(earth)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyHandle(pub(crate) usize);

/// Main simulation container with compile-time type parameters.
///
/// Generic over integrator and force calculator types for zero-cost abstraction.
///
/// # Type Parameters
///
/// * `I` - The integrator type
/// * `F` - The force calculator type
///
/// # Examples
///
/// ```
/// use gravwell::prelude::*;
///
/// let mut sim: Simulation<VelocityVerlet, BarnesHut> = Simulation::builder()
///     .integrator(VelocityVerlet::new())
///     .gravity(BarnesHut::new())
///     .build()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Simulation<I: Integrator, F: ForceCalculator> {
    particles: ParticleSet,
    integrator: I,
    force_calculator: F,
    collision_handler: Option<Box<dyn CollisionHandler>>,
    time: f64,
    timestep: f64,
    energy_history: Vec<f64>,  // For validation
    config: SimulationConfig,
}

/// Configuration options for the simulation.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Enable energy conservation monitoring
    pub monitor_energy: bool,
    /// Energy conservation threshold (relative)
    pub energy_threshold: f64,
    /// Enable momentum conservation monitoring
    pub monitor_momentum: bool,
    /// Momentum conservation threshold (absolute)
    pub momentum_threshold: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            monitor_energy: true,
            energy_threshold: 1e-10,
            monitor_momentum: true,
            momentum_threshold: 1e-14,
        }
    }
}
```

## Rust Library Design Patterns

> 📘 **For comprehensive details, see [RUST_LIBRARY_BEST_PRACTICES.md](./RUST_LIBRARY_BEST_PRACTICES.md)**

### API Design Principles

**The Prelude Pattern**
Provide a convenience module for common imports:

```rust
// src/prelude.rs
//! Convenient re-exports for common use cases.
//!
//! # Examples
//!
//! ```
//! use gravwell::prelude::*;
//!
//! let mut sim = Simulation::builder()
//!     .integrator(SemiImplicitEuler::new())
//!     .build()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub use crate::core::{Body, BodyHandle, ParticleSet, Vector3, Scalar, Mass};
pub use crate::{Simulation, SimulationBuilder};
pub use crate::integrators::{SemiImplicitEuler, VelocityVerlet, Leapfrog};
pub use crate::forces::{DirectGravity, BarnesHut};
pub use crate::traits::{Integrator, ForceCalculator, CollisionHandler};
pub use crate::{Error, Result};
```

**Error Handling**

Use `thiserror` for ergonomic error types:

```rust
// src/error.rs
//! Error types for the gravity simulation library.

use std::fmt;

/// Main error type for simulation operations.
///
/// Marked `#[non_exhaustive]` to allow adding variants without breaking changes.
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    /// Invalid body handle
    #[error("invalid body handle: {0:?}")]
    InvalidHandle(BodyHandle),

    /// Physics validation failed
    #[error("physics error: {0}")]
    Physics(String),

    /// Configuration error
    #[error("configuration error: {0}")]
    Config(String),

    /// I/O error
    #[cfg(feature = "std")]
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[cfg(feature = "serde")]
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Specialized [`Result`] type for simulation operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Physics validation error with detailed diagnostics.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub kind: ValidationKind,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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

**Builder Pattern with Type-State**

```rust
// src/builder.rs
/// Builder for configuring a [`Simulation`].
///
/// # Examples
///
/// ```
/// use gravwell::prelude::*;
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
            timestep: Some(1.0 / 60.0),  // Default 60 Hz
            collision_handler: None,
            config: SimulationConfig::default(),
            _phantom: PhantomData,
        }
    }
}

impl<I, F> SimulationBuilder<I, F> {
    /// Set the numerical integrator.
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
    /// Returns [`Error::Config`] if required fields are not set.
    pub fn build(self) -> Result<Simulation<I, F>>
    where
        I: Integrator,
        F: ForceCalculator,
    {
        let integrator = self.integrator
            .ok_or_else(|| Error::Config("integrator not specified".into()))?;
        let force_calculator = self.force_calculator
            .ok_or_else(|| Error::Config("force calculator not specified".into()))?;
        let timestep = self.timestep.unwrap_or(1.0 / 60.0);

        Ok(Simulation {
            particles: ParticleSet::new(),
            integrator,
            force_calculator,
            collision_handler: self.collision_handler,
            timestep,
            time: 0.0,
            energy_history: Vec::new(),
            config: self.config,
        })
    }
}
```

### Testing Infrastructure

**Unit Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_circular_orbit_energy_conservation() {
        let mut sim = create_test_simulation();
        let initial_energy = sim.total_energy();

        for _ in 0..1000 {
            sim.step();
        }

        let final_energy = sim.total_energy();
        assert_relative_eq!(initial_energy, final_energy, epsilon = 1e-10);
    }
}
```

**Property-Based Tests**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn momentum_always_conserved(
        masses in prop::collection::vec(0.1f64..100.0, 2..10),
        velocities in prop::collection::vec(
            prop::array::uniform3(-10.0f64..10.0), 2..10
        )
    ) {
        let mut sim = create_sim_from_params(masses, velocities);
        let initial_momentum = sim.total_momentum();

        for _ in 0..100 {
            sim.step();
        }

        let final_momentum = sim.total_momentum();
        prop_assert!((initial_momentum - final_momentum).norm() < 1e-10);
    }
}
```

### Documentation Standards

Every public item must have:

1. **Doc comment** with description
2. **Examples** section with working code
3. **Errors** section if returning `Result`
4. **Panics** section if may panic
5. **Safety** section if unsafe

```rust
/// Add a body to the simulation.
///
/// Returns a handle that can be used to query or modify the body later.
///
/// # Examples
///
/// ```
/// use gravwell::prelude::*;
///
/// let mut sim = Simulation::builder().build()?;
///
/// let earth = sim.add_body(
///     Body::new()
///         .mass(Mass::EARTH_MASS)
///         .position([0.0, 0.0, 0.0])
///         .velocity([0.0, 0.0, 0.0])
/// )?;
/// # Ok::<(), gravwell::Error>(())
/// ```
///
/// # Errors
///
/// Returns [`Error::Config`] if the simulation is at maximum capacity.
///
/// # See Also
///
/// * [`remove_body`](Self::remove_body) - Remove a body
/// * [`get_body`](Self::get_body) - Query body properties
pub fn add_body(&mut self, body: Body) -> Result<BodyHandle> {
    // Implementation
}
```

> 📘 **For comprehensive implementation details, see [60FPS_REQUIREMENTS.md](./60FPS_REQUIREMENTS.md)**

Achieving 60 FPS (16.67ms frame budget) versus 30 FPS (33.33ms) requires
**fundamentally different optimization strategies**. The following approaches are
listed in priority order for implementation:

### Critical Optimization #1: Physics/Render Rate Decoupling

**The most impactful optimization** - update physics at 20-30 Hz while rendering
at 60 Hz with interpolation:

```rust
pub struct DecoupledSimulation {
    physics_rate: f64,        // e.g., 30 Hz
    render_rate: f64,         // 60 Hz
    physics_accumulator: f64,
    previous_state: ParticleSet,
    current_state: ParticleSet,
}

impl DecoupledSimulation {
    pub fn update(&mut self, frame_dt: f64) {
        self.physics_accumulator += frame_dt;
        let physics_dt = 1.0 / self.physics_rate;
        
        // Run physics at lower rate
        while self.physics_accumulator >= physics_dt {
            self.previous_state = self.current_state.clone();
            self.step_physics(physics_dt);
            self.physics_accumulator -= physics_dt;
        }
    }
    
    // Interpolate for smooth rendering
    pub fn get_render_state(&self) -> ParticleSet {
        let alpha = self.physics_accumulator / (1.0 / self.physics_rate);
        self.previous_state.lerp(&self.current_state, alpha)
    }
}
```

**Impact**: 2-3× more time for physics calculations with no visual quality loss.

### Critical Optimization #2: SIMD Vectorization

Process 4-8 particles simultaneously with AVX/AVX-512 instructions:

```rust
use std::simd::*;

// Process 8 particles at once
let forces_simd = calculate_forces_avx512(
    &positions_x_vec,  // f64x8
    &positions_y_vec,
    &positions_z_vec,
    &masses_vec
);
```

**Impact**: 4-8× speedup for force calculations.

### Critical Optimization #3: Level of Detail (LOD)

Dynamically adjust simulation fidelity based on distance/importance:

```rust
pub enum LODLevel {
    High,      // Full physics every frame
    Medium,    // Physics every 2 frames
    Low,       // Physics every 4 frames
    Culled,    // Orbit approximation only
}

// Assign LOD based on camera distance
pub fn assign_lod(&self, particle_pos: Vector3<f64>) -> LODLevel {
    let distance = (particle_pos - camera_pos).norm();
    match distance {
        d if d < 100.0 => LODLevel::High,
        d if d < 500.0 => LODLevel::Medium,
        d if d < 2000.0 => LODLevel::Low,
        _ => LODLevel::Culled,
    }
}
```

**Impact**: Handle 10-100× more particles by updating only nearby/visible ones.

### Performance Targets by Optimization Level

| Particle Count | Basic (30 FPS) | + Decoupling | + SIMD | + LOD | + All |
|----------------|----------------|--------------|--------|-------|-------|
| 500 | ✅ Easy | ✅ Trivial | ✅ Trivial | ✅ Trivial | ✅ 60 FPS |
| 1,000 | ✅ Achievable | ✅ Easy | ✅ Easy | ✅ Easy | ✅ 60 FPS |
| 5,000 | ⚠️ Hard | ✅ Achievable | ✅ Achievable | ✅ Easy | ✅ 60 FPS |
| 10,000 | ❌ GPU needed | ⚠️ Hard | ⚠️ Hard | ✅ Achievable | ✅ 60 FPS |
| 20,000+ | ❌ | ❌ | ⚠️ Very Hard | ✅ Achievable | ✅ 60 FPS |

### Optional Advanced Optimizations

**Spatial Culling**: Don't simulate distant particles

```rust
pub struct SpatialCuller {
    active_region: AABB,
    influence_radius: f64,
}
// Only simulate particles within active region
```

**Async Physics Thread**: Run physics on separate thread

```rust
pub struct AsyncSimulation {
    physics_thread: JoinHandle<()>,
    state_channel: mpsc::Receiver<ParticleSet>,
}
// Physics never blocks rendering
```

**GPU Threshold Changes**: For 60 FPS, GPU becomes beneficial at **lower particle counts**:

- 30 FPS: GPU helps at N > 20,000
- 60 FPS: GPU helps at N > 5,000

### Recommended 60 FPS Configuration

```rust
// Targeting 1,000-5,000 particles @ 60 FPS
let sim = Simulation::builder()
    .physics_rate(30.0)      // 30 Hz physics (33ms budget)
    .render_rate(60.0)       // 60 Hz rendering
    .integrator(SemiImplicitEuler::new())
    .gravity(BarnesHut::new()
        .theta(0.6)          // Balanced accuracy/speed
        .simd(true))         // Enable SIMD
    .parallel(true)          // Multi-threading
    .lod_system(LODSystem::new()
        .distances([100.0, 500.0, 2000.0])
        .update_frequencies([1, 2, 4, 8]))
    .spatial_culling(true)
    .target_frame_time(16.0) // Try to stay under 16ms
    .build();
```

### Implementation Priority for 60 FPS

1. **Phase 1**: Physics/render decoupling + interpolation
2. **Phase 2**: SIMD vectorization + basic LOD
3. **Phase 3**: Spatial culling + async physics (optional)

**See [60FPS_REQUIREMENTS.md](./60FPS_REQUIREMENTS.md) for complete implementation details, benchmarking strategies, and advanced optimization techniques.**

## Implementation Phases

### Phase 1: Foundation (Weeks 1-3)

**Goal**: Establish core abstractions and validate basic physics

**Deliverables**:

- Core traits defined (`Integrator`, `ForceCalculator`, `CollisionHandler`)
- `ParticleSet` with SoA layout
- Semi-implicit Euler integrator (game mode)
- Velocity Verlet integrator (science mode)
- Direct O(N²) force calculation
- Two-body Kepler orbit validation tests
- Energy and momentum conservation monitoring
- Basic API with builder pattern

**Success Metrics**:

- Two-body circular orbit maintains energy drift < 10⁻⁶ over 100 orbits
- API ergonomics validated with 3+ example scenarios
- All tests passing with 80%+ coverage

**60 FPS Stretch Goals**:

- ⭐ Physics/render rate decoupling implemented
- ⭐ 500 particles @ 60 FPS (direct O(N²) with interpolation)
- ⭐ Frame time profiling system in place

**Example Usage (Phase 1)**:

```rust
use gravwell::prelude::*;

// Game mode: fast, stable, bounded
let mut sim = Simulation::builder()
    .integrator(SemiImplicitEuler::new())
    .gravity(DirectGravity::new())
    .timestep(0.016) // 60 FPS
    .build();

// Add Earth orbiting Sun
sim.add_body(Body::sun());
sim.add_body(Body::earth());

for _ in 0..1000 {
    sim.step();
    println!("Energy: {}", sim.total_energy());
}
```

### Phase 2: Optimization & Scale (Weeks 4-8)

**Goal**: Scale to 10,000 particles with real-time performance

**Deliverables**:

- Barnes-Hut tree force calculation (O(N log N))
- Broad-phase collision detection (spatial grid or AABB tree)
- Narrow-phase collision with GJK algorithm
- Rayon-based CPU parallelization
- Leapfrog and RK4 integrators
- Softening parameter support
- Adaptive timestep system
- Comprehensive benchmarking suite

**Success Metrics**:

- 10,000 particles at 30 FPS (with Barnes-Hut + parallel)
- 80%+ parallel efficiency on 8+ cores
- Energy drift < 10⁻¹⁰ for Leapfrog on Kepler problem
- Barnes-Hut produces < 1% force error at θ=0.5

**60 FPS Stretch Goals**:

- ⭐ 1,000 particles @ 60 FPS (Barnes-Hut + SIMD + decoupling)
- ⭐ SIMD vectorization for force calculations (4-8× speedup)
- ⭐ Basic LOD system implementation
- ⭐ 5,000 particles @ 60 FPS with LOD enabled
- ⭐ Frame time variance < 2ms

**Example Usage (Phase 2)**:

```rust
// Science mode: accurate, energy-conserving
let mut sim = Simulation::builder()
    .integrator(Leapfrog::new())
    .gravity(BarnesHut::new()
        .theta(0.5)
        .softening(0.01))
    .collision_detection(SpatialGrid::new(1.0))
    .parallel(true)  // Enable Rayon
    .build();

// Load 10,000 particle galaxy
sim.load_galaxy_ic("galaxy_ic.dat")?;

// Run with adaptive timesteps
for step in 0..100_000 {
    sim.step_adaptive();
    if step % 1000 == 0 {
        sim.validate_energy()?;
        sim.checkpoint(format!("galaxy_{}.dat", step))?;
    }
}
```

### Phase 3: Advanced Features (Weeks 9-12)

**Goal**: Research-grade capabilities and ecosystem integration

**Deliverables**:

- IAS15 adaptive 15th-order integrator
- WHFast integrator for planetary systems
- GPU acceleration via WGPU (optional feature)
- WASM compatibility and browser demo
- Bevy engine integration plugin
- Post-Newtonian corrections (optional)
- Built-in visualization module
- Fast Multipole Method (stretch goal)
- Published crate with full documentation

**Success Metrics**:

- Matches REBOUND energy conservation on standard benchmarks
- 100,000+ particles with GPU acceleration
- Bevy plugin works with existing Bevy physics architecture
- WASM demo runs in browser at 30+ FPS
- Documentation complete with 20+ examples
- Published paper or blog post demonstrating novel optimizations

**60 FPS Stretch Goals**:

- ⭐ 10,000+ particles @ 60 FPS with GPU acceleration
- ⭐ Spatial culling system for open-world scenarios
- ⭐ Async physics thread for guaranteed smooth frames
- ⭐ VR-ready performance (90 FPS) for N ≤ 1,000
- ⭐ 50,000+ particles @ 60 FPS in "visual showcase" mode
- ⭐ Complete 60 FPS optimization guide published

**Example Usage (Phase 3)**:

```rust
// GPU-accelerated massive simulation
let mut sim = Simulation::builder()
    .integrator(IAS15::new()
        .tolerance(1e-12)
        .adaptive(true))
    .gravity(BarnesHut::new()
        .theta(0.3)
        .softening(0.001))
    .accelerator(GpuAccelerator::new()?)  // Optional GPU
    .build();

// 1 million particle dark matter halo
sim.load_halo_ic("halo_1M.dat")?;

sim.evolve_until(t_final, |sim, t| {
    if t % snapshot_interval < dt {
        sim.save_snapshot(format!("snap_{:.2}.dat", t))?;
    }
    Ok(())
})?;
```

## Detailed Technical Specifications

### Integrators

#### Semi-Implicit Euler (Game Mode)

- **Order**: 1st order
- **Symplectic**: No
- **Use Case**: Real-time games requiring stability over accuracy
- **Characteristics**: Bounded energy behavior, excellent for oscillators, fast
- **Timestep**: Fixed, typically 1/60 to 1/120 second
- **Implementation**:

  ```rust
  v_new = v + a * dt
  x_new = x + v_new * dt  // Use updated velocity
  ```

#### Velocity Verlet (Basic Science)

- **Order**: 2nd order
- **Symplectic**: Yes
- **Use Case**: General-purpose accurate integration
- **Characteristics**: Time-reversible, energy-conserving, 1 force eval per step
- **Timestep**: Adaptive or fixed based on shortest dynamical time
- **Implementation**:

  ```rust
  x_new = x + v * dt + 0.5 * a * dt²
  a_new = compute_acceleration(x_new)
  v_new = v + 0.5 * (a + a_new) * dt
  ```

#### Leapfrog (Symplectic)

- **Order**: 2nd order
- **Symplectic**: Yes
- **Use Case**: Long-term orbital stability
- **Characteristics**: Exactly energy-conserving for harmonic systems
- **Timestep**: Fixed, chosen for stability
- **Implementation**: Interleaved velocity and position updates

#### RK4 (High Accuracy)

- **Order**: 4th order
- **Symplectic**: No
- **Use Case**: Short-term high-precision calculations
- **Characteristics**: Low error per step, 4 force evaluations
- **Timestep**: Adaptive based on error estimation
- **Implementation**: Classic 4-stage Runge-Kutta

#### IAS15 (Research Grade)

- **Order**: 15th order adaptive
- **Symplectic**: No (but excellent energy conservation)
- **Use Case**: Extreme accuracy requirements, close encounters
- **Characteristics**: Embedded error estimation, automatic timestep adaptation
- **Timestep**: Individually adaptive per particle
- **Reference**: REBOUND's IAS15 implementation

### Force Calculation Methods

#### Direct O(N²)

- **Complexity**: O(N²)
- **Use Case**: N < 1,000 particles, exact forces needed
- **Optimization**: SIMD vectorization, symmetry exploitation
- **Parameters**: Optional softening length ε

#### Barnes-Hut Tree

- **Complexity**: O(N log N)
- **Use Case**: 1,000 < N < 1,000,000
- **Accuracy**: Controlled by θ parameter (typical: 0.3-0.7)
- **Implementation**: Octree with center-of-mass approximation
- **Parameters**:
  - `theta`: Opening angle (0.5 recommended)
  - `softening`: Gravitational softening length
  - `max_depth`: Tree depth limit (auto-computed default)

#### Fast Multipole Method (Future)

- **Complexity**: O(N)
- **Use Case**: N > 100,000, extreme accuracy needed
- **Implementation**: Spherical harmonic expansions
- **Note**: Complex implementation, future stretch goal

### Collision Detection

#### Broad Phase Options

1. **Uniform Grid**: O(N), best for uniform distributions
2. **Spatial Hash**: O(N), good for dynamic systems
3. **AABB Tree**: O(N log N), excellent for clustered objects
4. **Sweep and Prune**: O(N + K) where K = collision pairs

#### Narrow Phase

- **GJK Algorithm**: Exact distance and collision detection for convex shapes
- **Sphere-Sphere**: Fast analytical test for spherical bodies
- **EPA (Expanding Polytope Algorithm)**: Contact point generation

#### Collision Response

1. **Perfectly Inelastic Merger** (Science Mode):

   ```rust
   m_new = m1 + m2
   v_new = (m1*v1 + m2*v2) / m_new
   ```

2. **Impulse-Based Resolution** (Game Mode):

   ```rust
   J = -(1 + e) * v_rel / (1/m1 + 1/m2)
   v1 += J/m1
   v2 -= J/m2
   ```

### Precision and Numerical Stability

#### Floating Point Strategy

- **Default**: f64 for all physics calculations
- **Optional**: f32 for game mode when using local coordinates
- **GPU**: Mixed precision (f32 computation, f64 accumulation)

#### Numerical Safeguards

1. **Kahan Summation**: For energy and momentum accumulation
2. **Compensated Arithmetic**: Optional for extreme precision needs
3. **Softening**: Prevent singularities at r=0
4. **Timestep Limits**: Maximum dt based on shortest dynamical time
5. **Energy Monitoring**: Automatic detection of integration failure

#### Coordinate Systems

- **Barycentric Frame**: Center-of-mass reference for N-body systems
- **Local Coordinates**: For game physics to maintain precision
- **Optional**: Arbitrary precision using `rug` crate for pathological cases

### Parallelization

#### CPU Parallelization (Rayon)

```rust
// Parallel force calculation
particles.positions
    .par_iter()
    .zip(forces.par_iter_mut())
    .for_each(|(pos, force)| {
        *force = calculate_force(*pos, &all_particles);
    });
```

**Expected Efficiency**: 80-95% on 8+ cores for N > 10,000

#### GPU Acceleration (WGPU)

- **Compute Shaders**: Particle force calculation
- **Memory Strategy**: Persistent GPU buffers, minimize transfers
- **Precision**: Mixed f32/f64 depending on hardware support
- **Portability**: Vulkan/Metal/DirectX12/WebGPU backends

### Validation and Testing

#### Analytical Test Cases

1. **Two-Body Kepler Problem**:
   - Circular orbit: Constant radius ± 10⁻¹⁰
   - Elliptical orbit: Correct period, eccentricity, orientation
   - Energy conservation: |ΔE/E| < 10⁻¹⁰ for symplectic

2. **Figure-8 Choreography**:
   - Three equal masses following figure-8 path
   - Period: 6.3259 time units
   - Validate stability over 10+ periods

3. **Pythagorean Three-Body**:
   - Masses 3:4:5, chaotic dynamics
   - Statistical comparison with high-precision reference

4. **Solar System**:
   - 8-planet simulation
   - Validate against JPL HORIZONS ephemeris
   - Energy drift < 10⁻⁸ over 100 years

#### Conservation Laws

- **Energy**: Monitor E(t) = T + V, check drift
- **Linear Momentum**: Should be exactly conserved (< 10⁻¹⁴ for f64)
- **Angular Momentum**: Conserved for isolated systems
- **Center of Mass**: Should remain at origin

#### Property-Based Testing

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn momentum_conserved_in_collisions(
            m1 in 0.1f64..10.0,
            m2 in 0.1f64..10.0,
            v1 in prop::array::uniform3(-10.0f64..10.0),
            v2 in prop::array::uniform3(-10.0f64..10.0),
        ) {
            let p_initial = m1*v1 + m2*v2;
            let (v1_new, v2_new) = collide(m1, m2, v1, v2);
            let p_final = m1*v1_new + m2*v2_new;
            assert!((p_initial - p_final).norm() < 1e-10);
        }
    }
}
```

#### Benchmarking

- **Criterion.rs**: Statistical performance analysis
- **Track Metrics**:
  - Force calculations per second
  - Timesteps per second
  - Scaling with particle count (1K, 10K, 100K, 1M)
  - Parallel efficiency
  - Memory bandwidth utilization

## API Design Philosophy

### Builder Pattern with Type Safety

```rust
// Compile-time validation of configuration
let sim = Simulation::builder()
    .integrator(Leapfrog::new())  // Type carries integrator choice
    .gravity(BarnesHut::new().theta(0.5))
    .collision_detection(SpatialGrid::new(1.0))
    .timestep(0.01)
    .validate_energy(true)
    .build()?;  // Returns Result for runtime validation
```

### Handle-Based Access

```rust
// No lifetimes, no borrow checker fights
let sun = sim.add_body(Body::sun());
let earth = sim.add_body(Body::earth());

// Query by handle
let earth_pos = sim.position(earth)?;
let sun_mass = sim.mass(sun)?;

// Handles remain valid until explicit removal
sim.remove_body(earth)?;
```

### Fluent Interface for Bodies

```rust
let body = Body::new()
    .mass(5.972e24)  // Earth mass in kg
    .radius(6.371e6)  // Earth radius in m
    .position([1.496e11, 0.0, 0.0])  // 1 AU from sun
    .velocity([0.0, 2.978e4, 0.0])  // Orbital velocity
    .name("Earth")
    .color([0.2, 0.5, 1.0]);  // For visualization
```

### Mode Selection

```rust
// Explicit mode selection for clarity
use gravwell::prelude::*;
use gravwell::game::*;     // Game-optimized types
use gravwell::science::*;  // Science-optimized types

// Game mode
let game_sim = GameSimulation::new()
    .fixed_timestep(1.0 / 60.0)
    .deterministic(true)  // For networked games
    .sleeping_enabled(true);  // Cull resting bodies

// Science mode
let science_sim = ScienceSimulation::new()
    .integrator(IAS15::new().tolerance(1e-12))
    .adaptive_timesteps(true)
    .monitor_energy(true);
```

## Dependencies and Ecosystem

### Core Dependencies (minimal)

```toml
[dependencies]
nalgebra = { version = "0.32", default-features = false }  # Linear algebra
num-traits = "0.2"  # Numeric trait bounds

[dev-dependencies]
criterion = "0.5"   # Benchmarking
proptest = "1.0"    # Property-based testing
approx = "0.5"      # Floating point comparisons
```

### Optional Feature Dependencies

```toml
[features]
default = []
std = []
parallel = ["rayon"]
gpu = ["wgpu", "bytemuck"]
serde = ["dep:serde", "nalgebra/serde-serialize"]
arbitrary-precision = ["rug"]
visualization = ["plotters"]
simd = []  # Enable SIMD optimizations
performance-60fps = ["simd", "parallel"]  # All 60 FPS optimizations

[dependencies]
rayon = { version = "1.7", optional = true }
wgpu = { version = "0.18", optional = true }
bytemuck = { version = "1.14", optional = true }
serde = { version = "1.0", optional = true, features = ["derive"] }
rug = { version = "1.21", optional = true }
plotters = { version = "0.3", optional = true }
```

### Integration with Bevy

```toml
# Separate crate: bevy_gravwell
[dependencies]
bevy = "0.12"
gravwell = "0.1"
```

## Documentation Requirements

### API Documentation

- Every public function with doc comments
- Examples in doc tests (runnable with `cargo test`)
- Module-level documentation explaining purpose and usage
- Links to related functions and types

### User Guide (mdBook)

1. **Getting Started**
   - Installation
   - First simulation in 5 minutes
   - Basic concepts

2. **Game Development Guide**
   - Real-time constraints
   - Integration with game engines
   - Performance optimization
   - Determinism for networking

3. **Scientific Computing Guide**
   - Accuracy vs performance trade-offs
   - Integrator selection
   - Validation methodology
   - Comparison with established codes

4. **API Reference**
   - Complete trait documentation
   - Implementation examples
   - Common patterns

5. **Advanced Topics**
   - GPU acceleration
   - Custom integrators
   - Parallel performance tuning
   - Numerical stability

### Examples

- **Minimum 20 examples** covering:
  - Basic orbital mechanics
  - Solar system simulation
  - Galaxy collisions
  - Planetary landing physics
  - Asteroid deflection
  - Binary star systems
  - Lagrange points
  - Hill sphere calculations
  - N-body chaos
  - Game integration demos

## Quality Assurance

### Testing Strategy

- **Unit Tests**: Every function with edge cases
- **Integration Tests**: Complete workflows
- **Property Tests**: Conservation laws with random inputs
- **Benchmark Tests**: Performance regression detection
- **Example Tests**: All examples compile and run
- **Doc Tests**: All documentation examples work

### CI/CD Pipeline

```yaml
# .github/workflows/ci.yml
- Run tests on Linux/macOS/Windows
- Check formatting (rustfmt)
- Run clippy lints
- Build documentation
- Run benchmarks (store historical data)
- Check code coverage (>90% target)
- Test WASM build
- Test all feature combinations
```

### Code Quality Standards

- **Rustfmt**: Consistent formatting
- **Clippy**: All warnings addressed
- **No Unsafe**: Except in performance-critical sections with extensive comments
- **Panic-Free**: Physics code should return Results, not panic
- **Zero Warnings**: Clean compilation

## Performance Targets

### Tier 1 (MVP)

**30 FPS Baseline:**

- 1,000 particles @ 30 FPS (single-threaded, direct N²)
- 100 particles @ 60 FPS (for game integration)

**60 FPS Stretch:**

- 500 particles @ 60 FPS (with physics/render decoupling)
- 100 particles @ 120 FPS (for high-refresh displays)

### Tier 2 (Production)

**30 FPS Baseline:**

- 10,000 particles @ 30 FPS (multithreaded, Barnes-Hut)
- 80%+ parallel efficiency on 8 cores
- Memory usage < 1 KB per particle

**60 FPS Stretch:**

- 1,000-5,000 particles @ 60 FPS (Barnes-Hut + SIMD + LOD)
- Frame time variance < 2ms
- 0.1% frame drops (< 1 in 1000 frames miss target)

### Tier 3 (Research)

**30 FPS Baseline:**

- 100,000 particles @ 30 FPS (GPU-accelerated)
- 1,000,000 particles @ 1 FPS (for offline simulations)
- WASM: 1,000 particles @ 30 FPS in browser

**60 FPS Stretch:**

- 10,000+ particles @ 60 FPS (GPU + LOD + culling)
- 50,000+ particles @ 60 FPS (visual showcase mode)
- VR-ready: 1,000 particles @ 90 FPS
- WASM: 1,000 particles @ 60 FPS in browser

> 📘 **See [60FPS_REQUIREMENTS.md](./60FPS_REQUIREMENTS.md)** for detailed optimization strategies, implementation priorities, and code examples for achieving 60 FPS performance.

## Lessons from Existing Implementations

### Anti-Patterns to Avoid (from kavan010/gravity_sim review)

❌ Hard-coded timesteps with magic numbers (dividing by 94, 96)
❌ O(N²) without spatial acceleration for all particle counts
❌ Tight coupling of physics and rendering
❌ Global state preventing multiple simulation instances
❌ No testing against analytical solutions
❌ Missing validation of conservation laws

### Best Practices to Adopt (from REBOUND, Rapier, etc.)

✅ Clean separation of physics engine from visualization
✅ Trait-based abstractions for extensibility
✅ Handle-based API avoiding lifetime issues
✅ Comprehensive validation test suite
✅ Optional features behind Cargo flags
✅ `no_std` core for maximum portability
✅ Builder pattern for ergonomic configuration
✅ Detailed documentation with examples for both audiences

## Success Metrics and Validation

### Quantitative Metrics

- **Energy Conservation**: |ΔE/E| < 10⁻¹⁰ for symplectic integrators
- **Momentum Conservation**: |Δp| < 10⁻¹⁴ (machine precision for f64)
- **Performance**: Meets all tier targets
- **Test Coverage**: >90% line coverage, 100% of public API
- **Documentation**: Every public item documented with examples

### Qualitative Metrics

- **API Ergonomics**: New users can run first simulation in <10 minutes
- **Community Adoption**: 100+ stars on GitHub, active issues/PRs
- **Industry Use**: At least one game or research project using the crate
- **Comparison**: Favorable mention in Rust physics engine comparisons

### Validation Against Established Codes

- **REBOUND**: Energy conservation within 10× on standard benchmarks
- **Rapier**: Performance parity for collision-heavy scenarios
- **NBODY6**: Comparable accuracy for star cluster simulations
- **JPL HORIZONS**: Solar system matches ephemeris within error bounds

## Maintenance and Community

### Semantic Versioning and Stability

> 📘 **See [RUST_LIBRARY_BEST_PRACTICES.md](./RUST_LIBRARY_BEST_PRACTICES.md#semantic-versioning-and-stability) for complete versioning guidelines**

Follow [SemVer 2.0](https://semver.org/) strictly:

- **0.1.x** - Initial development, API may change
- **0.2.x** - Major API revisions during development
- **1.0.0** - Stable API, breaking changes bump major version
- **1.x.y** - Bug fixes (patch) and backward-compatible additions (minor)

**API Stability Markers:**

```rust
/// This API is stable and follows SemVer.
#[stable(since = "1.0.0")]
pub struct Simulation { }

/// This function is deprecated.
#[deprecated(since = "1.2.0", note = "use `new_function` instead")]
pub fn old_function() { }

/// Mark non-exhaustive for future extensibility
#[non_exhaustive]
pub enum Error { }
```

### Publishing Checklist

Before publishing to crates.io:

- [ ] All tests passing (`cargo test --all-features`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Formatted (`cargo fmt --check`)
- [ ] Documentation builds (`cargo doc --all-features`)
- [ ] Examples compile and run
- [ ] CHANGELOG.md updated
- [ ] Version bumped according to SemVer
- [ ] README.md up to date
- [ ] LICENSE files present (MIT and Apache-2.0)
- [ ] CI passing on all platforms

**CHANGELOG.md Format:**

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- New feature X

### Changed
- Improved performance of Y

### Fixed
- Bug in collision detection

## [0.2.0] - 2024-01-15

### Added
- Barnes-Hut tree optimization (#42)
```

### Long-term Support Plan

- Semantic versioning (SemVer 2.0)
- CHANGELOG.md with detailed release notes
- Issue templates for bug reports and feature requests
- Contributing guidelines (CONTRIBUTING.md)
- Code of conduct (CODE_OF_CONDUCT.md)

### Community Engagement

- Regular blog posts on implementation details
- Tutorial videos for common use cases
- Active Discord/Zulip channel
- Participation in Rust gamedev and scientific computing communities
- Conference talks and papers

## Risk Mitigation

### Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| GPU acceleration complexity | High | Make optional, focus on CPU first |
| Numerical instability | Critical | Extensive testing, multiple integrators |
| Performance bottlenecks | High | Profile early, benchmark continuously |
| SIMD portability issues | Medium | Use portable abstractions (nalgebra) |
| Determinism across platforms | Medium | Optional strict mode, document limitations |

### Project Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Scope creep | High | Strict phase gating, MVP first |
| Maintenance burden | Medium | Clean architecture, good tests |
| Lack of adoption | Medium | Focus on documentation, examples |
| Competition from established libraries | Low | Unique dual-mode positioning |

## Deliverables Checklist

### Phase 1 Deliverables

- [ ] Core traits defined and documented
- [ ] ParticleSet with SoA layout
- [ ] Semi-implicit Euler integrator
- [ ] Velocity Verlet integrator
- [ ] Direct force calculation
- [ ] Basic collision handling
- [ ] Builder API
- [ ] 5+ examples
- [ ] Test suite with >80% coverage
- [ ] README with quickstart

**60 FPS Stretch:**

- [ ] Physics/render rate decoupling system
- [ ] Interpolation for smooth 60 FPS rendering
- [ ] Frame time profiling utilities
- [ ] 500 particles @ 60 FPS demo

### Phase 2 Deliverables

- [ ] Barnes-Hut implementation
- [ ] Leapfrog integrator
- [ ] RK4 integrator
- [ ] Rayon parallelization
- [ ] Broad-phase collision detection
- [ ] Adaptive timestep system
- [ ] Comprehensive benchmarks
- [ ] 15+ examples
- [ ] User guide (mdBook)
- [ ] Published to crates.io

**60 FPS Stretch:**

- [ ] SIMD vectorization for force calculations
- [ ] LOD (Level of Detail) system
- [ ] Spatial culling for large scenes
- [ ] 1,000 particles @ 60 FPS demo
- [ ] 5,000 particles @ 60 FPS with LOD demo
- [ ] Frame time variance analysis tools

### Phase 3 Deliverables

- [ ] IAS15 adaptive integrator
- [ ] WHFast integrator
- [ ] GPU acceleration (WGPU)
- [ ] WASM compatibility
- [ ] Bevy integration plugin
- [ ] Visualization module
- [ ] 20+ examples
- [ ] Complete documentation
- [ ] Published paper/blog post
- [ ] Version 1.0 release

**60 FPS Stretch:**

- [ ] Async physics thread implementation
- [ ] GPU-accelerated 10,000 particles @ 60 FPS
- [ ] VR demo (90 FPS with 1,000 particles)
- [ ] 50,000+ particle visual showcase @ 60 FPS
- [ ] 60 FPS optimization guide published
- [ ] WASM 60 FPS browser demo

## Getting Started: First Steps

> 📘 **Follow Rust library best practices from day one - see [RUST_LIBRARY_BEST_PRACTICES.md](./RUST_LIBRARY_BEST_PRACTICES.md)**

1. **Initialize Project** (Workspace Structure):

   ```bash
   mkdir gravwell && cd gravwell
   
   # Create workspace
   cargo new --lib crates/gravwell
   
   # Create workspace Cargo.toml
   cat > Cargo.toml <<EOF
   [workspace]
   members = ["crates/*"]
   resolver = "2"
   EOF
   
   cd crates/gravwell
   ```

2. **Configure Cargo.toml** (Library Metadata):

   ```bash
   # Add essential metadata to crates/gravwell/Cargo.toml
   # - Dual license: MIT OR Apache-2.0
   # - Keywords and categories for discoverability
   # - Documentation URL
   # - Repository link
   # - MSRV (Minimum Supported Rust Version)
   ```

3. **Set Up Core Dependencies**:

   ```bash
   cargo add nalgebra --no-default-features --features libm
   cargo add num-traits --no-default-features
   cargo add thiserror  # For error handling
   
   # Dev dependencies
   cargo add --dev criterion --features html_reports
   cargo add --dev proptest
   cargo add --dev approx
   ```

4. **Define Core Traits** (in `src/core/mod.rs`):
   - Start with trait definitions (Integrator, ForceCalculator)
   - Add comprehensive doc comments with examples
   - Include trait bounds for Send + Sync where needed

5. **Implement ParticleSet** (in `src/core/particle.rs`):
   - Use Structure-of-Arrays layout
   - Add newtype wrappers (Mass, BodyHandle)
   - Implement proper error handling

6. **Build First Integrator** (Semi-implicit Euler):
   - Implement the Integrator trait
   - Add unit tests with known solutions
   - Document with examples

7. **Create Prelude Module** (in `src/prelude.rs`):
   - Re-export commonly used types
   - Make imports ergonomic for users

8. **Set Up Testing Infrastructure**:

   ```bash
   # Create test directories
   mkdir -p tests/integration tests/validation tests/property
   
   # Add first test
   touch tests/integration/basic_orbit.rs
   ```

9. **Configure CI/CD** (`.github/workflows/ci.yml`):

   ```yaml
   name: CI
   on: [push, pull_request]
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
         - run: cargo test --all-features
         - run: cargo clippy -- -D warnings
         - run: cargo fmt --check
   ```

10. **Create Initial Documentation**:

    ```bash
    # README.md with badges, quick start, features
    # CONTRIBUTING.md with development workflow
    # CODE_OF_CONDUCT.md (use Contributor Covenant)
    # LICENSE-MIT and LICENSE-APACHE
    ```

**Development Workflow:**

1. Write tests first (TDD)
2. Implement feature
3. Document with examples
4. Run full test suite
5. Check with clippy
6. Format code
7. Update CHANGELOG.md

**Quality Gates (all must pass before commit):**

```bash
cargo test --all-features         # All tests pass
cargo clippy -- -D warnings       # No clippy warnings
cargo fmt --check                 # Code formatted
cargo doc --all-features --no-deps # Docs build
cargo build --no-default-features # no_std works
```

## References and Further Reading

### Project Documentation

- [60FPS_REQUIREMENTS.md](./60FPS_REQUIREMENTS.md) - Comprehensive guide to achieving 60 FPS performance
- [RUST_LIBRARY_BEST_PRACTICES.md](./RUST_LIBRARY_BEST_PRACTICES.md) - Complete Rust library design patterns and conventions

### Scientific Computing

- [REBOUND Documentation](https://rebound.readthedocs.io/)
- [Scholarpedia: N-body Simulations](http://www.scholarpedia.org/article/N-body_simulations_(gravitational))
- [Numerical Recipes Chapter on ODEs](http://numerical.recipes/)

### Game Physics

- [Gaffer on Games: Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/)
- [Erin Catto's GDC Physics Talks](https://box2d.org/publications/)
- [Rapier Documentation](https://rapier.rs/)

### Rust Resources

- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)

---

## Final Notes

This INITIAL_PROMPT.md represents a comprehensive blueprint for building a
production-quality gravity simulation crate that serves both game developers and
scientific researchers. The key insights are:

1. **Dual-Mode Architecture**: Game and science modes have fundamentally different
    needs—serve both excellently rather than compromising
2. **Library-First Design**: Follow Rust ecosystem conventions from day one for
    maximum adoption and maintainability
3. **Zero-Cost Abstractions**: Leverage Rust's type system to provide ergonomic
    APIs without runtime overhead

**Critical Success Factors:**

- **Start with Foundation**: Traits, error handling, and core abstractions matter more than features
- **Follow Conventions**: Use the patterns in [RUST_LIBRARY_BEST_PRACTICES.md](./RUST_LIBRARY_BEST_PRACTICES.md) to ensure idiomatic design
- **Test Ruthlessly**: Property-based tests, integration tests, and validation against analytical solutions
- **Document Everything**: Every public item with examples, errors, and panics documented
- **Optimize Later**: Get correctness and API ergonomics right before performance

**Start with Phase 1, validate thoroughly, then proceed.** Resist the temptation
to optimize prematurely or add features before the foundation is solid. The Rust
ecosystem provides unique advantages—zero-cost abstractions, memory safety,
excellent tooling—that make this project feasible where similar efforts in other
languages might struggle.

### Library Design Philosophy

Building a **reusable library** requires different thinking than an application:

- **Public API is Forever**: Design carefully, use `#[non_exhaustive]`, version semantically
- **Minimal Dependencies**: Keep the core `no_std`, make features optional
- **Ergonomic Defaults**: Make common use cases trivial, advanced use cases possible
- **Zero Surprises**: Follow Rust conventions, use Result for fallible ops, document panics
- **Extensibility**: Traits for customization, handles instead of borrows

The [RUST_LIBRARY_BEST_PRACTICES.md](./RUST_LIBRARY_BEST_PRACTICES.md) document
provides detailed patterns for:

- Workspace structure and organization
- Feature flag management
- Error handling with `thiserror`
- Builder patterns for configuration
- Documentation standards
- Testing strategies
- Publishing to crates.io

### About 60 FPS Performance

The **60 FPS stretch goals** represent aspirational targets that demonstrate the
crate's capabilities under optimal conditions. Achieving 60 FPS requires careful
attention to performance from the start, but should not compromise the core goal
of building a solid, correct, well-architected foundation.

**Recommended approach**:

1. **Phase 1**: Get physics/render decoupling working early—it's the biggest single win
2. **Phase 2**: Add SIMD and LOD systems as stretch goals once core physics is solid
3. **Phase 3**: Polish 60 FPS performance for showcase demos and demanding applications

The [60FPS_REQUIREMENTS.md](./60FPS_REQUIREMENTS.md) document provides complete
implementation details, but these optimizations should be considered enhancements
rather than requirements for a successful MVP.

Remember: *"Make it work, make it right, make it fast"* —in that order.

And always: *"Design for users, not yourself"* —follow Rust library conventions.

Good luck building the definitive Rust gravity simulation library! 🚀
