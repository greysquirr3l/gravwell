//! Error control algorithms for adaptive timestep integration
//!
//! This module provides algorithms for controlling integration errors
//! and automatically adjusting timesteps to maintain accuracy targets.

use crate::adaptive::stability::{ErrorEstimationMethod, ErrorEstimator};
use crate::error::GravwellError;
use crate::types::{Force, Mass, Position, Scalar, Velocity};
use std::collections::VecDeque;

/// Error control strategy for adaptive integration.
#[derive(Debug, Clone, Copy)]
pub enum ErrorControlStrategy {
    /// Proportional Integral (PI) controller
    PI { kp: Scalar, ki: Scalar },

    /// Proportional Integral Derivative (PID) controller
    PID { kp: Scalar, ki: Scalar, kd: Scalar },

    /// Elementary controller (simple proportional)
    Elementary { exponent: Scalar },

    /// Gustafsson controller
    Gustafsson { alpha: Scalar, beta: Scalar },

    /// Custom controller with user-defined logic
    Custom {
        adjustment_fn: fn(Scalar, Scalar, Scalar) -> Scalar,
    },
}

/// Error control results and recommendations.
#[derive(Debug, Clone)]
pub struct ErrorControlResult {
    /// Current error estimate
    pub current_error: Scalar,

    /// Target error tolerance
    pub target_tolerance: Scalar,

    /// Recommended timestep
    pub recommended_timestep: Scalar,

    /// Timestep change factor
    pub change_factor: Scalar,

    /// Error control status
    pub status: ErrorControlStatus,

    /// Additional diagnostic information
    pub diagnostics: ErrorDiagnostics,
}

/// Status of error control operation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorControlStatus {
    /// Error acceptable, continue
    Accept,

    /// Error too high, reject step and retry with smaller timestep
    Reject,

    /// Error acceptable but recommend timestep adjustment
    AcceptWithAdjustment,

    /// Error control disabled or not applicable
    Disabled,
}

/// Diagnostic information for error control.
#[derive(Debug, Clone)]
pub struct ErrorDiagnostics {
    /// Error components by type
    pub position_error: Scalar,
    pub velocity_error: Scalar,
    pub energy_error: Scalar,

    /// Control variables
    pub integral_term: Scalar,
    pub derivative_term: Scalar,
    pub proportional_term: Scalar,

    /// Step statistics
    pub accepted_steps: usize,
    pub rejected_steps: usize,
    pub step_efficiency: Scalar,
}

/// Adaptive error controller for timestep management.
#[derive(Debug)]
pub struct ErrorController {
    /// Error control strategy
    strategy: ErrorControlStrategy,

    /// Target error tolerance
    tolerance: Scalar,

    /// Minimum allowed timestep
    min_timestep: Scalar,

    /// Maximum allowed timestep
    max_timestep: Scalar,

    /// Safety factor for timestep changes
    safety_factor: Scalar,

    /// Maximum allowed timestep increase factor
    max_increase_factor: Scalar,

    /// Maximum allowed timestep decrease factor
    max_decrease_factor: Scalar,

    /// Error estimator
    error_estimator: ErrorEstimator,

    /// Error history for control algorithms
    error_history: VecDeque<Scalar>,

    /// Timestep history
    timestep_history: VecDeque<Scalar>,

    /// Control variable history (for integral/derivative terms)
    control_history: VecDeque<ControlState>,

    /// Step statistics
    accepted_steps: usize,
    rejected_steps: usize,

    /// Current timestep
    current_timestep: Scalar,

    /// Previous timestep for derivative control
    previous_timestep: Scalar,
}

/// Control state for PID controllers.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ControlState {
    error: Scalar,
    timestep: Scalar,
    time: Scalar,
    integral_sum: Scalar,
}

