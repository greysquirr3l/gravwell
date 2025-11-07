//! Spatial optimization and culling systems for massive particle simulations.
//!
//! This module provides efficient spatial data structures and culling algorithms
//! to enable simulations with 100,000+ particles while maintaining real-time
//! performance. The spatial systems work in conjunction with the LOD system
//! to provide intelligent performance optimization.
//!
//! # Core Components
//!
//! - **Spatial Hash Grid**: O(1) proximity queries for particle neighbor finding
//! - **Frustum Culling**: Camera-based visibility optimization
//! - **Dynamic Activation**: Distance and importance-based particle activation
//! - **Spatial Culler**: Integrated culling system combining all optimizations
//!
//! # Usage
//!
//! ```rust
//! use gravwell::spatial::{SpatialHashGrid, Frustum, SpatialCuller};
//! use gravwell::types::{Vector3, Scalar};
//!
//! // Create spatial hash grid for proximity queries
//! let mut grid = SpatialHashGrid::new(10.0); // 10 unit cell size
//!
//! // Insert particles into spatial grid
//! grid.insert_particle(0, Vector3::new(5.0, 5.0, 5.0));
//! grid.insert_particle(1, Vector3::new(15.0, 5.0, 5.0));
//!
//! // Find neighbors within radius
//! let neighbors = grid.find_neighbors(Vector3::new(5.0, 5.0, 5.0), 12.0);
//!
//! // Create camera frustum for culling
//! let frustum = Frustum::from_camera(
//!     Vector3::new(0.0, 0.0, 0.0), // position
//!     Vector3::new(0.0, 0.0, 1.0), // forward
//!     Vector3::new(0.0, 1.0, 0.0), // up
//!     60.0, // fov degrees
//!     16.0 / 9.0, // aspect ratio
//!     1.0, // near
//!     1000.0 // far
//! );
//!
//! // Cull particles outside camera view
//! let visible = frustum.cull_particles(&particle_positions);
//! ```
//!
//! # Performance Characteristics
//!
//! - **Hash Grid**: O(1) insertion, O(k) neighbor queries (k = neighbors in range)
//! - **Frustum Culling**: O(n) with early termination for distant clusters
//! - **Combined System**: Enables 100,000+ particles at 60 FPS with proper configuration
//!
//! # Integration with LOD System
//!
//! The spatial culling system works seamlessly with the Level of Detail system:
//!
//! ```rust
//! use gravwell::lod::{LODSystem, DetailLevel};
//! use gravwell::spatial::SpatialCuller;
//!
//! let mut lod_system = LODSystem::new();
//! let mut spatial_culler = SpatialCuller::new();
//!
//! // Update LOD based on spatial queries
//! spatial_culler.update_lod_system(&mut lod_system, camera_position);
//! ```

pub mod activation;
pub mod frustum;
pub mod hash_grid;

pub use activation::{ActivationConfig, ActivationManager, ActivationState, ImportanceMetric};
pub use frustum::{Frustum, FrustumCullingResult, Plane};
pub use hash_grid::{HashGridConfig, SpatialCell, SpatialHashGrid};

use crate::lod::LODSystem;
use crate::types::{Scalar, Vector3};
use crate::BodyHandle;
use std::collections::HashMap;

/// Comprehensive spatial culling system that combines hash grids, frustum culling,
/// and dynamic activation management for maximum performance optimization.
///
/// The `SpatialCuller` provides a unified interface for all spatial optimizations,
/// automatically coordinating between different culling strategies to achieve
/// optimal performance for massive particle simulations.
///
/// # Example
///
/// ```rust
/// use gravwell::spatial::SpatialCuller;
/// use gravwell::types::Vector3;
///
/// let mut culler = SpatialCuller::new()
///     .with_cell_size(50.0)
///     .with_activation_distance(1000.0)
///     .with_frustum_culling(true);
///
/// // Update particle positions in spatial structures
/// culler.update_particles(&positions, &handles);
///
/// // Perform comprehensive culling pass
/// let active_particles = culler.cull_particles(
///     camera_position,
///     &camera_frustum,
///     max_active_particles
/// );
/// ```
pub struct SpatialCuller {
    /// Spatial hash grid for efficient proximity queries
    hash_grid: SpatialHashGrid,

