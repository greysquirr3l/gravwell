//! SIMD-optimized force calculation kernels.
//!
//! This module provides vectorized implementations of gravitational force calculations
//! using platform-specific SIMD instruction sets (AVX-512, AVX2, NEON) with automatic
//! fallback to scalar implementations on unsupported hardware.
//!
//! # Supported Instruction Sets
//!
//! - **x86_64**: AVX-512 (8x f64), AVX2 (4x f64), SSE2 (2x f64)
//! - **AArch64**: NEON (2x f64, 4x f32)
//! - **Fallback**: Scalar implementation for all other architectures
//!
//! # Performance Impact
//!
//! SIMD optimizations provide significant speedup for force calculations:
//! - AVX-512: Up to 8x speedup for f64 operations
//! - AVX2: Up to 4x speedup for f64 operations  
//! - NEON: Up to 2x speedup for f64 operations
//!
//! # Usage
//!
//! The SIMD kernels are automatically selected at runtime based on CPU capabilities:
//!
//! ```
//! use gravwell::simd::VectorizedGravity;
//! 
//! let force_calc = VectorizedGravity::new(); // Auto-detects best SIMD level
//! // Use with any existing ForceCalculator-compatible code
//! ```

pub mod cpu_detection;
pub mod kernels;
pub mod vectorized_gravity;

pub use cpu_detection::{CpuFeatures, detect_cpu_features};
pub use kernels::{SimdKernel, AvxKernel, NeonKernel, ScalarKernel};
pub use vectorized_gravity::VectorizedGravity;

/// SIMD optimization level based on detected CPU features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdLevel {
    /// No SIMD support - use scalar fallback
    Scalar,
    /// SSE2 support (x86_64) - 2x f64 vectors
    Sse2,
    /// AVX2 support (x86_64) - 4x f64 vectors
    Avx2,
    /// AVX-512 support (x86_64) - 8x f64 vectors
    Avx512,
    /// NEON support (AArch64) - 2x f64 vectors
    Neon,
}

impl SimdLevel {
    /// Get the theoretical speedup factor for this SIMD level.
    pub fn speedup_factor(&self) -> f64 {
        match self {
            SimdLevel::Scalar => 1.0,
            SimdLevel::Sse2 => 2.0,
            SimdLevel::Avx2 => 4.0,
            SimdLevel::Avx512 => 8.0,
            SimdLevel::Neon => 2.0,
        }
    }
    
    /// Get the vector width (number of f64 elements) for this SIMD level.
    pub fn vector_width(&self) -> usize {
        match self {
            SimdLevel::Scalar => 1,
            SimdLevel::Sse2 => 2,
            SimdLevel::Avx2 => 4,
            SimdLevel::Avx512 => 8,
            SimdLevel::Neon => 2,
        }
    }
    
    /// Get a human-readable description of this SIMD level.
    pub fn description(&self) -> &'static str {
        match self {
            SimdLevel::Scalar => "Scalar (no SIMD)",
            SimdLevel::Sse2 => "SSE2 (128-bit vectors)",
            SimdLevel::Avx2 => "AVX2 (256-bit vectors)",
            SimdLevel::Avx512 => "AVX-512 (512-bit vectors)",
            SimdLevel::Neon => "NEON (128-bit vectors)",
        }
    }
}