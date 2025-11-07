//! Dynamic object activation and deactivation system for massive particle simulations.
//!
//! The activation system provides intelligent particle management based on distance,
//! importance, and performance constraints. It enables simulations with 100,000+
//! particles by dynamically activating only the most relevant subset for physics
//! processing while maintaining smooth transitions and visual coherence.
//!
//! # Core Concepts
//!
//! - **Activation State**: Particles can be Active, Transitioning, or Inactive
//! - **Importance Metrics**: Distance, velocity, mass, interaction history
//! - **Hysteresis**: Prevents rapid activation/deactivation oscillations
//! - **Budget Management**: Limits active particles to maintain target frame rate
//!
//! # Performance Benefits
//!
//! - **Memory Efficiency**: Inactive particles use minimal memory
//! - **CPU Optimization**: Only active particles undergo physics calculations
//! - **Scalability**: Enables simulations with orders of magnitude more particles
//! - **Quality Control**: Most important particles always receive full processing
//!
//! # Usage Example
//!
//! ```rust
//! use gravwell::spatial::{ActivationManager, ActivationConfig, ImportanceMetric};
//! use gravwell::types::{Vector3, BodyHandle};
//!
//! // Configure activation system
//! let config = ActivationConfig::new()
//!     .with_activation_distance(500.0)
//!     .with_deactivation_distance(600.0)
//!     .with_max_active_particles(1000)
//!     .with_importance_metric(ImportanceMetric::Distance);
//!
//! let mut manager = ActivationManager::with_config(config);
//!
//! // Update particle positions
//! manager.update_positions(&positions, &handles);
//!
//! // Get active particles for physics processing
//! let camera_pos = Vector3::new(0.0, 0.0, 0.0);
//! let active_particles = manager.update_activation(camera_pos);
//! ```

use crate::types::{Scalar, Vector3};
use crate::BodyHandle;
use std::cmp::Ordering;
use std::collections::HashMap;

/// Configuration for the activation management system
#[derive(Debug, Clone)]
pub struct ActivationConfig {
    /// Distance at which particles become eligible for activation
    pub activation_distance: Scalar,

    /// Distance at which particles become eligible for deactivation
    /// Should be larger than activation_distance to provide hysteresis
    pub deactivation_distance: Scalar,

    /// Maximum number of particles that can be active simultaneously
    pub max_active_particles: usize,

    /// Minimum number of particles to keep active for quality
    pub min_active_particles: usize,

    /// Metric used for importance-based selection
    pub importance_metric: ImportanceMetric,

    /// Time delay before activation/deactivation to prevent flickering
    pub transition_delay_frames: u32,

    /// Whether to use importance-based selection when over budget
    pub use_importance_selection: bool,

    /// Quality vs performance bias (0.0 = performance, 1.0 = quality)
    pub quality_bias: f32,
}

impl Default for ActivationConfig {
    fn default() -> Self {
        Self {
            activation_distance: 1000.0,
            deactivation_distance: 1200.0,
            max_active_particles: 1000,
            min_active_particles: 100,
            importance_metric: ImportanceMetric::Distance,
            transition_delay_frames: 5,
            use_importance_selection: true,
            quality_bias: 0.5,
        }
    }
}

impl ActivationConfig {
    /// Create a new activation configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the activation distance threshold
    pub fn with_activation_distance(mut self, distance: Scalar) -> Self {
        self.activation_distance = distance;
        self
    }

    /// Set the deactivation distance threshold (should be > activation_distance)
    pub fn with_deactivation_distance(mut self, distance: Scalar) -> Self {
        self.deactivation_distance = distance;
        self
    }

    /// Set the maximum number of active particles
    pub fn with_max_active_particles(mut self, max: usize) -> Self {
        self.max_active_particles = max;
        self
    }

    /// Set the minimum number of active particles to maintain
    pub fn with_min_active_particles(mut self, min: usize) -> Self {
        self.min_active_particles = min;
        self
    }

