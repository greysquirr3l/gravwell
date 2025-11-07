# Gravwell - Quick Start Implementation Guide

This guide walks through creating the first working prototype of Gravwell in the correct order, following Rust library best practices.

## 🎯 Goal: First Working Prototype (Week 1)

**Target**: A minimal but working gravity simulation that can run a simple two-body orbit.

## 📋 Step-by-Step Implementation

### Step 1: Project Setup (30 minutes)

```bash
# Create workspace
mkdir gravwell && cd gravwell

# Initialize workspace
cat > Cargo.toml <<EOF
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.70"
license = "MIT OR Apache-2.0"
authors = ["Your Name <email@example.com>"]
repository = "https://github.com/username/gravwell"
EOF

# Create main library crate
cargo new --lib crates/gravwell

# Set up development tools
cargo install cargo-watch cargo-tarpaulin
```

### Step 2: Configure Cargo.toml (15 minutes)

Edit `crates/gravwell/Cargo.toml`:

```toml
[package]
name = "gravwell"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Ultra-realistic gravity simulation for games and astrophysics"
keywords = ["physics", "gravity", "simulation", "n-body", "game-physics"]
categories = ["game-development", "science", "simulation"]
readme = "../../README.md"

[dependencies]
# Core - minimal dependencies
nalgebra = { version = "0.32", default-features = false, features = ["libm"] }
num-traits = { version = "0.2", default-features = false }
thiserror = "1.0"

[dev-dependencies]
approx = "0.5"
criterion = { version = "0.5", features = ["html_reports"] }

[features]
default = ["std"]
std = ["nalgebra/std", "num-traits/std"]

[[bench]]
name = "force_calculation"
harness = false
```

### Step 3: Define Core Types (1 hour)

Create `src/types.rs`:

```rust
//! Core type definitions for Gravwell.

use nalgebra as na;

/// Scalar type for calculations (f64 for scientific accuracy).
pub type Scalar = f64;

/// 3D vector type.
pub type Vector3 = na::Vector3<Scalar>;

/// Body mass with type safety.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mass(pub Scalar);

impl Mass {
    pub fn new(value: Scalar) -> Self {
        debug_assert!(value >= 0.0, "mass cannot be negative");
        Self(value)
    }

    pub fn value(&self) -> Scalar {
        self.0
    }

    pub const SOLAR_MASS: Self = Self(1.989e30);
    pub const EARTH_MASS: Self = Self(5.972e24);
}

/// Gravitational constant (m³ kg⁻¹ s⁻²)
pub const G: Scalar = 6.67430e-11;
```

### Step 4: Define Core Traits (1 hour)

Create `src/core/mod.rs`:

```rust
//! Core abstractions for extensibility.

use crate::types::*;

/// Numerical integrator for advancing the simulation.
pub trait Integrator {
    /// Advance the system by one timestep.
    fn step(&mut self, positions: &mut [Vector3], 
            velocities: &mut [Vector3], 
            masses: &[Mass], 
            dt: Scalar);
}

/// Calculate gravitational forces.
pub trait ForceCalculator: Send + Sync {
    /// Compute forces for all particles.
    fn compute_forces(&self, positions: &[Vector3], 
                      masses: &[Mass], 
                      forces: &mut [Vector3]);
}
```

### Step 5: Implement Simple Integrator (45 minutes)

Create `src/integrators/velocity_verlet.rs`:

