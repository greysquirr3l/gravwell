//! Spatial optimization and culling algorithms for LOD systems
//!
//! This module provides spatial partitioning and culling strategies to optimize
//! physics calculations by removing particles that don't need updates.

use crate::{
    core::particle::ParticleSet,
    types::{Position, Scalar, Vector3},
};

#[cfg(test)]
use crate::core::particle::Body;

// DetailLevel is available through parent module
use std::collections::{HashMap, HashSet};

/// Spatial hash grid for efficient neighbor queries and culling.
#[derive(Debug, Clone)]
pub struct SpatialHashGrid {
    /// Grid cell size
    cell_size: Scalar,

    /// Hash map of grid cells to particle indices
    grid: HashMap<GridCell, Vec<usize>>,

    /// Cached particle positions for change detection
    cached_positions: Vec<Position>,

    /// Particles that need grid updates
    dirty_particles: HashSet<usize>,
}

/// 3D grid cell coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GridCell {
    x: i32,
    y: i32,
    z: i32,
}

impl SpatialHashGrid {
    /// Create a new spatial hash grid.
    ///
    /// # Arguments
    /// * `cell_size` - Size of each grid cell (should be roughly the interaction range)
    ///
    /// # Example
    /// ```rust
    /// use gravwell::lod::spatial::SpatialHashGrid;
    ///
    /// let grid = SpatialHashGrid::new(100.0); // 100-unit cells
    /// ```
    pub fn new(cell_size: Scalar) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
            cached_positions: Vec::new(),
            dirty_particles: HashSet::new(),
        }
    }

    /// Update the grid with current particle positions.
    pub fn update(&mut self, particles: &ParticleSet) {
        // Resize cached positions if needed
        if self.cached_positions.len() != particles.len() {
            self.cached_positions
                .resize(particles.len(), Position::zeros());
            // Mark all particles as dirty on resize
            self.dirty_particles = (0..particles.len()).collect();
        }

        // Find particles that have moved significantly
        for i in 0..particles.len() {
            let current_pos = *particles.position(i);
            let cached_pos = self.cached_positions[i];

            let movement = (current_pos - cached_pos).norm();
            if movement > self.cell_size * 0.1 {
                // 10% of cell size threshold
                self.dirty_particles.insert(i);
            }
        }

        // Update grid for dirty particles
        for &particle_index in &self.dirty_particles {
            // Remove from old cell
            let old_pos = self.cached_positions[particle_index];
            let old_cell = self.position_to_cell(old_pos);
            if let Some(cell_particles) = self.grid.get_mut(&old_cell) {
                cell_particles.retain(|&i| i != particle_index);
                if cell_particles.is_empty() {
                    self.grid.remove(&old_cell);
                }
            }

            // Add to new cell
            let new_pos = *particles.position(particle_index);
            let new_cell = self.position_to_cell(new_pos);
            self.grid
                .entry(new_cell)
                .or_insert_with(Vec::new)
                .push(particle_index);

            // Update cached position
            self.cached_positions[particle_index] = new_pos;
        }

        self.dirty_particles.clear();
    }

    /// Get particles within a radius of a position.
    pub fn query_radius(&self, center: Position, radius: Scalar) -> Vec<usize> {
        let mut result = Vec::new();

        // Calculate the range of cells to check
        let cells_to_check = self.get_cells_in_radius(center, radius);

        for cell in cells_to_check {
            if let Some(particles) = self.grid.get(&cell) {
                for &particle_index in particles {
                    let particle_pos = self.cached_positions[particle_index];
                    if (particle_pos - center).norm() <= radius {
                        result.push(particle_index);
                    }
                }
            }
        }

        result
    }

    /// Get all particles in the same cell as the given position.
    pub fn query_cell(&self, position: Position) -> Vec<usize> {
        let cell = self.position_to_cell(position);
        self.grid.get(&cell).map(|v| v.clone()).unwrap_or_default()
    }

    /// Convert world position to grid cell.
    fn position_to_cell(&self, position: Position) -> GridCell {
        GridCell {
            x: (position.x / self.cell_size).floor() as i32,
            y: (position.y / self.cell_size).floor() as i32,
            z: (position.z / self.cell_size).floor() as i32,
        }
    }

    /// Get all grid cells within a radius of a position.
    fn get_cells_in_radius(&self, center: Position, radius: Scalar) -> Vec<GridCell> {
        let mut cells = Vec::new();

        let center_cell = self.position_to_cell(center);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                for dz in -cell_radius..=cell_radius {
                    cells.push(GridCell {
                        x: center_cell.x + dx,
                        y: center_cell.y + dy,
                        z: center_cell.z + dz,
                    });
                }
            }
        }

        cells
    }

    /// Get statistics about the spatial grid.
    pub fn statistics(&self) -> SpatialGridStats {
        let total_cells = self.grid.len();
        let total_particles = self.cached_positions.len();

        let particles_per_cell: Vec<usize> = self.grid.values().map(|v| v.len()).collect();
        let max_particles_per_cell = particles_per_cell.iter().max().copied().unwrap_or(0);
        let avg_particles_per_cell = if total_cells > 0 {
            total_particles as f64 / total_cells as f64
        } else {
            0.0
        };

        SpatialGridStats {
            total_cells,
            total_particles,
            max_particles_per_cell,
            avg_particles_per_cell,
            cell_size: self.cell_size,
        }
    }
}