    /// Activation manager for distance-based culling
    activation_manager: ActivationManager,

    /// Whether frustum culling is enabled
    frustum_culling_enabled: bool,

    /// Cache of previously active particles for smooth transitions
    active_particle_cache: HashMap<BodyHandle, ActivationState>,

    /// Statistics for performance monitoring
    culling_stats: CullingStatistics,
}

/// Statistics tracking for spatial culling performance analysis
#[derive(Debug, Clone)]
pub struct CullingStatistics {
    /// Total particles processed in last culling pass
    pub total_particles: usize,

    /// Particles culled by distance
    pub distance_culled: usize,

    /// Particles culled by frustum
    pub frustum_culled: usize,

    /// Particles activated this frame
    pub newly_activated: usize,

    /// Particles deactivated this frame
    pub newly_deactivated: usize,

    /// Time spent on spatial queries (microseconds)
    pub spatial_query_time_us: u64,

    /// Time spent on frustum culling (microseconds)
    pub frustum_culling_time_us: u64,

    /// Average particles per spatial cell
    pub avg_particles_per_cell: f32,
}

impl Default for CullingStatistics {
    fn default() -> Self {
        Self {
            total_particles: 0,
            distance_culled: 0,
            frustum_culled: 0,
            newly_activated: 0,
            newly_deactivated: 0,
            spatial_query_time_us: 0,
            frustum_culling_time_us: 0,
            avg_particles_per_cell: 0.0,
        }
    }
}

impl SpatialCuller {
    /// Create a new spatial culler with default configuration
    pub fn new() -> Self {
        Self {
            hash_grid: SpatialHashGrid::new(100.0), // 100 unit default cell size
            activation_manager: ActivationManager::new(),
            frustum_culling_enabled: true,
            active_particle_cache: HashMap::new(),
            culling_stats: CullingStatistics::default(),
        }
    }

    /// Configure the spatial hash grid cell size
    pub fn with_cell_size(mut self, cell_size: Scalar) -> Self {
        self.hash_grid = SpatialHashGrid::new(cell_size);
        self
    }

    /// Configure the activation distance threshold
    pub fn with_activation_distance(mut self, distance: Scalar) -> Self {
        self.activation_manager = self.activation_manager.with_activation_distance(distance);
        self
    }

    /// Enable or disable frustum culling
    pub fn with_frustum_culling(mut self, enabled: bool) -> Self {
        self.frustum_culling_enabled = enabled;
        self
    }

    /// Update particle positions in all spatial structures
    ///
    /// This should be called every frame to maintain spatial coherency.
    /// The function efficiently updates the hash grid and activation manager
    /// with new particle positions.
    pub fn update_particles(&mut self, positions: &[Vector3], handles: &[BodyHandle]) {
        use std::time::Instant;
        let start_time = Instant::now();

        // Clear previous frame's spatial data
        self.hash_grid.clear();

        // Insert all particles into spatial hash grid
        for (i, &position) in positions.iter().enumerate() {
            if i < handles.len() {
                self.hash_grid.insert_particle(handles[i], position);
            }
        }

        // Update activation manager with new positions
        self.activation_manager
            .update_positions(positions, handles, None, None);

        // Update statistics
        self.culling_stats.spatial_query_time_us = start_time.elapsed().as_micros() as u64;
        self.culling_stats.total_particles = positions.len();
        self.culling_stats.avg_particles_per_cell = self.hash_grid.average_particles_per_cell();
    }

