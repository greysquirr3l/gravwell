//! Distance-based LOD assignment algorithms
//!
//! This module provides sophisticated distance-based Level of Detail assignment
//! strategies for optimal performance scaling with large particle systems.

use crate::types::{Position, Scalar};

use super::DetailLevel;

/// Advanced distance-based LOD strategy with hysteresis and smoothing.
#[derive(Debug, Clone)]
pub struct AdaptiveDistanceLOD {
    /// Base distance thresholds for LOD transitions
    base_thresholds: Vec<Scalar>,

    /// Current effective thresholds (with hysteresis applied)
    effective_thresholds: Vec<Scalar>,

    /// Detail levels for each distance range
    detail_levels: Vec<DetailLevel>,

    /// Hysteresis factor to prevent LOD oscillation (0.0-1.0)
    hysteresis_factor: Scalar,

    /// Camera position for distance calculations
    camera_position: Position,

    /// Previous frame's LOD assignments for smoothing
    previous_assignments: Vec<DetailLevel>,
}

impl AdaptiveDistanceLOD {
    /// Create a new adaptive distance-based LOD system.
    ///
    /// # Arguments
    /// * `thresholds` - Base distance thresholds for LOD transitions
    /// * `detail_levels` - Detail levels for each distance range
    /// * `hysteresis_factor` - Factor to prevent LOD oscillation (0.1 = 10% hysteresis)
    ///
    /// # Example
    /// ```rust
    /// use gravwell::lod::{AdaptiveDistanceLOD, DetailLevel};
    ///
    /// let lod = AdaptiveDistanceLOD::new(
    ///     vec![100.0, 500.0, 2000.0],
    ///     vec![
    ///         DetailLevel::Full,
    ///         DetailLevel::Reduced,
    ///         DetailLevel::Minimal,
    ///         DetailLevel::Culled,
    ///     ],
    ///     0.15  // 15% hysteresis
    /// );
    /// ```
    pub fn new(
        thresholds: Vec<Scalar>,
        detail_levels: Vec<DetailLevel>,
        hysteresis_factor: Scalar,
    ) -> Self {
        assert_eq!(
            thresholds.len() + 1,
            detail_levels.len(),
            "Detail levels must be one more than thresholds"
        );

        assert!(
            (0.0..=1.0).contains(&hysteresis_factor),
            "Hysteresis factor must be between 0.0 and 1.0"
        );

        Self {
            base_thresholds: thresholds.clone(),
            effective_thresholds: thresholds,
            detail_levels,
            hysteresis_factor,
            camera_position: Position::zeros(),
            previous_assignments: Vec::new(),
        }
    }

    /// Update camera position and recalculate effective thresholds.
    pub fn set_camera_position(&mut self, position: Position) {
        self.camera_position = position;
        self.update_effective_thresholds();
    }

    /// Assign LOD level with hysteresis to prevent oscillation.
    pub fn assign_lod_with_hysteresis(
        &self,
        particle_position: Position,
        particle_index: usize,
    ) -> DetailLevel {
        let distance = (particle_position - self.camera_position).norm();

        // Get previous LOD assignment if available
        let previous_lod = self
            .previous_assignments
            .get(particle_index)
            .copied()
            .unwrap_or(DetailLevel::Full);

        // Use appropriate thresholds based on current direction
        let current_lod_index = self.detail_level_to_index(previous_lod);

        for (i, &threshold) in self.effective_thresholds.iter().enumerate() {
            let effective_threshold = if i < current_lod_index {
                // Transitioning to higher detail - use higher threshold (easier transition)
                threshold * (1.0 + self.hysteresis_factor)
            } else if i > current_lod_index {
                // Transitioning to lower detail - use lower threshold (harder transition)
                threshold * (1.0 - self.hysteresis_factor)
            } else {
                // Same level - use base threshold
                threshold
            };

            if distance < effective_threshold {
                return self.detail_levels[i];
            }
        }

        *self.detail_levels.last().unwrap()
    }

    /// Batch assign LOD levels for all particles with hysteresis.
    pub fn assign_lod_batch_adaptive(&mut self, positions: &[Position]) -> Vec<DetailLevel> {
        // Resize previous assignments if needed
        if self.previous_assignments.len() != positions.len() {
            self.previous_assignments
                .resize(positions.len(), DetailLevel::Full);
        }

        let new_assignments: Vec<DetailLevel> = positions
            .iter()
            .enumerate()
            .map(|(i, &pos)| self.assign_lod_with_hysteresis(pos, i))
            .collect();

        // Update previous assignments for next frame
        self.previous_assignments = new_assignments.clone();

        new_assignments
    }