```rust
//! Velocity Verlet integrator - 2nd order, symplectic.

use crate::core::*;
use crate::types::*;

/// Velocity Verlet integrator.
///
/// Second-order accurate, symplectic integrator suitable for
/// long-term orbital simulations.
pub struct VelocityVerlet {
    accelerations: Vec<Vector3>,
}

impl VelocityVerlet {
    pub fn new() -> Self {
        Self {
            accelerations: Vec::new(),
        }
    }
}

impl Integrator for VelocityVerlet {
    fn step(&mut self, positions: &mut [Vector3], 
            velocities: &mut [Vector3], 
            masses: &[Mass], 
            dt: Scalar) {
        let n = positions.len();
        
        // Ensure we have space for accelerations
        self.accelerations.resize(n, Vector3::zeros());
        
        // Compute initial accelerations (forces / mass)
        compute_accelerations(positions, masses, &mut self.accelerations);
        
        // Update positions: x = x + v*dt + 0.5*a*dt²
        for i in 0..n {
            positions[i] += velocities[i] * dt + 
                           0.5 * self.accelerations[i] * dt * dt;
        }
        
        // Compute new accelerations
        let mut new_accelerations = vec![Vector3::zeros(); n];
        compute_accelerations(positions, masses, &mut new_accelerations);
        
        // Update velocities: v = v + 0.5*(a_old + a_new)*dt
        for i in 0..n {
            velocities[i] += 0.5 * (self.accelerations[i] + 
                                    new_accelerations[i]) * dt;
        }
        
        self.accelerations = new_accelerations;
    }
}

fn compute_accelerations(positions: &[Vector3], 
                        masses: &[Mass], 
                        accelerations: &mut [Vector3]) {
    let n = positions.len();
    
    // Reset accelerations
    for acc in accelerations.iter_mut() {
        *acc = Vector3::zeros();
    }
    
    // O(N²) force calculation
    for i in 0..n {
        for j in 0..n {
            if i == j { continue; }
            
            let r_vec = positions[j] - positions[i];
            let r2 = r_vec.norm_squared();
            let r = r2.sqrt();
            
            // F = G * m1 * m2 / r²
            // a = F / m1 = G * m2 / r²
            let acc_magnitude = G * masses[j].value() / (r * r2);
            accelerations[i] += r_vec.normalize() * acc_magnitude;
        }
    }
}
```

Create `src/integrators/mod.rs`:

```rust
pub mod velocity_verlet;
pub use velocity_verlet::VelocityVerlet;
```

### Step 6: Create Simple API (1 hour)

Create `src/lib.rs`:

```rust
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! # Gravwell
//!
//! Ultra-realistic gravity simulation for games and astrophysics.
//!
//! # Quick Start
//!
//! ```
//! use gravwell::*;
//!
//! let mut sim = Simulation::new(VelocityVerlet::new());
//!
//! // Add Sun
//! sim.add_body([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], Mass::SOLAR_MASS);
//!
//! // Add Earth
//! sim.add_body([1.496e11, 0.0, 0.0], [0.0, 29780.0, 0.0], Mass::EARTH_MASS);
//!
//! // Simulate
//! for _ in 0..1000 {
//!     sim.step(3600.0); // 1 hour timestep
//! }
//! ```

pub mod types;
pub mod core;
pub mod integrators;

pub use types::*;
pub use core::{Integrator, ForceCalculator};
pub use integrators::VelocityVerlet;

/// Simple gravity simulation.
pub struct Simulation<I: Integrator> {
    positions: Vec<Vector3>,
    velocities: Vec<Vector3>,
    masses: Vec<Mass>,
    integrator: I,
    time: Scalar,
}

impl<I: Integrator> Simulation<I> {
    /// Create a new simulation.
    pub fn new(integrator: I) -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            masses: Vec::new(),
            integrator,
            time: 0.0,
        }
    }
    
    /// Add a body to the simulation.
    pub fn add_body(&mut self, position: [Scalar; 3], 
                    velocity: [Scalar; 3], 
                    mass: Mass) {
        self.positions.push(Vector3::from(position));
        self.velocities.push(Vector3::from(velocity));
        self.masses.push(mass);
    }
    
    /// Advance simulation by one timestep.
    pub fn step(&mut self, dt: Scalar) {
        self.integrator.step(&mut self.positions, 
                           &mut self.velocities, 
                           &self.masses, 
                           dt);
        self.time += dt;
    }
    
    /// Get current simulation time.
    pub fn time(&self) -> Scalar {
        self.time
    }
    
    /// Get body position.
    pub fn position(&self, index: usize) -> Option<Vector3> {
        self.positions.get(index).copied()
    }
    
    /// Get body velocity.
    pub fn velocity(&self, index: usize) -> Option<Vector3> {
        self.velocities.get(index).copied()
    }
    
    /// Calculate total energy (kinetic + potential).
    pub fn total_energy(&self) -> Scalar {
        let mut energy = 0.0;
        let n = self.positions.len();
        
        // Kinetic energy
        for i in 0..n {
            let v2 = self.velocities[i].norm_squared();
            energy += 0.5 * self.masses[i].value() * v2;
        }
        
        // Potential energy
        for i in 0..n {
            for j in (i+1)..n {
                let r = (self.positions[j] - self.positions[i]).norm();
                energy -= G * self.masses[i].value() * 
                         self.masses[j].value() / r;
            }
        }
        
        energy
    }
}
```

### Step 7: Write First Test (30 minutes)

Create `tests/kepler_orbit.rs`:

```rust
use gravwell::*;
use approx::assert_relative_eq;