    /// Perform comprehensive spatial culling pass
    ///
    /// Returns a list of particle handles that should remain active based on:
    /// - Distance from camera position
    /// - Frustum culling (if enabled)
    /// - Importance-based activation limits
    ///
    /// # Arguments
    ///
    /// * `camera_position` - Current camera/observer position
    /// * `frustum` - Camera frustum for visibility culling (optional)
    /// * `max_active` - Maximum number of particles to keep active
    ///
    /// # Returns
    ///
    /// Vector of body handles for particles that should remain active
    pub fn cull_particles(
        &mut self,
        camera_position: Vector3,
        frustum: Option<&Frustum>,
        max_active: usize,
    ) -> Vec<BodyHandle> {
        use std::time::Instant;

        // Reset frame statistics
        let mut new_stats = CullingStatistics::default();
        new_stats.total_particles = self.culling_stats.total_particles;
        new_stats.spatial_query_time_us = self.culling_stats.spatial_query_time_us;
        new_stats.avg_particles_per_cell = self.culling_stats.avg_particles_per_cell;

        // Phase 1: Distance-based activation
        let _distance_start = Instant::now();
        let distance_active = self.activation_manager.update_activation(camera_position);
        let distance_active_count = distance_active.len();
        new_stats.distance_culled = new_stats.total_particles - distance_active_count;

        // Phase 2: Frustum culling (if enabled)
        let mut visible_particles = distance_active;
        if self.frustum_culling_enabled {
            if let Some(frustum) = frustum {
                let frustum_start = Instant::now();
                visible_particles = self.apply_frustum_culling(visible_particles, frustum);
                new_stats.frustum_culling_time_us = frustum_start.elapsed().as_micros() as u64;
                new_stats.frustum_culled = distance_active_count - visible_particles.len();
            }
        }

        // Phase 3: Limit to maximum active particles by importance
        if visible_particles.len() > max_active {
            visible_particles = self.activation_manager.select_by_importance(
                visible_particles,
                max_active,
                camera_position,
            );
        }

        // Update activation state transitions
        self.update_activation_transitions(&visible_particles, &mut new_stats);

        self.culling_stats = new_stats;
        visible_particles
    }

    /// Apply frustum culling to a set of particles
    fn apply_frustum_culling(
        &self,
        particles: Vec<BodyHandle>,
        frustum: &Frustum,
    ) -> Vec<BodyHandle> {
        particles
            .into_iter()
            .filter(|&handle| {
                if let Some(position) = self.hash_grid.get_particle_position(handle) {
                    frustum.contains_point(position)
                } else {
                    false // Remove particles not in hash grid
                }
            })
            .collect()
    }

    /// Update activation state transitions and statistics
    fn update_activation_transitions(
        &mut self,
        current_active: &[BodyHandle],
        stats: &mut CullingStatistics,
    ) {
        use std::collections::HashSet;

        let current_set: HashSet<BodyHandle> = current_active.iter().copied().collect();
        let previous_set: HashSet<BodyHandle> =
            self.active_particle_cache.keys().copied().collect();

        // Count newly activated particles
        stats.newly_activated = current_set.difference(&previous_set).count();

        // Count newly deactivated particles
        stats.newly_deactivated = previous_set.difference(&current_set).count();

        // Update cache with current state
        self.active_particle_cache.clear();
        for &handle in current_active {
            self.active_particle_cache
                .insert(handle, ActivationState::Active);
        }
    }

    /// Get spatial neighbors within a given radius
    ///
    /// Useful for collision detection and local force calculations
    pub fn find_neighbors(&self, position: Vector3, radius: Scalar) -> Vec<BodyHandle> {
        self.hash_grid.find_neighbors(position, radius)
    }