    /// Set the importance metric for particle selection
    pub fn with_importance_metric(mut self, metric: ImportanceMetric) -> Self {
        self.importance_metric = metric;
        self
    }

    /// Set the transition delay in frames
    pub fn with_transition_delay(mut self, frames: u32) -> Self {
        self.transition_delay_frames = frames;
        self
    }

    /// Enable or disable importance-based selection
    pub fn with_importance_selection(mut self, enabled: bool) -> Self {
        self.use_importance_selection = enabled;
        self
    }

    /// Set quality vs performance bias (0.0 = performance, 1.0 = quality)
    pub fn with_quality_bias(mut self, bias: f32) -> Self {
        self.quality_bias = bias.clamp(0.0, 1.0);
        self
    }

    /// Validate configuration and fix common issues
    pub fn validate(&mut self) {
        // Ensure deactivation distance is larger than activation distance
        if self.deactivation_distance <= self.activation_distance {
            self.deactivation_distance = self.activation_distance * 1.2;
        }

        // Ensure min_active is less than max_active
        if self.min_active_particles >= self.max_active_particles {
            self.min_active_particles = (self.max_active_particles as f32 * 0.1) as usize;
        }

        // Clamp quality bias
        self.quality_bias = self.quality_bias.clamp(0.0, 1.0);
    }
}

/// Metrics for determining particle importance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportanceMetric {
    /// Distance from observer (closer = more important)
    Distance,

    /// Particle mass (larger = more important)
    Mass,

    /// Particle velocity magnitude (faster = more important)
    Velocity,

    /// Combined metric using distance and mass
    DistanceMass,

    /// Combined metric using all factors
    Comprehensive,

    /// Random selection (for testing/debugging)
    Random,
}

/// Current activation state of a particle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationState {
    /// Particle is fully active and participating in physics
    Active,

    /// Particle is transitioning between active and inactive states
    Transitioning {
        /// Target state after transition completes
        target_state: ActivationTarget,
        /// Frames remaining in transition
        remaining_frames: u32,
    },

    /// Particle is inactive (position may still be updated for culling)
    Inactive,
}

/// Target state for transitioning particles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationTarget {
    /// Particle should be active
    Active,
    /// Particle should be inactive  
    Inactive,
}

/// Information about a particle for activation management
#[derive(Debug, Clone)]
pub struct ParticleInfo {
    /// Current position
    pub position: Vector3,

    /// Current velocity (for importance calculation)
    pub velocity: Vector3,

    /// Particle mass (for importance calculation)
    pub mass: Scalar,

    /// Current activation state
    pub state: ActivationState,

    /// Distance to observer (cached for efficiency)
    pub distance_to_observer: Scalar,

    /// Calculated importance score
    pub importance_score: f64,

    /// Frame when this particle was last updated
    pub last_update_frame: u64,
}

impl ParticleInfo {
    /// Create new particle info with default values
    pub fn new(position: Vector3) -> Self {
        Self {
            position,
            velocity: Vector3::zeros(),
            mass: 1.0,
            state: ActivationState::Inactive,
            distance_to_observer: f64::INFINITY,
            importance_score: 0.0,
            last_update_frame: 0,
        }
    }

    /// Update position and velocity
    pub fn update_motion(&mut self, position: Vector3, velocity: Vector3) {
        self.position = position;
        self.velocity = velocity;
    }

    /// Update distance to observer
    pub fn update_distance(&mut self, observer_position: Vector3) {
        self.distance_to_observer = (self.position - observer_position).norm();
    }

