//! Stability detection and error estimation algorithms
//!
//! This module provides specialized algorithms for detecting numerical
//! instabilities and estimating integration errors for gravitational
//! N-body simulations.

use crate::error::GravwellError;
use crate::types::{Mass, Position, Scalar, Velocity};
use std::collections::VecDeque;

/// Stability detection results for gravitational systems.
#[derive(Debug, Clone)]
pub struct StabilityDetection {
    /// Overall stability score (0.0 = unstable, 1.0 = highly stable)
    pub stability_score: Scalar,

    /// Detected instability types
    pub instabilities: Vec<InstabilityType>,

    /// Lyapunov exponent estimate (positive indicates chaos)
    pub lyapunov_exponent: Option<Scalar>,

    /// Close encounter count
    pub close_encounters: usize,

    /// Recommended action
    pub recommended_action: StabilityAction,
}

/// Types of numerical instabilities in N-body systems.
#[derive(Debug, Clone, PartialEq)]
pub enum InstabilityType {
    /// Close gravitational encounter
    CloseEncounter {
        particle_indices: (usize, usize),
        separation: Scalar,
        relative_velocity: Scalar,
    },

    /// Exponential error growth detected
    ExponentialGrowth { growth_rate: Scalar },

    /// Energy conservation violation
    EnergyViolation { relative_error: Scalar },

    /// Momentum conservation violation
    MomentumViolation { relative_error: Scalar },

    /// Numerical overflow/underflow risk
    NumericalRange { affected_particles: Vec<usize> },

    /// High-frequency oscillations
    HighFrequencyNoise { frequency_estimate: Scalar },
}

/// Recommended actions for stability issues.
#[derive(Debug, Clone, PartialEq)]
pub enum StabilityAction {
    /// Continue with current settings
    Continue,

    /// Reduce timestep
    ReduceTimestep { factor: Scalar },

    /// Switch to more stable integrator
    SwitchIntegrator { recommended: String },

    /// Add numerical damping
    AddDamping { damping_factor: Scalar },

    /// Require manual intervention
    ManualIntervention { reason: String },
}

/// Advanced error estimation using multiple methods.
#[derive(Debug)]
pub struct ErrorEstimator {
    /// Error estimation method
    method: ErrorEstimationMethod,

    /// History length for trend analysis
    history_length: usize,

    /// Position error history
    position_errors: VecDeque<Scalar>,

    /// Velocity error history
    velocity_errors: VecDeque<Scalar>,

    /// Energy error history
    energy_errors: VecDeque<Scalar>,

    /// Previous state for comparison
    previous_state: Option<SystemState>,

    /// Reference solutions for comparison
    reference_solutions: Vec<ReferenceSolution>,
}

/// Error estimation methods.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorEstimationMethod {
    /// Richardson extrapolation
    Richardson,

    /// Embedded Runge-Kutta pairs
    EmbeddedRK,

    /// Energy conservation monitoring
    EnergyConservation,

    /// Multiple timestep comparison
    MultipleTimesteps,

    /// Analytical solution comparison (when available)
    AnalyticalComparison,
}

/// System state for error tracking.
#[derive(Debug, Clone)]
pub struct SystemState {
    pub positions: Vec<Position>,
    pub velocities: Vec<Velocity>,
    pub energy: Scalar,
    pub momentum: Position, // Using Position type for 3D momentum vector
    pub angular_momentum: Position,
    pub time: Scalar,
}

/// Reference solution for analytical comparison.
#[derive(Debug, Clone)]
pub struct ReferenceSolution {
    /// Solution type (e.g., "kepler_orbit", "circular_binary")
    pub solution_type: String,

    /// Applicable particle indices
    pub particle_indices: Vec<usize>,

    /// Solution function (time -> state)
    pub solution_fn: fn(Scalar, &[Scalar]) -> SystemState, // parameters: time, solution_params

    /// Solution parameters
    pub parameters: Vec<Scalar>,
}

/// Stability detector for gravitational N-body systems.
#[derive(Debug)]
pub struct StabilityDetector {
    /// Detection sensitivity (0.0 = low, 1.0 = high)
    #[allow(dead_code)]
    sensitivity: Scalar,

    /// Close encounter threshold (relative to system scale)
    close_encounter_threshold: Scalar,

    /// Energy conservation tolerance
    energy_tolerance: Scalar,

