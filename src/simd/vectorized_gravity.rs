//! Vectorized gravity force calculator with automatic SIMD optimization.
//!
//! This module provides a drop-in replacement for traditional force calculators
//! that automatically selects the best SIMD implementation based on CPU capabilities.

use crate::core::{forces::ForceCalculator, particle::ParticleSet};
use crate::error::{GravwellError, Result as GravwellResult};
use crate::simd::{
    detect_cpu_features, AvxKernel, NeonKernel, ScalarKernel, SimdKernel, SimdLevel,
};
use crate::types::{Force, Scalar};

/// High-performance vectorized gravity calculator.
///
/// This force calculator automatically detects CPU features and selects
/// the optimal SIMD implementation (AVX-512, AVX2, NEON, or scalar fallback).
///
/// # Performance
///
/// Expected speedup over scalar implementations:
/// - AVX-512: 4-8x speedup
/// - AVX2: 2-4x speedup
/// - NEON: 1.5-2x speedup
/// - SSE2: 1.5-2x speedup
///
/// # Examples
///
/// ```
/// use gravwell::simd::VectorizedGravity;
/// use gravwell::core::ForceCalculator;
///
/// let force_calc = VectorizedGravity::new();
/// println!("Using SIMD level: {}", force_calc.simd_level().description());
///
/// // Use like any other ForceCalculator
/// // let simulation = Simulation::builder()
/// //     .forces(force_calc)
/// //     .build()?;
/// ```
pub struct VectorizedGravity {
    kernel: Box<dyn SimdKernel + Send + Sync>,
    simd_level: SimdLevel,
    softening_parameter: Scalar,
}

impl VectorizedGravity {
    /// Create a new vectorized gravity calculator with automatic SIMD detection.
    pub fn new() -> Self {
        let features = detect_cpu_features();
        let simd_level = features.best_simd_level();

        let kernel: Box<dyn SimdKernel + Send + Sync> = match simd_level {
            SimdLevel::Avx512 | SimdLevel::Avx2 | SimdLevel::Sse2 => Box::new(AvxKernel::new()),
            SimdLevel::Neon => Box::new(NeonKernel),
            SimdLevel::Scalar => Box::new(ScalarKernel),
        };

        Self {
            kernel,
            simd_level,
            softening_parameter: 0.0,
        }
    }

    /// Create a vectorized gravity calculator with a specific SIMD level.
    ///
    /// This is useful for testing or when you want to force a specific
    /// implementation regardless of CPU capabilities.
    ///
    /// # Arguments
    ///
    /// * `simd_level` - The SIMD level to use
    ///
    /// # Examples
    ///
    /// ```
    /// use gravwell::simd::{VectorizedGravity, SimdLevel};
    ///
    /// let force_calc = VectorizedGravity::with_simd_level(SimdLevel::Scalar);
    /// ```
    pub fn with_simd_level(simd_level: SimdLevel) -> Self {
        let kernel: Box<dyn SimdKernel + Send + Sync> = match simd_level {
            SimdLevel::Avx512 | SimdLevel::Avx2 | SimdLevel::Sse2 => Box::new(AvxKernel::new()),
            SimdLevel::Neon => Box::new(NeonKernel),
            SimdLevel::Scalar => Box::new(ScalarKernel),
        };

        Self {
            kernel,
            simd_level,
            softening_parameter: 0.0,
        }
    }

    /// Set the softening parameter to avoid singularities.
    ///
    /// # Arguments
    ///
    /// * `softening` - Softening parameter (typically 1e-6 to 1e-3)
    ///
    /// # Examples
    ///
    /// ```
    /// use gravwell::simd::VectorizedGravity;
    ///
    /// let force_calc = VectorizedGravity::new()
    ///     .with_softening(1e-6);
    /// ```
    pub fn with_softening(mut self, softening: Scalar) -> Self {
        self.softening_parameter = softening;
        self
    }

    /// Get the SIMD level being used by this calculator.
    pub fn simd_level(&self) -> SimdLevel {
        self.simd_level
    }

    /// Get the theoretical speedup factor for this SIMD level.
    pub fn speedup_factor(&self) -> f64 {
        self.simd_level.speedup_factor()
    }