    /// Calculate importance score based on metric
    pub fn calculate_importance(&mut self, metric: ImportanceMetric, observer_position: Vector3) {
        self.update_distance(observer_position);

        self.importance_score = match metric {
            ImportanceMetric::Distance => {
                // Closer particles are more important (inverse distance)
                if self.distance_to_observer > 0.0 {
                    1.0 / self.distance_to_observer
                } else {
                    f64::MAX
                }
            }

            ImportanceMetric::Mass => {
                // More massive particles are more important
                self.mass as f64
            }

            ImportanceMetric::Velocity => {
                // Faster moving particles are more important
                self.velocity.norm() as f64
            }

            ImportanceMetric::DistanceMass => {
                // Combined distance and mass metric
                let distance_score = if self.distance_to_observer > 0.0 {
                    1.0 / self.distance_to_observer
                } else {
                    f64::MAX
                };
                distance_score * (self.mass as f64)
            }

            ImportanceMetric::Comprehensive => {
                // Comprehensive metric using all available factors
                let distance_factor = if self.distance_to_observer > 0.0 {
                    1.0 / (1.0 + self.distance_to_observer)
                } else {
                    1.0
                };
                let mass_factor = (self.mass as f64).sqrt();
                let velocity_factor = 1.0 + (self.velocity.norm() as f64 * 0.1);

                distance_factor * mass_factor * velocity_factor
            }

            ImportanceMetric::Random => {
                // Random importance for testing (using deterministic fallback if rand unavailable)
                #[cfg(test)]
                {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};

                    let mut hasher = DefaultHasher::new();
                    self.position.x.to_bits().hash(&mut hasher);
                    self.position.y.to_bits().hash(&mut hasher);
                    self.position.z.to_bits().hash(&mut hasher);

                    (hasher.finish() as f64) / (u64::MAX as f64)
                }

                #[cfg(not(test))]
                {
                    // Use a simple hash-based pseudo-random value in production
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};

                    let mut hasher = DefaultHasher::new();
                    self.position.x.to_bits().hash(&mut hasher);
                    self.position.y.to_bits().hash(&mut hasher);
                    self.position.z.to_bits().hash(&mut hasher);

                    (hasher.finish() as f64) / (u64::MAX as f64)
                }
            }
        };
    }

    /// Check if particle should be activated based on distance
    pub fn should_activate(&self, config: &ActivationConfig) -> bool {
        match self.state {
            ActivationState::Inactive => self.distance_to_observer <= config.activation_distance,
            ActivationState::Transitioning { target_state, .. } => {
                target_state == ActivationTarget::Active
            }
            ActivationState::Active => true,
        }
    }

    /// Check if particle should be deactivated based on distance
    pub fn should_deactivate(&self, config: &ActivationConfig) -> bool {
        match self.state {
            ActivationState::Active => self.distance_to_observer > config.deactivation_distance,
            ActivationState::Transitioning { target_state, .. } => {
                target_state == ActivationTarget::Inactive
            }
            ActivationState::Inactive => true,
        }
    }

    /// Update activation state and handle transitions
    pub fn update_state(&mut self, frame: u64) {
        if let ActivationState::Transitioning {
            target_state,
            remaining_frames,
        } = self.state
        {
            if remaining_frames <= 1 {
                // Transition complete
                self.state = match target_state {
                    ActivationTarget::Active => ActivationState::Active,
                    ActivationTarget::Inactive => ActivationState::Inactive,
                };
            } else {
                // Continue transition
                self.state = ActivationState::Transitioning {
                    target_state,
                    remaining_frames: remaining_frames - 1,
                };
            }
        }

        self.last_update_frame = frame;
    }

    /// Start transition to new state
    pub fn start_transition(&mut self, target: ActivationTarget, delay_frames: u32) {
        self.state = ActivationState::Transitioning {
            target_state: target,
            remaining_frames: delay_frames,
        };
    }

    /// Check if particle is currently active (including transitioning to active)
    pub fn is_active(&self) -> bool {
        match self.state {
            ActivationState::Active => true,
            ActivationState::Transitioning { target_state, .. } => {
                target_state == ActivationTarget::Active
            }
            ActivationState::Inactive => false,
        }
    }
}

/// Manages dynamic activation and deactivation of particles for performance optimization
pub struct ActivationManager {
    /// Configuration for activation behavior
    config: ActivationConfig,

    /// Information about all tracked particles
    particles: HashMap<BodyHandle, ParticleInfo>,

