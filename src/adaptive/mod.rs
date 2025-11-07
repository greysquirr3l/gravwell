//! Adaptive Timestep Control System
//!
//! This module provides advanced timestep control mechanisms for maintaining
//! simulation stability and accuracy through automatic timestep adjustment
//! based on multiple error metrics and stability criteria.

#![allow(missing_docs)]

pub mod error_control;
pub mod stability;

use crate::error::GravwellError;
use crate::types::{Force, Mass, Position, Scalar, Velocity};
use std::collections::VecDeque;

/// Error metrics for timestep adaptation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorMetric {
    /// Position-based error estimation
    Position,
    /// Velocity-based error estimation  
    Velocity,
    /// Energy conservation error
    Energy,
    /// Total acceleration magnitude
    Acceleration,
    /// Combined multi-metric approach
    Combined,
}

/// Timestep adaptation strategy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdaptationStrategy {
    /// Conservative adaptation (slow changes)
    Conservative,
    /// Balanced adaptation (moderate changes)
    Balanced,
    /// Aggressive adaptation (fast changes)
    Aggressive,
    /// Custom adaptation with user-defined parameters
    Custom {
        increase_factor: Scalar,
        decrease_factor: Scalar,
        stability_threshold: Scalar,
    },
}

/// Timestep stability analysis results.
#[derive(Debug, Clone)]
pub struct StabilityAnalysis {
    /// Current estimated error
    pub current_error: Scalar,
    /// Recommended timestep
    pub recommended_timestep: Scalar,
    /// Stability status
    pub is_stable: bool,
    /// Error trend over recent steps
    pub error_trend: ErrorTrend,
    /// Critical stability warnings
    pub warnings: Vec<StabilityWarning>,
}

/// Error trend analysis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorTrend {
    /// Error is decreasing (good)
    Decreasing,
    /// Error is stable
    Stable,
    /// Error is increasing slowly
    IncreasingSlowly,
    /// Error is increasing rapidly (concerning)
    IncreasingRapidly,
}

/// Stability warnings for timestep control.
#[derive(Debug, Clone, PartialEq)]
pub enum StabilityWarning {
    /// Timestep approaching minimum limit
    ApproachingMinimumTimestep,
    /// Error increasing beyond tolerance
    ErrorTrendConcerning,
    /// Possible numerical instability detected
    NumericalInstability,
    /// Close encounter requiring smaller timestep
    CloseEncounter {
        particle_indices: (usize, usize),
        distance: Scalar,
    },
}

/// Advanced timestep controller with multiple error metrics.
#[derive(Debug)]
pub struct AdaptiveTimestepController {
    /// Current timestep
    current_timestep: Scalar,

    /// Minimum allowed timestep
    min_timestep: Scalar,

    /// Maximum allowed timestep  
    max_timestep: Scalar,

    /// Target error tolerance
    error_tolerance: Scalar,

    /// Error metric to use for adaptation
    error_metric: ErrorMetric,

    /// Adaptation strategy
    adaptation_strategy: AdaptationStrategy,

    /// Error history for trend analysis
    error_history: VecDeque<Scalar>,

    /// Timestep history for stability analysis
    timestep_history: VecDeque<Scalar>,

    /// Maximum history length to keep in memory
    max_history_length: usize,

    /// Previous step data for error estimation
    previous_positions: Vec<Position>,
    previous_velocities: Vec<Velocity>,
    previous_energy: Option<Scalar>,

    /// Step counter
    step_count: u64,

    /// Stability analysis cache
    last_stability_analysis: Option<StabilityAnalysis>,
}