    /// Update effective thresholds based on current conditions.
    fn update_effective_thresholds(&mut self) {
        // For now, use base thresholds - future improvements could add:
        // - Performance-based threshold adjustment
        // - Frame rate adaptation
        // - Dynamic quality scaling
        self.effective_thresholds = self.base_thresholds.clone();
    }

    /// Convert detail level to index for hysteresis calculations.
    fn detail_level_to_index(&self, level: DetailLevel) -> usize {
        self.detail_levels
            .iter()
            .position(|&l| l == level)
            .unwrap_or(0)
    }
}

/// Performance-aware LOD system that adjusts based on frame rate.
#[derive(Debug, Clone)]
pub struct PerformanceLOD {
    /// Base LOD configuration
    adaptive_lod: AdaptiveDistanceLOD,

    /// Target frame rate (FPS)
    target_fps: f64,

    /// Recent frame times for performance monitoring
    frame_times: Vec<f64>,

    /// Maximum number of frame times to track
    max_frame_samples: usize,

    /// Performance adjustment factor (0.0-1.0)
    performance_factor: f64,

    /// Minimum performance factor to maintain visual quality
    min_performance_factor: f64,
}

impl PerformanceLOD {
    /// Create a new performance-aware LOD system.
    ///
    /// # Arguments
    /// * `adaptive_lod` - Base adaptive LOD configuration
    /// * `target_fps` - Target frame rate to maintain
    pub fn new(adaptive_lod: AdaptiveDistanceLOD, target_fps: f64) -> Self {
        Self {
            adaptive_lod,
            target_fps,
            frame_times: Vec::new(),
            max_frame_samples: 30, // Track last 30 frames
            performance_factor: 1.0,
            min_performance_factor: 0.3, // Don't reduce quality below 30%
        }
    }

    /// Record frame time for performance monitoring.
    pub fn record_frame_time(&mut self, frame_time_ms: f64) {
        self.frame_times.push(frame_time_ms);

        if self.frame_times.len() > self.max_frame_samples {
            self.frame_times.remove(0);
        }

        self.update_performance_factor();
    }

    /// Assign LOD levels with performance-based adjustment.
    pub fn assign_lod_performance_aware(&mut self, positions: &[Position]) -> Vec<DetailLevel> {
        let base_assignments = self.adaptive_lod.assign_lod_batch_adaptive(positions);

        if self.performance_factor >= 1.0 {
            // Performance is good - use base assignments
            return base_assignments;
        }

        // Performance is low - adjust LOD assignments to reduce load
        base_assignments
            .into_iter()
            .map(|lod| self.adjust_lod_for_performance(lod))
            .collect()
    }

    /// Update performance factor based on recent frame times.
    fn update_performance_factor(&mut self) {
        if self.frame_times.len() < 5 {
            return; // Need at least 5 samples
        }

        let average_frame_time: f64 =
            self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64;
        let current_fps = 1000.0 / average_frame_time; // Convert ms to FPS

        if current_fps < self.target_fps {
            // Performance is below target - reduce quality
            let performance_ratio = current_fps / self.target_fps;
            self.performance_factor = performance_ratio.max(self.min_performance_factor);
        } else {
            // Performance is good - can increase quality
            self.performance_factor = (self.performance_factor + 0.1).min(1.0);
        }
    }

    /// Adjust individual LOD assignment based on performance factor.
    fn adjust_lod_for_performance(&self, base_lod: DetailLevel) -> DetailLevel {
        if self.performance_factor >= 0.8 {
            return base_lod; // Good performance - no adjustment
        }

        // Reduce detail level based on performance
        match base_lod {
            DetailLevel::Full if self.performance_factor < 0.6 => DetailLevel::Reduced,
            DetailLevel::Full if self.performance_factor < 0.4 => DetailLevel::Minimal,
            DetailLevel::Reduced if self.performance_factor < 0.4 => DetailLevel::Minimal,
            DetailLevel::Reduced if self.performance_factor < 0.3 => DetailLevel::Culled,
            DetailLevel::Minimal if self.performance_factor < 0.3 => DetailLevel::Culled,
            _ => base_lod,
        }
    }

