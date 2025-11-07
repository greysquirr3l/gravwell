//! Spatial hash grid for collision detection.

/// Spatial hash grid for efficient collision detection.
#[derive(Debug, Clone)]
#[allow(dead_code)] // TODO: Remove when implementation is complete
pub struct SpatialHash {
    cell_size: f64,
}

impl SpatialHash {
    /// Create a new spatial hash grid.
    pub fn new(cell_size: f64) -> Self {
        Self { cell_size }
    }
}

impl Default for SpatialHash {
    fn default() -> Self {
        Self::new(1.0)
    }
}