impl AdaptiveTimestepController {
    /// Create a new adaptive timestep controller.
    ///
    /// # Arguments
    /// * `initial_timestep` - Starting timestep value
    /// * `min_timestep` - Minimum allowed timestep
    /// * `max_timestep` - Maximum allowed timestep
    /// * `error_tolerance` - Target error tolerance for adaptation
    /// * `error_metric` - Error metric to use for timestep adaptation
    /// * `adaptation_strategy` - Strategy for timestep changes
    pub fn new(
        initial_timestep: Scalar,
        min_timestep: Scalar,
        max_timestep: Scalar,
        error_tolerance: Scalar,
        error_metric: ErrorMetric,
        adaptation_strategy: AdaptationStrategy,
    ) -> Result<Self, GravwellError> {
        if initial_timestep <= 0.0 {
            return Err(GravwellError::InvalidTimestep {
                timestep: initial_timestep,
            });
        }
        if min_timestep <= 0.0 || min_timestep > max_timestep {
            return Err(GravwellError::InvalidTimestep {
                timestep: min_timestep,
            });
        }
        if error_tolerance <= 0.0 {
            return Err(GravwellError::Configuration(
                "Error tolerance must be positive".to_string(),
            ));
        }

        Ok(Self {
            current_timestep: initial_timestep.clamp(min_timestep, max_timestep),
            min_timestep,
            max_timestep,
            error_tolerance,
            error_metric,
            adaptation_strategy,
            error_history: VecDeque::with_capacity(100),
            timestep_history: VecDeque::with_capacity(100),
            max_history_length: 100,
            previous_positions: Vec::new(),
            previous_velocities: Vec::new(),
            previous_energy: None,
            step_count: 0,
            last_stability_analysis: None,
        })
    }

    /// Create a conservative timestep controller for stable simulations.
    pub fn conservative(
        initial_timestep: Scalar,
        error_tolerance: Scalar,
    ) -> Result<Self, GravwellError> {
        Self::new(
            initial_timestep,
            initial_timestep * 1e-6, // Very small minimum
            initial_timestep * 10.0, // Conservative maximum
            error_tolerance,
            ErrorMetric::Combined,
            AdaptationStrategy::Conservative,
        )
    }

    /// Create an aggressive timestep controller for performance.
    pub fn aggressive(
        initial_timestep: Scalar,
        error_tolerance: Scalar,
    ) -> Result<Self, GravwellError> {
        Self::new(
            initial_timestep,
            initial_timestep * 1e-8,  // Smaller minimum
            initial_timestep * 100.0, // Larger maximum
            error_tolerance,
            ErrorMetric::Position,
            AdaptationStrategy::Aggressive,
        )
    }

    /// Get the current timestep.
    pub fn current_timestep(&self) -> Scalar {
        self.current_timestep
    }

    /// Get timestep bounds.
    pub fn timestep_bounds(&self) -> (Scalar, Scalar) {
        (self.min_timestep, self.max_timestep)
    }

    /// Set new timestep bounds.
    pub fn set_timestep_bounds(
        &mut self,
        min_timestep: Scalar,
        max_timestep: Scalar,
    ) -> Result<(), GravwellError> {
        if min_timestep <= 0.0 || min_timestep > max_timestep {
            return Err(GravwellError::InvalidTimestep {
                timestep: min_timestep,
            });
        }
        self.min_timestep = min_timestep;
        self.max_timestep = max_timestep;
        self.current_timestep = self.current_timestep.clamp(min_timestep, max_timestep);
        Ok(())
    }

    /// Update error tolerance.
    pub fn set_error_tolerance(&mut self, error_tolerance: Scalar) -> Result<(), GravwellError> {
        if error_tolerance <= 0.0 {
            return Err(GravwellError::Configuration(
                "Error tolerance must be positive".to_string(),
            ));
        }
        self.error_tolerance = error_tolerance;
        Ok(())
    }

    /// Change error metric.
    pub fn set_error_metric(&mut self, error_metric: ErrorMetric) {
        self.error_metric = error_metric;
    }

    /// Change adaptation strategy.
    pub fn set_adaptation_strategy(&mut self, adaptation_strategy: AdaptationStrategy) {
        self.adaptation_strategy = adaptation_strategy;
    }

    /// Analyze current stability and recommend timestep adjustment.
    pub fn analyze_stability(
        &mut self,
        positions: &[Position],
        velocities: &[Velocity],
        forces: &[Force],
        masses: &[Mass],
        current_energy: Option<Scalar>,
    ) -> StabilityAnalysis {
        let current_error =
            self.estimate_error(positions, velocities, forces, masses, current_energy);

        // Update error history
        self.error_history.push_back(current_error);
        if self.error_history.len() > self.max_history_length {
            self.error_history.pop_front();
        }

        // Analyze error trend
        let error_trend = self.analyze_error_trend();

        // Calculate recommended timestep
        let recommended_timestep = self.calculate_recommended_timestep(current_error);

        // Check stability
        let is_stable =
            current_error <= self.error_tolerance && error_trend != ErrorTrend::IncreasingRapidly;

        // Generate warnings
        let warnings = self.generate_stability_warnings(current_error, positions, &error_trend);

        let analysis = StabilityAnalysis {
            current_error,
            recommended_timestep,
            is_stable,
            error_trend,
            warnings,
        };

        self.last_stability_analysis = Some(analysis.clone());
        analysis
    }