    /// Momentum conservation tolerance  
    momentum_tolerance: Scalar,

    /// History for trend analysis
    history_length: usize,

    /// System state history
    state_history: VecDeque<SystemState>,

    /// Instability event history
    instability_history: VecDeque<InstabilityType>,
}

impl ErrorEstimator {
    /// Create a new error estimator.
    pub fn new(method: ErrorEstimationMethod, history_length: usize) -> Self {
        Self {
            method,
            history_length,
            position_errors: VecDeque::with_capacity(history_length),
            velocity_errors: VecDeque::with_capacity(history_length),
            energy_errors: VecDeque::with_capacity(history_length),
            previous_state: None,
            reference_solutions: Vec::new(),
        }
    }

    /// Add a reference solution for analytical comparison.
    pub fn add_reference_solution(&mut self, solution: ReferenceSolution) {
        self.reference_solutions.push(solution);
    }

    /// Estimate current integration error.
    pub fn estimate_error(
        &mut self,
        positions: &[Position],
        velocities: &[Velocity],
        masses: &[Mass],
        time: Scalar,
    ) -> Result<Scalar, GravwellError> {
        let current_state = self.compute_system_state(positions, velocities, masses, time)?;

        let error = match self.method {
            ErrorEstimationMethod::Richardson => {
                self.richardson_extrapolation_error(&current_state)?
            }
            ErrorEstimationMethod::EmbeddedRK => self.embedded_rk_error(&current_state)?,
            ErrorEstimationMethod::EnergyConservation => {
                self.energy_conservation_error(&current_state)?
            }
            ErrorEstimationMethod::MultipleTimesteps => {
                self.multiple_timestep_error(&current_state)?
            }
            ErrorEstimationMethod::AnalyticalComparison => {
                self.analytical_comparison_error(&current_state)?
            }
        };

        self.update_error_history(error);
        self.previous_state = Some(current_state);

        Ok(error)
    }

    /// Get error trend over recent history.
    pub fn error_trend(&self) -> Option<Scalar> {
        if self.position_errors.len() < 2 {
            return None;
        }

        let recent = self.position_errors.back()?;
        let older = self.position_errors.front()?;

        if *older > 1e-15 {
            Some((recent - older) / older)
        } else {
            Some(*recent - *older)
        }
    }

    /// Get error statistics.
    pub fn error_statistics(&self) -> ErrorStatistics {
        let compute_stats = |errors: &VecDeque<Scalar>| -> (Scalar, Scalar, Scalar) {
            if errors.is_empty() {
                return (0.0, 0.0, 0.0);
            }

            let mean = errors.iter().sum::<Scalar>() / errors.len() as Scalar;
            let max = errors.iter().fold(0.0_f64, |a, &b| a.max(b));
            let variance =
                errors.iter().map(|&x| (x - mean).powi(2)).sum::<Scalar>() / errors.len() as Scalar;

            (mean, max, variance.sqrt())
        };

        let (pos_mean, pos_max, pos_std) = compute_stats(&self.position_errors);
        let (vel_mean, vel_max, vel_std) = compute_stats(&self.velocity_errors);
        let (energy_mean, energy_max, energy_std) = compute_stats(&self.energy_errors);

        ErrorStatistics {
            position_error_mean: pos_mean,
            position_error_max: pos_max,
            position_error_std: pos_std,
            velocity_error_mean: vel_mean,
            velocity_error_max: vel_max,
            velocity_error_std: vel_std,
            energy_error_mean: energy_mean,
            energy_error_max: energy_max,
            energy_error_std: energy_std,
        }
    }

