# Gravwell Tech Stack and Dependencies

## Core Rust Ecosystem

### Primary Dependencies
- **nalgebra 0.32+**: Linear algebra (Vector3, Matrix operations, SIMD support)
- **rayon 1.7+**: Data parallelism and multi-threading
- **wgpu 0.19**: WebGPU compute shaders for GPU acceleration
- **thiserror 1.0+**: Structured error handling with derive macros
- **serde 1.0+**: Serialization/deserialization (optional feature)

### Development Tools
- **criterion 0.5+**: Performance benchmarking with HTML reports
- **proptest 1.0**: Property-based testing for mathematical validation
- **approx 0.5**: Floating-point equality testing with epsilon tolerance

### Target Platforms
- **Native**: x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc, x86_64-apple-darwin, aarch64-apple-darwin
- **Web**: wasm32-unknown-unknown (with WebGPU support)

## Feature Flags (Cargo.toml)

### Core Features
- **`std`** (default): Standard library support
- **`parallel`**: Multi-threading with rayon
- **`simd`**: SIMD vectorization (AVX/AVX-512)
- **`gpu`**: GPU acceleration with WebGPU
- **`serde`**: Serialization support

### Build Configuration
```toml
[dependencies]
nalgebra = { version = "0.32", features = ["serde-serialize"] }
rayon = { version = "1.7", optional = true }
wgpu = { version = "0.19", optional = true }
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"], optional = true }

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.0"
approx = "0.5"
```

## Performance Optimization Stack

### SIMD Support
- **Target Features**: AVX, AVX2, AVX-512 (when available)
- **Portable SIMD**: std::simd for cross-platform vectorization
- **Manual Vectorization**: Hand-tuned force calculations

### GPU Compute Pipeline
- **WebGPU**: Cross-platform GPU compute (native + web)
- **WGSL Shaders**: Modern shader language for compute kernels
- **Buffer Management**: Efficient CPU-GPU memory transfers

### Parallel Processing
- **Thread Pool**: Configurable worker thread count
- **Data Parallelism**: Parallel iterators with rayon
- **NUMA Awareness**: Memory-local processing on multi-socket systems

## Build System Integration

### Rust Toolchain
- **MSRV**: 1.70+ (minimum supported Rust version)
- **Edition**: 2021
- **Profile Optimization**: LTO, codegen-units=1 for release builds

### Development Tools
- **rustfmt**: Code formatting (enforced in CI)
- **clippy**: Linting with physics-specific rules
- **cargo-watch**: Auto-rebuild on file changes
- **flamegraph**: Performance profiling integration

### Cross-Compilation
- **Native Targets**: All major desktop platforms
- **WebAssembly**: Browser deployment with WebGPU
- **CI/CD**: GitHub Actions with cross-platform testing

## Scientific Computing Stack

### Mathematical Libraries
- **nalgebra**: Vector/matrix operations with SIMD
- **num-traits**: Generic numeric programming
- **approx**: Fuzzy floating-point comparisons

### Validation Framework
- **proptest**: Property-based testing for physics laws
- **criterion**: Rigorous performance benchmarking
- **Integration**: Comparison with established N-body codes (REBOUND)

### Logging and Debugging
- **tracing**: Structured logging for performance analysis
- **env_logger**: Runtime log level configuration
- **Debug Builds**: Comprehensive assertion checking