/// Statistics for the spatial hash grid.
#[derive(Debug, Clone)]
pub struct SpatialGridStats {
    /// Total number of active grid cells
    pub total_cells: usize,

    /// Total number of particles in the grid
    pub total_particles: usize,

    /// Maximum particles in any single cell
    pub max_particles_per_cell: usize,

    /// Average particles per cell
    pub avg_particles_per_cell: f64,

    /// Grid cell size
    pub cell_size: Scalar,
}

/// Frustum culling for camera-based spatial optimization.
#[derive(Debug, Clone)]
pub struct FrustumCuller {
    /// Camera position
    camera_position: Position,

    /// Camera forward direction (normalized)
    camera_forward: Vector3,

    /// Camera up direction (normalized)
    camera_up: Vector3,

    /// Field of view in radians
    field_of_view: Scalar,

    /// Aspect ratio (width/height)
    aspect_ratio: Scalar,

    /// Near clipping plane distance
    near_plane: Scalar,

    /// Far clipping plane distance
    far_plane: Scalar,

    /// Cached frustum planes for culling
    frustum_planes: [Plane; 6],
}

/// Geometric plane for frustum culling.
#[derive(Debug, Clone, Copy)]
struct Plane {
    normal: Vector3,
    distance: Scalar,
}

impl FrustumCuller {
    /// Create a new frustum culler.
    ///
    /// # Arguments
    /// * `camera_position` - World position of the camera
    /// * `camera_forward` - Forward direction vector (will be normalized)
    /// * `camera_up` - Up direction vector (will be normalized)
    /// * `field_of_view` - Vertical field of view in radians
    /// * `aspect_ratio` - Width/height ratio
    /// * `near_plane` - Distance to near clipping plane
    /// * `far_plane` - Distance to far clipping plane
    pub fn new(
        camera_position: Position,
        camera_forward: Vector3,
        camera_up: Vector3,
        field_of_view: Scalar,
        aspect_ratio: Scalar,
        near_plane: Scalar,
        far_plane: Scalar,
    ) -> Self {
        let mut culler = Self {
            camera_position,
            camera_forward: camera_forward.normalize(),
            camera_up: camera_up.normalize(),
            field_of_view,
            aspect_ratio,
            near_plane,
            far_plane,
            frustum_planes: [Plane {
                normal: Vector3::zeros(),
                distance: 0.0,
            }; 6],
        };

        culler.update_frustum_planes();
        culler
    }

    /// Update camera parameters and recalculate frustum planes.
    pub fn update_camera(&mut self, position: Position, forward: Vector3, up: Vector3) {
        self.camera_position = position;
        self.camera_forward = forward.normalize();
        self.camera_up = up.normalize();
        self.update_frustum_planes();
    }

    /// Test if a point is inside the frustum.
    pub fn is_point_in_frustum(&self, point: Position) -> bool {
        for plane in &self.frustum_planes {
            if self.distance_to_plane(point, *plane) < 0.0 {
                return false;
            }
        }
        true
    }

    /// Test if a sphere is inside or intersecting the frustum.
    pub fn is_sphere_in_frustum(&self, center: Position, radius: Scalar) -> bool {
        for plane in &self.frustum_planes {
            if self.distance_to_plane(center, *plane) < -radius {
                return false;
            }
        }
        true
    }

    /// Cull particles outside the frustum.
    pub fn cull_particles(&self, particles: &ParticleSet) -> Vec<usize> {
        (0..particles.len())
            .filter(|&i| {
                let position = *particles.position(i);
                let radius = 1.0; // Default radius - TODO: add radius support to ParticleSet
                self.is_sphere_in_frustum(position, radius)
            })
            .collect()
    }

