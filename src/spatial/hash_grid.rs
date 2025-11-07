//! Spatial hash grid implementation for efficient proximity queries and collision detection.
//!
//! The spatial hash grid provides O(1) insertion and O(k) neighbor queries where k is the
//! number of neighbors within the query radius. This is essential for large-scale simulations
//! where particles need to find nearby neighbors for collision detection, force calculations,
//! or other spatial queries.
//!
//! # Algorithm Overview
//!
//! The hash grid divides 3D space into uniform cubic cells, each identified by discrete
//! coordinates (i, j, k). Particles are inserted into cells based on their world position,
//! and neighbor queries search only the relevant cells within the query radius.
//!
//! # Performance Characteristics
//!
//! - **Insertion**: O(1) - Direct cell coordinate calculation and HashMap insertion
//! - **Removal**: O(1) - Direct cell lookup and particle removal
//! - **Neighbor Query**: O(k + c) where k = neighbors in range, c = cells to check
//! - **Memory**: O(n) where n = total particles
//!
//! # Usage Example
//!
//! ```rust
//! use gravwell::spatial::SpatialHashGrid;
//! use gravwell::types::{Vector3, BodyHandle};
//!
//! let mut grid = SpatialHashGrid::new(10.0); // 10 unit cell size
//!
//! // Insert particles
//! let handle1 = BodyHandle::new(1, 0);
//! let handle2 = BodyHandle::new(2, 0);
//! grid.insert_particle(handle1, Vector3::new(5.0, 5.0, 5.0));
//! grid.insert_particle(handle2, Vector3::new(15.0, 5.0, 5.0));
//!
//! // Find neighbors within 12 units of a position
//! let neighbors = grid.find_neighbors(Vector3::new(5.0, 5.0, 5.0), 12.0);
//! assert!(neighbors.contains(&handle1));
//! assert!(neighbors.contains(&handle2));
//! ```

use crate::types::{Scalar, Vector3};
use crate::BodyHandle;
use std::collections::HashMap;

/// Configuration for spatial hash grid behavior
#[derive(Debug, Clone)]
pub struct HashGridConfig {
    /// Size of each cubic cell in world units
    pub cell_size: Scalar,

    /// Initial capacity for the particle HashMap
    pub initial_capacity: usize,

    /// Whether to automatically optimize cell size based on particle distribution
    pub auto_optimize: bool,

    /// Maximum particles per cell before triggering optimization warnings
    pub max_particles_per_cell: usize,
}

impl Default for HashGridConfig {
    fn default() -> Self {
        Self {
            cell_size: 100.0,
            initial_capacity: 1000,
            auto_optimize: false,
            max_particles_per_cell: 50,
        }
    }
}

/// A single cell in the spatial hash grid containing multiple particles
#[derive(Debug, Clone)]
pub struct SpatialCell {
    /// Particles in this cell
    pub particles: Vec<BodyHandle>,

    /// Center position of this cell in world coordinates
    pub center: Vector3,

    /// Discrete cell coordinates in grid space
    pub coordinates: (i32, i32, i32),
}

impl SpatialCell {
    /// Create a new spatial cell with given coordinates and center
    pub fn new(coordinates: (i32, i32, i32), center: Vector3) -> Self {
        Self {
            particles: Vec::new(),
            center,
            coordinates,
        }
    }

    /// Add a particle to this cell
    pub fn add_particle(&mut self, handle: BodyHandle) {
        if !self.particles.contains(&handle) {
            self.particles.push(handle);
        }
    }

    /// Remove a particle from this cell
    pub fn remove_particle(&mut self, handle: BodyHandle) -> bool {
        if let Some(pos) = self.particles.iter().position(|&h| h == handle) {
            self.particles.swap_remove(pos);
            true
        } else {
            false
        }
    }

    /// Get the number of particles in this cell
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Check if this cell is empty
    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }
}