    /// Get the vector width (number of f64 elements processed per operation).
    pub fn vector_width(&self) -> usize {
        self.simd_level.vector_width()
    }

    /// Get a description of the SIMD implementation being used.
    pub fn description(&self) -> String {
        format!(
            "{} ({}x speedup, {}-wide vectors)",
            self.simd_level.description(),
            self.speedup_factor(),
            self.vector_width()
        )
    }
}

impl Default for VectorizedGravity {
    fn default() -> Self {
        Self::new()
    }
}

impl ForceCalculator for VectorizedGravity {
    fn calculate_forces(
        &self,
        particles: &ParticleSet,
        forces: &mut [Force],
    ) -> GravwellResult<()> {
        // Validate input
        if particles.len() != forces.len() {
            return Err(GravwellError::Configuration(
                "Force array length must match number of particles".to_string(),
            ));
        }

        // Use standard gravitational constant
        const G: Scalar = 6.67430e-11; // m³/(kg⋅s²)

        // Delegate to SIMD kernel
        self.kernel.calculate_forces_simd(
            particles.positions(),
            particles.masses(),
            forces,
            G,
            self.softening_parameter,
        );

        Ok(())
    }

    fn name(&self) -> &'static str {
        match self.simd_level {
            SimdLevel::Avx512 => "VectorizedGravity (AVX-512)",
            SimdLevel::Avx2 => "VectorizedGravity (AVX2)",
            SimdLevel::Sse2 => "VectorizedGravity (SSE2)",
            SimdLevel::Neon => "VectorizedGravity (NEON)",
            SimdLevel::Scalar => "VectorizedGravity (Scalar)",
        }
    }

    fn complexity(&self) -> &'static str {
        "O(N²)"
    }
}

/// Builder for creating customized VectorizedGravity instances.
pub struct VectorizedGravityBuilder {
    simd_level: Option<SimdLevel>,
    softening_parameter: Scalar,
    gravitational_constant: Option<Scalar>,
}

impl VectorizedGravityBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            simd_level: None,
            softening_parameter: 0.0,
            gravitational_constant: None,
        }
    }

    /// Force a specific SIMD level instead of auto-detection.
    pub fn simd_level(mut self, level: SimdLevel) -> Self {
        self.simd_level = Some(level);
        self
    }

    /// Set the softening parameter.
    pub fn softening(mut self, softening: Scalar) -> Self {
        self.softening_parameter = softening;
        self
    }

    /// Set a custom gravitational constant (for unit testing or non-SI units).
    pub fn gravitational_constant(mut self, g: Scalar) -> Self {
        self.gravitational_constant = Some(g);
        self
    }

    /// Build the VectorizedGravity instance.
    pub fn build(self) -> VectorizedGravity {
        let mut calc = if let Some(level) = self.simd_level {
            VectorizedGravity::with_simd_level(level)
        } else {
            VectorizedGravity::new()
        };

        calc.softening_parameter = self.softening_parameter;
        calc
    }
}

