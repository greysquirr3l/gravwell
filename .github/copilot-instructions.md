# 🌌 Gravwell - GitHub Copilot Instructions

## Project Overview

**Project Name**: GRAVWELL  
**Primary Language**: Rust  
**Framework/Stack**: nalgebra + rayon + criterion + WASM  
**Architecture Pattern**: Library-First Design + Trait-Based Abstractions  
**Domain**: Ultra-Realistic Gravity Simulation for Games & Astrophysics  

### Tech Stack

```text
Primary Language: Rust (stable, MSRV 1.70+)
Linear Algebra: nalgebra 0.32+
Parallel Computing: rayon 1.7+
Benchmarking: criterion 0.5+
Error Handling: thiserror 1.0+
Serialization: serde 1.0+ (optional)
Logging: tracing 0.1+
Testing: cargo test + proptest (property-based)
CI/CD: GitHub Actions
Target Platform: Native + Web (WASM)
```

## 🏗️ Architecture Patterns

### Library Architecture + Trait-Based Design

```text
- Core Traits: Integrator, ForceCalculator, CollisionHandler (extensible abstractions)
- Builder Pattern: SimulationBuilder for complex configuration with sensible defaults
- Zero-Cost Abstractions: Compile-time dispatch via generics, runtime dispatch only when needed
- Data Layout: Structure-of-Arrays (SoA) for SIMD-friendly vectorization
- Error Handling: Result<T, E> with thiserror for ergonomic error types
```

### Core Physics Systems

- **Integrator System**: VelocityVerlet, Leapfrog, RK4, IAS15 (symplectic + non-symplectic)
- **Force Calculation**: DirectGravity O(N²), BarnesHut O(N log N), FastMultipole O(N)
- **Collision System**: Spatial grid, AABB trees, sweep-and-prune algorithms
- **Performance System**: SIMD vectorization, parallel computation, GPU acceleration
- **Validation System**: Energy conservation, analytical solution comparison
- **Serialization System**: Save/load simulation state, reproducible physics

## 📁 Directory Structure

```text
gravwell/                         # Workspace root
├── Cargo.toml                    # Workspace manifest
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── CHANGELOG.md
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
│
├── crates/
│   └── gravwell/                 # Main library crate
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs            # Public API surface and re-exports
│       │   ├── prelude.rs        # Convenient imports module
│       │   ├── error.rs          # Error types with thiserror
│       │   ├── types.rs          # Core type definitions
│       │   ├── builder.rs        # SimulationBuilder pattern
│       │   ├── core/             # Core abstractions (no_std compatible)
│       │   │   ├── mod.rs
│       │   │   ├── integrator.rs # Integrator trait and implementations
│       │   │   ├── forces.rs     # ForceCalculator trait
│       │   │   ├── particle.rs   # ParticleSet and Body types
│       │   │   └── math.rs       # Vector math utilities
│       │   ├── integrators/      # Numerical integration methods
│       │   │   ├── mod.rs
│       │   │   ├── verlet.rs     # Velocity Verlet (symplectic)
│       │   │   ├── leapfrog.rs   # Leapfrog (symplectic)
│       │   │   ├── rk4.rs        # Runge-Kutta 4th order
│       │   │   └── ias15.rs      # IAS15 adaptive 15th order
│       │   ├── forces/           # Force calculation algorithms
│       │   │   ├── mod.rs
│       │   │   ├── direct.rs     # Direct O(N²) calculation
│       │   │   ├── barnes_hut.rs # Barnes-Hut O(N log N)
│       │   │   └── fmm.rs        # Fast Multipole Method O(N)
│       │   ├── collision/        # Collision detection (optional)
│       │   │   ├── mod.rs
│       │   │   ├── spatial_hash.rs # Spatial hash grid
│       │   │   └── aabb.rs       # AABB tree
│       │   └── utils/            # Utility functions
│       │       ├── mod.rs
│       │       ├── constants.rs  # Physical constants
│       │       └── validation.rs # Analytical solution validation
│       ├── tests/                # Integration tests
│       │   ├── energy_conservation.rs
│       │   ├── kepler_orbits.rs
│       │   └── nbody_validation.rs
│       ├── benches/              # Criterion benchmarks
│       │   ├── force_calculation.rs
│       │   ├── integration_step.rs
│       │   └── full_simulation.rs
│       └── examples/             # Executable examples
│           ├── solar_system.rs
│           ├── binary_orbit.rs
│           └── performance_test.rs
│
├── examples/                     # Workspace-level examples
│   ├── basic_usage.rs
│   ├── custom_integrator.rs
│   └── gpu_acceleration.rs
│
├── docs/                         # Documentation
│   ├── architecture.md
│   ├── performance_guide.md
│   └── scientific_validation.md
│
└── scripts/                      # Development scripts
    ├── benchmark.sh
    ├── validate.sh
    └── release.sh
```