    /// Update the timestep based on stability analysis.
    pub fn update_timestep(
        &mut self,
        positions: &[Position],
        velocities: &[Velocity],
        forces: &[Force],
        masses: &[Mass],
        current_energy: Option<Scalar>,
    ) -> Scalar {
        let analysis =
            self.analyze_stability(positions, velocities, forces, masses, current_energy);

        let new_timestep =
            if analysis.is_stable && analysis.current_error < self.error_tolerance * 0.5 {
                // Error is well below tolerance, can potentially increase timestep
                self.calculate_increased_timestep(analysis.current_error)
            } else if !analysis.is_stable || analysis.current_error > self.error_tolerance {
                // Error too high or unstable, decrease timestep
                analysis.recommended_timestep
            } else {
                // Maintain current timestep
                self.current_timestep
            };

        // Apply adaptation strategy constraints
        let adapted_timestep = self.apply_adaptation_constraints(new_timestep);

        // Update timestep history
        self.timestep_history.push_back(adapted_timestep);
        if self.timestep_history.len() > self.max_history_length {
            self.timestep_history.pop_front();
        }

        self.current_timestep = adapted_timestep;
        self.step_count += 1;

        // Store current state for next iteration
        self.previous_positions = positions.to_vec();
        self.previous_velocities = velocities.to_vec();
        self.previous_energy = current_energy;

        self.current_timestep
    }

    /// Force a specific timestep (bypassing adaptation).
    pub fn force_timestep(&mut self, timestep: Scalar) -> Result<(), GravwellError> {
        if timestep <= 0.0 {
            return Err(GravwellError::InvalidTimestep { timestep });
        }
        self.current_timestep = timestep.clamp(self.min_timestep, self.max_timestep);
        Ok(())
    }

    /// Get the last stability analysis results.
    pub fn last_stability_analysis(&self) -> Option<&StabilityAnalysis> {
        self.last_stability_analysis.as_ref()
    }

    /// Get error history for analysis.
    pub fn error_history(&self) -> &VecDeque<Scalar> {
        &self.error_history
    }

    /// Get timestep history for analysis.
    pub fn timestep_history(&self) -> &VecDeque<Scalar> {
        &self.timestep_history
    }

    /// Get step count.
    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Reset the controller state.
    pub fn reset(&mut self, initial_timestep: Scalar) -> Result<(), GravwellError> {
        if initial_timestep <= 0.0 {
            return Err(GravwellError::InvalidTimestep {
                timestep: initial_timestep,
            });
        }

        self.current_timestep = initial_timestep.clamp(self.min_timestep, self.max_timestep);
        self.error_history.clear();
        self.timestep_history.clear();
        self.previous_positions.clear();
        self.previous_velocities.clear();
        self.previous_energy = None;
        self.step_count = 0;
        self.last_stability_analysis = None;

        Ok(())
    }

    /// Estimate error based on the selected error metric.
    fn estimate_error(
        &self,
        positions: &[Position],
        velocities: &[Velocity],
        forces: &[Force],
        masses: &[Mass],
        current_energy: Option<Scalar>,
    ) -> Scalar {
        match self.error_metric {
            ErrorMetric::Position => self.estimate_position_error(positions),
            ErrorMetric::Velocity => self.estimate_velocity_error(velocities),
            ErrorMetric::Energy => self.estimate_energy_error(current_energy),
            ErrorMetric::Acceleration => self.estimate_acceleration_error(forces, masses),
            ErrorMetric::Combined => {
                self.estimate_combined_error(positions, velocities, forces, masses, current_energy)
            }
        }
    }

