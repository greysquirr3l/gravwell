# Gravwell Project Purpose and Overview

## Project Mission
Gravwell is a high-performance Rust library for gravity simulation designed for both **gaming** and **astrophysics** applications. It provides ultra-realistic N-body gravity simulation with:

- **Dual-mode operation**: Game mode (real-time 60 FPS) and Science mode (maximum accuracy)
- **Multiple algorithms**: O(N²) Direct, O(N log N) Barnes-Hut, O(N) Fast Multipole Method
- **Performance optimization**: SIMD vectorization (5-6x speedup), GPU acceleration (1,276x speedup), multi-threading
- **Scientific accuracy**: Energy conservation, symplectic integrators, analytical solution validation

## Key Features

### Performance Targets
- **Game Mode**: 1,000 particles @ 60 FPS (basic), 5,000 particles @ 60 FPS (optimized)
- **Science Mode**: 10,000+ particles with energy conservation < 1e-12 relative error
- **GPU Mode**: 50,000+ particles with WebGPU acceleration

### Algorithm Implementations
- **Integrators**: Semi-Implicit Euler, Velocity Verlet, Leapfrog, RK4, IAS15 adaptive
- **Force Calculation**: Direct Gravity, Barnes-Hut Tree, Fast Multipole Method, GPU Barnes-Hut
- **Optimization**: SIMD (AVX/AVX-512), Parallel computation (rayon), GPU compute shaders

### Platform Support
- **Native**: x86_64, ARM64 (macOS, Linux, Windows)
- **Web**: WebAssembly with WebGPU acceleration
- **Cross-platform**: Consistent physics results across all targets

## Current Status (v0.2.0)
- ✅ Core trait-based architecture implemented
- ✅ All major integrators and force calculators
- ✅ SIMD optimization with 5-6x performance boost
- ✅ GPU Barnes-Hut with WebGPU (recently committed)
- ✅ Comprehensive test suite and benchmarks
- ✅ Scientific validation framework
- 🔄 Ongoing: Advanced GPU features and multi-GPU distribution

## Use Cases
1. **Game Development**: Real-time space simulations, asteroid fields, planetary systems
2. **Education**: Interactive astrophysics demonstrations, orbital mechanics teaching
3. **Scientific Computing**: N-body simulations, celestial mechanics research
4. **Visualization**: Large-scale cosmic structure simulation and rendering

## Unique Selling Points
- **Zero-cost abstractions**: Compile-time optimization with trait-based design
- **Scientific rigor**: Validation against analytical solutions and established codes
- **Performance-first**: Benchmark-driven development with 60 FPS guarantees
- **Extensible architecture**: Easy to add custom integrators and force calculators
- **Cross-platform**: Native and web deployment with identical physics