impl ErrorController {
    /// Create a new error controller.
    pub fn new(
        strategy: ErrorControlStrategy,
        tolerance: Scalar,
        min_timestep: Scalar,
        max_timestep: Scalar,
        initial_timestep: Scalar,
    ) -> Result<Self, GravwellError> {
        if tolerance <= 0.0 {
            return Err(GravwellError::Configuration(
                "Tolerance must be positive".to_string(),
            ));
        }
        if min_timestep <= 0.0 || min_timestep > max_timestep {
            return Err(GravwellError::InvalidTimestep {
                timestep: min_timestep,
            });
        }
        if initial_timestep <= 0.0 {
            return Err(GravwellError::InvalidTimestep {
                timestep: initial_timestep,
            });
        }

        let clamped_timestep = initial_timestep.clamp(min_timestep, max_timestep);

        Ok(Self {
            strategy,
            tolerance,
            min_timestep,
            max_timestep,
            safety_factor: 0.9,
            max_increase_factor: 1.5,
            max_decrease_factor: 0.3,
            error_estimator: ErrorEstimator::new(ErrorEstimationMethod::EnergyConservation, 50),
            error_history: VecDeque::with_capacity(50),
            timestep_history: VecDeque::with_capacity(50),
            control_history: VecDeque::with_capacity(50),
            accepted_steps: 0,
            rejected_steps: 0,
            current_timestep: clamped_timestep,
            previous_timestep: clamped_timestep,
        })
    }

    /// Create a PI controller for stable error control.
    pub fn pi_controller(
        tolerance: Scalar,
        min_timestep: Scalar,
        max_timestep: Scalar,
        initial_timestep: Scalar,
    ) -> Result<Self, GravwellError> {
        let strategy = ErrorControlStrategy::PI { kp: 0.7, ki: 0.4 };
        Self::new(
            strategy,
            tolerance,
            min_timestep,
            max_timestep,
            initial_timestep,
        )
    }

    /// Create a PID controller for advanced error control.
    pub fn pid_controller(
        tolerance: Scalar,
        min_timestep: Scalar,
        max_timestep: Scalar,
        initial_timestep: Scalar,
    ) -> Result<Self, GravwellError> {
        let strategy = ErrorControlStrategy::PID {
            kp: 0.7,
            ki: 0.4,
            kd: 0.01,
        };
        Self::new(
            strategy,
            tolerance,
            min_timestep,
            max_timestep,
            initial_timestep,
        )
    }

    /// Create an elementary controller for simple applications.
    pub fn elementary_controller(
        tolerance: Scalar,
        min_timestep: Scalar,
        max_timestep: Scalar,
        initial_timestep: Scalar,
    ) -> Result<Self, GravwellError> {
        let strategy = ErrorControlStrategy::Elementary { exponent: 0.2 };
        Self::new(
            strategy,
            tolerance,
            min_timestep,
            max_timestep,
            initial_timestep,
        )
    }

    /// Control timestep based on error estimation.
    pub fn control_timestep(
        &mut self,
        positions: &[Position],
        velocities: &[Velocity],
        _forces: &[Force],
        masses: &[Mass],
        time: Scalar,
    ) -> Result<ErrorControlResult, GravwellError> {
        // Estimate current error
        let current_error = self
            .error_estimator
            .estimate_error(positions, velocities, masses, time)?;

        // Calculate timestep adjustment
        let (new_timestep, status) = self.calculate_timestep_adjustment(current_error, time)?;

        // Update statistics
        match status {
            ErrorControlStatus::Accept | ErrorControlStatus::AcceptWithAdjustment => {
                self.accepted_steps += 1;
            }
            ErrorControlStatus::Reject => {
                self.rejected_steps += 1;
            }
            _ => {}
        }

        // Update histories
        self.update_histories(current_error, new_timestep, time);

        // Calculate change factor
        let change_factor = new_timestep / self.current_timestep;

        // Update current timestep if accepting the step
        if status != ErrorControlStatus::Reject {
            self.previous_timestep = self.current_timestep;
            self.current_timestep = new_timestep;
        }

        // Create diagnostics
        let diagnostics = self.create_diagnostics(current_error);

        Ok(ErrorControlResult {
            current_error,
            target_tolerance: self.tolerance,
            recommended_timestep: new_timestep,
            change_factor,
            status,
            diagnostics,
        })
    }

    /// Get current timestep.
    pub fn current_timestep(&self) -> Scalar {
        self.current_timestep
    }