    /// Current frame number for transition timing
    current_frame: u64,

    /// Observer position for distance calculations
    observer_position: Vector3,

    /// Statistics for performance monitoring
    stats: ActivationStatistics,
}

/// Statistics for activation system performance analysis
#[derive(Debug, Clone, Default)]
pub struct ActivationStatistics {
    /// Total number of tracked particles
    pub total_particles: usize,

    /// Number of currently active particles
    pub active_particles: usize,

    /// Number of particles in transition
    pub transitioning_particles: usize,

    /// Number of inactive particles
    pub inactive_particles: usize,

    /// Activations this frame
    pub activations_this_frame: u32,

    /// Deactivations this frame
    pub deactivations_this_frame: u32,

    /// Total activations over all time
    pub total_activations: u64,

    /// Total deactivations over all time
    pub total_deactivations: u64,

    /// Average importance score of active particles
    pub avg_active_importance: f64,

    /// Performance budget utilization (0.0 = underutilized, 1.0 = at limit)
    pub budget_utilization: f32,
}

impl ActivationManager {
    /// Create a new activation manager with default configuration
    pub fn new() -> Self {
        Self::with_config(ActivationConfig::default())
    }

    /// Create activation manager with custom configuration
    pub fn with_config(mut config: ActivationConfig) -> Self {
        config.validate();

        Self {
            config,
            particles: HashMap::new(),
            current_frame: 0,
            observer_position: Vector3::zeros(),
            stats: ActivationStatistics::default(),
        }
    }

    /// Set activation distance threshold
    pub fn with_activation_distance(mut self, distance: Scalar) -> Self {
        self.config.activation_distance = distance;
        self
    }

    /// Update particle positions and velocities
    ///
    /// This should be called every frame to maintain spatial coherency.
    /// Particles not in the provided arrays will be marked as removed.
    ///
    /// # Arguments
    ///
    /// * `positions` - Array of current particle positions
    /// * `handles` - Array of particle handles corresponding to positions
    /// * `velocities` - Optional array of particle velocities
    /// * `masses` - Optional array of particle masses
    pub fn update_positions(
        &mut self,
        positions: &[Vector3],
        handles: &[BodyHandle],
        velocities: Option<&[Vector3]>,
        masses: Option<&[Scalar]>,
    ) {
        self.current_frame += 1;

        // Update existing particles and add new ones
        for (i, &handle) in handles.iter().enumerate() {
            if i < positions.len() {
                let position = positions[i];

                let particle = self
                    .particles
                    .entry(handle)
                    .or_insert_with(|| ParticleInfo::new(position));

                // Update motion
                let velocity = velocities
                    .and_then(|v| v.get(i))
                    .copied()
                    .unwrap_or(Vector3::zeros());

                particle.update_motion(position, velocity);

                // Update mass if provided
                if let Some(masses) = masses {
                    if let Some(&mass) = masses.get(i) {
                        particle.mass = mass;
                    }
                }

                // Update state transitions
                particle.update_state(self.current_frame);
            }
        }

        // Remove particles that are no longer present
        let current_handles: std::collections::HashSet<BodyHandle> =
            handles.iter().copied().collect();

        self.particles
            .retain(|&handle, _| current_handles.contains(&handle));

        self.update_statistics();
    }

    /// Simplified position update for common use case
    pub fn update_positions_simple(&mut self, positions: &[Vector3], handles: &[BodyHandle]) {
        self.update_positions(positions, handles, None, None);
    }