/// Spatial hash grid for efficient proximity queries and collision detection.
///
/// The hash grid divides 3D space into uniform cells and provides fast lookups
/// for particles within a given radius. This is essential for large-scale
/// simulations where O(N²) proximity checks would be prohibitively expensive.
pub struct SpatialHashGrid {
    /// Grid configuration
    config: HashGridConfig,

    /// Hash table mapping cell coordinates to spatial cells
    cells: HashMap<(i32, i32, i32), SpatialCell>,

    /// Mapping from particle handles to their current cell coordinates
    particle_to_cell: HashMap<BodyHandle, (i32, i32, i32)>,

    /// Mapping from particle handles to their world positions (for efficient queries)
    particle_positions: HashMap<BodyHandle, Vector3>,

    /// Statistics for performance monitoring
    stats: HashGridStats,
}

/// Statistics for hash grid performance analysis
#[derive(Debug, Clone, Default)]
pub struct HashGridStats {
    /// Total number of particles in the grid
    pub total_particles: usize,

    /// Number of occupied cells
    pub occupied_cells: usize,

    /// Total number of cells ever created
    pub total_cells: usize,

    /// Average particles per occupied cell
    pub avg_particles_per_cell: f32,

    /// Maximum particles in any single cell
    pub max_particles_per_cell: usize,

    /// Number of neighbor queries performed
    pub neighbor_queries: u64,

    /// Total neighbor query time in microseconds
    pub total_query_time_us: u64,
}

impl SpatialHashGrid {
    /// Create a new spatial hash grid with specified cell size
    ///
    /// # Arguments
    ///
    /// * `cell_size` - Size of each cubic cell in world units
    ///
    /// # Example
    ///
    /// ```rust
    /// use gravwell::spatial::SpatialHashGrid;
    ///
    /// // Create grid with 50-unit cells (good for particles with ~25 unit interaction radius)
    /// let grid = SpatialHashGrid::new(50.0);
    /// ```
    pub fn new(cell_size: Scalar) -> Self {
        let config = HashGridConfig {
            cell_size,
            ..Default::default()
        };

        Self::with_config(config)
    }

    /// Create a new spatial hash grid with custom configuration
    pub fn with_config(config: HashGridConfig) -> Self {
        Self {
            cells: HashMap::with_capacity(config.initial_capacity),
            particle_to_cell: HashMap::with_capacity(config.initial_capacity),
            particle_positions: HashMap::with_capacity(config.initial_capacity),
            config,
            stats: HashGridStats::default(),
        }
    }

    /// Insert a particle into the spatial hash grid
    ///
    /// If the particle already exists, it will be moved to the new position.
    ///
    /// # Arguments
    ///
    /// * `handle` - Unique identifier for the particle
    /// * `position` - World position of the particle
    ///
    /// # Example
    ///
    /// ```rust
    /// use gravwell::spatial::SpatialHashGrid;
    /// use gravwell::types::{Vector3, BodyHandle};
    ///
    /// let mut grid = SpatialHashGrid::new(10.0);
    /// let handle = BodyHandle::new(1, 0);
    /// grid.insert_particle(handle, Vector3::new(25.0, 35.0, 15.0));
    /// ```
    pub fn insert_particle(&mut self, handle: BodyHandle, position: Vector3) {
        // Remove particle from previous cell if it exists
        if let Some(&old_coords) = self.particle_to_cell.get(&handle) {
            if let Some(cell) = self.cells.get_mut(&old_coords) {
                cell.remove_particle(handle);
                if cell.is_empty() {
                    self.cells.remove(&old_coords);
                }
            }
        }

        // Calculate new cell coordinates
        let coords = self.world_to_cell_coords(position);

        // Precompute world coordinates to avoid borrowing issues
        let world_coords = self.cell_coords_to_world(coords);
        let new_cell_created = !self.cells.contains_key(&coords);

        // Insert into new cell
        let cell = self
            .cells
            .entry(coords)
            .or_insert_with(|| SpatialCell::new(coords, world_coords));

        if new_cell_created {
            self.stats.total_cells += 1;
        }

        cell.add_particle(handle);

        // Update tracking maps
        self.particle_to_cell.insert(handle, coords);
        self.particle_positions.insert(handle, position);

        // Update statistics
        self.update_stats();
    }