    /// Set new error tolerance.
    pub fn set_tolerance(&mut self, tolerance: Scalar) -> Result<(), GravwellError> {
        if tolerance <= 0.0 {
            return Err(GravwellError::Configuration(
                "Tolerance must be positive".to_string(),
            ));
        }
        self.tolerance = tolerance;
        Ok(())
    }

    /// Set timestep bounds.
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

    /// Set safety factor for timestep changes.
    pub fn set_safety_factor(&mut self, safety_factor: Scalar) {
        self.safety_factor = safety_factor.clamp(0.1, 1.0);
    }

    /// Set maximum timestep change factors.
    pub fn set_change_factors(&mut self, max_increase: Scalar, max_decrease: Scalar) {
        self.max_increase_factor = max_increase.max(1.0);
        self.max_decrease_factor = max_decrease.clamp(0.1, 1.0);
    }

    /// Get step efficiency statistics.
    pub fn step_efficiency(&self) -> Scalar {
        let total_steps = self.accepted_steps + self.rejected_steps;
        if total_steps > 0 {
            self.accepted_steps as Scalar / total_steps as Scalar
        } else {
            1.0
        }
    }

    /// Reset controller state.
    pub fn reset(&mut self, initial_timestep: Scalar) -> Result<(), GravwellError> {
        if initial_timestep <= 0.0 {
            return Err(GravwellError::InvalidTimestep {
                timestep: initial_timestep,
            });
        }

        self.current_timestep = initial_timestep.clamp(self.min_timestep, self.max_timestep);
        self.previous_timestep = self.current_timestep;
        self.error_history.clear();
        self.timestep_history.clear();
        self.control_history.clear();
        self.accepted_steps = 0;
        self.rejected_steps = 0;

        Ok(())
    }

    /// Calculate timestep adjustment based on error and control strategy.
    fn calculate_timestep_adjustment(
        &mut self,
        current_error: Scalar,
        time: Scalar,
    ) -> Result<(Scalar, ErrorControlStatus), GravwellError> {
        if current_error <= 0.0 {
            // No error or negative error (which shouldn't happen)
            return Ok((
                self.current_timestep * self.max_increase_factor.min(1.2),
                ErrorControlStatus::Accept,
            ));
        }

        let error_ratio = current_error / self.tolerance;

        // Determine if step should be accepted
        let status = if error_ratio <= 1.0 {
            if error_ratio <= 0.5 {
                ErrorControlStatus::AcceptWithAdjustment // Can potentially increase timestep
            } else {
                ErrorControlStatus::Accept
            }
        } else {
            ErrorControlStatus::Reject // Error too large
        };

        // Calculate new timestep based on control strategy
        let timestep_factor = match self.strategy {
            ErrorControlStrategy::Elementary { exponent } => {
                self.safety_factor * (1.0 / error_ratio).powf(exponent)
            }
            ErrorControlStrategy::PI { kp, ki } => {
                self.calculate_pi_factor(current_error, time, kp, ki)?
            }
            ErrorControlStrategy::PID { kp, ki, kd } => {
                self.calculate_pid_factor(current_error, time, kp, ki, kd)?
            }
            ErrorControlStrategy::Gustafsson { alpha, beta } => {
                self.calculate_gustafsson_factor(current_error, alpha, beta)?
            }
            ErrorControlStrategy::Custom { adjustment_fn } => {
                adjustment_fn(current_error, self.tolerance, self.current_timestep)
            }
        };

        // Apply bounds and safety constraints
        let bounded_factor =
            timestep_factor.clamp(self.max_decrease_factor, self.max_increase_factor);
        let new_timestep =
            (self.current_timestep * bounded_factor).clamp(self.min_timestep, self.max_timestep);

        Ok((new_timestep, status))
    }