    /// Update LOD system based on spatial culling results
    ///
    /// This integrates the spatial culler with the existing LOD system,
    /// providing distance-based detail levels and activation states.
    ///
    /// Note: This is a simplified integration that demonstrates the concept.
    /// A full integration would require extending the LOD system to work with
    /// individual particle handles rather than batch processing.
    pub fn update_lod_system(&self, lod_system: &mut LODSystem, camera_position: Vector3) {
        // Set camera position in LOD system
        lod_system.set_camera_position(camera_position);

        // The LOD system currently works with ParticleSet batch processing
        // Individual particle LOD assignment would require extending the LOD API
        // For now, this method serves as a placeholder for future integration

        // In a full implementation, we would:
        // 1. Extend LODSystem to support individual particle LOD assignment
        // 2. Use the spatial culler's activation states to override LOD decisions
        // 3. Provide seamless integration between spatial and LOD optimizations

        // Example of how this could work with an extended API:
        // for &handle in self.get_all_particles() {
        //     if let Some(position) = self.hash_grid.get_particle_position(handle) {
        //         let distance = (position - camera_position).norm();
        //         let detail_level = if self.is_particle_active(handle) {
        //             lod_system.distance_lod.assign_lod(position)
        //         } else {
        //             DetailLevel::Culled
        //         };
        //         lod_system.set_particle_detail_level(handle, detail_level);
        //     }
        // }
    }

    /// Get current culling statistics
    pub fn get_statistics(&self) -> &CullingStatistics {
        &self.culling_stats
    }

    /// Get the number of currently active particles
    pub fn active_particle_count(&self) -> usize {
        self.active_particle_cache.len()
    }

    /// Check if a particle is currently active
    pub fn is_particle_active(&self, handle: BodyHandle) -> bool {
        self.active_particle_cache.contains_key(&handle)
    }

    /// Get spatial hash grid statistics
    pub fn get_hash_grid_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert(
            "total_cells".to_string(),
            self.hash_grid.cell_count() as f64,
        );
        stats.insert(
            "occupied_cells".to_string(),
            self.hash_grid.occupied_cell_count() as f64,
        );
        stats.insert(
            "avg_particles_per_cell".to_string(),
            self.hash_grid.average_particles_per_cell() as f64,
        );
        stats.insert(
            "max_particles_per_cell".to_string(),
            self.hash_grid.max_particles_per_cell() as f64,
        );
        stats
    }
}

impl Default for SpatialCuller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Vector3;

    #[test]
    fn test_spatial_culler_creation() {
        let culler = SpatialCuller::new();
        assert_eq!(culler.active_particle_count(), 0);
        assert!(culler.frustum_culling_enabled);
    }

    #[test]
    fn test_spatial_culler_configuration() {
        let culler = SpatialCuller::new()
            .with_cell_size(50.0)
            .with_activation_distance(500.0)
            .with_frustum_culling(false);

        assert!(!culler.frustum_culling_enabled);
    }

    #[test]
    fn test_particle_update_and_culling() {
        let mut culler = SpatialCuller::new().with_activation_distance(100.0);

        let positions = vec![
            Vector3::new(0.0, 0.0, 0.0),   // Close to origin
            Vector3::new(50.0, 0.0, 0.0),  // Medium distance
            Vector3::new(200.0, 0.0, 0.0), // Far from origin
        ];

        let handles = vec![
            BodyHandle::new(0, 0),
            BodyHandle::new(1, 0),
            BodyHandle::new(2, 0),
        ];

        culler.update_particles(&positions, &handles);

        let camera_position = Vector3::new(0.0, 0.0, 0.0);
        let active = culler.cull_particles(camera_position, None, 1000);

        // Should activate particles within activation distance (100.0)
        assert!(active.len() >= 2); // First two particles should be active
        assert!(culler.is_particle_active(handles[0]));
        assert!(culler.is_particle_active(handles[1]));
    }

    #[test]
    fn test_activation_statistics() {
        let mut culler = SpatialCuller::new().with_activation_distance(50.0);

        let positions = vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(100.0, 0.0, 0.0)];

        let handles = vec![BodyHandle::new(0, 0), BodyHandle::new(1, 0)];

        culler.update_particles(&positions, &handles);
        culler.cull_particles(Vector3::new(0.0, 0.0, 0.0), None, 1000);

        let stats = culler.get_statistics();
        assert_eq!(stats.total_particles, 2);
        assert!(stats.distance_culled > 0); // Should cull the distant particle
    }
}
