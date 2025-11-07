//! CPU feature detection for SIMD optimization.
//!
//! This module provides runtime detection of CPU capabilities to select
//! the optimal SIMD instruction set for force calculations.

use crate::simd::SimdLevel;

/// CPU feature flags detected at runtime.
#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    /// SSE2 support (x86/x86_64 - 128-bit vectors, 2x f64).
    pub has_sse2: bool,
    /// AVX2 support (x86/x86_64 - 256-bit vectors, 4x f64).
    pub has_avx2: bool,
    /// AVX-512F support (x86/x86_64 - 512-bit vectors, 8x f64).
    pub has_avx512f: bool,
    /// NEON support (AArch64 - 128-bit vectors, 2x f64).
    pub has_neon: bool,
}

impl CpuFeatures {
    /// Determine the best SIMD level for the detected CPU features.
    pub fn best_simd_level(&self) -> SimdLevel {
        if self.has_avx512f {
            SimdLevel::Avx512
        } else if self.has_avx2 {
            SimdLevel::Avx2
        } else if self.has_sse2 {
            SimdLevel::Sse2
        } else if self.has_neon {
            SimdLevel::Neon
        } else {
            SimdLevel::Scalar
        }
    }
}

/// Detect CPU features at runtime.
///
/// This function uses the `is_x86_feature_detected!` macro on x86/x86_64
/// and feature detection mechanisms on other architectures.
///
/// # Examples
///
/// ```
/// use gravwell::simd::detect_cpu_features;
///
/// let features = detect_cpu_features();
/// println!("Best SIMD level: {:?}", features.best_simd_level());
/// ```
pub fn detect_cpu_features() -> CpuFeatures {
    CpuFeatures {
        has_sse2: detect_sse2(),
        has_avx2: detect_avx2(),
        has_avx512f: detect_avx512f(),
        has_neon: detect_neon(),
    }
}

fn detect_sse2() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        is_x86_feature_detected!("sse2")
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

fn detect_avx2() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        is_x86_feature_detected!("avx2")
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

fn detect_avx512f() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        is_x86_feature_detected!("avx512f")
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

fn detect_neon() -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        // NEON is mandatory on AArch64
        true
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_detection() {
        let features = detect_cpu_features();
        let simd_level = features.best_simd_level();

        // At minimum, we should have scalar support
        assert!(matches!(
            simd_level,
            SimdLevel::Scalar
                | SimdLevel::Sse2
                | SimdLevel::Avx2
                | SimdLevel::Avx512
                | SimdLevel::Neon
        ));

        println!(
            "Detected SIMD level: {} ({}x speedup)",
            simd_level.description(),
            simd_level.speedup_factor()
        );
    }
}