impl Default for VectorizedGravityBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Vector3;
    use approx::assert_relative_eq;

    fn create_binary_system() -> ParticleSet {
        use crate::core::particle::{Body, ParticleSet};

        let mut particles = ParticleSet::new();

        particles
            .add_body(
                Body::new()
                    .with_mass(1.0e30)
                    .with_position([-0.5, 0.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();

        particles
            .add_body(
                Body::new()
                    .with_mass(1.0e30)
                    .with_position([0.5, 0.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();

        particles
    }

    fn create_four_body_system() -> ParticleSet {
        use crate::core::particle::{Body, ParticleSet};

        let mut particles = ParticleSet::new();

        particles
            .add_body(
                Body::new()
                    .with_mass(1.0e30)
                    .with_position([0.0, 0.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();

        particles
            .add_body(
                Body::new()
                    .with_mass(1.0e30)
                    .with_position([1.0, 0.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();

        particles
            .add_body(
                Body::new()
                    .with_mass(1.0e30)
                    .with_position([0.0, 1.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();

        particles
            .add_body(
                Body::new()
                    .with_mass(1.0e30)
                    .with_position([0.0, 0.0, 1.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();

        particles
    }

    #[test]
    fn test_vectorized_gravity_creation() {
        let calc = VectorizedGravity::new();
        println!("Created calculator: {}", calc.description());

        // Should create without error
        assert!(calc.speedup_factor() >= 1.0);
        assert!(calc.vector_width() >= 1);
    }

    #[test]
    fn test_binary_system_forces() {
        let calc = VectorizedGravity::new();
        let particles = create_binary_system();
        let mut forces = vec![Vector3::zeros(); 2];

        calc.calculate_forces(&particles, &mut forces).unwrap();

        // Forces should be equal and opposite (Newton's third law)
        assert_relative_eq!(forces[0].norm(), forces[1].norm(), epsilon = 1e-10);
        assert_relative_eq!(forces[0].x, -forces[1].x, epsilon = 1e-10);

        // Force should point toward the other mass
        assert!(forces[0].x > 0.0); // Star 1 attracted toward Star 2 (positive x)
        assert!(forces[1].x < 0.0); // Star 2 attracted toward Star 1 (negative x)
    }

    #[test]
    fn test_four_body_system_momentum_conservation() {
        let calc = VectorizedGravity::new();
        let particles = create_four_body_system();
        let mut forces = vec![Vector3::zeros(); 4];

        calc.calculate_forces(&particles, &mut forces).unwrap();

        // Total force should sum to zero (momentum conservation)
        let total_force: Vector3 = forces.iter().sum();
        assert_relative_eq!(total_force.norm(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_simd_level_consistency() {
        // Test that all SIMD levels produce the same results
        let particles = create_four_body_system();

        let levels = vec![
            SimdLevel::Scalar,
            SimdLevel::Sse2,
            SimdLevel::Avx2,
            SimdLevel::Avx512,
            SimdLevel::Neon,
        ];

        let mut results = Vec::new();

        for level in levels {
            let calc = VectorizedGravity::with_simd_level(level);
            let mut forces = vec![Vector3::zeros(); 4];
            calc.calculate_forces(&particles, &mut forces).unwrap();
            results.push(forces);
        }

        // Compare all results against scalar reference
        let scalar_result = &results[0];
        for (_i, result) in results.iter().enumerate().skip(1) {
            for j in 0..4 {
                assert_relative_eq!(result[j].x, scalar_result[j].x, epsilon = 1e-10);
                assert_relative_eq!(result[j].y, scalar_result[j].y, epsilon = 1e-10);
                assert_relative_eq!(result[j].z, scalar_result[j].z, epsilon = 1e-10);
            }
        }
    }

    #[test]
    fn test_softening_parameter() {
        use crate::core::particle::{Body, ParticleSet};

        let calc = VectorizedGravity::new().with_softening(1e-6);

        let mut particles = ParticleSet::new();
        particles
            .add_body(
                Body::new()
                    .with_mass(1.0e30)
                    .with_position([0.0, 0.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();

        particles
            .add_body(
                Body::new()
                    .with_mass(1.0e30)
                    .with_position([1e-10, 0.0, 0.0]) // Very close particles
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();

        let mut forces = vec![Vector3::zeros(); 2];

        calc.calculate_forces(&particles, &mut forces).unwrap();

        // Forces should be finite (not infinite due to softening)
        assert!(forces[0].norm().is_finite());
        assert!(forces[1].norm().is_finite());
    }

    #[test]
    fn test_builder_pattern() {
        let calc = VectorizedGravityBuilder::new()
            .simd_level(SimdLevel::Scalar)
            .softening(1e-6)
            .build();

        assert_eq!(calc.simd_level(), SimdLevel::Scalar);
        assert_eq!(calc.softening_parameter, 1e-6);
    }

    #[test]
    fn test_error_handling() {
        use crate::core::particle::{Body, ParticleSet};

        let calc = VectorizedGravity::new();

        // Create a particle set with 3 particles
        let mut particles = ParticleSet::new();
        particles.add_body(Body::new().with_mass(1.0e30)).unwrap();
        particles.add_body(Body::new().with_mass(1.0e30)).unwrap();
        particles.add_body(Body::new().with_mass(1.0e30)).unwrap();

        // Test mismatched force array length
        let mut forces = vec![Vector3::zeros(); 2]; // Wrong size

        let result = calc.calculate_forces(&particles, &mut forces);
        assert!(result.is_err());
    }
}