## 🎯 Development Standards

### Naming Conventions (Rust)

**Rust Conventions:**
- **Types**: PascalCase (`ParticleSet`, `SimulationBuilder`)
- **Functions**: snake_case (`calculate_forces`, `step_simulation`)
- **Variables**: snake_case (`particle_count`, `timestep`)
- **Constants**: SCREAMING_SNAKE_CASE (`G`, `SOLAR_MASS`, `AU`)
- **Modules**: snake_case (`integrators`, `force_calculation`)
- **Traits**: PascalCase (`Integrator`, `ForceCalculator`)

**Physics-Specific Naming:**
- **Physical Quantities**: SI units in names (`Mass`, `Position`, `Velocity`)
- **Algorithms**: Descriptive (`VelocityVerlet`, `BarnesHut`, `DirectGravity`)
- **Errors**: Descriptive errors (`SimulationError`, `BuildError`, `IntegrationError`)
- **Builders**: Fluent API (`SimulationBuilder`, `BodyBuilder`)
- **Validation**: Test-oriented (`EnergyConservation`, `KeplerOrbit`)

### File Organization Patterns

```text
Physics algorithm-based grouping (integrators, forces, collision)
Separation of concerns (computation, validation, utilities)
Test files in dedicated tests/ directory for integration tests
Unit tests co-located with source files (#[cfg(test)] modules)
Benchmarks in dedicated benches/ directory
Examples demonstrating different use cases
Comprehensive documentation with doctests
```

### Code Quality Standards

```text
- Rust formatting with rustfmt (enforce in CI)
- Comprehensive error handling with Result<T, E> and thiserror
- Input validation on all public API functions
- Structured logging with tracing crate for performance analysis
- Performance-first approach (60 FPS capable, scientifically accurate)
- Memory safety through Rust's ownership system
- No unwrap() in library code (use proper error propagation)
- Extensive documentation with doctests for all public APIs
```

## 🧪 Testing Guidelines

### Testing Strategy

```text
- Unit Tests: Test physics algorithms (force calculation, integration accuracy)
- Integration Tests: Test complete simulation workflows
- Validation Tests: Compare against analytical solutions (Kepler orbits, N-body)
- Performance Tests: Ensure 60fps capability with criterion benchmarks
- Property Tests: Mathematical invariants (energy conservation, momentum)
```

### Test Patterns

```text
- Arrange-Act-Assert (AAA) pattern
- Deterministic physics testing with fixed seeds and epsilon comparisons
- Property-based testing with proptest for mathematical validation
- Benchmark-driven development for performance-critical paths
- Analytical solution comparison for accuracy validation
```

### Coverage Requirements

```text
- Core Physics: 95% minimum coverage (integrators, force calculation)
- Public API: 90% minimum coverage (builder, simulation methods)
- Error Handling: 85% minimum coverage (all error paths tested)
- Mathematical Properties: 100% coverage (energy conservation, symplectic)
```

## 🔒 Security Requirements

### Library Security Standards

```text
- Input validation for all public API functions
- Numerical stability (prevent NaN/infinity propagation)
- Memory safety through Rust's ownership system
- No unsafe code blocks without explicit justification and safety comments
- Bounds checking on all array/vector access
- Safe error handling (no panics in library code)
- Secure deserialization of simulation state
```

### Scientific Computing Security