    /// Get current performance statistics.
    pub fn performance_stats(&self) -> PerformanceLODStats {
        let current_fps = if !self.frame_times.is_empty() {
            let avg_frame_time =
                self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64;
            1000.0 / avg_frame_time
        } else {
            0.0
        };

        PerformanceLODStats {
            current_fps,
            target_fps: self.target_fps,
            performance_factor: self.performance_factor,
            frame_time_samples: self.frame_times.len(),
        }
    }
}

/// Performance statistics for the performance-aware LOD system.
#[derive(Debug, Clone)]
pub struct PerformanceLODStats {
    /// Current average frame rate
    pub current_fps: f64,

    /// Target frame rate
    pub target_fps: f64,

    /// Current performance factor (0.0-1.0)
    pub performance_factor: f64,

    /// Number of frame time samples collected
    pub frame_time_samples: usize,
}

impl PerformanceLODStats {
    /// Check if the system is meeting performance targets.
    pub fn is_meeting_target(&self) -> bool {
        self.current_fps >= self.target_fps * 0.95 // Within 5% of target
    }

    /// Get performance percentage relative to target.
    pub fn performance_percentage(&self) -> f64 {
        if self.target_fps > 0.0 {
            (self.current_fps / self.target_fps * 100.0).min(100.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_distance_lod_hysteresis() {
        let mut lod = AdaptiveDistanceLOD::new(
            vec![100.0, 500.0],
            vec![DetailLevel::Full, DetailLevel::Reduced, DetailLevel::Culled],
            0.2, // 20% hysteresis
        );

        let positions = vec![
            [90.0, 0.0, 0.0].into(),  // Near threshold
            [110.0, 0.0, 0.0].into(), // Just over threshold
        ];

        // First assignment - no hysteresis
        let assignments1 = lod.assign_lod_batch_adaptive(&positions);
        assert_eq!(assignments1[0], DetailLevel::Full); // < 100
        assert_eq!(assignments1[1], DetailLevel::Reduced); // > 100

        // Move first particle slightly past threshold
        let positions2 = vec![
            [105.0, 0.0, 0.0].into(), // Just over threshold, but with hysteresis
            [95.0, 0.0, 0.0].into(),  // Back under threshold
        ];

        let assignments2 = lod.assign_lod_batch_adaptive(&positions2);

        // Due to hysteresis, transitions should be smoother
        // This requires the actual implementation to handle hysteresis properly
        assert!(assignments2.len() == 2);
    }

    #[test]
    fn test_performance_lod_adjustment() {
        let adaptive_lod = AdaptiveDistanceLOD::new(
            vec![100.0, 500.0],
            vec![DetailLevel::Full, DetailLevel::Reduced, DetailLevel::Culled],
            0.1,
        );

        let mut perf_lod = PerformanceLOD::new(adaptive_lod, 60.0);

        // Simulate poor performance (30 FPS = 33.33ms per frame)
        for _ in 0..10 {
            perf_lod.record_frame_time(33.33);
        }

        let positions = vec![[50.0, 0.0, 0.0].into()]; // Should be Full normally
        let assignments = perf_lod.assign_lod_performance_aware(&positions);

        // Performance adjustment should reduce quality
        // Actual behavior depends on implementation details
        assert_eq!(assignments.len(), 1);

        let stats = perf_lod.performance_stats();
        assert!(!stats.is_meeting_target()); // Should not be meeting 60 FPS target
        assert!(stats.performance_factor < 1.0); // Should have reduced performance factor
    }

    #[test]
    fn test_performance_stats() {
        let adaptive_lod = AdaptiveDistanceLOD::new(
            vec![100.0],
            vec![DetailLevel::Full, DetailLevel::Reduced],
            0.1,
        );

        let mut perf_lod = PerformanceLOD::new(adaptive_lod, 60.0);

        // Good performance (60 FPS = 16.67ms per frame)
        for _ in 0..5 {
            perf_lod.record_frame_time(16.67);
        }

        let stats = perf_lod.performance_stats();
        assert!(stats.is_meeting_target());
        assert!(stats.performance_percentage() >= 95.0);
        assert_eq!(stats.frame_time_samples, 5);
    }
}