    /// Estimate position-based error.
    fn estimate_position_error(&self, positions: &[Position]) -> Scalar {
        if self.previous_positions.is_empty() || positions.len() != self.previous_positions.len() {
            return 0.0; // No comparison possible
        }

        let mut max_relative_change: Scalar = 0.0;
        for (current, previous) in positions.iter().zip(self.previous_positions.iter()) {
            let change = (current - previous).norm();
            let magnitude = current.norm();
            if magnitude > 1e-12 {
                let relative_change = change / magnitude;
                max_relative_change = max_relative_change.max(relative_change);
            }
        }

        max_relative_change
    }

    /// Estimate velocity-based error.
    fn estimate_velocity_error(&self, velocities: &[Velocity]) -> Scalar {
        if self.previous_velocities.is_empty() || velocities.len() != self.previous_velocities.len()
        {
            return 0.0;
        }

        let mut max_relative_change: Scalar = 0.0;
        for (current, previous) in velocities.iter().zip(self.previous_velocities.iter()) {
            let change = (current - previous).norm();
            let magnitude = current.norm();
            if magnitude > 1e-12 {
                let relative_change = change / magnitude;
                max_relative_change = max_relative_change.max(relative_change);
            }
        }

        max_relative_change
    }

    /// Estimate energy conservation error.
    fn estimate_energy_error(&self, current_energy: Option<Scalar>) -> Scalar {
        if let (Some(current), Some(previous)) = (current_energy, self.previous_energy) {
            if previous.abs() > 1e-12 {
                ((current - previous) / previous).abs()
            } else {
                (current - previous).abs()
            }
        } else {
            0.0
        }
    }

    /// Estimate acceleration-based error.
    fn estimate_acceleration_error(&self, forces: &[Force], masses: &[Mass]) -> Scalar {
        let mut max_acceleration: Scalar = 0.0;
        for (force, mass) in forces.iter().zip(masses.iter()) {
            let acceleration = force.norm() / mass;
            max_acceleration = max_acceleration.max(acceleration);
        }

        // Scale by timestep to get position error estimate
        max_acceleration * self.current_timestep * self.current_timestep
    }

    /// Estimate combined error using multiple metrics.
    fn estimate_combined_error(
        &self,
        positions: &[Position],
        velocities: &[Velocity],
        forces: &[Force],
        masses: &[Mass],
        current_energy: Option<Scalar>,
    ) -> Scalar {
        let position_error = self.estimate_position_error(positions);
        let velocity_error = self.estimate_velocity_error(velocities);
        let energy_error = self.estimate_energy_error(current_energy);
        let acceleration_error = self.estimate_acceleration_error(forces, masses);

        // Weighted combination
        let weights = [0.4, 0.3, 0.2, 0.1]; // Position, velocity, energy, acceleration
        let errors = [
            position_error,
            velocity_error,
            energy_error,
            acceleration_error,
        ];

        weights.iter().zip(errors.iter()).map(|(w, e)| w * e).sum()
    }

    /// Analyze error trend over recent history.
    fn analyze_error_trend(&self) -> ErrorTrend {
        if self.error_history.len() < 3 {
            return ErrorTrend::Stable;
        }

        let recent_errors: Vec<_> = self.error_history.iter().rev().take(5).collect();
        let recent_avg =
            recent_errors.iter().map(|&&e| e).sum::<Scalar>() / recent_errors.len() as Scalar;

        let older_errors: Vec<_> = self.error_history.iter().rev().skip(5).take(5).collect();
        if older_errors.is_empty() {
            return ErrorTrend::Stable;
        }
        let older_avg =
            older_errors.iter().map(|&&e| e).sum::<Scalar>() / older_errors.len() as Scalar;

        let relative_change = if older_avg > 1e-12 {
            (recent_avg - older_avg) / older_avg
        } else {
            recent_avg - older_avg
        };

        match relative_change {
            x if x < -0.1 => ErrorTrend::Decreasing,
            x if x > 0.5 => ErrorTrend::IncreasingRapidly,
            x if x > 0.1 => ErrorTrend::IncreasingSlowly,
            _ => ErrorTrend::Stable,
        }
    }

    /// Calculate recommended timestep based on error.
    fn calculate_recommended_timestep(&self, current_error: Scalar) -> Scalar {
        if current_error <= 1e-12 {
            return self.max_timestep; // Error negligible, use maximum
        }

        // PI controller approach: proportional to error ratio
        let error_ratio = self.error_tolerance / current_error;
        let safety_factor = 0.8; // Conservative safety margin

        let suggested_timestep = self.current_timestep * error_ratio.powf(0.2) * safety_factor;

        suggested_timestep.clamp(self.min_timestep, self.max_timestep)
    }