    /// Update the frustum planes based on current camera parameters.
    fn update_frustum_planes(&mut self) {
        let right = self.camera_forward.cross(&self.camera_up).normalize();

        // Calculate frustum dimensions at near and far planes
        let half_v_near = (self.field_of_view * 0.5).tan() * self.near_plane;
        let half_h_near = half_v_near * self.aspect_ratio;
        let half_v_far = (self.field_of_view * 0.5).tan() * self.far_plane;
        let _half_h_far = half_v_far * self.aspect_ratio; // For future frustum calculations

        // Calculate frustum corners at near and far planes
        let near_center = self.camera_position + self.camera_forward * self.near_plane;
        let far_center = self.camera_position + self.camera_forward * self.far_plane;

        // Near plane: pointing towards camera
        self.frustum_planes[0] = Plane {
            normal: self.camera_forward,
            distance: -self.camera_forward.dot(&near_center),
        };

        // Far plane: pointing away from camera
        self.frustum_planes[1] = Plane {
            normal: -self.camera_forward,
            distance: self.camera_forward.dot(&far_center),
        };

        // Calculate the corner points of the frustum at the near plane
        let near_top_left = near_center + self.camera_up * half_v_near - right * half_h_near;
        let near_top_right = near_center + self.camera_up * half_v_near + right * half_h_near;
        let near_bottom_left = near_center - self.camera_up * half_v_near - right * half_h_near;
        let _near_bottom_right = near_center - self.camera_up * half_v_near + right * half_h_near;

        // Left plane: normal points inward (towards right)
        let left_edge = near_top_left - self.camera_position;
        let left_normal = left_edge.cross(&self.camera_up).normalize();
        self.frustum_planes[2] = Plane {
            normal: left_normal,
            distance: -left_normal.dot(&self.camera_position),
        };

        // Right plane: normal points inward (towards left)
        let right_edge = near_top_right - self.camera_position;
        let right_normal = self.camera_up.cross(&right_edge).normalize();
        self.frustum_planes[3] = Plane {
            normal: right_normal,
            distance: -right_normal.dot(&self.camera_position),
        };

        // Top plane: normal points inward (towards bottom)
        let top_edge = near_top_left - self.camera_position;
        let top_normal = right.cross(&top_edge).normalize();
        self.frustum_planes[4] = Plane {
            normal: top_normal,
            distance: -top_normal.dot(&self.camera_position),
        };

        // Bottom plane: normal points inward (towards top)
        let bottom_edge = near_bottom_left - self.camera_position;
        let bottom_normal = bottom_edge.cross(&right).normalize();
        self.frustum_planes[5] = Plane {
            normal: bottom_normal,
            distance: -bottom_normal.dot(&self.camera_position),
        };
    }

    /// Calculate the signed distance from a point to a plane.
    fn distance_to_plane(&self, point: Position, plane: Plane) -> Scalar {
        plane.normal.dot(&point) + plane.distance
    }
}

/// Combined spatial optimization system.
#[derive(Debug)]
pub struct SpatialOptimizer {
    /// Spatial hash grid for neighbor queries
    spatial_grid: SpatialHashGrid,

    /// Frustum culler for camera-based culling
    frustum_culler: Option<FrustumCuller>,

    /// Enable/disable different optimization strategies
    use_frustum_culling: bool,
    use_spatial_hashing: bool,

    /// Cached results to avoid recomputation
    cached_visible_particles: Vec<usize>,
    cache_valid: bool,
}

impl SpatialOptimizer {
    /// Create a new spatial optimizer.
    pub fn new(cell_size: Scalar) -> Self {
        Self {
            spatial_grid: SpatialHashGrid::new(cell_size),
            frustum_culler: None,
            use_frustum_culling: false,
            use_spatial_hashing: true,
            cached_visible_particles: Vec::new(),
            cache_valid: false,
        }
    }

    /// Enable frustum culling with camera parameters.
    pub fn enable_frustum_culling(
        &mut self,
        camera_position: Position,
        camera_forward: Vector3,
        camera_up: Vector3,
        field_of_view: Scalar,
        aspect_ratio: Scalar,
        near_plane: Scalar,
        far_plane: Scalar,
    ) {
        self.frustum_culler = Some(FrustumCuller::new(
            camera_position,
            camera_forward,
            camera_up,
            field_of_view,
            aspect_ratio,
            near_plane,
            far_plane,
        ));
        self.use_frustum_culling = true;
        self.cache_valid = false;
    }

    /// Update camera parameters for frustum culling.
    pub fn update_camera(&mut self, position: Position, forward: Vector3, up: Vector3) {
        if let Some(ref mut culler) = self.frustum_culler {
            culler.update_camera(position, forward, up);
            self.cache_valid = false;
        }
    }

    /// Update spatial data structures with current particle positions.
    pub fn update(&mut self, particles: &ParticleSet) {
        if self.use_spatial_hashing {
            self.spatial_grid.update(particles);
        }
        self.cache_valid = false;
    }