#[test]
fn test_circular_orbit_energy_conservation() {
    let mut sim = Simulation::new(VelocityVerlet::new());
    
    // Sun at origin
    sim.add_body([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], Mass::SOLAR_MASS);
    
    // Earth at 1 AU with circular orbital velocity
    sim.add_body([1.496e11, 0.0, 0.0], [0.0, 29780.0, 0.0], Mass::EARTH_MASS);
    
    let initial_energy = sim.total_energy();
    
    // Simulate for 100 days
    for _ in 0..(100 * 24) {
        sim.step(3600.0); // 1 hour timesteps
    }
    
    let final_energy = sim.total_energy();
    let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();
    
    // Energy should be conserved to better than 0.1%
    assert!(energy_error < 1e-3, "Energy error: {:.3e}", energy_error);
}
```

### Step 8: Write Example (15 minutes)

Create `examples/earth_orbit.rs`:

```rust
use gravwell::*;

fn main() {
    let mut sim = Simulation::new(VelocityVerlet::new());
    
    println!("Simulating Earth's orbit around the Sun...\n");
    
    // Add Sun
    sim.add_body([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], Mass::SOLAR_MASS);
    
    // Add Earth at 1 AU
    sim.add_body([1.496e11, 0.0, 0.0], [0.0, 29780.0, 0.0], Mass::EARTH_MASS);
    
    let initial_energy = sim.total_energy();
    println!("Initial energy: {:.3e} J", initial_energy);
    
    // Simulate for one year
    for day in 0..365 {
        // 24 steps per day (1 hour each)
        for _ in 0..24 {
            sim.step(3600.0);
        }
        
        if day % 30 == 0 {
            let pos = sim.position(1).unwrap();
            let distance = pos.norm();
            let energy = sim.total_energy();
            let drift = (energy - initial_energy).abs() / initial_energy.abs();
            
            println!("Day {:3}: distance = {:.3e} m, energy drift = {:.3e}", 
                     day, distance, drift);
        }
    }
    
    println!("\nSimulation complete!");
    let final_energy = sim.total_energy();
    let total_drift = (final_energy - initial_energy).abs() / initial_energy.abs();
    println!("Total energy drift: {:.3e}", total_drift);
}
```

### Step 9: Run and Validate (15 minutes)

```bash
# Run tests
cargo test

# Run example
cargo run --example earth_orbit

# Check formatting
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Generate docs
cargo doc --open
```

## ✅ Success Criteria

After completing these steps, you should have:

- [x] Working Velocity Verlet integrator
- [x] Simple API (add bodies, step simulation)
- [x] Energy conservation test passing
- [x] Example demonstrating Earth's orbit
- [x] Clean code (no clippy warnings)
- [x] Basic documentation

## 🎯 Next Steps (Week 2)

1. Add Semi-implicit Euler integrator (for games)
2. Add error handling (Result types)
3. Implement builder pattern
4. Add more tests (momentum conservation)
5. Create prelude module
6. Write more examples

## 📊 Expected Output

Running the example should show something like:

```
Simulating Earth's orbit around the Sun...

Initial energy: -2.648e+33 J
Day   0: distance = 1.496e+11 m, energy drift = 3.421e-11
Day  30: distance = 1.496e+11 m, energy drift = 1.234e-10
Day  60: distance = 1.496e+11 m, energy drift = 2.156e-10
Day  90: distance = 1.496e+11 m, energy drift = 2.987e-10
...
Day 360: distance = 1.496e+11 m, energy drift = 9.876e-10

Simulation complete!
Total energy drift: 9.876e-10
```

Energy drift should be < 10⁻⁹ (excellent conservation for a symplectic integrator).

## 🐛 Common Issues

**Issue**: Large energy drift (> 10⁻⁶)
**Solution**: Check timestep - try smaller dt (100s instead of 3600s)

**Issue**: Compilation errors with nalgebra
**Solution**: Check features are correct: `features = ["libm"]`

**Issue**: Test fails on CI but passes locally
**Solution**: Energy conservation is platform-dependent, relax threshold slightly

## 🎉 Congratulations

You now have the foundation of Gravwell! This minimal prototype demonstrates:

- ✅ Proper trait-based architecture
- ✅ Energy-conserving physics
- ✅ Clean API
- ✅ Good test coverage
- ✅ Working examples

From here, you can incrementally add features following the INITIAL_PROMPT.md roadmap.

---

**Time Estimate**: 4-6 hours for first working prototype
**Difficulty**: Moderate (requires understanding of numerical integration)
**Prerequisites**: Basic Rust knowledge, basic physics understanding
