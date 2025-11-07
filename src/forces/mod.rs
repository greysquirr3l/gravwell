//! Force calculation algorithms.

pub mod barnes_hut;
pub mod direct;

// Re-export public types
pub use barnes_hut::BarnesHut;
pub use direct::DirectGravity;