    /// Compute current system state.
    fn compute_system_state(
        &self,
        positions: &[Position],
        velocities: &[Velocity],
        masses: &[Mass],
        time: Scalar,
    ) -> Result<SystemState, GravwellError> {
        if positions.len() != velocities.len() || positions.len() != masses.len() {
            return Err(GravwellError::Configuration(
                "Array length mismatch".to_string(),
            ));
        }

        // Calculate total energy
        let mut kinetic_energy = 0.0;
        let mut potential_energy = 0.0;

        for i in 0..positions.len() {
            kinetic_energy += 0.5 * masses[i] * velocities[i].norm_squared();

            for j in (i + 1)..positions.len() {
                let r = (positions[i] - positions[j]).norm();
                if r > 1e-15 {
                    potential_energy -= crate::utils::constants::G * masses[i] * masses[j] / r;
                }
            }
        }

        let total_energy = kinetic_energy + potential_energy;

        // Calculate total momentum
        let mut momentum = Position::zeros();
        for i in 0..positions.len() {
            momentum += masses[i] * velocities[i];
        }

        // Calculate center of mass
        let mut com = Position::zeros();
        let mut total_mass = 0.0;
        for i in 0..positions.len() {
            com += masses[i] * positions[i];
            total_mass += masses[i];
        }
        if total_mass > 1e-15 {
            com /= total_mass;
        }

        // Calculate angular momentum about center of mass
        let mut angular_momentum = Position::zeros();
        for i in 0..positions.len() {
            let r_rel = positions[i] - com;
            let v_rel = velocities[i];
            angular_momentum += masses[i] * r_rel.cross(&v_rel);
        }

        Ok(SystemState {
            positions: positions.to_vec(),
            velocities: velocities.to_vec(),
            energy: total_energy,
            momentum,
            angular_momentum,
            time,
        })
    }

    /// Richardson extrapolation error estimation.
    fn richardson_extrapolation_error(
        &self,
        _current_state: &SystemState,
    ) -> Result<Scalar, GravwellError> {
        // TODO: Implement Richardson extrapolation
        // This requires running integration with different timesteps
        Ok(0.0)
    }

    /// Embedded Runge-Kutta error estimation.
    fn embedded_rk_error(&self, _current_state: &SystemState) -> Result<Scalar, GravwellError> {
        // TODO: Implement embedded RK error estimation
        // This requires integration method cooperation
        Ok(0.0)
    }

    /// Energy conservation error estimation.
    fn energy_conservation_error(
        &self,
        current_state: &SystemState,
    ) -> Result<Scalar, GravwellError> {
        if let Some(ref previous) = self.previous_state {
            let energy_change = (current_state.energy - previous.energy).abs();
            let energy_scale = previous.energy.abs().max(1e-15);
            Ok(energy_change / energy_scale)
        } else {
            Ok(0.0)
        }
    }

    /// Multiple timestep comparison error.
    fn multiple_timestep_error(
        &self,
        _current_state: &SystemState,
    ) -> Result<Scalar, GravwellError> {
        // TODO: Implement multiple timestep comparison
        // This requires running with different timesteps
        Ok(0.0)
    }

    /// Analytical solution comparison error.
    fn analytical_comparison_error(
        &self,
        current_state: &SystemState,
    ) -> Result<Scalar, GravwellError> {
        if self.reference_solutions.is_empty() {
            return Ok(0.0);
        }

        let mut max_error: Scalar = 0.0;

        for ref_solution in &self.reference_solutions {
            let analytical_state =
                (ref_solution.solution_fn)(current_state.time, &ref_solution.parameters);

            // Compare positions for specified particles
            for &particle_idx in &ref_solution.particle_indices {
                if particle_idx < current_state.positions.len() {
                    let position_error = (current_state.positions[particle_idx]
                        - analytical_state.positions[particle_idx])
                        .norm();
                    let position_scale = analytical_state.positions[particle_idx].norm().max(1e-15);
                    let relative_error = position_error / position_scale;
                    max_error = max_error.max(relative_error);
                }
            }
        }

        Ok(max_error)
    }

    /// Update error history.
    fn update_error_history(&mut self, error: Scalar) {
        self.position_errors.push_back(error);
        if self.position_errors.len() > self.history_length {
            self.position_errors.pop_front();
        }

        // For now, use same error for all types
        // TODO: Implement separate error calculations
        self.velocity_errors.push_back(error);
        if self.velocity_errors.len() > self.history_length {
            self.velocity_errors.pop_front();
        }

        self.energy_errors.push_back(error);
        if self.energy_errors.len() > self.history_length {
            self.energy_errors.pop_front();
        }
    }
}

impl StabilityDetector {
    /// Create a new stability detector.
    pub fn new(sensitivity: Scalar) -> Self {
        Self {
            sensitivity: sensitivity.clamp(0.0, 1.0),
            close_encounter_threshold: 1e-6,
            energy_tolerance: 1e-9,
            momentum_tolerance: 1e-12,
            history_length: 100,
            state_history: VecDeque::with_capacity(100),
            instability_history: VecDeque::with_capacity(100),
        }
    }