    /// Update activation states based on observer position and return active particles
    ///
    /// This is the main method for managing particle activation. It updates
    /// distance calculations, importance scores, and activation states to
    /// maintain optimal performance while preserving quality.
    ///
    /// # Arguments
    ///
    /// * `observer_position` - Current observer/camera position
    ///
    /// # Returns
    ///
    /// Vector of handles for particles that should be active this frame
    pub fn update_activation(&mut self, observer_position: Vector3) -> Vec<BodyHandle> {
        self.observer_position = observer_position;

        // Reset frame statistics
        self.stats.activations_this_frame = 0;
        self.stats.deactivations_this_frame = 0;

        // Update distances and importance scores
        for particle in self.particles.values_mut() {
            particle.calculate_importance(self.config.importance_metric, observer_position);
        }

        // Determine which particles should be active
        let mut candidates = self.get_activation_candidates();

        // Apply budget constraints
        candidates = self.apply_budget_constraints(candidates);

        // Update activation states
        self.update_activation_states(&candidates);

        // Collect currently active particles
        let active_particles: Vec<BodyHandle> = self
            .particles
            .iter()
            .filter_map(
                |(&handle, info)| {
                    if info.is_active() {
                        Some(handle)
                    } else {
                        None
                    }
                },
            )
            .collect();

        self.update_statistics();
        active_particles
    }