    /// Remove a particle from the spatial hash grid
    ///
    /// # Arguments
    ///
    /// * `handle` - Handle of the particle to remove
    ///
    /// # Returns
    ///
    /// `true` if the particle was found and removed, `false` otherwise
    pub fn remove_particle(&mut self, handle: BodyHandle) -> bool {
        if let Some(&coords) = self.particle_to_cell.get(&handle) {
            if let Some(cell) = self.cells.get_mut(&coords) {
                let removed = cell.remove_particle(handle);

                // Remove empty cells to save memory
                if cell.is_empty() {
                    self.cells.remove(&coords);
                }

                if removed {
                    self.particle_to_cell.remove(&handle);
                    self.particle_positions.remove(&handle);
                    self.update_stats();
                    return true;
                }
            }
        }
        false
    }

    /// Find all particles within a given radius of a position
    ///
    /// This is the primary query method for proximity detection. It efficiently
    /// searches only the cells that could contain particles within the query radius.
    ///
    /// # Arguments
    ///
    /// * `position` - Center position for the query
    /// * `radius` - Search radius in world units
    ///
    /// # Returns
    ///
    /// Vector of particle handles within the specified radius
    ///
    /// # Example
    ///
    /// ```rust
    /// use gravwell::spatial::SpatialHashGrid;
    /// use gravwell::types::Vector3;
    ///
    /// let grid = SpatialHashGrid::new(10.0);
    /// let neighbors = grid.find_neighbors(Vector3::new(0.0, 0.0, 0.0), 50.0);
    /// ```
    pub fn find_neighbors(&self, position: Vector3, radius: Scalar) -> Vec<BodyHandle> {
        use std::time::Instant;
        let _start_time = Instant::now();

        let mut neighbors = Vec::new();
        let radius_squared = radius * radius;

        // Calculate the range of cells to check
        let cell_radius = (radius / self.config.cell_size).ceil() as i32;
        let center_coords = self.world_to_cell_coords(position);

        // Search all cells within the radius
        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                for dz in -cell_radius..=cell_radius {
                    let coords = (
                        center_coords.0 + dx,
                        center_coords.1 + dy,
                        center_coords.2 + dz,
                    );

                    if let Some(cell) = self.cells.get(&coords) {
                        // Check each particle in this cell
                        for &particle_handle in &cell.particles {
                            if let Some(&particle_pos) =
                                self.particle_positions.get(&particle_handle)
                            {
                                let distance_squared = (particle_pos - position).norm_squared();
                                if distance_squared <= radius_squared {
                                    neighbors.push(particle_handle);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update query statistics (would need mutable reference for this)
        // For now, we'll update stats only in mutable methods

        neighbors
    }

    /// Find all particles within a spherical region defined by center and radius
    ///
    /// This is similar to `find_neighbors` but includes the center particle if it exists.
    pub fn find_particles_in_sphere(&self, center: Vector3, radius: Scalar) -> Vec<BodyHandle> {
        self.find_neighbors(center, radius)
    }

    /// Get the position of a particle by its handle
    pub fn get_particle_position(&self, handle: BodyHandle) -> Option<Vector3> {
        self.particle_positions.get(&handle).copied()
    }

    /// Get all particles currently in the grid
    pub fn get_all_particles(&self) -> Vec<BodyHandle> {
        self.particle_positions.keys().copied().collect()
    }

    /// Clear all particles from the grid
    pub fn clear(&mut self) {
        self.cells.clear();
        self.particle_to_cell.clear();
        self.particle_positions.clear();
        self.stats = HashGridStats::default();
    }

    /// Get the number of particles currently in the grid
    pub fn particle_count(&self) -> usize {
        self.particle_positions.len()
    }

    /// Get the number of cells currently in use
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Get the number of occupied cells (cells with at least one particle)
    pub fn occupied_cell_count(&self) -> usize {
        self.cells.values().filter(|cell| !cell.is_empty()).count()
    }

    /// Get the average number of particles per occupied cell
    pub fn average_particles_per_cell(&self) -> f32 {
        let occupied = self.occupied_cell_count();
        if occupied > 0 {
            self.particle_count() as f32 / occupied as f32
        } else {
            0.0
        }
    }

    /// Get the maximum number of particles in any single cell
    pub fn max_particles_per_cell(&self) -> usize {
        self.cells
            .values()
            .map(|cell| cell.particle_count())
            .max()
            .unwrap_or(0)
    }

    /// Get current grid statistics
    pub fn get_statistics(&self) -> &HashGridStats {
        &self.stats
    }

    /// Get cells that intersect with a bounding box
    ///
    /// Useful for frustum culling and other spatial queries
    pub fn get_cells_in_box(&self, min: Vector3, max: Vector3) -> Vec<&SpatialCell> {
        let min_coords = self.world_to_cell_coords(min);
        let max_coords = self.world_to_cell_coords(max);

        let mut cells = Vec::new();

        for x in min_coords.0..=max_coords.0 {
            for y in min_coords.1..=max_coords.1 {
                for z in min_coords.2..=max_coords.2 {
                    if let Some(cell) = self.cells.get(&(x, y, z)) {
                        cells.push(cell);
                    }
                }
            }
        }

        cells
    }

    /// Optimize the grid by analyzing particle distribution
    ///
    /// This method can suggest better cell sizes based on current particle density
    pub fn analyze_optimization(&self) -> HashGridOptimization {
        let avg_particles = self.average_particles_per_cell();
        let max_particles = self.max_particles_per_cell();
        let _total_particles = self.particle_count();
        let _occupied_cells = self.occupied_cell_count();

        let current_cell_size = self.config.cell_size;

        // Calculate suggested cell size based on particle density
        let ideal_particles_per_cell = 10.0; // Target for good performance
        let density_ratio = avg_particles / ideal_particles_per_cell;
        let suggested_cell_size = if density_ratio > 1.0 {
            current_cell_size * (density_ratio as f64).sqrt() // Increase cell size for high density
        } else if density_ratio < 0.5 {
            current_cell_size * 0.7 // Decrease cell size for low density
        } else {
            current_cell_size // Current size is good
        };

        HashGridOptimization {
            current_cell_size,
            suggested_cell_size,
            current_avg_particles_per_cell: avg_particles,
            current_max_particles_per_cell: max_particles,
            optimization_score: 1.0 / (1.0 + (avg_particles - ideal_particles_per_cell).abs()),
        }
    }

    /// Convert world coordinates to cell coordinates
    fn world_to_cell_coords(&self, position: Vector3) -> (i32, i32, i32) {
        (
            (position.x / self.config.cell_size).floor() as i32,
            (position.y / self.config.cell_size).floor() as i32,
            (position.z / self.config.cell_size).floor() as i32,
        )
    }

    /// Convert cell coordinates to world center position
    fn cell_coords_to_world(&self, coords: (i32, i32, i32)) -> Vector3 {
        Vector3::new(
            coords.0 as Scalar * self.config.cell_size + self.config.cell_size * 0.5,
            coords.1 as Scalar * self.config.cell_size + self.config.cell_size * 0.5,
            coords.2 as Scalar * self.config.cell_size + self.config.cell_size * 0.5,
        )
    }

    /// Update internal statistics
    fn update_stats(&mut self) {
        self.stats.total_particles = self.particle_count();
        self.stats.occupied_cells = self.occupied_cell_count();
        self.stats.avg_particles_per_cell = self.average_particles_per_cell();
        self.stats.max_particles_per_cell = self.max_particles_per_cell();
    }
}

/// Optimization analysis result for hash grid performance tuning
#[derive(Debug, Clone)]
pub struct HashGridOptimization {
    /// Current cell size being used
    pub current_cell_size: Scalar,

    /// Suggested optimal cell size
    pub suggested_cell_size: Scalar,

    /// Current average particles per cell
    pub current_avg_particles_per_cell: f32,

    /// Current maximum particles in any cell
    pub current_max_particles_per_cell: usize,

    /// Optimization score (0.0 = poor, 1.0 = optimal)
    pub optimization_score: f32,
}

impl HashGridOptimization {
    /// Check if optimization is recommended
    pub fn should_optimize(&self) -> bool {
        self.optimization_score < 0.7
            || (self.suggested_cell_size - self.current_cell_size).abs()
                > self.current_cell_size * 0.2
    }

    /// Get optimization recommendations as a string
    pub fn get_recommendations(&self) -> String {
        if self.optimization_score > 0.9 {
            "Grid is well optimized for current particle distribution.".to_string()
        } else if self.suggested_cell_size > self.current_cell_size {
            format!(
                "Consider increasing cell size to {:.1} to reduce overcrowding (avg {:.1} particles/cell).",
                self.suggested_cell_size, self.current_avg_particles_per_cell
            )
        } else {
            format!(
                "Consider decreasing cell size to {:.1} to improve spatial resolution.",
                self.suggested_cell_size
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builder::BodyHandle, types::Vector3};

    #[test]
    fn test_hash_grid_creation() {
        let grid = SpatialHashGrid::new(10.0);
        assert_eq!(grid.config.cell_size, 10.0);
        assert_eq!(grid.particle_count(), 0);
        assert_eq!(grid.cell_count(), 0);
    }

    #[test]
    fn test_world_to_cell_coords() {
        let grid = SpatialHashGrid::new(10.0);

        // Test coordinate conversion
        assert_eq!(
            grid.world_to_cell_coords(Vector3::new(5.0, 5.0, 5.0)),
            (0, 0, 0)
        );
        assert_eq!(
            grid.world_to_cell_coords(Vector3::new(15.0, 25.0, 35.0)),
            (1, 2, 3)
        );
        assert_eq!(
            grid.world_to_cell_coords(Vector3::new(-5.0, -15.0, -25.0)),
            (-1, -2, -3)
        );
    }

    #[test]
    fn test_cell_coords_to_world() {
        let grid = SpatialHashGrid::new(10.0);

        // Test center calculation (should be at cell center)
        let center = grid.cell_coords_to_world((0, 0, 0));
        assert!((center.x - 5.0).abs() < 1e-6);
        assert!((center.y - 5.0).abs() < 1e-6);
        assert!((center.z - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_particle_insertion_and_removal() {
        let mut grid = SpatialHashGrid::new(10.0);
        let handle = BodyHandle::new(1, 0);
        let position = Vector3::new(5.0, 5.0, 5.0);

        // Insert particle
        grid.insert_particle(handle, position);
        assert_eq!(grid.particle_count(), 1);
        assert_eq!(grid.cell_count(), 1);
        assert_eq!(grid.get_particle_position(handle), Some(position));

        // Remove particle
        assert!(grid.remove_particle(handle));
        assert_eq!(grid.particle_count(), 0);
        assert_eq!(grid.cell_count(), 0); // Cell should be removed when empty
        assert_eq!(grid.get_particle_position(handle), None);

        // Try to remove non-existent particle
        assert!(!grid.remove_particle(handle));
    }

    #[test]
    fn test_particle_movement() {
        let mut grid = SpatialHashGrid::new(10.0);
        let handle = BodyHandle::new(1, 0);

        // Insert at one position
        grid.insert_particle(handle, Vector3::new(5.0, 5.0, 5.0));
        assert_eq!(grid.particle_count(), 1);
        assert_eq!(grid.cell_count(), 1);

        // Move to different cell
        grid.insert_particle(handle, Vector3::new(15.0, 15.0, 15.0));
        assert_eq!(grid.particle_count(), 1);
        assert_eq!(grid.cell_count(), 1); // Old cell removed, new cell created

        let new_pos = grid.get_particle_position(handle).unwrap();
        assert!((new_pos.x - 15.0).abs() < 1e-6);
        assert!((new_pos.y - 15.0).abs() < 1e-6);
        assert!((new_pos.z - 15.0).abs() < 1e-6);
    }

    #[test]
    fn test_neighbor_finding() {
        let mut grid = SpatialHashGrid::new(10.0);

        // Insert particles in a line
        let handles: Vec<BodyHandle> = (0..5).map(|i| BodyHandle::new(i, 0)).collect();
        let positions = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(5.0, 0.0, 0.0),
            Vector3::new(10.0, 0.0, 0.0),
            Vector3::new(20.0, 0.0, 0.0),
            Vector3::new(50.0, 0.0, 0.0),
        ];

        for (handle, position) in handles.iter().zip(positions.iter()) {
            grid.insert_particle(*handle, *position);
        }

        // Find neighbors within 12 units of origin
        let neighbors = grid.find_neighbors(Vector3::new(0.0, 0.0, 0.0), 12.0);

        // Should find first three particles (at 0, 5, 10 units)
        assert!(neighbors.contains(&handles[0]));
        assert!(neighbors.contains(&handles[1]));
        assert!(neighbors.contains(&handles[2]));
        assert!(!neighbors.contains(&handles[3])); // At 20 units
        assert!(!neighbors.contains(&handles[4])); // At 50 units
    }

    #[test]
    fn test_empty_neighbor_query() {
        let grid = SpatialHashGrid::new(10.0);
        let neighbors = grid.find_neighbors(Vector3::new(0.0, 0.0, 0.0), 50.0);
        assert!(neighbors.is_empty());
    }

    #[test]
    fn test_statistics() {
        let mut grid = SpatialHashGrid::new(10.0);

        // Insert multiple particles in different cells
        for i in 0..10 {
            let handle = BodyHandle::new(i, 0);
            let position = Vector3::new(i as f64 * 5.0, 0.0, 0.0);
            grid.insert_particle(handle, position);
        }

        assert_eq!(grid.particle_count(), 10);
        assert!(grid.occupied_cell_count() > 0);
        assert!(grid.average_particles_per_cell() > 0.0);
        assert!(grid.max_particles_per_cell() > 0);

        let stats = grid.get_statistics();
        assert_eq!(stats.total_particles, 10);
    }

    #[test]
    fn test_clear() {
        let mut grid = SpatialHashGrid::new(10.0);

        // Insert some particles
        for i in 0..5 {
            let handle = BodyHandle::new(i, 0);
            grid.insert_particle(handle, Vector3::new(i as f64, 0.0, 0.0));
        }

        assert_eq!(grid.particle_count(), 5);
        assert!(grid.cell_count() > 0);

        grid.clear();
        assert_eq!(grid.particle_count(), 0);
        assert_eq!(grid.cell_count(), 0);
        assert_eq!(grid.get_statistics().total_particles, 0);
    }

    #[test]
    fn test_optimization_analysis() {
        let mut grid = SpatialHashGrid::new(10.0);

        // Create a scenario with many particles in few cells (overcrowding)
        for i in 0..100 {
            let handle = BodyHandle::new(i, 0);
            let position = Vector3::new(i as f64 * 0.1, 0.0, 0.0); // All in same cell
            grid.insert_particle(handle, position);
        }

        let optimization = grid.analyze_optimization();
        assert!(optimization.current_avg_particles_per_cell > 10.0);
        assert!(optimization.should_optimize());
        assert!(optimization.suggested_cell_size < optimization.current_cell_size);

        let recommendations = optimization.get_recommendations();
        assert!(!recommendations.is_empty());
    }
}