    /// Calculate increased timestep when error is low.
    fn calculate_increased_timestep(&self, current_error: Scalar) -> Scalar {
        let error_ratio = current_error / self.error_tolerance;
        let increase_factor = match self.adaptation_strategy {
            AdaptationStrategy::Conservative => 1.05,
            AdaptationStrategy::Balanced => 1.1,
            AdaptationStrategy::Aggressive => 1.2,
            AdaptationStrategy::Custom {
                increase_factor, ..
            } => increase_factor,
        };

        // Only increase if error is significantly below tolerance
        if error_ratio < 0.3 {
            (self.current_timestep * increase_factor).min(self.max_timestep)
        } else {
            self.current_timestep
        }
    }

    /// Apply adaptation strategy constraints to timestep changes.
    fn apply_adaptation_constraints(&self, new_timestep: Scalar) -> Scalar {
        let change_ratio = new_timestep / self.current_timestep;

        let (max_increase, max_decrease) = match self.adaptation_strategy {
            AdaptationStrategy::Conservative => (1.1, 0.8),
            AdaptationStrategy::Balanced => (1.5, 0.5),
            AdaptationStrategy::Aggressive => (2.0, 0.3),
            AdaptationStrategy::Custom {
                increase_factor,
                decrease_factor,
                ..
            } => (increase_factor, decrease_factor),
        };

        let constrained_ratio = change_ratio.clamp(max_decrease, max_increase);
        (self.current_timestep * constrained_ratio).clamp(self.min_timestep, self.max_timestep)
    }

    /// Generate stability warnings based on current state.
    fn generate_stability_warnings(
        &self,
        current_error: Scalar,
        positions: &[Position],
        error_trend: &ErrorTrend,
    ) -> Vec<StabilityWarning> {
        let mut warnings = Vec::new();

        // Check if approaching minimum timestep
        if self.current_timestep <= self.min_timestep * 1.1 {
            warnings.push(StabilityWarning::ApproachingMinimumTimestep);
        }

        // Check error trend
        if *error_trend == ErrorTrend::IncreasingRapidly {
            warnings.push(StabilityWarning::ErrorTrendConcerning);
        }

        // Check for numerical instability
        if current_error > self.error_tolerance * 10.0 {
            warnings.push(StabilityWarning::NumericalInstability);
        }

        // Check for close encounters
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let distance = (positions[i] - positions[j]).norm();
                // Warning for very close approaches (relative to particle scales)
                let position_scale = positions[i].norm().max(positions[j].norm());
                if distance < position_scale * 1e-6 {
                    warnings.push(StabilityWarning::CloseEncounter {
                        particle_indices: (i, j),
                        distance,
                    });
                }
            }
        }

        warnings
    }
}