```text
- Deterministic physics for reproducible results
- Overflow/underflow protection in numerical calculations
- Safe floating-point comparisons with appropriate epsilon
- Validation of physical constraints (positive masses, etc.)
- Protection against degenerate cases (zero distances, etc.)
- Secure random number generation for Monte Carlo methods
```

## ⚡ Performance Guidelines

### Physics Performance Patterns

```text
- Target 60fps capability for real-time applications
- SIMD vectorization for force calculations (AVX/AVX-512)
- Parallel force computation with rayon
- Cache-friendly data layouts (Structure-of-Arrays)
- Algorithmic optimization (O(N²) → O(N log N) → O(N))
- Memory pool allocation for temporary vectors
- Adaptive timestep controllers for efficiency
```

### Rust-Specific Optimizations

```text
- Zero-cost abstractions and compile-time optimization
- Avoid unnecessary allocations in simulation loops
- Use Vec with pre-allocated capacity for particle data
- Prefer iterators and parallel iterators (rayon)
- Use const generics for compile-time physics parameters
- Profile with criterion for benchmark-driven optimization
- SIMD operations for vectorized force calculations
```

## 🚀 Development Workflow

### Branch Strategy

```text
- main: Stable releases with semantic versioning
- develop: Integration branch for new features
- feature/[algorithm-name]: Algorithm implementations
- perf/[optimization-name]: Performance improvements
- fix/[issue-name]: Bug fixes
- docs/[documentation-name]: Documentation updates
```

### Commit Standards

```text
Format: type(scope): description

Types:
- feat: New physics algorithm or API feature
- perf: Performance optimization
- fix: Bug fix or numerical accuracy improvement
- docs: Documentation or example updates
- test: Test additions or improvements
- refactor: Code restructuring without behavior change
- chore: Maintenance tasks

Examples:
- feat(integrator): add IAS15 adaptive integrator
- perf(forces): vectorize direct gravity calculation
- fix(verlet): correct energy drift in long simulations
```

### Code Review Guidelines

```text
- Physics algorithm correctness and numerical accuracy
- Error handling completeness (all Result types)
- Performance implications and benchmark validation
- Test coverage adequacy (unit + integration + validation)
- Documentation updates (API docs + examples)
- Scientific accuracy and citation requirements
- Memory safety and no-panic guarantees
```

## 📚 Documentation Integration Strategy

Reference Gravwell's comprehensive physics simulation documentation

### Smart Reference System

For comprehensive technical documentation, use references to avoid context overload:

```markdown
**High-Value References:**
- [Architecture Guide](../docs/architecture.md) - Physics engine design and trait abstractions
- [Performance Guide](../docs/performance_guide.md) - SIMD optimization and parallelization
- [Scientific Validation](../docs/scientific_validation.md) - Energy conservation and accuracy testing
- [API Documentation](https://docs.rs/gravwell) - Complete API reference
- [Examples Collection](../examples/) - Solar system, binary orbits, performance tests
```

### Context-Aware Usage

```text
For [SPECIFIC_TASK] implementation:
1. Apply basic patterns from these instructions
2. Reference [SPECIFIC_DOC] for advanced patterns:
   - Section X: [Specific advanced pattern]
   - Section Y: [Complex implementation details]
   - Section Z: [Edge cases and solutions]
```

## 🤖 AI Assistant Optimization

### AI Assistance Guidelines

```text
- Provide domain context when requesting business logic
- Always request comprehensive error handling
- Ask for test generation alongside implementation
- Request security considerations for external-facing code
- Include performance considerations for data processing
- Validate generated code against project patterns
```

### Persona-Based Development Integration

```text
When using AI assistance, consider these specialized contexts:
- Requirements Gathering: Focus on user stories and acceptance criteria
- Technical Design: Emphasize architecture patterns and system design
- Implementation: Follow coding standards and best practices
- Testing: Comprehensive test coverage and quality assurance
- Code Review: Security, performance, and maintainability focus
- Debugging: Problem identification and solution strategies
```

### Model Context Protocol (MCP) Integration

```text
Enhanced AI capabilities through MCP:
- Filesystem access for project understanding
- Git integration for version control context
- External tool integration (linters, formatters, etc.)
- Custom domain-specific tools and knowledge bases
```