    /// Set detection thresholds.
    pub fn set_thresholds(
        &mut self,
        close_encounter: Scalar,
        energy_tolerance: Scalar,
        momentum_tolerance: Scalar,
    ) {
        self.close_encounter_threshold = close_encounter;
        self.energy_tolerance = energy_tolerance;
        self.momentum_tolerance = momentum_tolerance;
    }

    /// Detect stability issues in the current system state.
    pub fn detect_instabilities(
        &mut self,
        positions: &[Position],
        velocities: &[Velocity],
        masses: &[Mass],
        time: Scalar,
    ) -> Result<StabilityDetection, GravwellError> {
        let current_state = self.compute_system_state(positions, velocities, masses, time)?;
        let mut instabilities = Vec::new();

        // Check for close encounters
        let close_encounters = self.detect_close_encounters(&current_state, &mut instabilities);

        // Check energy conservation
        self.check_energy_conservation(&current_state, &mut instabilities);

        // Check momentum conservation
        self.check_momentum_conservation(&current_state, &mut instabilities);

        // Check for numerical range issues
        self.check_numerical_range(&current_state, &mut instabilities);

        // Estimate Lyapunov exponent
        let lyapunov_exponent = self.estimate_lyapunov_exponent(&current_state);

        // Calculate overall stability score
        let stability_score = self.calculate_stability_score(&instabilities, lyapunov_exponent);

        // Determine recommended action
        let recommended_action = self.determine_recommended_action(&instabilities, stability_score);

        // Update history
        self.update_history(current_state, instabilities.clone());

        Ok(StabilityDetection {
            stability_score,
            instabilities,
            lyapunov_exponent,
            close_encounters,
            recommended_action,
        })
    }

    /// Detect close gravitational encounters.
    fn detect_close_encounters(
        &self,
        state: &SystemState,
        instabilities: &mut Vec<InstabilityType>,
    ) -> usize {
        let mut encounter_count = 0;

        for i in 0..state.positions.len() {
            for j in (i + 1)..state.positions.len() {
                let separation = (state.positions[i] - state.positions[j]).norm();
                let relative_velocity = (state.velocities[i] - state.velocities[j]).norm();

                // Determine characteristic scale
                let position_scale = state.positions[i]
                    .norm()
                    .max(state.positions[j].norm())
                    .max(1.0);
                let threshold = self.close_encounter_threshold * position_scale;

                if separation < threshold {
                    encounter_count += 1;
                    instabilities.push(InstabilityType::CloseEncounter {
                        particle_indices: (i, j),
                        separation,
                        relative_velocity,
                    });
                }
            }
        }

        encounter_count
    }

    /// Check energy conservation.
    fn check_energy_conservation(
        &self,
        current_state: &SystemState,
        instabilities: &mut Vec<InstabilityType>,
    ) {
        if let Some(initial_state) = self.state_history.front() {
            let energy_change = (current_state.energy - initial_state.energy).abs();
            let energy_scale = initial_state.energy.abs().max(1e-15);
            let relative_error = energy_change / energy_scale;

            if relative_error > self.energy_tolerance {
                instabilities.push(InstabilityType::EnergyViolation { relative_error });
            }
        }
    }

    /// Check momentum conservation.
    fn check_momentum_conservation(
        &self,
        current_state: &SystemState,
        instabilities: &mut Vec<InstabilityType>,
    ) {
        if let Some(initial_state) = self.state_history.front() {
            let momentum_change = (current_state.momentum - initial_state.momentum).norm();
            let momentum_scale = initial_state.momentum.norm().max(1e-15);
            let relative_error = momentum_change / momentum_scale;

            if relative_error > self.momentum_tolerance {
                instabilities.push(InstabilityType::MomentumViolation { relative_error });
            }
        }
    }

