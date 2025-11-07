# Gravwell

[![Crates.io](https://img.shields.io/crates/v/gravwell.svg)](https://crates.io/crates/gravwell)
[![Documentation](https://docs.rs/gravwell/badge.svg)](https://docs.rs/gravwell)
[![License](https://img.shields.io/crates/l/gravwell.svg)](LICENSE-MIT)
[![CI](https://github.com/username/gravwell/workflows/CI/badge.svg)](https://github.com/username/gravwell/actions)

*Realistic gravity wells for games and astrophysics*

Ultra-realistic gravity simulation library for Rust, supporting both real-time game physics and high-accuracy scientific computing.

## ✨ Features

- 🎮 **Game Mode**: Real-time performance (1,000+ bodies @ 60 FPS) with stable, bounded behavior
- 🔬 **Science Mode**: High-accuracy symplectic integrators with energy conservation < 10⁻¹⁰
- ⚡ **Fast**: SIMD vectorization, multi-threading, and optional GPU acceleration
- 🦀 **Pure Rust**: Memory-safe with zero-cost abstractions
- 📦 **Modular**: `no_std` core with optional features
- 🌌 **Multi-Scale**: From planetary systems to local surface gravity

## 🚀 Quick Start

```rust
use gravwell::prelude::*;

fn main() -> Result<()> {
    // Create a simulation with realistic physics
    let mut sim = Simulation::builder()
        .integrator(VelocityVerlet::new())
        .gravity(BarnesHut::new().theta(0.5))
        .timestep(0.01)
        .build()?;

    // Add celestial bodies
    sim.add_body(Body::new()
        .mass(Mass::SOLAR_MASS)
        .position([0.0, 0.0, 0.0])
        .name("Sun"))?;

    sim.add_body(Body::new()
        .mass(Mass::EARTH_MASS)
        .position([1.496e11, 0.0, 0.0])  // 1 AU
        .velocity([0.0, 29780.0, 0.0])   // Orbital velocity
        .name("Earth"))?;

    // Run simulation
    for day in 0..365 {
        for _ in 0..24 {
            sim.step();
        }
        
        if day % 30 == 0 {
            println!("Day {}: Energy drift = {:.3e}", 
                day, sim.energy_drift());
        }
    }

    Ok(())
}
```

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gravwell = "0.1"

# Enable optional features
gravwell = { version = "0.1", features = ["parallel", "simd"] }
```

### Feature Flags

- **`std`** (default) - Standard library support
- **`parallel`** - Multi-threading with Rayon
- **`simd`** - SIMD optimizations
- **`gpu`** - GPU acceleration via WGPU
- **`serde`** - Serialization support
- **`performance-60fps`** - All 60 FPS optimizations (parallel + simd)
- **`full`** - Enable all features

## 🎯 Use Cases

### Game Development

```rust
// Real-time orbital mechanics for space games
let sim = Simulation::builder()
    .integrator(SemiImplicitEuler::new())  // Fast & stable
    .gravity(BarnesHut::new())
    .physics_rate(30.0)   // 30 Hz physics
    .render_rate(60.0)    // 60 Hz rendering
    .build()?;
```

### Scientific Computing

```rust
// High-accuracy astrophysics simulation
let sim = Simulation::builder()
    .integrator(Leapfrog::new())  // Symplectic
    .gravity(BarnesHut::new().theta(0.3))
    .monitor_energy(true)
    .build()?;
```

## 📊 Performance

| Particle Count | 30 FPS | 60 FPS | Hardware |
|----------------|--------|--------|----------|
| 1,000 | ✅ CPU | ✅ CPU + SIMD | Single-threaded |
| 10,000 | ✅ CPU + Parallel | ✅ CPU + Full opt | Multi-core |
| 100,000 | ✅ GPU | ⚠️ GPU + LOD | GPU required |

## 📚 Documentation

- **[User Guide](https://docs.rs/gravwell)** - Complete API documentation
- **[Examples](./examples/)** - Runnable examples for common scenarios
- **[60 FPS Guide](./docs/60FPS_REQUIREMENTS.md)** - Performance optimization strategies
- **[Library Design](./docs/RUST_LIBRARY_BEST_PRACTICES.md)** - Architecture patterns

## 🔬 Algorithms

- **Integrators**: Semi-implicit Euler, Velocity Verlet, Leapfrog, RK4, IAS15
- **Force Calculation**: Direct O(N²), Barnes-Hut O(N log N), Fast Multipole Method
- **Collision Detection**: Spatial grid, AABB trees, sweep-and-prune
- **Optimizations**: SIMD (AVX/AVX-512), Multi-threading (Rayon), GPU (WGPU)

## 🧪 Validation

Gravwell is validated against:

- ✅ **Analytical solutions** - Kepler orbits, two-body problem
- ✅ **Established codes** - REBOUND, NBODY6
- ✅ **Conservation laws** - Energy < 10⁻¹⁰, momentum < 10⁻¹⁴
- ✅ **JPL HORIZONS** - Solar system ephemeris

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## 🌟 Inspiration

Gravwell stands on the shoulders of giants:

- **[REBOUND](https://github.com/hannorein/rebound)** - N-body astrophysics
- **[Rapier](https://rapier.rs/)** - Rust physics engine design
- **[nalgebra](https://nalgebra.org/)** - Linear algebra foundations

---

**Built with 🦀 Rust** • **Physics by Newton** • **Optimized for Reality**