## 🔧 Tool Integration

### Development Tools

```text
- Code Formatter: rustfmt (built-in)
- Linter: clippy (built-in)
- Package Manager: cargo (built-in)
- Build Tool: cargo build/run/test
- Benchmark Tool: criterion
- Documentation: cargo doc
- Cross-compilation: cargo build --target wasm32-unknown-unknown
```

### CI/CD Integration

```text
- Automated testing on all branches
- Code quality checks (formatting, linting, security)
- Test coverage reporting and enforcement
- Security vulnerability scanning
- Performance benchmarking
- Documentation generation and validation
```

## 🎯 Success Metrics

### Code Quality Metrics

```text
- Test Coverage: 95% minimum for physics algorithms
- Code Duplication: <3% across modules
- Cyclomatic Complexity: <8 per function
- Clippy Warnings: Zero in all builds
- Documentation Coverage: 100% of public APIs
```

### Performance Metrics

```text
- Simulation Speed: 60fps capable for 10K particles
- Force Calculation: <5ms per timestep (10K bodies)
- Memory Efficiency: <1KB per particle overhead
- Compilation Time: <30 seconds for full rebuild
- SIMD Utilization: >80% in force calculations
```

### Scientific Quality Metrics

```text
- Energy Conservation: <1e-12 relative error over 1M steps
- Kepler Orbit Accuracy: <1e-8 relative error over 100 periods
- Benchmark Performance: Within 5% of reference implementations
- Cross-Platform Consistency: Identical results across targets
```

## 🚦 Quality Gates

### Pre-Commit Requirements

```text
- All tests pass (cargo test)
- No clippy warnings (cargo clippy)
- Code formatted (cargo fmt --check)
- All benchmarks pass baseline thresholds
- Documentation builds successfully (cargo doc)
```

### Pre-Release Requirements

```text
- Full test suite passes (unit + integration + validation)
- Performance benchmarks meet targets
- Cross-platform builds successful (native + WASM)
- All examples run successfully
- API documentation complete and accurate
- Changelog updated with breaking changes
- Semantic versioning compliance verified
```

---

## � Rust Development Commands

### Essential Commands

```bash
# Development Workflow
cargo fmt                      # Format code with rustfmt
cargo clippy                   # Run Rust linter
cargo check                    # Quick syntax/type check
cargo build                    # Build library (debug mode)
cargo build --release        # Build optimized (release mode)
cargo test                    # Run all tests
cargo test -- --nocapture     # Run tests with println! output
cargo bench                   # Run criterion benchmarks
cargo doc --open              # Generate and open documentation

# Library Development Specific
cargo build --target wasm32-unknown-unknown  # Build for web (WASM)
cargo test --release          # Run tests in release mode (for performance)
cargo bench -- --baseline new  # Set new performance baseline
RUST_LOG=debug cargo test     # Run tests with debug logging
RUST_BACKTRACE=1 cargo test   # Run tests with stack traces

# Development Tools
cargo install cargo-watch     # Install file watcher
cargo watch -x test          # Auto-run tests on file changes
cargo install flamegraph     # Install performance profiler
cargo flamegraph --bench force_calculation  # Profile benchmarks

# Examples and Validation
cargo run --example solar_system     # Run solar system example
cargo run --example binary_orbit     # Run binary orbit example
cargo run --example performance_test # Run performance validation
```

### Physics-Specific Debugging

```bash
# Performance Analysis
RUST_LOG=gravwell::forces=debug cargo test    # Debug force calculations
RUST_LOG=gravwell::integrator=trace cargo test  # Trace integration steps

# Numerical Accuracy Analysis
cargo test energy_conservation -- --nocapture  # Energy conservation tests
cargo test kepler_orbit -- --nocapture        # Orbital accuracy tests

# Benchmark Analysis
cargo bench -- --output-format html           # Generate benchmark reports
cargo bench force_calculation                  # Benchmark force algorithms
cargo bench integration_step                   # Benchmark integrators
```

**This configuration provides maximum GitHub Copilot effectiveness for Rust physics library development with Gravwell.**
