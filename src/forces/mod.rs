//! Force calculation algorithms.

pub mod barnes_hut;
pub mod direct;

#[cfg(feature = "parallel")]
pub mod parallel;

// Re-export public types
pub use barnes_hut::BarnesHut;
pub use direct::DirectGravity;

#[cfg(feature = "parallel")]
pub use parallel::{ChunkSizeStrategy, ParallelDirectGravity};