    /// Calculate PI controller timestep factor.
    fn calculate_pi_factor(
        &self,
        current_error: Scalar,
        _time: Scalar,
        kp: Scalar,
        ki: Scalar,
    ) -> Result<Scalar, GravwellError> {
        let error_ratio = current_error / self.tolerance;

        // Proportional term
        let proportional = kp * (1.0 / error_ratio).ln();

        // Integral term
        let integral = if self.control_history.len() >= 2 {
            let mut integral_sum = 0.0;
            let history_vec: Vec<_> = self.control_history.iter().collect();
            for window in history_vec.windows(2) {
                let dt = window[1].time - window[0].time;
                let avg_error = (window[0].error + window[1].error) / 2.0;
                integral_sum += avg_error * dt;
            }
            ki * integral_sum / self.tolerance
        } else {
            0.0
        };

        let total_adjustment = proportional + integral;
        Ok(self.safety_factor * total_adjustment.exp())
    }

    /// Calculate PID controller timestep factor.
    fn calculate_pid_factor(
        &self,
        current_error: Scalar,
        time: Scalar,
        kp: Scalar,
        ki: Scalar,
        kd: Scalar,
    ) -> Result<Scalar, GravwellError> {
        let pi_factor = self.calculate_pi_factor(current_error, time, kp, ki)?;

        // Derivative term
        let derivative = if self.control_history.len() >= 2 {
            let prev_state = &self.control_history[self.control_history.len() - 1];
            let dt = time - prev_state.time;
            if dt > 1e-15 {
                let error_change = current_error - prev_state.error;
                kd * error_change / (dt * self.tolerance)
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok(pi_factor * (derivative).exp())
    }

    /// Calculate Gustafsson controller timestep factor.
    fn calculate_gustafsson_factor(
        &self,
        current_error: Scalar,
        alpha: Scalar,
        beta: Scalar,
    ) -> Result<Scalar, GravwellError> {
        let error_ratio = current_error / self.tolerance;

        let basic_factor = self.safety_factor * (1.0 / error_ratio).powf(alpha);

        // Use previous error if available for predictive component
        if let Some(prev_error) = self.error_history.back() {
            let prev_ratio = prev_error / self.tolerance;
            let predictive_factor = (prev_ratio / error_ratio).powf(beta);
            Ok(basic_factor * predictive_factor)
        } else {
            Ok(basic_factor)
        }
    }

    /// Update error and timestep histories.
    fn update_histories(&mut self, error: Scalar, timestep: Scalar, time: Scalar) {
        // Update error history
        self.error_history.push_back(error);
        if self.error_history.len() > 50 {
            self.error_history.pop_front();
        }

        // Update timestep history
        self.timestep_history.push_back(timestep);
        if self.timestep_history.len() > 50 {
            self.timestep_history.pop_front();
        }

        // Update control state history
        let integral_sum = if let Some(prev_state) = self.control_history.back() {
            let dt = time - prev_state.time;
            prev_state.integral_sum + error * dt
        } else {
            0.0
        };

        self.control_history.push_back(ControlState {
            error,
            timestep,
            time,
            integral_sum,
        });
        if self.control_history.len() > 50 {
            self.control_history.pop_front();
        }
    }

    /// Create diagnostic information.
    fn create_diagnostics(&self, current_error: Scalar) -> ErrorDiagnostics {
        let stats = self.error_estimator.error_statistics();

        // Calculate control terms based on current strategy
        let (proportional, integral, derivative) = match self.strategy {
            ErrorControlStrategy::PI { kp, ki } | ErrorControlStrategy::PID { kp, ki, .. } => {
                let proportional = kp * (self.tolerance / current_error).ln();
                let integral = if let Some(state) = self.control_history.back() {
                    ki * state.integral_sum / self.tolerance
                } else {
                    0.0
                };
                let derivative = if matches!(self.strategy, ErrorControlStrategy::PID { .. })
                    && self.error_history.len() >= 2
                {
                    let prev_error = self.error_history[self.error_history.len() - 2];
                    let error_change = current_error - prev_error;
                    let dt = self.current_timestep; // Approximate
                    if let ErrorControlStrategy::PID { kd, .. } = self.strategy {
                        kd * error_change / (dt * self.tolerance)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                (proportional, integral, derivative)
            }
            _ => (0.0, 0.0, 0.0),
        };

        ErrorDiagnostics {
            position_error: stats.position_error_mean,
            velocity_error: stats.velocity_error_mean,
            energy_error: stats.energy_error_mean,
            integral_term: integral,
            derivative_term: derivative,
            proportional_term: proportional,
            accepted_steps: self.accepted_steps,
            rejected_steps: self.rejected_steps,
            step_efficiency: self.step_efficiency(),
        }
    }
}

impl Default for ErrorController {
    fn default() -> Self {
        Self::pi_controller(1e-9, 1e-12, 1e-3, 1e-6).expect("Default parameters should be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_controller_creation() {
        let controller = ErrorController::pi_controller(1e-9, 1e-12, 1e-3, 1e-6).unwrap();
        assert_eq!(controller.current_timestep(), 1e-6);
        assert_eq!(controller.tolerance, 1e-9);
    }

    #[test]
    fn test_invalid_parameters() {
        // Invalid tolerance
        let result = ErrorController::pi_controller(-1e-9, 1e-12, 1e-3, 1e-6);
        assert!(result.is_err());

        // Invalid timestep bounds
        let result = ErrorController::pi_controller(1e-9, 1e-3, 1e-12, 1e-6);
        assert!(result.is_err());
    }

    #[test]
    fn test_timestep_control() {
        let mut controller =
            ErrorController::elementary_controller(1e-6, 1e-12, 1e-3, 1e-9).unwrap();

        let positions = vec![Position::new(1.0, 0.0, 0.0)];
        let velocities = vec![Velocity::new(0.0, 0.1, 0.0)];
        let forces = vec![Force::new(0.1, 0.0, 0.0)];
        let masses = vec![1.0];

        // First control step
        let result = controller
            .control_timestep(&positions, &velocities, &forces, &masses, 0.0)
            .unwrap();
        assert!(result.recommended_timestep > 0.0);
        assert!(result.change_factor > 0.0);

        // Should complete without error
        match result.status {
            ErrorControlStatus::Accept
            | ErrorControlStatus::AcceptWithAdjustment
            | ErrorControlStatus::Disabled => {
                // All acceptable
            }
            ErrorControlStatus::Reject => {
                // Also acceptable for first step
            }
        }
    }

    #[test]
    fn test_different_strategies() {
        let strategies = vec![
            ErrorControlStrategy::Elementary { exponent: 0.2 },
            ErrorControlStrategy::PI { kp: 0.7, ki: 0.4 },
            ErrorControlStrategy::PID {
                kp: 0.7,
                ki: 0.4,
                kd: 0.01,
            },
            ErrorControlStrategy::Gustafsson {
                alpha: 0.7,
                beta: 0.4,
            },
        ];

        for strategy in strategies {
            let mut controller = ErrorController::new(strategy, 1e-6, 1e-12, 1e-3, 1e-9).unwrap();

            let positions = vec![Position::new(1.0, 0.0, 0.0)];
            let velocities = vec![Velocity::new(0.0, 0.1, 0.0)];
            let forces = vec![Force::new(0.1, 0.0, 0.0)];
            let masses = vec![1.0];

            // Should work with all strategies
            let result = controller
                .control_timestep(&positions, &velocities, &forces, &masses, 0.0)
                .unwrap();
            assert!(result.recommended_timestep > 0.0);
        }
    }

    #[test]
    fn test_step_efficiency() {
        let mut controller = ErrorController::pi_controller(1e-6, 1e-12, 1e-3, 1e-9).unwrap();

        // Initially should have 100% efficiency (no steps taken)
        assert_eq!(controller.step_efficiency(), 1.0);

        // Simulate some accepted steps
        controller.accepted_steps = 8;
        controller.rejected_steps = 2;

        assert_eq!(controller.step_efficiency(), 0.8);
    }

    #[test]
    fn test_controller_reset() {
        let mut controller = ErrorController::pi_controller(1e-6, 1e-12, 1e-3, 1e-9).unwrap();

        // Add some history
        controller.accepted_steps = 10;
        controller.rejected_steps = 2;
        controller.error_history.push_back(1e-5);

        // Reset should clear everything
        controller.reset(2e-9).unwrap();

        assert_eq!(controller.current_timestep(), 2e-9);
        assert_eq!(controller.accepted_steps, 0);
        assert_eq!(controller.rejected_steps, 0);
        assert!(controller.error_history.is_empty());
    }
}
