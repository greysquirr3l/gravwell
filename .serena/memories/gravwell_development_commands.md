# Gravwell Development Commands and Workflows

## Essential Development Commands

### Code Quality and Formatting
```bash
# Format code with rustfmt (enforced in CI)
cargo fmt

# Run Rust linter with physics-specific rules
cargo clippy

# Check syntax and types quickly
cargo check

# Build library (debug mode)
cargo build

# Build optimized (release mode)
cargo build --release
```

### Testing Commands
```bash
# Run all tests
cargo test

# Run tests with output visible
cargo test -- --nocapture

# Run tests in release mode (for performance validation)
cargo test --release

# Run specific test categories
cargo test energy_conservation     # Energy conservation tests
cargo test integrator_accuracy     # Numerical integrator tests
cargo test force_calculation       # Force algorithm tests
cargo test gpu_barnes_hut          # GPU algorithm tests
cargo test kepler_orbit            # Orbital mechanics validation

# Run tests with debug logging
RUST_LOG=debug cargo test

# Run tests with stack traces on panic
RUST_BACKTRACE=1 cargo test

# Test specific physics algorithms
RUST_LOG=gravwell::forces=debug cargo test    # Debug force calculations
RUST_LOG=gravwell::integrator=trace cargo test  # Trace integration steps
```

### Performance and Benchmarking
```bash
# Run all criterion benchmarks
cargo bench

# Set new performance baseline
cargo bench -- --baseline new

# Run specific benchmarks
cargo bench force_calculation       # Benchmark force algorithms
cargo bench integration_step        # Benchmark integrators
cargo bench 60fps_target           # Test 60 FPS performance targets

# Generate benchmark reports (HTML)
cargo bench -- --output-format html

# Profile benchmarks with flamegraph
cargo install flamegraph
cargo flamegraph --bench force_calculation
```

### Cross-Platform Building
```bash
# Build for WebAssembly
cargo build --target wasm32-unknown-unknown

# Test WASM compatibility
cargo test --target wasm32-unknown-unknown

# Test Windows compatibility (from Linux/macOS)
cargo test --target x86_64-pc-windows-msvc
```

### Documentation and Examples
```bash
# Generate and open documentation
cargo doc --open

# Test documentation examples
cargo test --doc

# Run specific examples
cargo run --example solar_system      # Solar system simulation
cargo run --example binary_orbit      # Two-body problem
cargo run --example performance_test  # Performance validation
cargo run --example gpu_acceleration  # GPU usage demonstration
```

## Development Workflow Scripts

### Continuous Development
```bash
# Auto-run tests on file changes
cargo install cargo-watch
cargo watch -x test

# Watch specific test categories
cargo watch -x "test force"           # Watch force calculation tests
cargo watch -x "test energy"          # Watch energy conservation tests

# Continuous performance monitoring
cargo watch -x "bench --bench physics"
```

### Scientific Validation
```bash
# Run comprehensive physics validation
cargo test scientific_validation -- --nocapture

# Test against analytical solutions
cargo test analytical_solutions
cargo test kepler_orbits

# Compare with established N-body codes
cargo test nbody_benchmarks

# Validate energy conservation over time
cargo test energy_conservation -- --nocapture
```

### Performance Analysis
```bash
# Physics-specific debugging
RUST_LOG=gravwell::forces=debug cargo test     # Debug force calculations
RUST_LOG=gravwell::integrator=trace cargo test # Trace integration steps

# Numerical accuracy analysis
cargo test energy_conservation -- --nocapture  # Energy conservation
cargo test kepler_orbit -- --nocapture        # Orbital accuracy

# Memory usage profiling
cargo install heaptrack
heaptrack cargo test --release
```

## Development Environment Setup

### Required Tools
```bash
# Core Rust toolchain
rustup install stable
rustup default stable

# Development tools
cargo install cargo-watch      # File watching
cargo install flamegraph      # Performance profiling
cargo install cargo-criterion # Benchmark management

# Optional scientific tools
cargo install cargo-outdated  # Dependency management
cargo install cargo-audit     # Security scanning
```

### IDE Configuration (VS Code)
```json
// .vscode/settings.json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.features": ["simd", "parallel", "gpu"],
  "rust-analyzer.lens.enable": true,
  "rust-analyzer.inlayHints.enable": true
}
```

### Performance Profiling Setup
```bash
# CPU performance profiling
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Set CPU affinity for consistent benchmarks
taskset -c 0-7 cargo bench

# Memory profiling with valgrind
cargo install cargo-valgrind
cargo valgrind test --release
```

## CI/CD Integration Commands

### Pre-commit Validation
```bash
# Quality gates that must pass
cargo fmt --check               # Code formatting
cargo clippy -- -D warnings     # No clippy warnings
cargo test                      # All tests pass
cargo bench --no-run           # Benchmarks compile
cargo doc --no-deps            # Documentation builds
```

### Release Preparation
```bash
# Full validation suite
cargo test --release            # Release mode testing
cargo bench                     # Performance regression check
cargo build --target wasm32-unknown-unknown  # WASM compatibility
cargo audit                     # Security vulnerability scan

# Generate documentation
cargo doc --no-deps --open
```

### Platform Testing
```bash
# Test across platforms
cargo test --target x86_64-unknown-linux-gnu
cargo test --target x86_64-pc-windows-msvc
cargo test --target x86_64-apple-darwin
cargo test --target aarch64-apple-darwin
```

## Troubleshooting Commands

### Performance Issues
```bash
# Profile slow tests
cargo test --release -- --nocapture

# Benchmark comparison
cargo bench --save-baseline before
# Make changes
cargo bench --save-baseline after
cargo bench --baseline before --baseline after
```

### Memory Issues
```bash
# Check for memory leaks
cargo test --release
valgrind --tool=memcheck --leak-check=full cargo test
```

### GPU Issues
```bash
# Test GPU availability
cargo test gpu --features gpu -- --nocapture

# Fallback testing without GPU
cargo test --no-default-features --features std,parallel,simd
```