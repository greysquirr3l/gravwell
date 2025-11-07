//! SIMD kernel implementations for vectorized force calculations.
//!
//! This module contains the actual SIMD implementations for calculating
//! gravitational forces between particles using various instruction sets.

use crate::simd::SimdLevel;
use crate::types::{Scalar, Vector3};

/// Trait for SIMD kernel implementations.
///
/// Each kernel implementation provides vectorized force calculation
/// for a specific instruction set (AVX-512, AVX2, NEON, etc.).
pub trait SimdKernel {
    /// Calculate forces between particles using SIMD instructions.
    ///
    /// # Arguments
    ///
    /// * `positions` - Array of particle positions
    /// * `masses` - Array of particle masses
    /// * `forces` - Output array for calculated forces (will be modified)
    /// * `gravitational_constant` - Gravitational constant G
    /// * `softening_parameter` - Softening parameter to avoid singularities
    ///
    /// # Safety
    ///
    /// This function may use unsafe SIMD intrinsics. All implementations
    /// must ensure proper alignment and bounds checking.
    fn calculate_forces_simd(
        &self,
        positions: &[Vector3],
        masses: &[Scalar],
        forces: &mut [Vector3],
        gravitational_constant: Scalar,
        softening_parameter: Scalar,
    );

    /// Get the SIMD level supported by this kernel.
    fn simd_level(&self) -> SimdLevel;

    /// Get the optimal chunk size for this kernel's vector width.
    fn chunk_size(&self) -> usize {
        self.simd_level().vector_width()
    }
}

/// AVX-512 kernel implementation for x86_64.
pub struct AvxKernel {
    simd_level: SimdLevel,
}

impl AvxKernel {
    /// Create a new AVX kernel with the best available instruction set.
    pub fn new() -> Self {
        use crate::simd::detect_cpu_features;

        let features = detect_cpu_features();
        let simd_level = if features.has_avx512f {
            SimdLevel::Avx512
        } else if features.has_avx2 {
            SimdLevel::Avx2
        } else if features.has_sse2 {
            SimdLevel::Sse2
        } else {
            SimdLevel::Scalar
        };

        Self { simd_level }
    }
}

impl SimdKernel for AvxKernel {
    fn calculate_forces_simd(
        &self,
        positions: &[Vector3],
        masses: &[Scalar],
        forces: &mut [Vector3],
        gravitational_constant: Scalar,
        softening_parameter: Scalar,
    ) {
        match self.simd_level {
            SimdLevel::Avx512 => {
                #[cfg(target_arch = "x86_64")]
                {
                    if is_x86_feature_detected!("avx512f") {
                        unsafe {
                            self.calculate_forces_avx512(
                                positions,
                                masses,
                                forces,
                                gravitational_constant,
                                softening_parameter,
                            )
                        }
                    } else {
                        ScalarKernel.calculate_forces_simd(
                            positions,
                            masses,
                            forces,
                            gravitational_constant,
                            softening_parameter,
                        );
                    }
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    ScalarKernel.calculate_forces_simd(
                        positions,
                        masses,
                        forces,
                        gravitational_constant,
                        softening_parameter,
                    );
                }
            },
            SimdLevel::Avx2 => {
                #[cfg(target_arch = "x86_64")]
                {
                    if is_x86_feature_detected!("avx2") {
                        unsafe {
                            self.calculate_forces_avx2(
                                positions,
                                masses,
                                forces,
                                gravitational_constant,
                                softening_parameter,
                            )
                        }
                    } else {
                        ScalarKernel.calculate_forces_simd(
                            positions,
                            masses,
                            forces,
                            gravitational_constant,
                            softening_parameter,
                        );
                    }
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    ScalarKernel.calculate_forces_simd(
                        positions,
                        masses,
                        forces,
                        gravitational_constant,
                        softening_parameter,
                    );
                }
            },
            SimdLevel::Sse2 => {
                #[cfg(target_arch = "x86_64")]
                {
                    if is_x86_feature_detected!("sse2") {
                        unsafe {
                            self.calculate_forces_sse2(
                                positions,
                                masses,
                                forces,
                                gravitational_constant,
                                softening_parameter,
                            )
                        }
                    } else {
                        ScalarKernel.calculate_forces_simd(
                            positions,
                            masses,
                            forces,
                            gravitational_constant,
                            softening_parameter,
                        );
                    }
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    ScalarKernel.calculate_forces_simd(
                        positions,
                        masses,
                        forces,
                        gravitational_constant,
                        softening_parameter,
                    );
                }
            },
            _ => ScalarKernel.calculate_forces_simd(
                positions,
                masses,
                forces,
                gravitational_constant,
                softening_parameter,
            ),
        }
    }