impl Default for AdaptiveTimestepController {
    fn default() -> Self {
        Self::conservative(0.01, 1e-9).expect("Default parameters should be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_controller_creation() {
        let controller = AdaptiveTimestepController::conservative(0.01, 1e-9).unwrap();
        assert_eq!(controller.current_timestep(), 0.01);
        assert_eq!(controller.timestep_bounds(), (0.01 * 1e-6, 0.01 * 10.0));
    }

    #[test]
    fn test_timestep_bounds_validation() {
        let result = AdaptiveTimestepController::new(
            0.01,
            0.1,
            0.05,
            1e-9,
            ErrorMetric::Position,
            AdaptationStrategy::Conservative,
        );
        assert!(result.is_err()); // min > max should fail
    }

    #[test]
    fn test_error_estimation() {
        let mut controller = AdaptiveTimestepController::conservative(0.01, 1e-9).unwrap();

        let positions = vec![Position::new(1.0, 0.0, 0.0), Position::new(-1.0, 0.0, 0.0)];
        let velocities = vec![Velocity::new(0.0, 0.1, 0.0), Velocity::new(0.0, -0.1, 0.0)];
        let forces = vec![Force::new(0.1, 0.0, 0.0), Force::new(-0.1, 0.0, 0.0)];
        let masses = vec![1.0, 1.0];

        // First call - no previous data
        let analysis =
            controller.analyze_stability(&positions, &velocities, &forces, &masses, Some(-0.5));
        assert_eq!(analysis.current_error, 0.0); // No previous data for comparison

        // Modify positions slightly
        let new_positions = vec![
            Position::new(1.01, 0.0, 0.0),
            Position::new(-1.01, 0.0, 0.0),
        ];

        // Second call - should detect position change
        let analysis =
            controller.analyze_stability(&new_positions, &velocities, &forces, &masses, Some(-0.5));
        assert!(analysis.current_error > 0.0);
    }

    #[test]
    fn test_timestep_adaptation() {
        let mut controller = AdaptiveTimestepController::aggressive(0.01, 1e-6).unwrap();

        let positions = vec![Position::new(1.0, 0.0, 0.0)];
        let velocities = vec![Velocity::new(0.0, 0.1, 0.0)];
        let forces = vec![Force::new(0.001, 0.0, 0.0)]; // Very small force
        let masses = vec![1.0];

        // First update with small forces should allow timestep increase
        let initial_timestep = controller.current_timestep();
        controller.update_timestep(&positions, &velocities, &forces, &masses, Some(-0.5));

        // Add some history
        for _ in 0..5 {
            controller.update_timestep(&positions, &velocities, &forces, &masses, Some(-0.5));
        }

        // Timestep should have potential to increase (though may be limited by adaptation strategy)
        assert!(controller.current_timestep() >= initial_timestep);
    }

    #[test]
    fn test_stability_warnings() {
        let mut controller = AdaptiveTimestepController::new(
            1e-10,
            1e-12,
            1e-8,
            1e-9,
            ErrorMetric::Position,
            AdaptationStrategy::Conservative,
        )
        .unwrap();

        // Create close encounter scenario
        let positions = vec![
            Position::new(1.0, 0.0, 0.0),
            Position::new(1.0000001, 0.0, 0.0), // Very close
        ];
        let velocities = vec![Velocity::new(0.0, 0.1, 0.0), Velocity::new(0.0, -0.1, 0.0)];
        let forces = vec![
            Force::new(1000.0, 0.0, 0.0), // Large force
            Force::new(-1000.0, 0.0, 0.0),
        ];
        let masses = vec![1.0, 1.0];

        let analysis =
            controller.analyze_stability(&positions, &velocities, &forces, &masses, Some(-0.5));

        // Should generate warnings for close encounter and potentially other issues
        assert!(!analysis.warnings.is_empty());
    }

    #[test]
    fn test_different_error_metrics() {
        let mut pos_controller = AdaptiveTimestepController::new(
            0.01,
            1e-8,
            1.0,
            1e-6,
            ErrorMetric::Position,
            AdaptationStrategy::Balanced,
        )
        .unwrap();

        let mut vel_controller = AdaptiveTimestepController::new(
            0.01,
            1e-8,
            1.0,
            1e-6,
            ErrorMetric::Velocity,
            AdaptationStrategy::Balanced,
        )
        .unwrap();

        let positions = vec![Position::new(1.0, 0.0, 0.0)];
        let velocities = vec![Velocity::new(0.0, 0.1, 0.0)];
        let forces = vec![Force::new(0.1, 0.0, 0.0)];
        let masses = vec![1.0];

        // Both controllers should work but may give different results
        let pos_analysis =
            pos_controller.analyze_stability(&positions, &velocities, &forces, &masses, Some(-0.5));
        let vel_analysis =
            vel_controller.analyze_stability(&positions, &velocities, &forces, &masses, Some(-0.5));

        // Both should complete without error
        assert!(pos_analysis.recommended_timestep > 0.0);
        assert!(vel_analysis.recommended_timestep > 0.0);
    }

    #[test]
    fn test_controller_reset() {
        let mut controller = AdaptiveTimestepController::conservative(0.01, 1e-9).unwrap();

        // Add some history
        controller.error_history.push_back(1e-6);
        controller.timestep_history.push_back(0.005);
        controller.step_count = 100;

        // Reset should clear everything
        controller.reset(0.02).unwrap();

        assert_eq!(controller.current_timestep(), 0.02);
        assert_eq!(controller.step_count(), 0);
        assert!(controller.error_history().is_empty());
        assert!(controller.timestep_history().is_empty());
    }
}