    /// Check for numerical range issues.
    fn check_numerical_range(&self, state: &SystemState, instabilities: &mut Vec<InstabilityType>) {
        let mut affected_particles = Vec::new();

        for (i, position) in state.positions.iter().enumerate() {
            let pos_magnitude = position.norm();
            let vel_magnitude = state.velocities[i].norm();

            // Check for values approaching floating-point limits
            if pos_magnitude > 1e100 || pos_magnitude < 1e-100 && pos_magnitude > 0.0 {
                affected_particles.push(i);
            }
            if vel_magnitude > 1e100 || vel_magnitude < 1e-100 && vel_magnitude > 0.0 {
                affected_particles.push(i);
            }
        }

        if !affected_particles.is_empty() {
            affected_particles.sort_unstable();
            affected_particles.dedup();
            instabilities.push(InstabilityType::NumericalRange { affected_particles });
        }
    }

    /// Estimate Lyapunov exponent (simplified approach).
    fn estimate_lyapunov_exponent(&self, _current_state: &SystemState) -> Option<Scalar> {
        if self.state_history.len() < 10 {
            return None;
        }

        // TODO: Implement proper Lyapunov exponent calculation
        // This requires tracking deviation growth rates
        Some(0.0)
    }

    /// Calculate overall stability score.
    fn calculate_stability_score(
        &self,
        instabilities: &[InstabilityType],
        lyapunov_exponent: Option<Scalar>,
    ) -> Scalar {
        let mut score = 1.0;

        // Penalize based on instability types
        for instability in instabilities {
            match instability {
                InstabilityType::CloseEncounter { separation, .. } => {
                    let penalty = (1e-6 / separation.max(1e-12)).min(0.5);
                    score -= penalty;
                }
                InstabilityType::EnergyViolation { relative_error } => {
                    let penalty = (relative_error / self.energy_tolerance * 0.3).min(0.3);
                    score -= penalty;
                }
                InstabilityType::MomentumViolation { relative_error } => {
                    let penalty = (relative_error / self.momentum_tolerance * 0.2).min(0.2);
                    score -= penalty;
                }
                InstabilityType::NumericalRange { .. } => {
                    score -= 0.4; // Heavy penalty for numerical issues
                }
                InstabilityType::ExponentialGrowth { .. } => {
                    score -= 0.5; // Very heavy penalty
                }
                InstabilityType::HighFrequencyNoise { .. } => {
                    score -= 0.1; // Light penalty
                }
            }
        }

        // Factor in Lyapunov exponent if available
        if let Some(lyapunov) = lyapunov_exponent {
            if lyapunov > 0.1 {
                score -= 0.3; // Chaotic behavior penalty
            }
        }

        score.max(0.0)
    }

    /// Determine recommended action based on stability analysis.
    fn determine_recommended_action(
        &self,
        instabilities: &[InstabilityType],
        stability_score: Scalar,
    ) -> StabilityAction {
        if stability_score > 0.8 {
            return StabilityAction::Continue;
        }

        if stability_score < 0.3 {
            return StabilityAction::ManualIntervention {
                reason: "Multiple severe instabilities detected".to_string(),
            };
        }

        // Check for specific instability types
        for instability in instabilities {
            match instability {
                InstabilityType::CloseEncounter { .. } => {
                    return StabilityAction::ReduceTimestep { factor: 0.1 };
                }
                InstabilityType::EnergyViolation { relative_error } => {
                    if *relative_error > self.energy_tolerance * 100.0 {
                        return StabilityAction::SwitchIntegrator {
                            recommended: "VelocityVerlet or Leapfrog".to_string(),
                        };
                    }
                    return StabilityAction::ReduceTimestep { factor: 0.5 };
                }
                InstabilityType::NumericalRange { .. } => {
                    return StabilityAction::ManualIntervention {
                        reason: "Numerical overflow/underflow detected".to_string(),
                    };
                }
                _ => {}
            }
        }

        // Default action for moderate instability
        StabilityAction::ReduceTimestep { factor: 0.7 }
    }