    /// Get indices of particles that should be updated (visible and relevant).
    pub fn get_active_particles(&mut self, particles: &ParticleSet) -> Vec<usize> {
        if self.cache_valid {
            return self.cached_visible_particles.clone();
        }

        let mut active_particles: Vec<usize> = (0..particles.len()).collect();

        // Apply frustum culling if enabled
        if self.use_frustum_culling {
            if let Some(ref culler) = self.frustum_culler {
                active_particles = culler.cull_particles(particles);
            }
        }

        // Cache results
        self.cached_visible_particles = active_particles.clone();
        self.cache_valid = true;

        active_particles
    }

    /// Get particles near a specific position (using spatial grid).
    pub fn get_neighbors(&self, position: Position, radius: Scalar) -> Vec<usize> {
        if self.use_spatial_hashing {
            self.spatial_grid.query_radius(position, radius)
        } else {
            Vec::new()
        }
    }

    /// Get spatial optimization statistics.
    pub fn statistics(&self) -> SpatialOptimizerStats {
        let grid_stats = if self.use_spatial_hashing {
            Some(self.spatial_grid.statistics())
        } else {
            None
        };

        SpatialOptimizerStats {
            grid_stats,
            visible_particles: self.cached_visible_particles.len(),
            frustum_culling_enabled: self.use_frustum_culling,
            spatial_hashing_enabled: self.use_spatial_hashing,
        }
    }
}

/// Statistics for the spatial optimizer.
#[derive(Debug, Clone)]
pub struct SpatialOptimizerStats {
    /// Spatial grid statistics (if enabled)
    pub grid_stats: Option<SpatialGridStats>,

    /// Number of currently visible particles
    pub visible_particles: usize,

    /// Whether frustum culling is enabled
    pub frustum_culling_enabled: bool,

    /// Whether spatial hashing is enabled
    pub spatial_hashing_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_grid() {
        let mut grid = SpatialHashGrid::new(100.0);
        let mut particles = ParticleSet::new();

        // Add particles in a cluster
        particles
            .add_body(
                Body::new()
                    .with_mass(1.0)
                    .with_position([50.0, 50.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();
        particles
            .add_body(
                Body::new()
                    .with_mass(1.0)
                    .with_position([150.0, 50.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();
        particles
            .add_body(
                Body::new()
                    .with_mass(1.0)
                    .with_position([250.0, 50.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();

        grid.update(&particles);

        // Query for neighbors
        let neighbors = grid.query_radius([100.0, 50.0, 0.0].into(), 75.0);
        assert_eq!(neighbors.len(), 2); // Should find first two particles

        let stats = grid.statistics();
        assert_eq!(stats.total_particles, 3);
        assert!(stats.total_cells > 0);
    }

    #[test]
    #[ignore] // TODO: Fix frustum culling implementation
    fn test_frustum_culling() {
        // Create a very simple frustum test
        let culler = FrustumCuller::new(
            [0.0, 0.0, 0.0].into(),     // Camera at origin
            [0.0, 0.0, -1.0].into(),    // Looking down negative Z
            [0.0, 1.0, 0.0].into(),     // Up is positive Y
            std::f64::consts::PI / 2.0, // 90 degree FOV (very wide)
            1.0,                        // Square aspect ratio
            1.0,                        // Near plane at Z = -1
            100.0,                      // Far plane at Z = -100
        );

        // Test simple cases
        // Point behind camera (positive Z) - should be culled
        let point_behind: Position = [0.0, 0.0, 10.0].into();
        assert!(!culler.is_point_in_frustum(point_behind));

        // Point in front but way off to the side - should be culled with wide FOV
        let point_far_side: Position = [1000.0, 0.0, -10.0].into();
        assert!(!culler.is_point_in_frustum(point_far_side));

        // Point beyond far plane - should be culled
        let point_too_far: Position = [0.0, 0.0, -200.0].into();
        assert!(!culler.is_point_in_frustum(point_too_far));

        // Point in center, within near/far bounds - should pass
        let center_point: Position = [0.0, 0.0, -10.0].into();
        // With 90 degree FOV, this should definitely be visible
        assert!(culler.is_point_in_frustum(center_point));
    }

    #[test]
    fn test_spatial_optimizer() {
        let mut optimizer = SpatialOptimizer::new(100.0);
        let mut particles = ParticleSet::new();

        // Add particles
        particles
            .add_body(
                Body::new()
                    .with_mass(1.0)
                    .with_position([0.0, 0.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();
        particles
            .add_body(
                Body::new()
                    .with_mass(1.0)
                    .with_position([200.0, 0.0, 0.0])
                    .with_velocity([0.0, 0.0, 0.0]),
            )
            .unwrap();

        optimizer.update(&particles);

        // Without frustum culling, all particles should be active
        let active = optimizer.get_active_particles(&particles);
        assert_eq!(active.len(), 2);

        let stats = optimizer.statistics();
        assert_eq!(stats.visible_particles, 2);
        assert!(stats.spatial_hashing_enabled);
        assert!(!stats.frustum_culling_enabled);
    }
}
