//! AABB tree for collision detection.

/// Axis-Aligned Bounding Box tree for collision detection.
#[derive(Debug, Clone)]
pub struct AabbTree;

impl AabbTree {
    /// Create a new AABB tree.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AabbTree {
    fn default() -> Self {
        Self::new()
    }
}