    /// Compute system state (similar to ErrorEstimator).
    fn compute_system_state(
        &self,
        positions: &[Position],
        velocities: &[Velocity],
        masses: &[Mass],
        time: Scalar,
    ) -> Result<SystemState, GravwellError> {
        if positions.len() != velocities.len() || positions.len() != masses.len() {
            return Err(GravwellError::Configuration(
                "Array length mismatch".to_string(),
            ));
        }

        // Calculate energy
        let mut kinetic_energy = 0.0;
        let mut potential_energy = 0.0;

        for i in 0..positions.len() {
            kinetic_energy += 0.5 * masses[i] * velocities[i].norm_squared();

            for j in (i + 1)..positions.len() {
                let r = (positions[i] - positions[j]).norm();
                if r > 1e-15 {
                    potential_energy -= crate::utils::constants::G * masses[i] * masses[j] / r;
                }
            }
        }

        // Calculate momentum
        let mut momentum = Position::zeros();
        for i in 0..positions.len() {
            momentum += masses[i] * velocities[i];
        }

        // Calculate angular momentum
        let mut angular_momentum = Position::zeros();
        for i in 0..positions.len() {
            angular_momentum += masses[i] * positions[i].cross(&velocities[i]);
        }

        Ok(SystemState {
            positions: positions.to_vec(),
            velocities: velocities.to_vec(),
            energy: kinetic_energy + potential_energy,
            momentum,
            angular_momentum,
            time,
        })
    }

    /// Update state and instability history.
    fn update_history(&mut self, state: SystemState, instabilities: Vec<InstabilityType>) {
        self.state_history.push_back(state);
        if self.state_history.len() > self.history_length {
            self.state_history.pop_front();
        }

        for instability in instabilities {
            self.instability_history.push_back(instability);
        }
        if self.instability_history.len() > self.history_length {
            self.instability_history.pop_front();
        }
    }
}

/// Error statistics summary.
#[derive(Debug, Clone)]
pub struct ErrorStatistics {
    pub position_error_mean: Scalar,
    pub position_error_max: Scalar,
    pub position_error_std: Scalar,
    pub velocity_error_mean: Scalar,
    pub velocity_error_max: Scalar,
    pub velocity_error_std: Scalar,
    pub energy_error_mean: Scalar,
    pub energy_error_max: Scalar,
    pub energy_error_std: Scalar,
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_error_estimator_creation() {
        let estimator = ErrorEstimator::new(ErrorEstimationMethod::EnergyConservation, 50);
        assert_eq!(estimator.method, ErrorEstimationMethod::EnergyConservation);
        assert_eq!(estimator.history_length, 50);
    }

    #[test]
    fn test_stability_detector() {
        let mut detector = StabilityDetector::new(0.8);

        let positions = vec![Position::new(1.0, 0.0, 0.0), Position::new(-1.0, 0.0, 0.0)];
        let velocities = vec![Velocity::new(0.0, 0.1, 0.0), Velocity::new(0.0, -0.1, 0.0)];
        let masses = vec![1.0, 1.0];

        let detection = detector
            .detect_instabilities(&positions, &velocities, &masses, 0.0)
            .unwrap();

        // Should complete without error
        assert!(detection.stability_score >= 0.0);
        assert!(detection.stability_score <= 1.0);
    }

    #[test]
    fn test_close_encounter_detection() {
        let mut detector = StabilityDetector::new(1.0);
        detector.set_thresholds(1e-3, 1e-9, 1e-12);

        // Set up close encounter
        let positions = vec![
            Position::new(1.0, 0.0, 0.0),
            Position::new(1.0001, 0.0, 0.0), // Very close
        ];
        let velocities = vec![Velocity::new(0.0, 0.1, 0.0), Velocity::new(0.0, -0.1, 0.0)];
        let masses = vec![1.0, 1.0];

        let detection = detector
            .detect_instabilities(&positions, &velocities, &masses, 0.0)
            .unwrap();

        // Should detect close encounter
        assert!(detection
            .instabilities
            .iter()
            .any(|inst| matches!(inst, InstabilityType::CloseEncounter { .. })));
        assert!(detection.close_encounters > 0);
    }

    #[test]
    fn test_error_estimation() {
        let mut estimator = ErrorEstimator::new(ErrorEstimationMethod::EnergyConservation, 10);

        let positions = vec![Position::new(1.0, 0.0, 0.0)];
        let velocities = vec![Velocity::new(0.0, 0.1, 0.0)];
        let masses = vec![1.0];

        // First estimation should work
        let error1 = estimator
            .estimate_error(&positions, &velocities, &masses, 0.0)
            .unwrap();
        assert_eq!(error1, 0.0); // No previous state to compare

        // Slightly modify energy by changing velocity
        let velocities2 = vec![Velocity::new(0.0, 0.11, 0.0)];
        let error2 = estimator
            .estimate_error(&positions, &velocities2, &masses, 0.01)
            .unwrap();
        assert!(error2 > 0.0); // Should detect energy change
    }
}