    fn simd_level(&self) -> SimdLevel {
        self.simd_level
    }
}

impl AvxKernel {
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx512f")]
    unsafe fn calculate_forces_avx512(
        &self,
        positions: &[Vector3],
        masses: &[Scalar],
        forces: &mut [Vector3],
        gravitational_constant: Scalar,
        softening_parameter: Scalar,
    ) {
        use std::arch::x86_64::*;

        let n = positions.len();
        let softening_squared = softening_parameter * softening_parameter;

        // Process 8 particles at a time with AVX-512
        for i in (0..n).step_by(8) {
            let chunk_size = (n - i).min(8);

            // Load positions and masses for current chunk
            let mut pos_x = [0.0f64; 8];
            let mut pos_y = [0.0f64; 8];
            let mut pos_z = [0.0f64; 8];
            let mut mass_i = [0.0f64; 8];

            for k in 0..chunk_size {
                pos_x[k] = positions[i + k].x;
                pos_y[k] = positions[i + k].y;
                pos_z[k] = positions[i + k].z;
                mass_i[k] = masses[i + k];
            }

            let pos_x_vec = _mm512_load_pd(pos_x.as_ptr());
            let pos_y_vec = _mm512_load_pd(pos_y.as_ptr());
            let pos_z_vec = _mm512_load_pd(pos_z.as_ptr());
            let mass_i_vec = _mm512_load_pd(mass_i.as_ptr());

            let mut force_x_vec = _mm512_setzero_pd();
            let mut force_y_vec = _mm512_setzero_pd();
            let mut force_z_vec = _mm512_setzero_pd();

            // Calculate forces from all other particles
            for j in 0..n {
                if i <= j && j < i + chunk_size {
                    continue;
                }

                let pos_j_x = _mm512_set1_pd(positions[j].x);
                let pos_j_y = _mm512_set1_pd(positions[j].y);
                let pos_j_z = _mm512_set1_pd(positions[j].z);
                let mass_j = _mm512_set1_pd(masses[j]);

                // Calculate distance vectors
                let dx = _mm512_sub_pd(pos_j_x, pos_x_vec);
                let dy = _mm512_sub_pd(pos_j_y, pos_y_vec);
                let dz = _mm512_sub_pd(pos_j_z, pos_z_vec);

                // Calculate r_squared = dx^2 + dy^2 + dz^2 + softening^2
                let dx_sq = _mm512_mul_pd(dx, dx);
                let dy_sq = _mm512_mul_pd(dy, dy);
                let dz_sq = _mm512_mul_pd(dz, dz);
                let softening_vec = _mm512_set1_pd(softening_squared);

                let r_squared = _mm512_add_pd(
                    _mm512_add_pd(dx_sq, dy_sq),
                    _mm512_add_pd(dz_sq, softening_vec),
                );

                // Calculate 1/r_squared and 1/r
                let inv_r_squared = _mm512_div_pd(_mm512_set1_pd(1.0), r_squared);
                let inv_r = _mm512_sqrt_pd(inv_r_squared);

                // Calculate force magnitude = G * mass_i * mass_j / r_squared
                let g_vec = _mm512_set1_pd(gravitational_constant);
                let force_magnitude = _mm512_mul_pd(
                    _mm512_mul_pd(_mm512_mul_pd(g_vec, mass_i_vec), mass_j),
                    inv_r_squared,
                );

                // Calculate force components = force_magnitude * (dx/r, dy/r, dz/r)
                let force_x = _mm512_mul_pd(force_magnitude, _mm512_mul_pd(dx, inv_r));
                let force_y = _mm512_mul_pd(force_magnitude, _mm512_mul_pd(dy, inv_r));
                let force_z = _mm512_mul_pd(force_magnitude, _mm512_mul_pd(dz, inv_r));

                // Accumulate forces
                force_x_vec = _mm512_add_pd(force_x_vec, force_x);
                force_y_vec = _mm512_add_pd(force_y_vec, force_y);
                force_z_vec = _mm512_add_pd(force_z_vec, force_z);
            }

            // Store results back to forces array
            let mut result_x = [0.0f64; 8];
            let mut result_y = [0.0f64; 8];
            let mut result_z = [0.0f64; 8];

            _mm512_store_pd(result_x.as_mut_ptr(), force_x_vec);
            _mm512_store_pd(result_y.as_mut_ptr(), force_y_vec);
            _mm512_store_pd(result_z.as_mut_ptr(), force_z_vec);

            for k in 0..chunk_size {
                forces[i + k].x += result_x[k];
                forces[i + k].y += result_y[k];
                forces[i + k].z += result_z[k];
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn calculate_forces_avx2(
        &self,
        positions: &[Vector3],
        masses: &[Scalar],
        forces: &mut [Vector3],
        gravitational_constant: Scalar,
        softening_parameter: Scalar,
    ) {
        use std::arch::x86_64::*;

        let n = positions.len();
        let softening_squared = softening_parameter * softening_parameter;

        // Process 4 particles at a time with AVX2
        for i in (0..n).step_by(4) {
            let chunk_size = (n - i).min(4);

            // Load positions and masses for current chunk
            let mut pos_x = [0.0f64; 4];
            let mut pos_y = [0.0f64; 4];
            let mut pos_z = [0.0f64; 4];
            let mut mass_i = [0.0f64; 4];

            for k in 0..chunk_size {
                pos_x[k] = positions[i + k].x;
                pos_y[k] = positions[i + k].y;
                pos_z[k] = positions[i + k].z;
                mass_i[k] = masses[i + k];
            }

            let pos_x_vec = _mm256_load_pd(pos_x.as_ptr());
            let pos_y_vec = _mm256_load_pd(pos_y.as_ptr());
            let pos_z_vec = _mm256_load_pd(pos_z.as_ptr());
            let mass_i_vec = _mm256_load_pd(mass_i.as_ptr());

            let mut force_x_vec = _mm256_setzero_pd();
            let mut force_y_vec = _mm256_setzero_pd();
            let mut force_z_vec = _mm256_setzero_pd();

            // Calculate forces from all other particles
            for j in 0..n {
                if i <= j && j < i + chunk_size {
                    continue;
                }

                let pos_j_x = _mm256_set1_pd(positions[j].x);
                let pos_j_y = _mm256_set1_pd(positions[j].y);
                let pos_j_z = _mm256_set1_pd(positions[j].z);
                let mass_j = _mm256_set1_pd(masses[j]);

                // Calculate distance vectors
                let dx = _mm256_sub_pd(pos_j_x, pos_x_vec);
                let dy = _mm256_sub_pd(pos_j_y, pos_y_vec);
                let dz = _mm256_sub_pd(pos_j_z, pos_z_vec);

                // Calculate r_squared = dx^2 + dy^2 + dz^2 + softening^2
                let dx_sq = _mm256_mul_pd(dx, dx);
                let dy_sq = _mm256_mul_pd(dy, dy);
                let dz_sq = _mm256_mul_pd(dz, dz);
                let softening_vec = _mm256_set1_pd(softening_squared);

                let r_squared = _mm256_add_pd(
                    _mm256_add_pd(dx_sq, dy_sq),
                    _mm256_add_pd(dz_sq, softening_vec),
                );

                // Calculate 1/r_squared and 1/r
                let inv_r_squared = _mm256_div_pd(_mm256_set1_pd(1.0), r_squared);
                let inv_r = _mm256_sqrt_pd(inv_r_squared);

                // Calculate force magnitude = G * mass_i * mass_j / r_squared
                let g_vec = _mm256_set1_pd(gravitational_constant);
                let force_magnitude = _mm256_mul_pd(
                    _mm256_mul_pd(_mm256_mul_pd(g_vec, mass_i_vec), mass_j),
                    inv_r_squared,
                );

                // Calculate force components = force_magnitude * (dx/r, dy/r, dz/r)
                let force_x = _mm256_mul_pd(force_magnitude, _mm256_mul_pd(dx, inv_r));
                let force_y = _mm256_mul_pd(force_magnitude, _mm256_mul_pd(dy, inv_r));
                let force_z = _mm256_mul_pd(force_magnitude, _mm256_mul_pd(dz, inv_r));

                // Accumulate forces
                force_x_vec = _mm256_add_pd(force_x_vec, force_x);
                force_y_vec = _mm256_add_pd(force_y_vec, force_y);
                force_z_vec = _mm256_add_pd(force_z_vec, force_z);
            }

            // Store results back to forces array
            let mut result_x = [0.0f64; 4];
            let mut result_y = [0.0f64; 4];
            let mut result_z = [0.0f64; 4];

            _mm256_store_pd(result_x.as_mut_ptr(), force_x_vec);
            _mm256_store_pd(result_y.as_mut_ptr(), force_y_vec);
            _mm256_store_pd(result_z.as_mut_ptr(), force_z_vec);

            for k in 0..chunk_size {
                forces[i + k].x += result_x[k];
                forces[i + k].y += result_y[k];
                forces[i + k].z += result_z[k];
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn calculate_forces_sse2(
        &self,
        positions: &[Vector3],
        masses: &[Scalar],
        forces: &mut [Vector3],
        gravitational_constant: Scalar,
        softening_parameter: Scalar,
    ) {
        use std::arch::x86_64::*;

        let n = positions.len();
        let softening_squared = softening_parameter * softening_parameter;

        // Process 2 particles at a time with SSE2
        for i in (0..n).step_by(2) {
            let chunk_size = (n - i).min(2);

            // Load positions and masses for current chunk
            let mut pos_x = [0.0f64; 2];
            let mut pos_y = [0.0f64; 2];
            let mut pos_z = [0.0f64; 2];
            let mut mass_i = [0.0f64; 2];

            for k in 0..chunk_size {
                pos_x[k] = positions[i + k].x;
                pos_y[k] = positions[i + k].y;
                pos_z[k] = positions[i + k].z;
                mass_i[k] = masses[i + k];
            }

            let pos_x_vec = _mm_load_pd(pos_x.as_ptr());
            let pos_y_vec = _mm_load_pd(pos_y.as_ptr());
            let pos_z_vec = _mm_load_pd(pos_z.as_ptr());
            let mass_i_vec = _mm_load_pd(mass_i.as_ptr());

            let mut force_x_vec = _mm_setzero_pd();
            let mut force_y_vec = _mm_setzero_pd();
            let mut force_z_vec = _mm_setzero_pd();

            // Calculate forces from all other particles
            for j in 0..n {
                if i <= j && j < i + chunk_size {
                    continue;
                }

                let pos_j_x = _mm_set1_pd(positions[j].x);
                let pos_j_y = _mm_set1_pd(positions[j].y);
                let pos_j_z = _mm_set1_pd(positions[j].z);
                let mass_j = _mm_set1_pd(masses[j]);

                // Calculate distance vectors
                let dx = _mm_sub_pd(pos_j_x, pos_x_vec);
                let dy = _mm_sub_pd(pos_j_y, pos_y_vec);
                let dz = _mm_sub_pd(pos_j_z, pos_z_vec);

                // Calculate r_squared = dx^2 + dy^2 + dz^2 + softening^2
                let dx_sq = _mm_mul_pd(dx, dx);
                let dy_sq = _mm_mul_pd(dy, dy);
                let dz_sq = _mm_mul_pd(dz, dz);
                let softening_vec = _mm_set1_pd(softening_squared);

                let r_squared =
                    _mm_add_pd(_mm_add_pd(dx_sq, dy_sq), _mm_add_pd(dz_sq, softening_vec));

                // Calculate 1/r_squared and 1/r
                let inv_r_squared = _mm_div_pd(_mm_set1_pd(1.0), r_squared);
                let inv_r = _mm_sqrt_pd(inv_r_squared);

                // Calculate force magnitude = G * mass_i * mass_j / r_squared
                let g_vec = _mm_set1_pd(gravitational_constant);
                let force_magnitude = _mm_mul_pd(
                    _mm_mul_pd(_mm_mul_pd(g_vec, mass_i_vec), mass_j),
                    inv_r_squared,
                );

                // Calculate force components = force_magnitude * (dx/r, dy/r, dz/r)
                let force_x = _mm_mul_pd(force_magnitude, _mm_mul_pd(dx, inv_r));
                let force_y = _mm_mul_pd(force_magnitude, _mm_mul_pd(dy, inv_r));
                let force_z = _mm_mul_pd(force_magnitude, _mm_mul_pd(dz, inv_r));

                // Accumulate forces
                force_x_vec = _mm_add_pd(force_x_vec, force_x);
                force_y_vec = _mm_add_pd(force_y_vec, force_y);
                force_z_vec = _mm_add_pd(force_z_vec, force_z);
            }

            // Store results back to forces array
            let mut result_x = [0.0f64; 2];
            let mut result_y = [0.0f64; 2];
            let mut result_z = [0.0f64; 2];

            _mm_store_pd(result_x.as_mut_ptr(), force_x_vec);
            _mm_store_pd(result_y.as_mut_ptr(), force_y_vec);
            _mm_store_pd(result_z.as_mut_ptr(), force_z_vec);

            for k in 0..chunk_size {
                forces[i + k].x += result_x[k];
                forces[i + k].y += result_y[k];
                forces[i + k].z += result_z[k];
            }
        }
    }
}

/// NEON kernel implementation for AArch64.
pub struct NeonKernel;

impl SimdKernel for NeonKernel {
    fn calculate_forces_simd(
        &self,
        positions: &[Vector3],
        masses: &[Scalar],
        forces: &mut [Vector3],
        gravitational_constant: Scalar,
        softening_parameter: Scalar,
    ) {
        #[cfg(target_arch = "aarch64")]
        {
            self.calculate_forces_neon(
                positions,
                masses,
                forces,
                gravitational_constant,
                softening_parameter,
            );
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            ScalarKernel.calculate_forces_simd(
                positions,
                masses,
                forces,
                gravitational_constant,
                softening_parameter,
            );
        }
    }

    fn simd_level(&self) -> SimdLevel {
        #[cfg(target_arch = "aarch64")]
        {
            SimdLevel::Neon
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            SimdLevel::Scalar
        }
    }
}

impl NeonKernel {
    #[cfg(target_arch = "aarch64")]
    fn calculate_forces_neon(
        &self,
        positions: &[Vector3],
        masses: &[Scalar],
        forces: &mut [Vector3],
        gravitational_constant: Scalar,
        softening_parameter: Scalar,
    ) {
        use std::arch::aarch64::*;

        let n = positions.len();
        let softening_squared = softening_parameter * softening_parameter;

        // Process 2 particles at a time with NEON (f64x2)
        for i in (0..n).step_by(2) {
            let chunk_size = (n - i).min(2);

            // Load positions and masses for current chunk
            let mut pos_x = [0.0f64; 2];
            let mut pos_y = [0.0f64; 2];
            let mut pos_z = [0.0f64; 2];
            let mut mass_i = [0.0f64; 2];

            for k in 0..chunk_size {
                pos_x[k] = positions[i + k].x;
                pos_y[k] = positions[i + k].y;
                pos_z[k] = positions[i + k].z;
                mass_i[k] = masses[i + k];
            }

            unsafe {
                let pos_x_vec = vld1q_f64(pos_x.as_ptr());
                let pos_y_vec = vld1q_f64(pos_y.as_ptr());
                let pos_z_vec = vld1q_f64(pos_z.as_ptr());
                let mass_i_vec = vld1q_f64(mass_i.as_ptr());

                let mut force_x_vec = vdupq_n_f64(0.0);
                let mut force_y_vec = vdupq_n_f64(0.0);
                let mut force_z_vec = vdupq_n_f64(0.0);

                // Calculate forces from all other particles
                for j in 0..n {
                    if i <= j && j < i + chunk_size {
                        continue;
                    }

                    let pos_j_x = vdupq_n_f64(positions[j].x);
                    let pos_j_y = vdupq_n_f64(positions[j].y);
                    let pos_j_z = vdupq_n_f64(positions[j].z);
                    let mass_j = vdupq_n_f64(masses[j]);

                    // Calculate distance vectors
                    let dx = vsubq_f64(pos_j_x, pos_x_vec);
                    let dy = vsubq_f64(pos_j_y, pos_y_vec);
                    let dz = vsubq_f64(pos_j_z, pos_z_vec);

                    // Calculate r_squared = dx^2 + dy^2 + dz^2 + softening^2
                    let dx_sq = vmulq_f64(dx, dx);
                    let dy_sq = vmulq_f64(dy, dy);
                    let dz_sq = vmulq_f64(dz, dz);
                    let softening_vec = vdupq_n_f64(softening_squared);

                    let r_squared =
                        vaddq_f64(vaddq_f64(dx_sq, dy_sq), vaddq_f64(dz_sq, softening_vec));

                    // Calculate 1/r_squared and 1/r
                    let inv_r_squared = vdivq_f64(vdupq_n_f64(1.0), r_squared);
                    let inv_r = vsqrtq_f64(inv_r_squared);

                    // Calculate force magnitude = G * mass_i * mass_j / r_squared
                    let g_vec = vdupq_n_f64(gravitational_constant);
                    let force_magnitude = vmulq_f64(
                        vmulq_f64(vmulq_f64(g_vec, mass_i_vec), mass_j),
                        inv_r_squared,
                    );

                    // Calculate force components = force_magnitude * (dx/r, dy/r, dz/r)
                    let force_x = vmulq_f64(force_magnitude, vmulq_f64(dx, inv_r));
                    let force_y = vmulq_f64(force_magnitude, vmulq_f64(dy, inv_r));
                    let force_z = vmulq_f64(force_magnitude, vmulq_f64(dz, inv_r));

                    // Accumulate forces
                    force_x_vec = vaddq_f64(force_x_vec, force_x);
                    force_y_vec = vaddq_f64(force_y_vec, force_y);
                    force_z_vec = vaddq_f64(force_z_vec, force_z);
                }

                // Store results back to forces array
                let mut result_x = [0.0f64; 2];
                let mut result_y = [0.0f64; 2];
                let mut result_z = [0.0f64; 2];

                vst1q_f64(result_x.as_mut_ptr(), force_x_vec);
                vst1q_f64(result_y.as_mut_ptr(), force_y_vec);
                vst1q_f64(result_z.as_mut_ptr(), force_z_vec);

                for k in 0..chunk_size {
                    forces[i + k].x += result_x[k];
                    forces[i + k].y += result_y[k];
                    forces[i + k].z += result_z[k];
                }
            }
        }
    }
}

/// Scalar fallback kernel for unsupported architectures.
pub struct ScalarKernel;

impl SimdKernel for ScalarKernel {
    fn calculate_forces_simd(
        &self,
        positions: &[Vector3],
        masses: &[Scalar],
        forces: &mut [Vector3],
        gravitational_constant: Scalar,
        softening_parameter: Scalar,
    ) {
        let n = positions.len();
        let softening_squared = softening_parameter * softening_parameter;

        // Zero out forces
        for force in forces.iter_mut() {
            *force = Vector3::zeros();
        }

        // Calculate forces using standard scalar operations
        for i in 0..n {
            for j in (i + 1)..n {
                // Calculate distance vector
                let r_vec = positions[j] - positions[i];
                let r_squared = r_vec.norm_squared() + softening_squared;
                let r = r_squared.sqrt();

                // Calculate force magnitude
                let force_magnitude = gravitational_constant * masses[i] * masses[j] / r_squared;
                let force_vector = force_magnitude * r_vec / r;

                // Apply Newton's third law
                forces[i] += force_vector;
                forces[j] -= force_vector;
            }
        }
    }

    fn simd_level(&self) -> SimdLevel {
        SimdLevel::Scalar
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Vector3;
    use approx::assert_relative_eq;

    fn create_test_system() -> (Vec<Vector3>, Vec<Scalar>) {
        let positions = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ];
        let masses = vec![1.0, 1.0, 1.0, 1.0];
        (positions, masses)
    }

    #[test]
    fn test_scalar_kernel() {
        let (positions, masses) = create_test_system();
        let mut forces = vec![Vector3::zeros(); 4];

        let kernel = ScalarKernel;
        kernel.calculate_forces_simd(&positions, &masses, &mut forces, 1.0, 0.01);

        // Forces should not be zero
        assert!(forces.iter().any(|f| f.norm() > 0.0));

        // Total force should sum to zero (conservation of momentum)
        let total_force: Vector3 = forces.iter().sum();
        assert_relative_eq!(total_force.norm(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_avx_kernel_fallback() {
        let (positions, masses) = create_test_system();
        let mut forces = vec![Vector3::zeros(); 4];

        let kernel = AvxKernel::new();
        kernel.calculate_forces_simd(&positions, &masses, &mut forces, 1.0, 0.01);

        // Forces should not be zero
        assert!(forces.iter().any(|f| f.norm() > 0.0));
    }

    #[test]
    fn test_neon_kernel_fallback() {
        let (positions, masses) = create_test_system();
        let mut forces = vec![Vector3::zeros(); 4];

        let kernel = NeonKernel;
        kernel.calculate_forces_simd(&positions, &masses, &mut forces, 1.0, 0.01);

        // Forces should not be zero
        assert!(forces.iter().any(|f| f.norm() > 0.0));
    }
}