    /// Get particles that are candidates for activation based on distance
    fn get_activation_candidates(&self) -> Vec<BodyHandle> {
        self.particles
            .iter()
            .filter_map(|(&handle, info)| {
                if info.should_activate(&self.config) {
                    Some(handle)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Apply budget constraints using importance-based selection
    fn apply_budget_constraints(&self, mut candidates: Vec<BodyHandle>) -> Vec<BodyHandle> {
        if !self.config.use_importance_selection
            || candidates.len() <= self.config.max_active_particles
        {
            return candidates;
        }

        // Sort candidates by importance (descending)
        candidates.sort_by(|&a, &b| {
            let info_a = &self.particles[&a];
            let info_b = &self.particles[&b];
            info_b
                .importance_score
                .partial_cmp(&info_a.importance_score)
                .unwrap_or(Ordering::Equal)
        });

        // Take only the most important particles up to budget
        candidates.truncate(self.config.max_active_particles);
        candidates
    }

    /// Update activation states based on candidate list
    fn update_activation_states(&mut self, candidates: &[BodyHandle]) {
        let candidate_set: std::collections::HashSet<BodyHandle> =
            candidates.iter().copied().collect();

        for (&handle, particle) in self.particles.iter_mut() {
            let should_be_active = candidate_set.contains(&handle);

            match particle.state {
                ActivationState::Active => {
                    if !should_be_active && particle.should_deactivate(&self.config) {
                        particle.start_transition(
                            ActivationTarget::Inactive,
                            self.config.transition_delay_frames,
                        );
                        self.stats.deactivations_this_frame += 1;
                        self.stats.total_deactivations += 1;
                    }
                }

                ActivationState::Inactive => {
                    if should_be_active && particle.should_activate(&self.config) {
                        particle.start_transition(
                            ActivationTarget::Active,
                            self.config.transition_delay_frames,
                        );
                        self.stats.activations_this_frame += 1;
                        self.stats.total_activations += 1;
                    }
                }

                ActivationState::Transitioning { .. } => {
                    // Allow transitions to complete naturally
                    // Could add logic here to interrupt transitions if needed
                }
            }
        }
    }

    /// Select particles by importance from a given list
    ///
    /// This method can be used by external systems (like spatial cullers)
    /// to get the most important subset of particles.
    ///
    /// # Arguments
    ///
    /// * `particle_handles` - List of particle handles to select from
    /// * `max_count` - Maximum number of particles to select
    /// * `observer_position` - Observer position for importance calculation
    ///
    /// # Returns
    ///
    /// Vector of the most important particle handles (up to max_count)
    pub fn select_by_importance(
        &mut self,
        particle_handles: Vec<BodyHandle>,
        max_count: usize,
        observer_position: Vector3,
    ) -> Vec<BodyHandle> {
        if particle_handles.len() <= max_count {
            return particle_handles;
        }

        // Update importance scores for selection
        for &handle in &particle_handles {
            if let Some(particle) = self.particles.get_mut(&handle) {
                particle.calculate_importance(self.config.importance_metric, observer_position);
            }
        }

        // Sort by importance and take top entries
        let mut sorted_handles = particle_handles;
        sorted_handles.sort_by(|&a, &b| {
            let importance_a = self
                .particles
                .get(&a)
                .map(|p| p.importance_score)
                .unwrap_or(0.0);
            let importance_b = self
                .particles
                .get(&b)
                .map(|p| p.importance_score)
                .unwrap_or(0.0);

            importance_b
                .partial_cmp(&importance_a)
                .unwrap_or(Ordering::Equal)
        });

        sorted_handles.truncate(max_count);
        sorted_handles
    }

    /// Get the current activation state of a particle
    pub fn get_activation_state(&self, handle: BodyHandle) -> Option<ActivationState> {
        self.particles.get(&handle).map(|info| info.state)
    }

    /// Get the importance score of a particle
    pub fn get_importance_score(&self, handle: BodyHandle) -> Option<f64> {
        self.particles
            .get(&handle)
            .map(|info| info.importance_score)
    }

    /// Get particles within a distance range
    pub fn get_particles_in_range(
        &self,
        min_distance: Scalar,
        max_distance: Scalar,
    ) -> Vec<BodyHandle> {
        self.particles
            .iter()
            .filter_map(|(&handle, info)| {
                if info.distance_to_observer >= min_distance
                    && info.distance_to_observer <= max_distance
                {
                    Some(handle)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Force activation of specific particles (override distance checks)
    pub fn force_activate_particles(&mut self, handles: &[BodyHandle]) {
        for &handle in handles {
            if let Some(particle) = self.particles.get_mut(&handle) {
                if !particle.is_active() {
                    particle.start_transition(ActivationTarget::Active, 0); // Immediate activation
                    self.stats.activations_this_frame += 1;
                    self.stats.total_activations += 1;
                }
            }
        }
    }

    /// Force deactivation of specific particles
    pub fn force_deactivate_particles(&mut self, handles: &[BodyHandle]) {
        for &handle in handles {
            if let Some(particle) = self.particles.get_mut(&handle) {
                if particle.is_active() {
                    particle.start_transition(ActivationTarget::Inactive, 0); // Immediate deactivation
                    self.stats.deactivations_this_frame += 1;
                    self.stats.total_deactivations += 1;
                }
            }
        }
    }

    /// Update internal statistics
    fn update_statistics(&mut self) {
        self.stats.total_particles = self.particles.len();
        self.stats.active_particles = 0;
        self.stats.transitioning_particles = 0;
        self.stats.inactive_particles = 0;

        let mut total_active_importance = 0.0;

        for particle in self.particles.values() {
            match particle.state {
                ActivationState::Active => {
                    self.stats.active_particles += 1;
                    total_active_importance += particle.importance_score;
                }
                ActivationState::Transitioning { .. } => {
                    self.stats.transitioning_particles += 1;
                }
                ActivationState::Inactive => {
                    self.stats.inactive_particles += 1;
                }
            }
        }

        self.stats.avg_active_importance = if self.stats.active_particles > 0 {
            total_active_importance / self.stats.active_particles as f64
        } else {
            0.0
        };

        self.stats.budget_utilization = if self.config.max_active_particles > 0 {
            self.stats.active_particles as f32 / self.config.max_active_particles as f32
        } else {
            0.0
        };
    }

    /// Get current activation statistics
    pub fn get_statistics(&self) -> &ActivationStatistics {
        &self.stats
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ActivationConfig {
        &self.config
    }

    /// Update configuration (validates before applying)
    pub fn update_config(&mut self, mut config: ActivationConfig) {
        config.validate();
        self.config = config;
    }

    /// Get the number of tracked particles
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Get the number of currently active particles
    pub fn active_count(&self) -> usize {
        self.stats.active_particles
    }

    /// Check if the system is over budget
    pub fn is_over_budget(&self) -> bool {
        self.stats.active_particles > self.config.max_active_particles
    }

    /// Get budget utilization percentage
    pub fn budget_utilization(&self) -> f32 {
        self.stats.budget_utilization * 100.0
    }
}

impl Default for ActivationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builder::BodyHandle, types::Vector3};

    #[test]
    fn test_activation_config_creation() {
        let config = ActivationConfig::new()
            .with_activation_distance(500.0)
            .with_deactivation_distance(600.0)
            .with_max_active_particles(100);

        assert_eq!(config.activation_distance, 500.0);
        assert_eq!(config.deactivation_distance, 600.0);
        assert_eq!(config.max_active_particles, 100);
    }

    #[test]
    fn test_config_validation() {
        let mut config = ActivationConfig::new()
            .with_activation_distance(600.0)
            .with_deactivation_distance(500.0); // Invalid: smaller than activation

        config.validate();
        assert!(config.deactivation_distance > config.activation_distance);
    }

    #[test]
    fn test_particle_info_importance_calculation() {
        let mut particle = ParticleInfo::new(Vector3::new(10.0, 0.0, 0.0));
        particle.mass = 2.0;
        particle.velocity = Vector3::new(5.0, 0.0, 0.0);

        let observer = Vector3::new(0.0, 0.0, 0.0);

        // Test distance-based importance
        particle.calculate_importance(ImportanceMetric::Distance, observer);
        assert!(particle.importance_score > 0.0);
        let distance_score = particle.importance_score;

        // Test mass-based importance
        particle.calculate_importance(ImportanceMetric::Mass, observer);
        assert_eq!(particle.importance_score, 2.0);

        // Test velocity-based importance
        particle.calculate_importance(ImportanceMetric::Velocity, observer);
        assert_eq!(particle.importance_score, 5.0);

        // Test combined metric
        particle.calculate_importance(ImportanceMetric::DistanceMass, observer);
        assert!(particle.importance_score > distance_score); // Should be higher due to mass
    }

    #[test]
    fn test_activation_manager_creation() {
        let manager = ActivationManager::new();
        assert_eq!(manager.particle_count(), 0);
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_particle_position_updates() {
        let mut manager = ActivationManager::new();

        let positions = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(10.0, 0.0, 0.0),
            Vector3::new(20.0, 0.0, 0.0),
        ];

        let handles = vec![
            BodyHandle::new(0, 0),
            BodyHandle::new(1, 0),
            BodyHandle::new(2, 0),
        ];

        manager.update_positions_simple(&positions, &handles);

        assert_eq!(manager.particle_count(), 3);
        assert!(manager.particles.contains_key(&handles[0]));
        assert!(manager.particles.contains_key(&handles[1]));
        assert!(manager.particles.contains_key(&handles[2]));
    }

    #[test]
    fn test_distance_based_activation() {
        let config = ActivationConfig::new()
            .with_activation_distance(50.0)
            .with_deactivation_distance(60.0)
            .with_max_active_particles(10);

        let mut manager = ActivationManager::with_config(config);

        let positions = vec![
            Vector3::new(0.0, 0.0, 0.0),   // Very close
            Vector3::new(25.0, 0.0, 0.0),  // Close
            Vector3::new(75.0, 0.0, 0.0),  // Far
            Vector3::new(200.0, 0.0, 0.0), // Very far
        ];

        let handles = vec![
            BodyHandle::new(0, 0),
            BodyHandle::new(1, 0),
            BodyHandle::new(2, 0),
            BodyHandle::new(3, 0),
        ];

        manager.update_positions_simple(&positions, &handles);

        // Update activation from origin
        let active = manager.update_activation(Vector3::new(0.0, 0.0, 0.0));

        // Should activate particles within 50 units (first two particles)
        // Note: there may be transition delays, so we need to advance frames
        for _ in 0..10 {
            // Advance frames to complete transitions
            manager.current_frame += 1;
            for particle in manager.particles.values_mut() {
                particle.update_state(manager.current_frame);
            }
        }

        let final_active = manager.update_activation(Vector3::new(0.0, 0.0, 0.0));
        assert!(final_active.len() >= 2); // At least the close particles should be active
    }

    #[test]
    fn test_importance_based_selection() {
        let mut manager = ActivationManager::new().with_activation_distance(1000.0); // Large distance to include all

        let positions = vec![
            Vector3::new(5.0, 0.0, 0.0),  // Closest
            Vector3::new(10.0, 0.0, 0.0), // Medium
            Vector3::new(50.0, 0.0, 0.0), // Farthest
        ];

        let handles = vec![
            BodyHandle::new(0, 0),
            BodyHandle::new(1, 0),
            BodyHandle::new(2, 0),
        ];

        manager.update_positions_simple(&positions, &handles);
        manager.update_activation(Vector3::new(0.0, 0.0, 0.0));

        // Select 2 most important particles (should be the closest ones)
        let selected =
            manager.select_by_importance(handles.clone(), 2, Vector3::new(0.0, 0.0, 0.0));

        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&handles[0])); // Closest should be selected
        assert!(selected.contains(&handles[1])); // Second closest should be selected
    }

    #[test]
    fn test_forced_activation() {
        let mut manager = ActivationManager::new().with_activation_distance(10.0); // Small distance

        let positions = vec![Vector3::new(100.0, 0.0, 0.0)]; // Far from origin
        let handles = vec![BodyHandle::new(0, 0)];

        manager.update_positions_simple(&positions, &handles);
        manager.update_activation(Vector3::new(0.0, 0.0, 0.0));

        // Particle should not be active due to distance
        assert!(!manager.particles[&handles[0]].is_active());

        // Force activation
        manager.force_activate_particles(&handles);

        // Particle should now be active (or transitioning to active)
        let particle_state = manager.particles[&handles[0]].state;
        match particle_state {
            ActivationState::Active => assert!(true),
            ActivationState::Transitioning { target_state, .. } => {
                assert_eq!(target_state, ActivationTarget::Active);
            }
            ActivationState::Inactive => {
                panic!("Particle should be active or transitioning to active")
            }
        }
    }

    #[test]
    fn test_statistics_tracking() {
        let mut manager = ActivationManager::new();

        let positions = vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(10.0, 0.0, 0.0)];
        let handles = vec![BodyHandle::new(0, 0), BodyHandle::new(1, 0)];

        manager.update_positions_simple(&positions, &handles);
        manager.update_activation(Vector3::new(0.0, 0.0, 0.0));

        let stats = manager.get_statistics();
        assert_eq!(stats.total_particles, 2);
        assert!(stats.budget_utilization >= 0.0 && stats.budget_utilization <= 1.0);
    }

    #[test]
    fn test_budget_constraints() {
        let config = ActivationConfig::new()
            .with_activation_distance(1000.0) // Large distance to include all
            .with_max_active_particles(2); // Limit to 2 active particles

        let mut manager = ActivationManager::with_config(config);

        // Add 5 particles at different distances (closer = more important)
        let positions = vec![
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(2.0, 0.0, 0.0),
            Vector3::new(3.0, 0.0, 0.0),
            Vector3::new(4.0, 0.0, 0.0),
            Vector3::new(5.0, 0.0, 0.0),
        ];

        let handles: Vec<BodyHandle> = (0..5).map(|i| BodyHandle::new(i, 0)).collect();

        manager.update_positions_simple(&positions, &handles);

        // Allow transitions to complete
        for _ in 0..10 {
            manager.update_activation(Vector3::new(0.0, 0.0, 0.0));
            manager.current_frame += 1;
            for particle in manager.particles.values_mut() {
                particle.update_state(manager.current_frame);
            }
        }

        let final_active = manager.update_activation(Vector3::new(0.0, 0.0, 0.0));

        // Should have at most 2 active particles due to budget constraint
        assert!(final_active.len() <= 2);

        // The closest particles should be the ones activated
        if final_active.len() >= 2 {
            assert!(final_active.contains(&handles[0])); // Closest
            assert!(final_active.contains(&handles[1])); // Second closest
        }
    }
}
